use bevy::prelude::Component;
use bevy_turborand::DelegatedRng;
use bevy_turborand::prelude::RngComponent;

/// RNG to use when dealing with any entity that needs random movement behaviors.
#[derive(Clone, PartialEq, Debug, Default, Component)]
pub struct PhysicsRng(RngComponent);

impl PhysicsRng {
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        self.0.shuffle(slice);
    }
}
