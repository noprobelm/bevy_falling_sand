use bevy::prelude::*;
use bevy_turborand::{prelude::RngComponent, DelegatedRng};
use std::ops::RangeBounds;

/// RNG to use when dealing with any entity that needs random reaction behavior.

#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct ReactionRng(pub RngComponent);

impl ReactionRng {
    /// Shuffles a given slice.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        self.0.shuffle(slice);
    }

    /// Returns a boolean value based on a rate. rate represents the chance to return a true value, with 0.0 being no
    /// chance and 1.0 will always return true.
    pub fn chance(&mut self, rate: f64) -> bool {
        self.0.chance(rate)
    }

    /// Samples a random item from a slice of values.
    pub fn sample<'a, T>(&mut self, list: &'a [T]) -> Option<&'a T> {
        self.0.sample(&list)
    }

    /// Returns a usize value for stable indexing across different word size platforms.
    pub fn index(&mut self, bound: impl RangeBounds<usize>) -> usize {
        self.0.index(bound)
    }
}

