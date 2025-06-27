mod particle_definitions;
mod systems;

use bevy::prelude::*;

pub use particle_definitions::*;
use systems::*;

pub struct FallingSandColorPlugin;

impl Plugin for FallingSandColorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ParticleDefinitionsPlugin, SystemsPlugin));
    }
}
