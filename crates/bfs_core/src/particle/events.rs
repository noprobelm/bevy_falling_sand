use bevy::prelude::*;

use crate::Particle;

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MutateParticleEvent>();
    }
}

/// Changes a particle to the designated type
#[derive(Event)]
pub struct MutateParticleEvent {
    /// The entity to change the particle type of
    pub entity: Entity,
    /// The new particle type
    pub particle: Particle,
}

