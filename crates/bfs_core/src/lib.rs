//! Provides core functionality for `bevy_falling_sand`. The types exposed in this crate are
//! typically necessary for extending the functionality of the particle simulation, such as:
//!   - Basic Particle type definitions
//!   - Particle spatial mapping data structures
//!   - System sets
//!   - Particle mutation/reset events

#![forbid(missing_docs)]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]

//! This crate provides core functionality for particles.

mod common;
mod events;
mod map;
mod particle;
mod particle_type;

use bevy::prelude::*;

pub use common::*;
pub use events::*;
pub use map::*;
pub use particle::*;
pub use particle_type::*;

/// Core plugin for Bevy Falling Sand.
pub struct FallingSandCorePlugin;

impl Plugin for FallingSandCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ParticlePlugin,
            ParticleTypePlugin,
            ChunkMapPlugin,
            CommonUtilitiesPlugin,
            EventsPlugin,
        ));
    }
}
