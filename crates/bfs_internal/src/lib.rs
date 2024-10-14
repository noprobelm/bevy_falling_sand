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
pub mod handle_new_particles;

use bevy::prelude::{App, Plugin, Update};

pub use bfs_color as color;
pub use bfs_core as core;
pub use bfs_debug as debug;
pub use bfs_movement as movement;
pub use bfs_reactions as reactions;
pub use bfs_spatial as spatial;
pub use bfs_asset_loaders as asset_loaders;
pub use bfs_scenes as scenes;

pub use bundles::*;
pub use handle_new_particles::*;

/// Main plugin for Bevy Falling Sand
pub struct FallingSandPlugin;

impl Plugin for FallingSandPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            core::FallingSandCorePlugin,
            movement::FallingSandMovementPlugin,
            color::FallingSandColorPlugin,
            debug::FallingSandDebugPlugin,
            spatial::FallingSandSpatialPlugin,
            reactions::FallingSandReactionsPlugin,
            asset_loaders::FallingSandAssetLoadersPlugin,
	    scenes::FallingSandScenesPlugin
        ));

	app.add_systems(Update, handle_new_particles);
    }
}
