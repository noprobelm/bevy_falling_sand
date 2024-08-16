//! All systems driving particle behavior are found in these modules.

use bevy::prelude::SystemSet;

pub use particle::*;
pub use map::*;
pub use movement::*;
pub use hibernation::*;
pub use color::*;
pub use debug::*;

mod particle;
mod map;
mod movement;
mod hibernation;
mod color;
mod debug;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleSimulationSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleDebugSet;
