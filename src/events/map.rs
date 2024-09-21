//! Events for particle management.
use bevy::prelude::{IVec2, Event, Entity};
use crate::Particle;

/// Triggers [on_remove_particle](crate::on_remove_particle) to remove a particle from the simulation.
#[derive(Event)]
pub struct RemoveParticleEvent {
    /// The coordinates of the particle to remove.
    pub coordinates: IVec2,
    /// Whether the corresponding entity should be despawned from the world.
    pub despawn: bool
}

/// Triggers [on_clear_chunk_map](crate::on_clear_chunk_map) to remove a particle from the simulation.
#[derive(Event)]
pub struct ClearChunkMapEvent;

/// Changes a particle to the designated type
#[derive(Event)]
pub struct ChangeParticleEvent {
    /// The entity to change the particle type of
    pub entity: Entity,
    /// The new particle type
    pub particle: Particle
}

/// Resets all of a particle's components to its parent's.
#[derive(Event)]
pub struct ResetParticleEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

/// Triggers a particle to reset its Density information to its parent's.
#[derive(Event)]
pub struct ResetDensityEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

/// Triggers a particle to reset its MovementPriority information to its parent's.
#[derive(Event)]
pub struct ResetMovementPriorityEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

/// Triggers a particle to reset its Velocity information to its parent's.
#[derive(Event)]
pub struct ResetVelocityEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

/// Triggers a particle to reset its ParticleColor information to its parent's.
#[derive(Event)]
pub struct ResetParticleColorEvent {
    /// The entity to reset data for.
    pub entity: Entity,
}

/// Triggers a particle to reset its RandomizesColor information to its parent's.
#[derive(Event)]
pub struct ResetRandomizesColorEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

/// Triggers a particle to reset its FlowsColor information to its parent's.
#[derive(Event)]
pub struct ResetFlowsColorEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

/// Triggers a particle to reset its ParticleColor information to its parent's.
#[derive(Event)]
pub struct ResetMomentumEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

/// Triggers a particle to reset its Fire information to its parent's.
#[derive(Event)]
pub struct ResetFireEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

/// Triggers a particle to reset its Burns information to its parent's.
#[derive(Event)]
pub struct ResetBurnsEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

/// Triggers a particle to reset its Burning information to its parent's.
#[derive(Event)]
pub struct ResetBurningEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

/// Triggers a particle to reset its Reacts information to its parent's.
#[derive(Event)]
pub struct ResetReactsEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

