mod particle_definitions;
mod systems;

use bevy::prelude::*;

pub use particle_definitions::*;
pub use systems::*;

pub struct FallingSandReactionsPlugin;

impl Plugin for FallingSandReactionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ParticleDefinitionsPlugin, SystemsPlugin));
    }
}
