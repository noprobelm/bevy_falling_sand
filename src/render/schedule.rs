//! System set definitions for particle color and rendering.
//!
//! ## Schedule Overview
//!
//! **`Update`:**
//! - [`RenderingSystems::ChunkImage`] — world texture setup
//!
//! **`PostUpdate`:**
//! - [`RenderingSystems::ChunkEffectLayerUpdate`] — chunk effect layer updates

use bevy::prelude::*;

use crate::core::{ChunkSystems, ParticleSystems};

pub(super) struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, RenderingSystems::ChunkImage)
            .configure_sets(
                PostUpdate,
                RenderingSystems::ChunkEffectLayerUpdate
                    .after(ParticleSystems::Simulation)
                    .after(ChunkSystems::Cleanup),
            );
    }
}

/// System sets for particle rendering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum RenderingSystems {
    /// Runs in `Update`. Material systems that depend on
    /// [`WorldColorTexture`](super::WorldColorTexture) should be ordered
    /// `.after(RenderingSystems::ChunkImage)` to ensure deferred commands have been applied.
    ChunkImage,
    /// Runs in `PostUpdate`, ordered after
    /// [`ParticleSystems::Simulation`] and
    /// [`ChunkSystems::Cleanup`].
    ChunkEffectLayerUpdate,
}
