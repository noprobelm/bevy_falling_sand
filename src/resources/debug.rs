//! Resources related to particle debugging.
use bevy::prelude::Resource;

/// Indicates whether built-in debugging should be enabled.
#[derive(Default, Resource)]
pub struct DebugParticles;

#[derive(Default, Resource)]
pub struct DynamicParticleCount(pub u64);

#[derive(Default, Resource)]
pub struct TotalParticleCount(pub u64);
