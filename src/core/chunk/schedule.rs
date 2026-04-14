//! System set definitions for chunk management.
//!
//! ## Schedule Overview
//!
//! **`PreUpdate`:**
//! - [`ChunkSystems::Loading`] — handles chunk loading/unloading on origin shift
//! - [`ChunkSystems::DirtyAdvance`] — advances chunk dirty state (runs before movement)
//!
//! **`PostUpdate`:**
//! - [`ChunkSystems::Cleanup`] — drains stale particles from unloaded regions

use bevy::prelude::*;

use crate::ParticleSimulationRun;

pub(super) struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkLoadingRun>()
            .configure_sets(
                PreUpdate,
                (
                    ChunkSystems::Loading.run_if(resource_exists::<ChunkLoadingRun>),
                    ChunkSystems::DirtyAdvance.run_if(resource_exists::<ParticleSimulationRun>),
                ),
            )
            .configure_sets(
                PostUpdate,
                ChunkSystems::Cleanup
                    .run_if(resource_exists::<ParticleSimulationRun>)
                    .run_if(resource_exists::<ChunkLoadingRun>),
            );
    }
}

/// System sets for chunk management.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChunkSystems {
    /// Runs in `PreUpdate`. Detects when the [`ChunkLoader`](crate::ChunkLoader)
    /// crosses a chunk boundary and shifts the map origin accordingly. Systems that
    /// respond to chunk loading/unloading should run after this set.
    Loading,
    /// Runs in `PreUpdate`. Advances chunk dirty state so that movement systems
    /// know which chunks need processing.
    DirtyAdvance,
    /// Runs in `PostUpdate`. Drains particle data from unloaded regions and
    /// processes batched despawns across frames to avoid frame spikes.
    Cleanup,
}

/// Marker resource to control whether chunk loading sytsems will run.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkLoadingRun;
