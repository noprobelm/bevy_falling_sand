//! System set definitions for particle simulation.
//!
//! ## Schedule Overview
//!
//! **`PreUpdate`:**
//! - [`ParticleSystems::Registration`] — handles registration for new particles
//!
//! **`PostUpdate`:**
//! - [`ParticleSystems::Simulation`] — top-level gate for all simulation systems

use bevy::prelude::*;

use crate::simulation::{condition_msg_simulation_step_received, ParticleSimulationRun};

pub(super) struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(PreUpdate, ParticleSystems::Registration)
            .configure_sets(
                PostUpdate,
                ParticleSystems::Simulation.run_if(
                    resource_exists::<ParticleSimulationRun>
                        .or(condition_msg_simulation_step_received),
                ),
            );
    }
}

/// System sets for particle simulation.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParticleSystems {
    /// Runs in `PreUpdate`. Systems that spawn particles via
    /// [`SpawnParticleSignal`](crate::SpawnParticleSignal) run before this set;
    /// deferred command application and invalid-particle cleanup run after.
    Registration,
    /// Runs in `PostUpdate`. Gated by the presence of
    /// [`ParticleSimulationRun`] or receipt of a
    /// [`SimulationStepSignal`](crate::SimulationStepSignal).
    Simulation,
}
