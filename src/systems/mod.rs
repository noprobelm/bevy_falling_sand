use bevy::prelude::SystemSet;

pub use map::*;
pub use movement::*;
pub use hibernation::*;
pub use color::*;
pub use debug::*;

pub mod map;
pub mod movement;
pub mod hibernation;
pub mod color;
pub mod debug;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleMovementSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleDebugSet;
