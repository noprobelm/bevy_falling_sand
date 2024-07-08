use crate::ParticleType;
use ahash::HashMap;
use bevy::prelude::*;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use std::collections::hash_map::Entry;

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
    /// The size of each chunk
    chunk_size: u32,
    /// The size of the grid
    grid_size: usize,
    /// The particle maps
    chunks: Vec<HashMap<IVec2, Entity>>,
}

impl Default for ParticleMap {
    fn default() -> ParticleMap {
	let chunk_size: u32 = 64;
	let chunks_len: u32 = 32;
        ParticleMap {
	    chunk_size,
	    grid_size: (chunk_size * chunks_len) as usize,
            chunks: (0..chunks_len.pow(2)).map(|_| HashMap::default()).collect(),
        }
    }
}

impl ParticleMap {
    /// Gets the index of the appropriate chunk when given an &IVec2
    fn get_chunk_index(&self, coord: &IVec2) -> usize {
        let col = ((coord.x + self.grid_size as i32) / self.chunk_size as i32) as usize;
        let row = ((self.grid_size as i32 - coord.y) / self.chunk_size as i32) as usize;
	row * 16 + col
    }

    /// Gets an immutable reference to a chunk
    fn get_chunk(&self, coord: &IVec2) -> Option<&HashMap<IVec2, Entity>> {
        let index = self.get_chunk_index(coord);
        self.chunks.get(index)
    }

    /// Gets a mutable reference to a chunk
    fn get_chunk_mut(&mut self, coord: &IVec2) -> Option<&mut HashMap<IVec2, Entity>> {
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

    /// Inserts a new particle at a given coordinate if it is not already occupied
    pub fn insert_no_overwrite(&mut self, coords: IVec2, entity: Entity) -> &mut Entity {
        self.get_chunk_mut(&coords).unwrap().entry(coords).or_insert(entity)
    }

    /// Inserts a new particle at a given coordinate irrespective of its occupied state
    
    pub fn insert_overwrite(&mut self, coords: IVec2, entity: Entity) -> Option<Entity> {
        self.get_chunk_mut(&coords).unwrap().insert(coords, entity)
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
