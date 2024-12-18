//! This crate adds bevy_core and other extensions used by the `bevy_falling_sand` crate.
//!
//! Additionally, any logic utilizing types or systems from the core or extended plugins should be
//! defined here.

#![forbid(missing_docs)]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]

//! This crate sources bevy_falling_sand crates.
pub mod bundles;
pub mod particle_management;

use bevy::prelude::{App, Plugin, Update};
use bevy_turborand::prelude::*;

pub use bfs_asset_loaders as asset_loaders;
pub use bfs_color as color;
pub use bfs_core as core;
pub use bfs_debug as debug;
pub use bfs_movement as movement;
pub use bfs_reactions as reactions;
pub use bfs_scenes as scenes;
pub use bfs_spatial as spatial;

pub use bundles::*;
pub use particle_management::*;

/// Main plugin for Bevy Falling Sand.
pub struct FallingSandPlugin;

impl Plugin for FallingSandPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RngPlugin::default(),
            ParticleManagementPlugin,
            core::FallingSandCorePlugin,
            movement::FallingSandMovementPlugin,
            color::FallingSandColorPlugin,
            debug::FallingSandDebugPlugin,
            spatial::FallingSandSpatialPlugin,
            reactions::FallingSandReactionsPlugin,
            asset_loaders::FallingSandAssetLoadersPlugin,
            scenes::FallingSandScenesPlugin,
        ));
        app.add_systems(Update, handle_new_particles);
    }
}
