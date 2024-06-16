use bevy::prelude::*;

/// Marker component for querying for particles
#[derive(Component, Reflect, Debug, Default)]
pub struct Particle;

/// This component manages a particle's velocity
#[derive(Component, Reflect, Debug, Default)]
pub struct Velocity {
    /// The current velocity of the particle
    pub val: u8,
    /// The maximum velocity of the particle
    pub max: u8
}

impl Velocity {
    #[inline(always)]
    pub fn new(val: u8, max: u8) -> Self {
        Velocity { val, max }
    }

    #[inline(always)]
    pub fn increment(&mut self) {
        if self.val < self.max {
            self.val += 1;
        }
    }

    #[inline(always)]
    pub fn decrement(&mut self) {
        if self.val > 1 {
            self.val -= 1;
        }
    }

}

