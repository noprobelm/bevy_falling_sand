use bevy::prelude::{Entity, Event};
use super::Particle;

/// Changes a particle to the designated type
#[derive(Event)]
pub struct ChangeParticleEvent {
    /// The entity to change the particle type of
    pub entity: Entity,
    /// The new particle type
    pub particle: Particle
}
