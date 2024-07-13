//! Collection of particle components.

use bevy::prelude::*;

/// Marker component for particles.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Particle;

/// Marker component for particle parents.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct ParticleParent;

/// Coordinates component for particles.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Coordinates(pub IVec2);

/// The density of a particle.
#[derive(Component, Reflect, Debug, Default, PartialEq, PartialOrd)]
#[reflect(Component, Debug)]
pub struct Density(pub u32);

/// A particle's neighbors, represented as a nested vector of IVec2. The outer vectors represent tiered "groups" of
/// priority, whereas the inner vectors are representative of relative coordinates a particle might move to. See
/// the [handle_particles](crate::handle_particles) system.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Neighbors(pub Vec<Vec<IVec2>>);

/// Indicates whether a particle should be considered for movement or not. Examples of anchored particles might be the
/// ground, or walls. We want anchored particles to be considered as impenetrable neighbors (excluding their ability to
/// be destroyed), but without uninstigated movement.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Anchored;

/// A particle's velocity.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Velocity {
    /// The current velocity of the particle.
    pub val: u8,
    /// The maximum velocity of the particle.
    pub max: u8
}

impl Velocity {
    /// Creates a new velocity component.
    #[inline(always)]
    pub fn new(val: u8, max: u8) -> Self {
        Velocity { val, max }
    }

    /// Increment the velocity by 1
    #[inline(always)]
    pub fn increment(&mut self) {
        if self.val < self.max {
            self.val += 1;
        }
    }

    /// Decrement the velocity by 1
    #[inline(always)]
    pub fn decrement(&mut self) {
        if self.val > 1 {
            self.val -= 1;
        }
    }

}

/// Momentum component for particles. If a particle possesses this component, it will dynamically attempt to move in the
/// same direction it moved in the previous frame.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Momentum(pub IVec2);

impl Momentum {
    /// Use if the particle is capable of gaining momentum, but currently has none.
    pub const ZERO: Self = Self(IVec2::splat(0));
}
