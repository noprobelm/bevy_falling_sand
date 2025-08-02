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
//! Provides reaction functionality for particles in the Falling Sand simulation.
mod particle_definitions;
mod systems;

use bevy::prelude::*;

pub use particle_definitions::*;
use systems::SystemsPlugin;

/// Provides the constructs and systems necessary for particle reactions in the Falling Sand
/// Simulation.
pub struct FallingSandReactionsPlugin;

impl Plugin for FallingSandReactionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ParticleDefinitionsPlugin, SystemsPlugin));
    }
}
