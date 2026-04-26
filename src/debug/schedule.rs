//! System set definitions for particle debugging.
//!
//! ## Schedule Overview
//!
//! **`PostUpdate`:**
//! - [`ParticleDebugSystems`] — debug visualization and particle counting

use bevy::prelude::*;

use crate::core::ParticleSystems;

pub(super) struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PostUpdate,
            ParticleDebugSystems.after(ParticleSystems::Simulation),
        );
    }
}

/// Runs in `PostUpdate`, ordered after
/// [`ParticleSystems::Simulation`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleDebugSystems;
