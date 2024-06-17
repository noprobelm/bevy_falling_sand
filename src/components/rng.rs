use bevy::prelude::Component;
use bevy_turborand::prelude::RngComponent;

/// The physics rng to use when dealing with any entity that needs random movement behaviors.
#[derive(Component, Default)]
pub struct PhysicsRng(RngComponent);
