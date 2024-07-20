//! All components related to particle behavior are found in these modules.
pub use particle_types::*;
pub use particle::*;
pub use rng::*;
pub use color::*;
pub use hibernation::*;
pub use movement_priority::*;

mod particle_types;
mod particle;
mod rng;
mod color;
mod hibernation;
mod movement_priority;
