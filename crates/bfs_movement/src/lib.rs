use bevy::prelude::*;

pub mod material;
mod particle_definitions;
mod systems;

pub use material::*;
pub use particle_definitions::*;
pub use systems::*;

pub struct FallingSandMovementPlugin;

impl Plugin for FallingSandMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ParticleDefinitionsPlugin, MaterialPlugin, SystemsPlugin));
    }
}
