//! This crate sources all peripheral plugins for *Bevy Falling Sand* and provides some convenient
//! plugins and commonly used particle bundles.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links,
    missing_docs
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]

/// Provides bundles for commonly used particle types.
pub mod bundles;
/// Prelude for commonly accessed constructs
pub mod prelude;

use std::time::Duration;

use bevy::prelude::{App, Plugin, Vec2};
use bevy_turborand::prelude::*;

pub use bfs_assets as assets;
pub use bfs_color as color;
pub use bfs_core as core;
pub use bfs_debug as debug;
pub use bfs_movement as movement;
pub use bfs_physics as physics;
pub use bfs_reactions as reactions;
pub use bfs_scenes as scenes;
pub use bfs_spatial as spatial;

pub use bundles::*;

/// Plugin which includes all main *Bevy Falling Sand* sub-plugins.
pub struct FallingSandPlugin {
    /// The length unit to use for [avian2d]
    /// [avian2d](https://docs.rs/avian2d/latest/avian2d/)
    pub length_unit: f32,
    /// The spatial refresh frequency to use for [bevy_spatial](https://docs.rs/bevy_spatial/latest/bevy_spatial/)
    pub spatial_refresh_frequency: Duration,
    /// The value for [`GravityScale`](https://docs.rs/avian2d/latest/avian2d/dynamics/rigid_body/struct.GravityScale.html)
    /// in the avian2d crate.
    pub rigid_body_gravity: Vec2,
}

impl Default for FallingSandPlugin {
    fn default() -> Self {
        Self {
            length_unit: 8.0,
            spatial_refresh_frequency: Duration::from_millis(50),
            rigid_body_gravity: Vec2::NEG_Y * 50.0,
        }
    }
}

impl FallingSandPlugin {
    /// Change the units-per-meter scaling factor for avian2d, which influences some of the engine's
    /// internal properties with respect to the scale of the world.
    #[must_use]
    pub const fn with_length_unit(self, length_unit: f32) -> Self {
        Self {
            length_unit,
            ..self
        }
    }

    #[must_use]
    /// Change the update rate for particle spatial queries.
    ///
    /// Expects a [Duration] which is the delay between kdtree updates.
    pub const fn with_spatial_refresh_frequency(self, spatial_refresh_frequency: Duration) -> Self {
        Self {
            spatial_refresh_frequency,
            ..self
        }
    }

    #[must_use]
    /// Change the gravity for 2d rigid bodies.
    pub const fn with_gravity(self, rigid_body_gravity: Vec2) -> Self {
        Self {
            rigid_body_gravity,
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
                rigid_body_gravity: self.rigid_body_gravity,
            },
            assets::FallingSandAssetsPlugin,
        ));
    }
}

/// A minimal plugin for *Bevy Falling Sand*, which only adds the crate's core features.
///
/// This plugin is useful for users who want to selectively import the additional plugins provided
/// by the *Bevy Falling Sand* subcrates.
pub struct FallingSandMinimalPlugin;

impl Plugin for FallingSandMinimalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RngPlugin::default(), core::FallingSandCorePlugin));
    }
}
