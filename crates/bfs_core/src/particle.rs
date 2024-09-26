//! Basic components for marking particles.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::*;

/// Marker component for entities that act as a central reference for particle type information.
#[derive(
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Default,
    Component,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct ParticleType {
    /// The particle type's unique name.
    pub name: String,
}

impl ParticleType {
    /// Creates a new ParticleType
    pub fn new(name: &str) -> ParticleType {
        ParticleType {
            name: name.to_string(),
        }
    }
}

/// Holds the particle type's name. Used to map to particle type data.
#[derive(Component, Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Particle {
    /// The particle's unique name.
    pub name: String,
}

impl Particle {
    /// Creates a new Particle
    pub fn new(name: &str) -> Particle {
        Particle {
            name: name.to_string(),
        }
    }
}

