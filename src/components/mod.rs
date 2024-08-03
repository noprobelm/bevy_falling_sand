//! All components related to particle behavior are found in these modules.
pub use particle::*;
pub use rng::*;
pub use color::*;
pub use hibernation::*;
pub use movement::*;
pub use material::*;

mod particle;
mod rng;
mod color;
mod hibernation;
mod movement;
mod material;
