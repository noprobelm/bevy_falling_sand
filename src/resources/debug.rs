//! Resources related to particle debugging.
use bevy::prelude::Resource;

/// Indicates whether built-in debugging should be enabled.
#[derive(Default, Resource)]
pub struct DebugParticles;

/// The total number of dynamic particles in the simulation.
#[derive(Default, Resource)]
pub struct DynamicParticleCount(pub u64);

/// The total number of particles in the simulation.
#[derive(Default, Resource)]
pub struct TotalParticleCount(pub u64);
