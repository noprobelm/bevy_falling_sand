use crate::*;

#[derive(Bundle)]
pub struct ParticleBundle {
    particle: Particle,
    particle_type: ParticleType,
    coordinates: Coordinates,
    density: Density,
    neighbors: Neighbors,
    anchored: Anchored,
    velocity: Velocity,
    momentum: Momentum,
    sleeping: Sleeping,
    color: ParticleColor,
    fire: Fire,
    burning: Burning
}
