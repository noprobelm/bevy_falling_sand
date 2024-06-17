use crate::ParticleType;
use ahash::HashMap;
use bevy::prelude::*;

/// A map of all parent particle types to their corresponding entity. This is used exclusively for
/// assigning child particles to their respective parent when initially spawned or have otherwise
/// changed types (e.g., a reaction has occurred)
#[derive(Resource, Clone, Default, Debug)]
pub struct ParentParticleMap {
    /// The mapping resource for particle types.
    map: HashMap<ParticleType, Entity>,
}

impl ParticleTypeMap {
    /// Insert a new particle type to the map
    pub fn insert(&mut self, ptype: ParticleType, entity: Entity) -> &mut Entity {
        self.map.entry(ptype).or_insert(entity)
    }

    /// Get an immutable reference to the corresponding entity, if it exists.
    pub fn get(&self, ptype: &ParticleType) -> Option<&Entity> {
        self.map.get(ptype)
    }
}
