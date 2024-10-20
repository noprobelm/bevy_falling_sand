use bevy::prelude::*;

mod events;
pub mod material;
mod movement;

pub use events::*;
pub use material::*;
pub use movement::*;

pub struct FallingSandMovementPlugin;

impl Plugin for FallingSandMovementPlugin {
    fn build(&self, app: &mut App) {
	app.add_plugins(MovementPlugin);
    }
}
