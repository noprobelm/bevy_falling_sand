#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]

pub mod bundles;
pub mod prelude;

use std::time::Duration;

use bevy::prelude::{App, Plugin};
use bevy_turborand::prelude::*;

pub use bfs_color as color;
pub use bfs_core as core;
pub use bfs_debug as debug;
pub use bfs_movement as movement;
pub use bfs_physics as physics;
pub use bfs_reactions as reactions;
pub use bfs_scenes as scenes;
pub use bfs_spatial as spatial;

pub use bundles::*;

pub struct FallingSandPlugin {
    pub length_unit: f32,
    pub spatial_refresh_frequency: Duration,
}

impl Default for FallingSandPlugin {
    fn default() -> Self {
        Self {
            length_unit: 8.0,
            spatial_refresh_frequency: Duration::from_millis(50),
        }
    }
}

impl FallingSandPlugin {
    #[must_use]
    pub const fn with_length_unit(self, length_unit: f32) -> Self {
        Self {
            length_unit,
            ..self
        }
    }

    #[must_use]
    pub const fn with_spatial_refresh_frequency(self, spatial_refresh_frequency: Duration) -> Self {
        Self {
            spatial_refresh_frequency,
            ..self
        }
    }
}

impl Plugin for FallingSandPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RngPlugin::default(),
            core::FallingSandCorePlugin,
            movement::FallingSandMovementPlugin,
            color::FallingSandColorPlugin,
            spatial::FallingSandSpatialPlugin {
                frequency: self.spatial_refresh_frequency,
            },
            reactions::FallingSandReactionsPlugin,
            scenes::FallingSandScenesPlugin,
            physics::FallingSandPhysicsPlugin {
                length_unit: self.length_unit,
            },
        ));
    }
}

pub struct FallingSandMinimalPlugin;

impl Plugin for FallingSandMinimalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RngPlugin::default(), core::FallingSandCorePlugin));
    }
}
