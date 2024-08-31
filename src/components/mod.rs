//! All components related to particle behavior are found in these modules.
mod particle;
mod rng;
mod color;
mod hibernation;
mod movement;
pub mod material;
mod burning;
mod reaction;
mod scenes;

pub use particle::*;
pub use rng::*;
pub use color::*;
pub use hibernation::*;
pub use movement::*;
pub use material::*;
pub use burning::*;
pub use reaction::*;
pub use scenes::*;
