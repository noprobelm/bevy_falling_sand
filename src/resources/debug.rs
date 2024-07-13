//! Resources related to particle debugging.
use bevy::prelude::Resource;

/// Indicates whether built-in debugging should be enabled.
#[derive(Default, Resource)]
pub struct DebugParticles;
