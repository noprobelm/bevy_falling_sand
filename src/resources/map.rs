use crate::ParticleType;
use ahash::HashMap;
use std::collections::hash_map::Entry;
use bevy::prelude::*;
use rayon::prelude::*;
use rayon::iter::IntoParallelRefIterator;

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

/// The mapping resource for the position of all particles in the world. This is used primarily when
/// we need to move particles for each tick of the simulation.
#[derive(Resource, Default, Debug, Clone)]
pub struct ParticleMap {
    /// The mapping resource for all particles
    map: HashMap<IVec2, Entity>,
}

impl ParticleMap {
    /// Clear all existing particles from the map
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if particles are not
    /// simultaneously cleared within the same system from which this method was called.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Gets the given coordinate's corresponding entry in the map for in-place manipulation.
    pub fn entry(&mut self, coords: IVec2) -> Entry<'_, IVec2, Entity> {
        self.map.entry(coords)
    }
    /// Inserts a new particle at a given coordinate if it is not already occupied
    pub fn insert_no_overwrite(&mut self, coords: IVec2, entity: Entity) -> &mut Entity {
        self.map.entry(coords).or_insert(entity)
    }

    /// Inserts a new particle at a given coordinate irrespective of its occupied state
    #[inline(always)]
    pub fn insert_overwrite(&mut self, coords: IVec2, entity: Entity) -> Option<Entity> {
        self.map.insert(coords, entity)
    }

    /// Get an immutable reference to the corresponding entity, if it exists.
    #[inline(always)]
    pub fn get(&self, coords: &IVec2) -> Option<&Entity> {
        self.map.get(coords)
    }

    /// Get an immutable reference to the corresponding entity, if it exists.
    #[inline(always)]
    pub fn get_mut(&mut self, coords: &IVec2) -> Option<&mut Entity> {
        self.map.get_mut(coords)
    }

    /// Remove a particle from the map
    /// > **⚠️ Warning:**
    /// > Calling this method will cause major breakage to the simulation if particles are not
    /// simultaneously cleared within the same system from which this method was called.
    #[inline(always)]
    pub fn remove(&mut self, coords: &IVec2) -> Option<Entity> {
        self.map.remove(coords)
    }

    /// Iterate through all key, value instances of the particle map
    #[inline(always)]
    #[allow(unused)]
    pub fn iter(&self) -> impl Iterator<Item = (&IVec2, &Entity)> {
        self.map.iter()
    }

    /// Parallel iter through all the key, value instances of the particle map
    #[inline(always)]
    pub fn par_iter(&self) -> impl IntoParallelIterator<Item = (&IVec2, &Entity)> {
        self.map.par_iter()
    }

    /// Get the total numebr of particles
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.map.len()
    }
}
