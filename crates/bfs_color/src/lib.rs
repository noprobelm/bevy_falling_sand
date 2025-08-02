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
//! Provides basic rendering functionality for the Falling Sand simulation.
mod particle_definitions;
mod systems;

use bevy::prelude::*;

pub use particle_definitions::*;
use systems::SystemsPlugin;

/// Provides the constructs and systems necessary for rendering particles in the Falling Sand
/// simulation.
pub struct FallingSandColorPlugin;

impl Plugin for FallingSandColorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ParticleDefinitionsPlugin, SystemsPlugin));
    }
}
