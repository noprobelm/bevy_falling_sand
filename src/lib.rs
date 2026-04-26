#![cfg_attr(docsrs, feature(doc_cfg))]
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
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::type_complexity,
    clippy::float_cmp
)]
//! # Overview
//!
//! **Bevy Falling Sand** (`bfs`) provides a [Falling Sand](https://en.wikipedia.org/wiki/Falling-sand_game)
//! engine for [Bevy](https://bevy.org) apps.
//!
//! | `bevy_falling_sand`   | `bevy`    |
//! |-----------------------|-----------|
//! | 0.7.x                 | 0.18.x    |
//!
//! # Feature Flags
//!
//! This crate aims to modularized. Opt out of any simulation features you don't want/like
//! in favor of your own implementations.
//!
//! All features are enabled by default.
//!
//! | Feature              | Description                                                              | Implies                    |
//! | -------------------- | ------------------------------------------------------------------------ | -------------------------- |
//! | [`mod@render`]       | Particle color profiles and chunk-based rendering                        | —                          |
//! | [`mod@movement`]     | Particle movement systems                                                | —                          |
//! | [`mod@reactions`]    | Inter-particle reactions                                                 | `render`, `movement`       |
//! | [`mod@physics`]      | [avian2d](https://docs.rs/avian2d) static and dynamic rigid body integration | —                          |
//! | [`mod@debug`]        | Debug resources                                                          | —                          |
//! | [`mod@persistence`]  | Save/load chunks, particle types, and scenes to disk                     | `bfs`, `bfc`               |
//! | `bfs`                | Enables [`persistence::bfs`] — compact particle scene format             | —                          |
//! | `bfc`                | Enables [`persistence::bfc`] — scene format with per-particle color      | `render`                   |
//!
//! # Quick start
//!
//! Add the [`FallingSandPlugin`] plugin, overriding defaults as desired (see also
//! [`FallingSandMinimalPlugin`]). Common overrides:
//! - `with_chunk_size`: side length of a chunk in particles. Must be a power of 2.
//! - `with_map_size`: side length of the loaded region in chunks. Must be a power of 2.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_falling_sand::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins((
//!             DefaultPlugins,
//!             FallingSandPlugin::default()
//!                 // Create a map with 64x64 chunks, each of which can hold 64x64 particles
//!                 .with_chunk_size(64)
//!                 .with_map_size(64),
//!         ))
//!         .add_systems(Startup, setup)
//!         .add_systems(Update, sand_emitter)
//!         .run();
//! }
//!
//! // Spawn a simple particle type with colors and movement behavior resembling sand.
//! fn setup(mut commands: Commands) {
//!     commands.spawn((
//!         ParticleType::new("Sand"),
//!         ColorProfile::palette(vec![
//!             Color::Srgba(Srgba::hex("#FFEB8A").unwrap()),
//!             Color::Srgba(Srgba::hex("#F2E06B").unwrap()),
//!         ]),
//!         // First tier: look directly below. Second tier: look diagonally down.
//!         Movement::from(vec![
//!             vec![IVec2::NEG_Y],
//!             vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
//!         ]),
//!         Density(1250),
//!         Speed::new(5, 10),
//!     ));
//! }
//!
//! // Continuously emit sand between (0, 0) and (10, 10)
//! fn sand_emitter(mut writer: MessageWriter<SpawnParticleSignal>) {
//!     for x in 0..10 {
//!         for y in 0..10 {
//!             writer.write(SpawnParticleSignal::new(
//!                 Particle::new("Sand"),
//!                 IVec2::new(x, y),
//!             ));
//!         }
//!     }
//! }
//!
//! ```
//! # Particle types
//!
//! The [`ParticleType`] component acts as an interface for creating new particles. When
//! [`ParticleType`] is inserted on an entity, it becomes a point of synchronization for all
//! [`Particle`] entities of the same identifier.
//!
//! The [`ParticleType::name`] and [`Particle::name`] fields are used to associate particles with
//! their parents.
//!
//! `bfs` provides several components that add behaviors particle types. Inserting any of the
//! components from the table below on a [`ParticleType`] entity will influence its child
//! [`Particle`] entity's behavior.
//!
//! | Particle Behavior Component | Description                                                          | Feature      |
//! | --------------------------- | -------------------------------------------------------------------- | ------------ |
//! | [`ColorProfile`]            | Color profile for particles from a predefined palette or gradient    | `render`     |
//! | [`ForceColor`]              | Overrides [`ColorProfile`] assignemnts with another color.           | `render`     |
//! | [`Movement`]                | Movement rulesets for a particle                                     | `movement`   |
//! | [`Density`]                 | Density of a particle, used for displacement comparisons             | `movement`   |
//! | [`Speed`]                   | Controls how many positions a particle can move per frame            | `movement`   |
//! | [`AirResistance`]           | Chance that a particle will skip movement to a vacant location       | `movement`   |
//! | [`ParticleResistor`]        | How much a particle resists being displaced by other particles       | `movement`   |
//! | [`Momentum`]                | Directional hint that biases movement toward the last direction      | `movement`   |
//! | [`ContactReaction`]         | Defines reaction rulesets for a particle type                        | `reactions`  |
//! | [`Fire`]                    | Makes a particle spread fire                                         | `reactions`  |
//! | [`Flammable`]               | Flammability properties for particles                                | `reactions`  |
//! | [`StaticRigidBodyParticle`] | Mark particles for inclusion in rigid body mesh generation           | `physics`    |
//! | [`TimedLifetime`]           | Despawns a particle after a specified duration                       | —            |
//! | [`ChanceLifetime`]          | Chance to despawn an entity on a per-tick basis                      | —            |
//!
//! # Table of Contents
//!
//! ## [Particle lifecycles](crate::lifecycle)
//!
//! - [Spawning particles](`SpawnParticleSignal`)
//! - [Despawning particles](`DespawnParticleSignal`)
//!
//! ## [Rendering](`crate::render`)
//!
//! - [Adding color to particles](`ColorProfile`)
//! - [Overriding color assignment](`ForceColor`)
//! - [Implementing a custom shader for particle types](crate::render#custom-shaders-and-effect-layers)
//!
//! ## [Movement](`crate::movement`)
//!
//! - [Movement rulesets](`Movement`)
//! - [Density]
//! - [Speed]
//! - [Air resistance](`AirResistance`)
//! - [Particle resistance](`ParticleResistor`)
//! - [Momentum]
//! - [Selecting movement algorithms](crate::movement#movement-processing-modes)
//!
//! ## [Reactions](`crate::reactions`)
//!
//! - [Contact Reactions](ContactReaction)
//! - [Fire emitting particles](Fire)
//! - [Flammable particles](Flammable)
//!
//! ## [Avian2d integration](`crate::physics`)
//!
//! - [Dynamic rigid bodies](`crate::physics::dynamic`) — promote particles into physics-driven
//!   rigid body proxies and rejoin them back into the simulation
//! - [Static rigid bodies](`crate::physics::static`) — per-chunk collision mesh generation from
//!   marked particles
//! - [Tagging particles as static rigid bodies](`StaticRigidBodyParticle`)
//! - [Promoting particles to dynamic rigid bodies](`crate::physics::PromoteDynamicRigidBodyParticle`)
//! - [Configuring collision mesh calculation intervals](DirtyChunkUpdateInterval)
//! - [Configuring polygon simplification tolerances](`DouglasPeuckerEpsilon`)
//!
//! ## [World persistence](crate::persistence)
//!
//! - [Saving chunks to disk](PersistChunksSignal)
//! - [Saving particle types to disk](`PersistParticleTypesSignal`)
//! - [Loading particle types from disk](`LoadParticleTypesSignal`)
//!
//! ## [Scenes](crate::scenes)
//!
//! - [Scene asset format](ParticleScene)
//! - [Scene registry](ParticleSceneRegistry)
//! - [Spawning a scene](SpawnSceneSignal)
//!
//! ## Map origin shifts and dynamic chunk loading
//!
//! - [Attaching a chunk loader to an entity](`ChunkLoader`)
//! - [Chunk loading configuration](`ChunkLoadingConfig`)
//! - [Querying per-frame loading state](`ChunkLoadingState`)
//! - [System scheduling](`ChunkSystems`)
//! - [Batched despawn configuration](`DespawnBatchConfig`)
//!
//!
//! ## [Particle synchronization](crate::sync)
//!
//! - [Triggering individual particle resync](SyncParticleSignal)
//! - [Triggering particle type resync](SyncParticleTypeChildrenSignal)
//! - [Filtering which components to sync](`PropagatorFilter`)
//!
//! ### Registering custom particle components
//!
//! - [Sync components](`ParticleSyncExt::register_particle_sync_component`)
//! - [Custom propagators](`ParticleSyncExt::register_particle_propagator`)
//!
//! ## Spatial queries and raycasting
//!
//! - [Radius search](crate::core::SpatialMap::within_radius)
//! - [Rectangular search](crate::core::SpatialMap::within_rect)
//! - [Generic raycasting](crate::core::SpatialMap::raycast)
//! - [Generic line-of-sight](crate::core::SpatialMap::has_line_of_sight_by)
//! - [Generic radius with line-of-sight](crate::core::SpatialMap::within_radius_los_by)
//!
//! [`ParticleMap`] convenience methods that accept a Bevy
//! [`Query`](bevy::prelude::Query) filter to define what counts as a "hit" or "blocker":
//!
//! - [Raycasting against a query filter](`ParticleMap::raycast_query`)
//! - [Line-of-sight against a query filter](`ParticleMap::has_line_of_sight`)
//! - [Radius with line-of-sight against a query filter](`ParticleMap::within_radius_los`)
//!
//! ## [Debug stats and visuals](crate::debug)
//!
//! - [Total particle count](DebugParticleCount)
//! - [Static particle (non-movable) count](`StaticParticleCount`)
//! - [Dynamic particle (movable) count](`DynamicParticleCount`)
//! - [Active particle count](`DynamicParticleCount`)
//! - [Chunk overlay](`DebugParticleMap`)
//! - [Dirty rect overlay](`DebugDirtyRects`)
//!

pub mod prelude;

#[cfg(feature = "physics")]
use bevy::prelude::Vec2;
use bevy::prelude::{App, Plugin};
use bevy_turborand::prelude::*;
// Reduce doc link verbosity
#[allow(unused_imports)]
use prelude::*;

pub mod core;
pub use core::*;

#[cfg(feature = "debug")]
#[cfg_attr(docsrs, doc(cfg(feature = "debug")))]
pub mod debug;
#[cfg(feature = "movement")]
#[cfg_attr(docsrs, doc(cfg(feature = "movement")))]
pub mod movement;
#[cfg(feature = "persistence")]
#[cfg_attr(docsrs, doc(cfg(feature = "persistence")))]
pub mod persistence;
#[cfg(feature = "physics")]
#[cfg_attr(docsrs, doc(cfg(feature = "physics")))]
pub mod physics;
#[cfg(feature = "reactions")]
#[cfg_attr(docsrs, doc(cfg(feature = "reactions")))]
pub mod reactions;
#[cfg(feature = "render")]
#[cfg_attr(docsrs, doc(cfg(feature = "render")))]
pub mod render;
#[cfg(feature = "scenes")]
#[cfg_attr(docsrs, doc(cfg(feature = "scenes")))]
pub mod scenes;

#[cfg(feature = "physics")]
const DEFAULT_LENGTH_UNIT: f32 = 8.0;
#[cfg(feature = "physics")]
const DEFAULT_GRAVITY: Vec2 = Vec2::new(0.0, -50.0);
const DEFAULT_MAP_CHUNKS: u32 = 32;
const DEFAULT_CHUNK_SIZE: u32 = 64;

/// Plugin that registers all feature-gated *Bevy Falling Sand* sub-plugins
/// enabled at compile time.
pub struct FallingSandPlugin {
    /// The avian2d physics length unit.
    #[cfg(feature = "physics")]
    pub length_unit: f32,
    /// The gravity vector for avian2d rigid bodies.
    #[cfg(feature = "physics")]
    pub rigid_body_gravity_scale: Vec2,
    /// Width of the loaded region in chunks.
    pub width_chunks: u32,
    /// Height of the loaded region in chunks.
    pub height_chunks: u32,
    /// Size of each chunk in world units (must be power of 2).
    pub chunk_size: u32,
}

impl Default for FallingSandPlugin {
    fn default() -> Self {
        Self {
            #[cfg(feature = "physics")]
            length_unit: DEFAULT_LENGTH_UNIT,
            #[cfg(feature = "physics")]
            rigid_body_gravity_scale: DEFAULT_GRAVITY,
            width_chunks: DEFAULT_MAP_CHUNKS,
            height_chunks: DEFAULT_MAP_CHUNKS,
            chunk_size: DEFAULT_CHUNK_SIZE,
        }
    }
}

impl FallingSandPlugin {
    /// Change the units-per-meter scaling factor for avian2d, which influences some of the engine's
    /// internal properties with respect to the scale of the world.
    #[cfg(feature = "physics")]
    #[cfg_attr(docsrs, doc(cfg(feature = "physics")))]
    #[must_use]
    pub const fn with_length_unit(self, length_unit: f32) -> Self {
        Self {
            length_unit,
            ..self
        }
    }

    /// Change the gravity for 2d rigid bodies.
    #[cfg(feature = "physics")]
    #[cfg_attr(docsrs, doc(cfg(feature = "physics")))]
    #[must_use]
    pub const fn with_gravity(self, rigid_body_gravity: Vec2) -> Self {
        Self {
            rigid_body_gravity_scale: rigid_body_gravity,
            ..self
        }
    }

    /// Set the map width in chunks.
    #[must_use]
    pub const fn with_map_width(self, chunks: u32) -> Self {
        Self {
            width_chunks: chunks,
            ..self
        }
    }

    /// Set the map height in chunks.
    #[must_use]
    pub const fn with_map_height(self, chunks: u32) -> Self {
        Self {
            height_chunks: chunks,
            ..self
        }
    }

    /// Set the map width and height to the same number of chunks.
    #[must_use]
    pub const fn with_map_size(self, chunks: u32) -> Self {
        Self {
            width_chunks: chunks,
            height_chunks: chunks,
            ..self
        }
    }

    /// Change the chunk size in world units.
    #[must_use]
    pub const fn with_chunk_size(self, chunk_size: u32) -> Self {
        Self { chunk_size, ..self }
    }
}

impl Plugin for FallingSandPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RngPlugin::default(),
            core::FallingSandCorePlugin {
                width: self.width_chunks * self.chunk_size,
                height: self.height_chunks * self.chunk_size,
                chunk_size: self.chunk_size,
            },
        ));

        #[cfg(feature = "movement")]
        app.add_plugins(movement::FallingSandMovementPlugin);
        #[cfg(feature = "render")]
        app.add_plugins(render::FallingSandRenderPlugin);
        #[cfg(feature = "reactions")]
        app.add_plugins(reactions::FallingSandReactionsPlugin);
        #[cfg(feature = "physics")]
        app.add_plugins(physics::FallingSandPhysicsPlugin {
            length_unit: self.length_unit,
            rigid_body_gravity: self.rigid_body_gravity_scale,
        });
        #[cfg(feature = "scenes")]
        app.add_plugins(scenes::FallingSandScenesPlugin);
    }
}

/// A minimal plugin for *Bevy Falling Sand*, which only adds the crate's core features.
///
/// This plugin is useful for users who want to selectively import the additional plugins provided
/// by the *Bevy Falling Sand* subcrates.
pub struct FallingSandMinimalPlugin {
    /// Width of the loaded region in chunks.
    pub width_chunks: u32,
    /// Height of the loaded region in chunks.
    pub height_chunks: u32,
    /// Size of each chunk in world units (must be power of 2).
    pub chunk_size: u32,
}

impl Default for FallingSandMinimalPlugin {
    fn default() -> Self {
        Self {
            width_chunks: DEFAULT_MAP_CHUNKS,
            height_chunks: DEFAULT_MAP_CHUNKS,
            chunk_size: DEFAULT_CHUNK_SIZE,
        }
    }
}

impl FallingSandMinimalPlugin {
    /// Set the map width in chunks.
    #[must_use]
    pub const fn with_map_width(self, chunks: u32) -> Self {
        Self {
            width_chunks: chunks,
            ..self
        }
    }

    /// Set the map height in chunks.
    #[must_use]
    pub const fn with_map_height(self, chunks: u32) -> Self {
        Self {
            height_chunks: chunks,
            ..self
        }
    }

    /// Set the map width and height to the same number of chunks.
    #[must_use]
    pub const fn with_map_size(self, chunks: u32) -> Self {
        Self {
            width_chunks: chunks,
            height_chunks: chunks,
            ..self
        }
    }

    /// Change the chunk size in world units.
    #[must_use]
    pub const fn with_chunk_size(self, chunk_size: u32) -> Self {
        Self { chunk_size, ..self }
    }
}

impl Plugin for FallingSandMinimalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RngPlugin::default(),
            core::FallingSandCorePlugin {
                width: self.width_chunks * self.chunk_size,
                height: self.height_chunks * self.chunk_size,
                chunk_size: self.chunk_size,
            },
        ));
    }
}
