use crate::{ParticleType, Sleeping};
use ahash::HashMap;
use bevy::prelude::*;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;

/// A map of all parent particle types to their corresponding entity. This is used exclusively for
/// assigning child particles to their respective parent when initially spawned or have otherwise
/// changed types (e.g., a reaction has occurred). For accessing common particle data, we have
/// dedicated entities with the ParticleParent component to provide us faster access than what
/// a HashMap could provide.
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

/// The mapping resource for the position of all particles in the world.
#[derive(Resource, Debug, Clone)]
pub struct ParticleMap {
    /// The particle chunk maps
    chunks: Vec<ChunkMap>,
}

impl Default for ParticleMap {
    fn default() -> ParticleMap {
        ParticleMap {
            chunks: (0..32_u32.pow(2)).map(|_| ChunkMap::default()).collect(),
        }
    }
}

impl ParticleMap {
    /// Gets the index of the appropriate chunk when given an &IVec2
    fn get_chunk_index(&self, coord: &IVec2) -> usize {
        let col = ((coord.x + 1024) / 32) as usize;
        let row = ((1024 - coord.y) / 32) as usize;
        row * 16 + col
    }

    /// Gets an immutable reference to a chunk
    fn get_chunk(&self, coord: &IVec2) -> Option<&ChunkMap> {
        let index = self.get_chunk_index(coord);
        self.chunks.get(index)
    }

    /// Gets a mutable reference to a chunk
    fn get_chunk_mut(&mut self, coord: &IVec2) -> Option<&mut ChunkMap> {
        let index = self.get_chunk_index(coord);
        self.chunks.get_mut(index)
    }
}

impl ParticleMap {
    /// Clear all existing particles from the map
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if particles are not
    /// simultaneously cleared within the same system from which this method was called.
    pub fn clear(&mut self) {
        for map in &mut self.chunks {
            map.clear();
        }
    }

    /// Checks each chunk for activity in the previous frame.
    ///
    /// If a chunk was active and is currently sleeping, wake it up and remove the Sleeping flag component from its
    /// HashMap.
    ///
    /// If a chunk was not activated and is currently awake, put it to sleep and add the Sleeping component to its
    /// HashMap.
    pub fn sleep_chunks(&mut self, mut commands: Commands) {
        self.chunks.iter_mut().for_each(|chunk| {
            if chunk.activated == true && chunk.sleeping == true {
                chunk.iter().for_each(|(_, entity)| {
                    commands.entity(*entity).remove::<Sleeping>();
                });
                chunk.sleeping = false;
            } else if chunk.activated == false && chunk.sleeping == false {
                chunk.sleeping = true;
                chunk.iter().for_each(|(_, entity)| {
                    commands.entity(*entity).insert(Sleeping);
                });
            }
        });
    }

    /// Puts all chunks in an inactive state prior to the start of the next frame.
    pub fn deactivate_all_chunks(&mut self) {
        self.chunks
            .iter_mut()
            .for_each(|chunk| chunk.activated = false);
    }

    /// Inserts a new particle at a given coordinate if it is not already occupied
    pub fn insert_no_overwrite(&mut self, coords: IVec2, entity: Entity) -> &mut Entity {
        self.get_chunk_mut(&coords)
            .unwrap()
            .insert_no_overwrite(coords, entity)
    }

    /// Inserts a new particle at a given coordinate irrespective of its occupied state
    pub fn insert_overwrite(&mut self, coords: IVec2, entity: Entity) -> Option<Entity> {
        self.get_chunk_mut(&coords)
            .unwrap()
            .insert_overwrite(coords, entity)
    }

    /// Get an immutable reference to the corresponding entity, if it exists.
    pub fn get(&self, coords: &IVec2) -> Option<&Entity> {
        self.get_chunk(&coords).unwrap().get(coords)
    }

    /// Get an immutable reference to the corresponding entity, if it exists.
    pub fn get_mut(&mut self, coords: &IVec2) -> Option<&mut Entity> {
        self.get_chunk_mut(&coords).unwrap().get_mut(coords)
    }

    /// Remove a particle from the map
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if particles are not
    /// simultaneously cleared within the same system from which this method was called.
    pub fn remove(&mut self, coords: &IVec2) -> Option<Entity> {
        self.get_chunk_mut(&coords).unwrap().remove(coords)
    }

    /// Iterate through all key, value instances of the particle map
    #[allow(unused)]
    pub fn iter(&self) -> impl Iterator<Item = (&IVec2, &Entity)> {
        self.chunks.iter().flat_map(|chunk| chunk.iter())
    }

    /// Parallel iter through all the key, value instances of the particle map
    pub fn par_iter(&self) -> impl IntoParallelIterator<Item = (&IVec2, &Entity)> {
        self.chunks.par_iter().flat_map(|chunk| chunk.par_iter())
    }
}

#[derive(Debug, Clone)]
pub struct ChunkMap {
    /// The chunk containing the particle data
    chunk: HashMap<IVec2, Entity>,
    /// Flag indicating the chunk was active at some point during the frame
    pub activated: bool,
    /// Flag indicating the chunk is sleeping
    pub sleeping: bool,
}

impl Default for ChunkMap {
    fn default() -> ChunkMap {
        ChunkMap {
            chunk: HashMap::default(),
            activated: true,
            sleeping: false,
        }
    }
}

impl ChunkMap {
    /// Clear all existing particles from the map
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if particles are not
    /// simultaneously cleared within the same system from which this method was called.
    pub fn clear(&mut self) {
        self.chunk.clear();
    }

    /// Inserts a new particle at a given coordinate if it is not already occupied
    pub fn insert_no_overwrite(&mut self, coords: IVec2, entity: Entity) -> &mut Entity {
        self.activated = true;
        self.chunk.entry(coords).or_insert(entity)
    }

    /// Inserts a new particle at a given coordinate irrespective of its occupied state
    pub fn insert_overwrite(&mut self, coords: IVec2, entity: Entity) -> Option<Entity> {
        self.activated = true;
        self.chunk.insert(coords, entity)
    }

    /// Get an immutable reference to the corresponding entity, if it exists.
    pub fn get(&self, coords: &IVec2) -> Option<&Entity> {
        self.chunk.get(coords)
    }

    /// Get an immutable reference to the corresponding entity, if it exists.
    pub fn get_mut(&mut self, coords: &IVec2) -> Option<&mut Entity> {
        self.activated = true;
        self.chunk.get_mut(coords)
    }

    /// Remove a particle from the map
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if particles are not
    /// simultaneously cleared within the same system from which this method was called.
    pub fn remove(&mut self, coords: &IVec2) -> Option<Entity> {
        self.activated = true;
        self.chunk.remove(coords)
    }

    /// Iterate through all key, value instances of the particle map
    #[allow(unused)]
    pub fn iter(&self) -> impl Iterator<Item = (&IVec2, &Entity)> {
        self.chunk.iter()
    }

    /// Parallel iter through all the key, value instances of the particle map
    pub fn par_iter(&self) -> impl IntoParallelIterator<Item = (&IVec2, &Entity)> {
        self.chunk.par_iter()
    }
}
