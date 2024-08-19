//! All systems driving particle behavior are found in these modules.

mod particle;
mod map;
mod movement;
mod hibernation;
mod color;
mod debug;
mod scenes;

use bevy::prelude::SystemSet;

pub use particle::*;
pub use map::*;
pub use movement::*;
pub use hibernation::*;
pub use color::*;
pub use debug::*;
pub use scenes::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleSimulationSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleDebugSet;
