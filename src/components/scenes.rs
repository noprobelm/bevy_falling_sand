use serde::{Serialize, Deserialize};

use crate::*;

#[derive(Serialize, Deserialize)]
pub struct ParticleData {
    pub particle_type: ParticleType,
    pub coordinates: Coordinates 
}

#[derive(Serialize, Deserialize)]
pub struct ParticleScene {
    pub particles: Vec<ParticleData>
}
