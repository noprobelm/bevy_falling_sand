use crate::{Hibernating, ParticleType};
use ahash::HashMap;
use bevy::prelude::*;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;

/// A map of all parent particle types to their corresponding entity. Used to map child particles to their corresponding
/// parent types
#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleParentMap {
    /// The mapping resource for particle types.
    map: HashMap<ParticleType, Entity>,
}

impl ParticleParentMap {
    /// Insert a new particle type to the map
    pub fn insert(&mut self, ptype: ParticleType, entity: Entity) -> &mut Entity {
        self.map.entry(ptype).or_insert(entity)
    }

    /// Get an immutable reference to the corresponding entity, if it exists.
    pub fn get(&self, ptype: &ParticleType) -> Option<&Entity> {
        self.map.get(ptype)
    }
}

/// The mapping resource for the position of all particles in the world. This map utilizes "chunks" consisting of particles
/// in a given region. This allows for putting chunks to "sleep" so their particles don't induce unecessary processing
/// overhead each frame.
#[derive(Resource, Debug, Clone)]
pub struct ChunkMap {
    /// The particle chunk maps
    chunks: Vec<Chunk>,
}

impl Default for ChunkMap {
    fn default() -> ChunkMap {
        use bevy::math::IVec2;

        let chunks: Vec<Chunk> = (0..32_i32.pow(2))
            .map(|i| {
                let x = (i % 32) * 32 - 512;
                let y = 512 - (i / 32) * 32;
                let upper_left = IVec2::new(x, y - 31);
                let lower_right = IVec2::new(x + 31, y);
                Chunk::new(upper_left, lower_right)
            })
            .collect();

        ChunkMap { chunks }
    }
}

impl ChunkMap {
    /// Gets the index of the corresponding chunk when given an &IVec2
    fn chunk_index(&self, coord: &IVec2) -> usize {
        let col = ((coord.x + 512) / 32) as usize;
        let row = ((512 - coord.y) / 32) as usize;
        row * 32 + col
    }

    /// Gets an immutable reference to a chunk
    fn chunk(&self, coord: &IVec2) -> Option<&Chunk> {
        let index = self.chunk_index(coord);
        self.chunks.get(index)
    }

    /// Gets a mutable reference to a chunk
    fn chunk_mut(&mut self, coord: &IVec2) -> Option<&mut Chunk> {
        let index = self.chunk_index(coord);
        self.chunks.get_mut(index)
    }
}

impl ChunkMap {
    /// Clear all existing particles from the map
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if particles are not
    /// simultaneously cleared within the same system from which this method was called.
    pub fn clear(&mut self) {
        for map in &mut self.chunks {
            map.clear();
        }
    }

    /// Remove a particle from the map
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if particles are not
    /// simultaneously cleared within the same system from which this method was called.
    pub fn remove(&mut self, coords: &IVec2) -> Option<Entity> {
        self.chunk_mut(&coords).unwrap().remove(coords)
    }
}

impl ChunkMap {
    /// Immutable iterator over all chunks in the ChunkMap
    pub fn iter_chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.iter()
    }

    pub fn iter_chunks_mut(&mut self) -> impl Iterator<Item = &mut Chunk> {
        self.chunks.iter_mut()
    }
}

impl ChunkMap {
    /// Checks each chunk for activity in the previous frame.
    ///
    /// If a chunk was active and is currently sleeping, wake it up and remove the Hibernating flag component from its
    /// HashMap.
    ///
    /// If a chunk was not activated and is currently awake, put it to sleep and add the Hibernating component to its
    /// HashMap.
    pub fn reset_chunks(&mut self, mut commands: Commands) {
        self.chunks.iter_mut().for_each(|chunk| {
            // Check for both so we're not needlessly removing components every frame
            if chunk.should_process_next_frame == true && chunk.should_process_this_frame == true {
                chunk.iter().for_each(|(_, entity)| {
                    commands.entity(*entity).remove::<Hibernating>();
                });
                chunk.should_process_this_frame = false;

            // Deactivate before the start of the next frame
            } else if chunk.should_process_next_frame == false {
                chunk.iter().for_each(|(_, entity)| {
                    commands.entity(*entity).insert(Hibernating);
                });
                chunk.should_process_this_frame = true;
            }

            chunk.should_process_next_frame = false;
        });
    }

    /// Checks if a coordinate lies on the border of a neighboring chunk and activates it if true
    fn activate_neighbor_chunks(&mut self, coord: &IVec2, chunk_idx: usize) {
        let chunk = &self.chunks[chunk_idx];

        if coord.x == chunk.min().x {
            self.chunks[chunk_idx - 1].should_process_next_frame = true;
        } else if coord.x == chunk.max().x {
            self.chunks[chunk_idx + 1].should_process_next_frame = true;
        } else if coord.y == chunk.min().y {
            self.chunks[chunk_idx + 32].should_process_next_frame = true;
        } else if coord.y == chunk.max().y {
            self.chunks[chunk_idx - 32].should_process_next_frame = true;
        }
    }
}

impl ChunkMap {
    /// Inserts a new particle at a given coordinate if it is not already occupied
    pub fn insert_no_overwrite(&mut self, coords: IVec2, entity: Entity) -> &mut Entity {
        self.chunk_mut(&coords)
            .unwrap()
            .insert_no_overwrite(coords, entity)
    }

    /// Inserts a new particle at a given coordinate irrespective of its occupied state
    pub fn insert_overwrite(&mut self, coords: IVec2, entity: Entity) -> Option<Entity> {
        self.chunk_mut(&coords)
            .unwrap()
            .insert_overwrite(coords, entity)
    }

    pub fn swap(&mut self, first: IVec2, second: IVec2) {
        let (first_chunk_idx, second_chunk_idx) =
            (self.chunk_index(&first), self.chunk_index(&second));

        if let Some(entity) = self.chunks[second_chunk_idx].remove(&second) {
            if let Some(swapped) = self.chunks[first_chunk_idx].insert_overwrite(first, entity) {
                self.chunks[second_chunk_idx].insert_overwrite(second, swapped);
            }
        } else {
            let entity = self.chunks[first_chunk_idx].remove(&first).unwrap();
            self.chunks[second_chunk_idx].insert_overwrite(second, entity);
        }

        self.activate_neighbor_chunks(&first, first_chunk_idx);
        self.activate_neighbor_chunks(&second, second_chunk_idx);
    }

    /// Get an immutable reference to an entity, if it exists.
    pub fn entity(&self, coords: &IVec2) -> Option<&Entity> {
        self.chunk(&coords).unwrap().get(coords)
    }

    /// Get an immutable reference to an entity, if it exists.
    pub fn get_mut(&mut self, coords: &IVec2) -> Option<&mut Entity> {
        self.chunk_mut(&coords).unwrap().get_mut(coords)
    }

    /// Iterator through a flattened map of all the particles in the ChunkMap
    #[allow(unused)]
    pub fn iter(&self) -> impl Iterator<Item = (&IVec2, &Entity)> {
        self.chunks.iter().flat_map(|chunk| chunk.iter())
    }

    /// Parallel iterator through a flattened map of all the particles in the ChunkMap
    pub fn par_iter(&self) -> impl IntoParallelIterator<Item = (&IVec2, &Entity)> {
        self.chunks.par_iter().flat_map(|chunk| chunk.par_iter())
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    /// The chunk containing the particle data
    chunk: HashMap<IVec2, Entity>,
    /// The area of the chunk
    irect: IRect,
    /// Flag indicating whether the chunk should be processed in the next frame
    pub should_process_next_frame: bool,
    /// Flag indicating whether the chunk should be processed this frame
    pub should_process_this_frame: bool,
}

impl Chunk {
    pub fn new(upper_left: IVec2, lower_right: IVec2) -> Chunk {
        Chunk {
            chunk: HashMap::default(),
            irect: IRect::from_corners(upper_left, lower_right),
            should_process_next_frame: false,
            should_process_this_frame: false,
        }
    }
}

impl Chunk {
    /// The minimum (upper left) point of the chunk's area
    pub fn min(&self) -> IVec2 {
        self.irect.min
    }

    /// The maximum (lower right) point of the chunk's area
    pub fn max(&self) -> IVec2 {
        self.irect.max
    }
}

impl Chunk {
    /// Get an immutable reference to the corresponding entity, if it exists.
    pub fn get(&self, coords: &IVec2) -> Option<&Entity> {
        self.chunk.get(coords)
    }

    /// Get an immutable reference to the corresponding entity, if it exists.
    pub fn get_mut(&mut self, coords: &IVec2) -> Option<&mut Entity> {
        self.should_process_next_frame = true;
        self.chunk.get_mut(coords)
    }
}

impl Chunk {
    /// Iterate through all key, value instances of the particle map
    pub fn iter(&self) -> impl Iterator<Item = (&IVec2, &Entity)> {
        self.chunk.iter()
    }

    /// Parallel iter through all the key, value instances of the particle map
    pub fn par_iter(&self) -> impl IntoParallelIterator<Item = (&IVec2, &Entity)> {
        self.chunk.par_iter()
    }
}

impl Chunk {
    /// Clear all existing particles from the map
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if particles are not
    /// simultaneously cleared within the same system from which this method was called.
    pub fn clear(&mut self) {
        self.chunk.clear();
    }

    /// Remove a particle from the map
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if particles are not
    /// simultaneously cleared within the same system from which this method was called.
    pub fn remove(&mut self, coords: &IVec2) -> Option<Entity> {
        self.should_process_next_frame = true;
        self.chunk.remove(coords)
    }

    /// Inserts a new particle at a given coordinate if it is not already occupied
    pub fn insert_no_overwrite(&mut self, coords: IVec2, entity: Entity) -> &mut Entity {
        self.should_process_next_frame = true;
        self.chunk.entry(coords).or_insert(entity)
    }

    /// Inserts a new particle at a given coordinate irrespective of its occupied state
    pub fn insert_overwrite(&mut self, coords: IVec2, entity: Entity) -> Option<Entity> {
        self.should_process_next_frame = true;
        self.chunk.insert(coords, entity)
    }
}
