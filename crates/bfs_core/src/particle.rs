//! Minimum components a particle is comprised of.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Plugin for basic particle components and events, including the minimal components necessary for adding a particle
/// to the simulation.
pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MutateParticleEvent>()
            .add_event::<ResetParticleEvent>()
            .add_event::<RemoveParticleEvent>()
            .register_type::<Coordinates>()
            .register_type::<ParticleType>()
            .register_type::<Particle>();
    }
}

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

/// Holds the particle type's name. Used to map to particle type data.
#[derive(Component, Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Particle {
    /// The particle's unique name.
    pub name: String,
}

impl Particle {
    /// Creates a new Particle
    pub fn new(name: &str) -> Particle {
        Particle {
            name: name.to_string(),
        }
    }
}

/// Coordinate component for particles.
#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct Coordinates(pub IVec2);

/// Maps particle type names to a corresponding entity
#[derive(Resource, Clone, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct ParticleTypeMap {
    /// The mapping resource for particle types.
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

/// Changes a particle to the designated type
#[derive(Event)]
pub struct MutateParticleEvent {
    /// The entity to change the particle type of
    pub entity: Entity,
    /// The new particle type
    pub particle: Particle,
}

/// Resets all of a particle's components to its parent's.
#[derive(Event)]
pub struct ResetParticleEvent {
    /// The entity to reset data for.
    pub entity: Entity,
}

/// Triggers the removal of a particle from the simulation.
#[derive(Event)]
pub struct RemoveParticleEvent {
    /// The coordinates of the particle to remove.
    pub coordinates: IVec2,
    /// Whether the corresponding entity should be despawned from the world.
    pub despawn: bool
}

/// Map all particles to their respective parent when added/changed within the simulation
pub fn handle_new_particle_types(
    mut commands: Commands,
    particle_type_query: Query<(Entity, &ParticleType), Changed<ParticleType>>,
    mut type_map: ResMut<ParticleTypeMap>,
) {
    particle_type_query
        .iter()
        .for_each(|(entity, particle_type)| {
            commands
                .entity(entity)
                .insert(SpatialBundle::from_transform(Transform::from_xyz(
                    0., 0., 0.,
                )));
            type_map.insert(particle_type.name.clone(), entity);
        });
}
