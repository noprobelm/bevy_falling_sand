//! Provides rigid body integration with particle movement systems

use bevy::prelude::*;

/// Marker component which can be added to rigid body colliders in order to include their boundaries
/// for evaluation in particle movement systems.
#[derive(Component)]
pub struct ParticleCollider;
