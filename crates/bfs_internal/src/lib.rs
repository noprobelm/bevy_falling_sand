#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]

pub mod bundles;

use bevy::prelude::{App, Plugin};
use bevy_turborand::prelude::*;

pub use bfs_color as color;
pub use bfs_core as core;
pub use bfs_debug as debug;
pub use bfs_movement as movement;
pub use bfs_reactions as reactions;
pub use bfs_scenes as scenes;
pub use bfs_spatial as spatial;

pub use bundles::*;

pub struct FallingSandPlugin;

impl Plugin for FallingSandPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RngPlugin::default(),
            core::FallingSandCorePlugin,
            movement::FallingSandMovementPlugin,
            color::FallingSandColorPlugin,
            debug::FallingSandDebugPlugin,
            spatial::FallingSandSpatialPlugin,
            reactions::FallingSandReactionsPlugin,
            scenes::FallingSandScenesPlugin,
        ));
    }
}
