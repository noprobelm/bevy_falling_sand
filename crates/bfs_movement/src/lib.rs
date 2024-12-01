use bevy::prelude::*;

pub mod material;
mod events;
mod rng;
mod particle_definitions;
mod systems;

pub use material::*;
pub use events::*;
pub use rng::*;
pub use particle_definitions::*;
pub use systems::*;

pub struct FallingSandMovementPlugin;

impl Plugin for FallingSandMovementPlugin {
    fn build(&self, app: &mut App) {
	app.add_plugins((ParticleDefinitionsPlugin, MaterialPlugin, SystemsPlugin, EventsPlugin));
    }
}
