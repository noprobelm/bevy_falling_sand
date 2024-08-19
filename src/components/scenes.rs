/// Data structures used for reading particle scenes.
use serde::{Serialize, Deserialize};

use crate::*;

/// Particle data for loading scenes.
#[derive(Serialize, Deserialize)]
pub struct ParticleData {
    /// The particle type to load.
    pub particle_type: ParticleType,
    /// The coordinates of the particle.
    pub coordinates: Coordinates 
}

/// A collection of particles that make up a scene.
#[derive(Serialize, Deserialize)]
pub struct ParticleScene {
    /// The particles to load.
    pub particles: Vec<ParticleData>
}
