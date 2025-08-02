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
//! Provides movement functionality for particles in the Falling Sand simulation. Custom movement
//! behavior can be implemented by omitting this plugin and writing your own constructs and systems.
use bevy::prelude::{App, Plugin};

mod material;
mod particle_definitions;
mod systems;

pub use material::*;
pub use particle_definitions::*;
pub use systems::*;

/// The movement plugin, which provides constructs and systems for particle movement.
pub struct FallingSandMovementPlugin;

impl Plugin for FallingSandMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ParticleDefinitionsPlugin, MaterialPlugin, SystemsPlugin));
    }
}
