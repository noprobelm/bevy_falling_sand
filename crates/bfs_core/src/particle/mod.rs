use bevy::prelude::*;

mod particle_definitions;
mod systems;

pub use particle_definitions::*;
use systems::*;

/// Plugin for basic particle components and events, including the minimal components necessary for adding a particle
/// to the simulation.
pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ParticleSystemsPlugin)
            .add_event::<MutateParticleEvent>()
            .register_type::<Coordinates>()
            .register_type::<Particle>()
            .add_event::<ResetParticleEvent>()
            .observe(on_reset_particle);
    }
}
