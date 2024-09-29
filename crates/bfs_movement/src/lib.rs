use bevy::prelude::*;

mod movement;
mod movement_priority;
mod physics_components;

pub use movement::*;
pub use movement_priority::*;
pub use physics_components::*;

pub struct ParticleMovementPlugin;

impl Plugin for ParticleMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_movement)
            .register_type::<Density>()
            .register_type::<Velocity>()
            .register_type::<Momentum>();
    }
}
