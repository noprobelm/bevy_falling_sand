use bevy::prelude::{Entity, Event};

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
