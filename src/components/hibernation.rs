//! Components related to particle "hibernation". See [ChunkMap](crate::ChunkMap) for more information on how this works.
use bevy::prelude::*;

/// Provides a flag for indicating whether a Particle is in a hibernated state. This is used to indicate whether a
/// particle should be included in any kind of movements for the frame.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Component)]
pub struct Hibernating;
