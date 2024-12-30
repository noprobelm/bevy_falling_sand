use bevy::prelude::*;
use bevy_turborand::{prelude::RngComponent, DelegatedRng};
use std::ops::RangeBounds;

#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct ReactionRng(pub RngComponent);

impl ReactionRng {
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        self.0.shuffle(slice);
    }

    pub fn chance(&mut self, rate: f64) -> bool {
        self.0.chance(rate)
    }

    pub fn sample<'a, T>(&mut self, list: &'a [T]) -> Option<&'a T> {
        self.0.sample(&list)
    }

    pub fn index(&mut self, bound: impl RangeBounds<usize>) -> usize {
        self.0.index(bound)
    }
}

