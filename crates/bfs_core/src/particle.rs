//! Minimum components a particle is comprised of.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{events::*, ParticleSimulationSet};

/// Plugin for basic particle components and events, including the minimal components necessary for adding a particle
/// to the simulation.
pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                ev_change_particle.in_set(ParticleSimulationSet),
            ),
        )
        .register_type::<Coordinates>()
        .register_type::<Particle>()
        .observe(on_reset_particle);
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

/// Event reader for particle type updates
pub fn ev_change_particle(
    mut ev_change_particle: EventReader<MutateParticleEvent>,
    mut particle_query: Query<&mut Particle>,
) {
    for ev in ev_change_particle.read() {
        let mut particle = particle_query.get_mut(ev.entity).unwrap();
        particle.name = ev.particle.name.clone();
    }
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
