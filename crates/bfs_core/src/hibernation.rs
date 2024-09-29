use bevy::prelude::*;

/// Provides a flag for indicating whether an entity is in a hibernating state. Entities with the Hibernating component
/// can be used with bevy query filters to manage which particles are actually being simulated.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Component)]
pub struct Hibernating;

