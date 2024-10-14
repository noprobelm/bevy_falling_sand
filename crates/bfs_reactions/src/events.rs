use bevy::prelude::{Entity, Event};

/// Triggers a particle to reset its Reacts information to its parent's.
#[derive(Event)]
pub struct ResetReactsEvent {
    /// The entity to reset data for.
    pub entity: Entity,
}

/// Triggers a particle to reset its Burning information to its parent's.
#[derive(Event)]
pub struct ResetBurningEvent {
    /// The entity to reset data for.
    pub entity: Entity,
}

/// Triggers a particle to reset its Burns information to its parent's.
#[derive(Event)]
pub struct ResetBurnsEvent {
    /// The entity to reset data for.
    pub entity: Entity,
}

/// Triggers a particle to reset its Fire information to its parent's.
#[derive(Event)]
pub struct ResetFireEvent {
    /// The entity to reset data for.
    pub entity: Entity,
}
