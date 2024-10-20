use crate::Particle;
use bevy::prelude::*;

/// Core plugin for Bevy Falling Sand.
pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MutateParticleEvent>()
            .add_event::<ResetParticleEvent>()
            .add_event::<RemoveParticleEvent>();
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

/// Resets all of a particle's components to its parent's.
#[derive(Event)]
pub struct ResetParticleEvent {
    /// The entity to reset data for.
    pub entity: Entity,
}

/// Triggers the removal of a particle from the simulation.
#[derive(Event)]
pub struct RemoveParticleEvent {
    /// The coordinates of the particle to remove.
    pub coordinates: IVec2,
    /// Whether the corresponding entity should be despawned from the world.
    pub despawn: bool,
}
