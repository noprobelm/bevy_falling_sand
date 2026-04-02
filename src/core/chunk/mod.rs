//! Chunk types for efficient simulation and rendering of particles.
//!
//! By subdividing the [`ParticleMap`](crate::ParticleMap) into chunks, we create the
//! possibility for efficient particle simulation and rendering. Each chunk is an entity carrying a
//! [`ChunkRegion`] (its world-space bounds) and a [`ChunkDirtyState`] (which regions have changed
//! and need reprocessing).
//!
//! # Loading
//!
//! Chunks are dynamically loaded and unloaded as a [`ChunkLoader`] entity moves through the world.
//! The [`ChunkIndex`] resource maps [`ChunkCoord`] values to chunk entities using toroidal
//! addressing, enabling O(1) lookups and seamless origin shifts without copying data.
//! See [`ChunkLoadingConfig`] and [`ChunkLoadingState`] for tuning and inspecting loading behavior.
//!
//! The [`persistence`](crate::persistence) module relies on `ChunkLoadingState` to know when to
//! read or write chunk data to disk as the `ChunkIndex` origin is shifted.
//!
//! # Dirty tracking
//!
//! [`ChunkDirtyState`] accumulates dirty rectangles and individual positions as particles change.
//! Each frame, the dirty state advances: the previous frame's accumulated mutations become the
//! current frame's processing region, inflated by 2 pixels to account for movement spillover.
//! Border flags ([`BORDER_N`], [`BORDER_E`], etc.) mark 2-pixel edge strips where neighboring chunk
//! activity may require rescanning.
//!
//! ## Movement and rendering
//!
//! If using the `movement` feature, the [`movement`](crate::movement) module iterates chunks in
//! checkerboard groups (via [`ChunkCoord::group`]) -- a concept inspired by the Noita developer's
//! 2019 [GDC talk](https://www.gdcvault.com/play/1025695/Exploring-the-Tech-and-Design) -- so that
//! non-adjacent chunks can be processed in parallel without data races.
//!
//! Rendering systems read each chunk's [`ChunkDirtyState`] to update only the pixels that changed,
//! avoiding full-texture rewrites.
//!
//! # Cleanup
//!
//! When chunks are unloaded, their particles are drained from the [`ParticleMap`](crate::ParticleMap)
//! and their entities are despawned incrementally across frames to prevent frame spikes.
//! See [`DespawnBatchConfig`] for tuning.
mod coord;
mod dirty;
mod index;
mod loading;
mod region;
mod schedule;

use bevy::prelude::*;

pub use coord::ChunkCoord;
pub use dirty::{
    ChunkDirtyState, BORDER_E, BORDER_N, BORDER_NE, BORDER_NW, BORDER_S, BORDER_SE, BORDER_SW,
    BORDER_W,
};
pub use index::ChunkIndex;
pub use loading::{
    ChunkLoader, ChunkLoadingConfig, ChunkLoadingState, DespawnBatchConfig, PendingDespawn,
};
pub use region::ChunkRegion;
pub use schedule::ChunkSystems;

use crate::core::chunk::{
    dirty::DirtyTrackingPlugin, loading::LoadingPlugin,
    schedule::SchedulePlugin as ChunkSchedulePlugin,
};

pub(super) struct ChunkPlugin {
    pub chunks_wide: u32,
    pub chunks_tall: u32,
    pub chunk_size: u32,
}

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ChunkSchedulePlugin, DirtyTrackingPlugin, LoadingPlugin))
            .insert_resource(ChunkIndex::new(
                self.chunks_wide,
                self.chunks_tall,
                self.chunk_size,
                IVec2::new(
                    -(self.chunks_wide as i32 / 2),
                    -(self.chunks_tall as i32 / 2),
                ),
            ))
            .init_resource::<ChunkLoadingConfig>()
            .init_resource::<ChunkLoadingState>()
            .init_resource::<DespawnBatchConfig>()
            .register_type::<ChunkRegion>()
            .register_type::<ChunkDirtyState>();
    }
}
