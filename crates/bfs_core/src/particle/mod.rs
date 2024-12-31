use bevy::prelude::*;

mod particle_definitions;
mod systems;

pub use particle_definitions::*;
use systems::*;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ParticleSystemsPlugin, ParticleDefinitionsPlugin));
    }
}
