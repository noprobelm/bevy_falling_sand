//! System set definitions for particle movement.
//!
//! ## Schedule Overview
//!
//! **`PostUpdate`:**
//! - [`ParticleMovementSystems`] — processes particle movement by chunks or individually

use bevy::prelude::*;

use crate::core::{ChunkSystems, ParticleSystems};

pub(super) struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PostUpdate,
            ParticleMovementSystems
                .in_set(ParticleSystems::Simulation)
                .after(ChunkSystems::DirtyAdvance),
        );
    }
}

/// Runs in `PostUpdate`, nested inside
/// [`ParticleSystems::Simulation`].
/// Ordered after [`ChunkSystems::DirtyAdvance`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleMovementSystems;
