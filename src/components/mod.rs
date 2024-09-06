//! All components related to particle behavior are found in these modules. The `material` module offers template 
//! structs for defining realistic movement behavior.
mod particle;
mod hibernation;
mod movement;
mod rng;
mod color;
mod burning;
mod reaction;
mod scenes;
pub mod material;

pub use particle::*;
pub use hibernation::*;
pub use movement::*;
pub use rng::*;
pub use color::*;
pub use burning::*;
pub use reaction::*;
pub use scenes::*;
pub use material::*;
