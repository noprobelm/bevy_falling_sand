use bevy::prelude::{Event, Entity};

/// Triggers a particle to reset its ParticleColor information to its parent's.
#[derive(Event)]
pub struct ResetMomentumEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

/// Triggers a particle to reset its Velocity information to its parent's.
#[derive(Event)]
pub struct ResetVelocityEvent {
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
