use bevy::prelude::*;
use bfs_core::{ParticleSimulationSet, reset_chunks};

mod events;
pub mod material;
mod movement;
mod physics_components;

pub use events::*;
pub use material::*;
pub use movement::*;
pub use physics_components::*;

pub struct FallingSandMovementPlugin;

impl Plugin for FallingSandMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_movement
                .in_set(ParticleSimulationSet)
                .before(reset_chunks),
        )
        .register_type::<Density>()
        .register_type::<Velocity>()
        .register_type::<Momentum>();
    }
}
