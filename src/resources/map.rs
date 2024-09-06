//! All resources related to tracking/mapping particles.
use crate::Hibernating;
use ahash::{HashMap, HashMapExt};
use bevy::prelude::*;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;

/// Map of all parent particle types to their corresponding entity. Used to map particle types to their corresponding data
#[derive(Resource, Clone, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct ParticleTypeMap {
    /// The mapping resource for particle types.
    /// The std collections HashMap is used here because it supports type reflection by default, whereas ahash::HashMap does not.
    map: std::collections::HashMap<String, Entity>,
}

impl ParticleTypeMap {
    /// Provides an iterator of the particle type map
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Entity)> {
        self.map.iter()
    }

    /// Provides an iterator over the keys of the particle type map
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.map.keys()
    }

    /// Insert a new particle type to the map
    pub fn insert(&mut self, ptype: String, entity: Entity) -> &mut Entity {
        self.map.entry(ptype).or_insert(entity)
    }

    /// Get an immutable reference to the corresponding entity, if it exists.
    pub fn get(&self, ptype: &String) -> Option<&Entity> {
        self.map.get(ptype)
    }
}

/// Generic resource for mapping IVec2 to entities. This resource uses underlying "chunks" which can optimize
/// performance when processing large numbers of entities that may be inactive for some time. Chunks will enter a
/// "hibernating" status if none of their contents are processed in a given frame.
///
/// Entities that move in a given frame which also border another chunk will send a "wake up" signal to that chunk,
/// causing it to be active for the next frame.
#[derive(Resource, Debug, Clone)]
pub struct ChunkMap {
    /// The entity chunk maps
    chunks: Vec<Chunk>,
}

impl Default for ChunkMap {
    /// A default chunk has a size of 32x32 entities. A default chunk map can hold 32 chunks, effectively capable of
    /// storing 1024x1024 (1,048,576) total entities.
    fn default() -> ChunkMap {
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
    /// Gets the index of the corresponding chunk
    fn chunk_index(&self, coord: &IVec2) -> usize {
        const OFFSET: i32 = 512;
        const GRID_WIDTH: usize = 32;

        let col = ((coord.x + OFFSET) >> 5) as usize;
        let row = ((OFFSET - coord.y) >> 5) as usize;

        row * GRID_WIDTH + col
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
    /// Clear all existing key, value pairs from all chunks.
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if entities are not despawned before another
    /// system attempts to access them.
    pub fn clear(&mut self) {
        for map in &mut self.chunks {
            map.clear();
        }
    }

    /// Remove a particle from the map.
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if the target entity is not despawned before
    /// another system attempts to access it.
    pub fn remove(&mut self, coords: &IVec2) -> Option<Entity> {
        self.chunk_mut(&coords).unwrap().remove(coords)
    }
}

impl ChunkMap {
    /// Immutable iterator over all chunks.
    pub fn iter_chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.iter()
    }
}

impl ChunkMap {
    /// Checks each chunk for activity in the current frame. This method is meant to be called after all
    /// movement logic has occurred for this frame.
    ///
    /// If a chunk was active and is currently hibernating, wake it up and remove the Hibernating marker
    /// component from its entity.
    ///
    /// If a chunk was not activated and is currently awake, put it to sleep and add the Hibernating
    /// component to its entity.
    pub fn reset_chunks(&mut self, mut commands: Commands) {
        self.chunks.iter_mut().for_each(|chunk| {
            // Check for both so we're not needlessly removing components every frame
            if chunk.should_process_next_frame == true && chunk.hibernating == true {
                chunk.iter().for_each(|(_, entity)| {
                    commands.entity(*entity).remove::<Hibernating>();
                });
                chunk.hibernating = false;

            // Deactivate before the start of the next frame
            } else if chunk.should_process_next_frame == false && chunk.hibernating == false {
                chunk.iter().for_each(|(_, entity)| {
                    commands.entity(*entity).insert(Hibernating);
                });
                chunk.hibernating = true;
            }

            chunk.should_process_next_frame = false;
        });
    }

    /// Checks if a coordinate lies on the border of a neighboring chunk and activates it if true.
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
    /// Inserts a new particle at a given coordinate if it is not already occupied. Calls to this method will
    /// wake up the subject chunk.
    pub fn insert_no_overwrite(&mut self, coords: IVec2, entity: Entity) -> &mut Entity {
        self.chunk_mut(&coords)
            .unwrap()
            .insert_no_overwrite(coords, entity)
    }

    /// Inserts a new particle at a given coordinate irrespective of its occupied state. Calls to this method will
    /// wake up the subject chunk.
    pub fn insert_overwrite(&mut self, coords: IVec2, entity: Entity) -> Option<Entity> {
        self.chunk_mut(&coords)
            .unwrap()
            .insert_overwrite(coords, entity)
    }

    /// Swaps two entities in the ChunkMap. This method is the preferred interface when carrying out component-based
    /// interactions between entities due to the facilities this provides for waking up neighboring chunks.
    /// 'insert_overwrite' and 'insert_no_overwrite' will wake up the subject chunk, but they will NOT wake up
    /// neighboring chunks.
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

/// A chunk which stores location information for entities.
#[derive(Debug, Clone)]
pub struct Chunk {
    /// The chunk containing the particle data
    chunk: HashMap<IVec2, Entity>,
    /// The area of the chunk
    irect: IRect,
    /// Flag indicating whether the chunk should be processed in the next frame
    should_process_next_frame: bool,
    /// Flag indicating whether the chunk should be processed this frame
    hibernating: bool,
}

impl Chunk {
    /// Creates a new Chunk instance
    pub fn new(upper_left: IVec2, lower_right: IVec2) -> Chunk {
        Chunk {
            chunk: HashMap::with_capacity(1024),
            irect: IRect::from_corners(upper_left, lower_right),
            should_process_next_frame: false,
            hibernating: false,
        }
    }
}

impl Chunk {
    /// The minimum (upper left) point of the chunk's area
    pub fn min(&self) -> &IVec2 {
        &self.irect.min
    }

    /// The maximum (lower right) point of the chunk's area
    pub fn max(&self) -> &IVec2 {
        &self.irect.max
    }
}

impl Chunk {
    /// The chunk should be processed in the current frame
    pub fn hibernating(&self) -> bool {
        self.hibernating
    }

    /// The chunk should be processed in the next frame
    pub fn should_process_next_frame(&self) -> bool {
        self.should_process_next_frame
    }
}

impl Chunk {
    /// Get an immutable reference to the corresponding entity, if it exists.
    pub fn get(&self, coords: &IVec2) -> Option<&Entity> {
        self.chunk.get(coords)
    }
}

impl Chunk {
    /// Iterate through all key, value instances of the entity map
    pub fn iter(&self) -> impl Iterator<Item = (&IVec2, &Entity)> {
        self.chunk.iter()
    }

    /// Parallel iter through all the key, value instances of the entity map
    pub fn par_iter(&self) -> impl IntoParallelIterator<Item = (&IVec2, &Entity)> {
        self.chunk.par_iter()
    }
}

impl Chunk {
    /// Clear all existing entities from the map
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if entities are not
    /// simultaneously cleared within the same system from which this method was called.
    pub fn clear(&mut self) {
        self.chunk.clear();
    }

    /// Remove a entity from the map
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if entities are not
    /// simultaneously cleared within the same system from which this method was called.
    pub fn remove(&mut self, coords: &IVec2) -> Option<Entity> {
        self.should_process_next_frame = true;
        self.chunk.remove(coords)
    }

    /// Inserts a new entity at a given coordinate if it is not already occupied. Calls to this method will
    /// wake up the subject chunk.
    pub fn insert_no_overwrite(&mut self, coords: IVec2, entity: Entity) -> &mut Entity {
        self.should_process_next_frame = true;
        self.chunk.entry(coords).or_insert(entity)
    }

    /// Inserts a new entity at a given coordinate irrespective of its occupied state. Calls to this method will
    /// wake up the subject chunk.
    pub fn insert_overwrite(&mut self, coords: IVec2, entity: Entity) -> Option<Entity> {
        self.should_process_next_frame = true;
        self.chunk.insert(coords, entity)
    }
}
