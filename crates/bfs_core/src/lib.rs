#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links,
    missing_docs
)]
#![allow(
    clippy::default_trait_access,
    clippy::module_name_repetitions,
    clippy::inline_always,
    clippy::cast_possible_wrap
)]
//! Provides all of the core constructs required for a falling sand simulation. All
//! extensions to *Bevy Falling Sand* require the constructs provided in this crate.
mod registration;
mod simulation;
mod spatial;

use bevy::prelude::*;

pub use registration::*;
pub use simulation::*;
pub use spatial::*;

/// The core plugin, which manages particle definitions and map setup.
pub struct FallingSandCorePlugin {
    /// The map size for the [`ParticleMap`] resource.
    pub map_size: usize,
    /// The chunk size for the [`ParticleMap`] resource.
    pub chunk_size: usize,
}

impl Default for FallingSandCorePlugin {
    fn default() -> Self {
        Self {
            map_size: 32,
            chunk_size: 32,
        }
    }
}

impl Plugin for FallingSandCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ParticleRegistrationPlugin,
            ParticleSimulationPlugin,
            ParticleSpatialPlugin {
                map_size: self.map_size,
                chunk_size: self.chunk_size,
            },
        ));
    }
}
