//! Defines additional components for particle types to be used as blueprint data when spawning or
//! resetting particles.
//!
//! This module is a standard template that can be followed when extending particle types. Its
//! structure is as follows:
//!   - Defines new components which will be associated with particle types as blueprint information
//!     for child particles.
//!   - Adds events for each new component which manage resetting information for child particles
//!   - Adds observers for each event to specify granular logic through which a particle should have
//!     its information reset. This usually involves referencing the parent `ParticleType`.
//!
//! When a particle should have its information reset (e.g., when spawning or resetting), we can
//! trigger the events defined in this module and communicate with higher level systems that
//! something needs to happen with a given particle.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Plugin for basic particle definitions.
pub struct ParticleDefinitionsPlugin;

impl Plugin for ParticleDefinitionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MutateParticleEvent>()
            .register_type::<Coordinates>()
            .register_type::<Particle>()
            .add_event::<ResetParticleEvent>()
            .add_event::<RemoveParticleEvent>()
            .add_observer(on_reset_particle);
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

/// Blueprint for the particle data
#[derive(Component, Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ParticleBlueprint(pub Particle);

/// Coordinate component for particles.
#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct Coordinates(pub IVec2);

/// Changes a particle to the designated type
#[derive(Event)]
pub struct MutateParticleEvent {
    /// The entity to change the particle type of
    pub entity: Entity,
    /// The new particle type
    pub particle: Particle,
}

/// Triggers the removal of a particle from the simulation.
#[derive(Event)]
pub struct RemoveParticleEvent {
    /// The coordinates of the particle to remove.
    pub coordinates: IVec2,
    /// Whether the corresponding entity should be despawned from the world.
    pub despawn: bool,
}

/// Resets all of a particle's components to its parent's.
#[derive(Event)]
pub struct ResetParticleEvent {
    /// The entity to reset data for.
    pub entity: Entity,
}

/// Observer for resetting all of a particle's data. This system simply marks the Particle as changed so it gets picked
/// up by `handle_new_particles` the next frame.
pub fn on_reset_particle(
    trigger: Trigger<ResetParticleEvent>,
    mut particle_query: Query<&mut Particle>,
) {
    particle_query
        .get_mut(trigger.event().entity)
        .unwrap()
        .into_inner();
}
