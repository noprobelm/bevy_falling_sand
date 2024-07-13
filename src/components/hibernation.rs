use bevy::prelude::*;

/// Provides a flag for indicating whether a Particle is in a hibernated state. This is used to indicate whether a
/// particle should be included in any kind of movements for the frame.
#[derive(Component)]
pub struct ShouldProcessThisFrame;
