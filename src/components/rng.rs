//! RNG used for particle movement.[^note]
//! [^note]: It might be better to move this to Chunks in the future. This would eliminate the need of having potentially hundreds of thousands of RNG's for every particle in the world
use bevy::prelude::*;
use bevy_turborand::DelegatedRng;
use bevy_turborand::prelude::RngComponent;

/// RNG to use when dealing with any entity that needs random movement behaviors.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct PhysicsRng(pub RngComponent);

impl PhysicsRng {
    /// Shuffles a given slice.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        self.0.shuffle(slice);
    }

    /// Returns a boolean value based on a rate. rate represents the chance to return a true value, with 0.0 being no
    /// chance and 1.0 will always return true.
    pub fn chance(&mut self, rate: f64) -> bool {
        self.0.chance(rate)
    }
}
