//! Marker resource for running particle debugging systems
use bevy::prelude::Resource;

/// Indicates whether built-in debugging should be enabled
#[derive(Default, Resource)]
pub struct DebugParticles;
