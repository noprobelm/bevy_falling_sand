use bevy::prelude::*;

mod movement;
mod physics_components;
pub mod material;
mod events;

pub use material::*;
pub use movement::*;
pub use physics_components::*;
pub use events::*;

pub struct FallingSandMovementPlugin;

impl Plugin for FallingSandMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_movement)
            .register_type::<Density>()
            .register_type::<Velocity>()
            .register_type::<Momentum>();
    }
}
