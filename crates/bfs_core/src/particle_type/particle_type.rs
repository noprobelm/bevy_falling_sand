use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Marker component for entities that act as a central reference for particle type information.
#[derive(
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Default,
    Component,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct ParticleType {
    /// The particle type's unique name.
    pub name: String,
}

impl ParticleType {
    /// Creates a new ParticleType
    pub fn new(name: &str) -> ParticleType {
        ParticleType {
            name: name.to_string(),
        }
    }
}

/// Maps particle type names to a corresponding entity
#[derive(Resource, Clone, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct ParticleTypeMap {
    /// The mapping resource for particle types.
    map: HashMap<String, Entity>,
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
