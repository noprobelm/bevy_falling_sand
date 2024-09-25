//! Components directly related to particle movement
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Coordinates component for particles.
#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct Coordinates(pub IVec2);

mod movement_priority;
mod physics_components;
mod rng;
mod behavior;

pub use movement_priority::*;
pub use physics_components::*;
pub use rng::*;
pub use behavior::*;
