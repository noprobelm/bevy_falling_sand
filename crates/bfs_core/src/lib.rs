#![forbid(missing_docs)]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]

//! This crate provides core functionality for particles.

mod particle;
mod map;
mod events;

pub use particle::*;
pub use map::*;
pub use events::*;
