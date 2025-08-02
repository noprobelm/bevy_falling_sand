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
pub struct FallingSandCorePlugin;

impl Plugin for FallingSandCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ParticleRegistrationPlugin,
            ParticleSimulationPlugin,
            ParticleSpatialPlugin,
        ));
    }
}
