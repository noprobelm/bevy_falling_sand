#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links,
    missing_docs
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]
//! This crate provides all of the core constructs required for a falling sand simulation. All
//! extensions to *Bevy Falling Sand* require the constructs provided in this crate.
mod map;
mod particle;
mod rng;

use bevy::prelude::*;

pub use map::*;
pub use particle::*;
pub use rng::*;

/// The core plugin, which manages particle definitions and map setup.
pub struct FallingSandCorePlugin;

impl Plugin for FallingSandCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ParticleCorePlugin, ParticleMapPlugin));
    }
}
