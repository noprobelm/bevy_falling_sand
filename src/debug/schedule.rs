//! System set definitions for particle debugging.
//!
//! ## Schedule Overview
//!
//! **`PostUpdate`:**
//! - [`ParticleDebugSet`] — debug visualization and particle counting

use bevy::prelude::*;

use crate::core::ParticleSystems;

pub(super) struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PostUpdate,
            ParticleDebugSet.after(ParticleSystems::Simulation),
        );
    }
}

/// Runs in `PostUpdate`, ordered after
/// [`ParticleSystems::Simulation`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleDebugSet;
