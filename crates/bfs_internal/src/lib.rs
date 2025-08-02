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
/// File I/O utilities for particle data
pub mod file_utils;
/// Central particle registry for serialization
pub mod particle_registry;
/// Prelude for commonly accessed constructs
pub mod prelude;

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

pub use bundles::*;

const DEFAULT_LENGTH_UNIT: f32 = 8.0;
const DEFAULT_RIGID_BODY_GRAVITY_SCALE: Vec2 = Vec2::new(0.0, -50.0);
const DEFAULT_MAP_SIZE: usize = 32;
const DEFAULT_CHUNK_SIZE: usize = 32;

/// Plugin which includes all main *Bevy Falling Sand* sub-plugins.
pub struct FallingSandPlugin {
    /// The length unit to use for [avian2d]
    /// [avian2d](https://docs.rs/avian2d/latest/avian2d/)
    pub length_unit: f32,
    /// The value for [`GravityScale`](https://docs.rs/avian2d/latest/avian2d/dynamics/rigid_body/struct.GravityScale.html)
    /// in the avian2d crate.
    pub rigid_body_gravity_scale: Vec2,
    /// The map size for the ParticleMap resource.
    pub map_size: usize,
    /// The chunk size for the ParticleMap resource.
    pub chunk_size: usize,
}

impl Default for FallingSandPlugin {
    fn default() -> Self {
        Self {
            length_unit: DEFAULT_LENGTH_UNIT,
            rigid_body_gravity_scale: DEFAULT_RIGID_BODY_GRAVITY_SCALE,
            map_size: DEFAULT_MAP_SIZE,
            chunk_size: DEFAULT_CHUNK_SIZE,
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
    /// Change the gravity for 2d rigid bodies.
    pub const fn with_gravity(self, rigid_body_gravity: Vec2) -> Self {
        Self {
            rigid_body_gravity_scale: rigid_body_gravity,
            ..self
        }
    }

    #[must_use]
    /// Change the map size for the ParticleMap resource.
    pub const fn with_map_size(self, map_size: usize) -> Self {
        Self { map_size, ..self }
    }

    #[must_use]
    /// Change the chunk size for the ParticleMap resource.
    pub const fn with_chunk_size(self, chunk_size: usize) -> Self {
        Self { chunk_size, ..self }
    }
}

impl Plugin for FallingSandPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RngPlugin::default(),
            core::FallingSandCorePlugin {
                map_size: self.map_size,
                chunk_size: self.chunk_size,
            },
            movement::FallingSandMovementPlugin,
            color::FallingSandColorPlugin,
            reactions::FallingSandReactionsPlugin,
            scenes::FallingSandScenesPlugin,
            physics::FallingSandPhysicsPlugin {
                length_unit: self.length_unit,
                rigid_body_gravity: self.rigid_body_gravity_scale,
            },
            assets::FallingSandAssetsPlugin,
        ));
    }
}

/// A minimal plugin for *Bevy Falling Sand*, which only adds the crate's core features.
///
/// This plugin is useful for users who want to selectively import the additional plugins provided
/// by the *Bevy Falling Sand* subcrates.
pub struct FallingSandMinimalPlugin {
    /// The map size for the ParticleMap resource.
    pub map_size: usize,
    /// The chunk size for the ParticleMap resource.
    pub chunk_size: usize,
}

impl Default for FallingSandMinimalPlugin {
    fn default() -> Self {
        Self {
            map_size: 32,
            chunk_size: 32,
        }
    }
}

impl Plugin for FallingSandMinimalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RngPlugin::default(), core::FallingSandCorePlugin::default()));
    }
}
