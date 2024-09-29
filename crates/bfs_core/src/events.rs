use bevy::prelude::{Entity, Event, Plugin, App};
use super::Particle;

/// Main plugin for Bevy Falling Sand
pub struct CoreEventsPlugin;

impl Plugin for CoreEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ParticleMutateEvent>();
    }
}


/// Changes a particle to the designated type
#[derive(Event)]
pub struct ParticleMutateEvent {
    /// The entity to change the particle type of
    pub entity: Entity,
    /// The new particle type
    pub particle: Particle
}
