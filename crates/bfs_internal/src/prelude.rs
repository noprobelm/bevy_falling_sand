//! Commonly used imports for *Bevy Falling Sand*.

#[cfg(feature = "bundles")]
pub use crate::bundles::*;
#[cfg(feature = "assets")]
pub use crate::file_utils::*;
#[cfg(feature = "assets")]
pub use crate::particle_registry::*;

pub use crate::core::*;

#[cfg(feature = "color")]
pub use crate::color::*;

#[cfg(feature = "movement")]
pub use crate::movement::*;

#[cfg(feature = "debug")]
pub use crate::debug::*;

#[cfg(feature = "reactions")]
pub use crate::reactions::*;

#[cfg(feature = "scenes")]
pub use crate::scenes::*;

#[cfg(feature = "physics")]
pub use crate::physics::*;

#[cfg(feature = "assets")]
pub use crate::assets::*;

pub use super::{FallingSandMinimalPlugin, FallingSandPlugin};
