use bevy::prelude::*;

/// Marker component for querying for particles
#[derive(Component, Reflect, Debug, Default)]
pub struct Particle;

/// Marker component for particle parent entities.
pub struct ParticleParent;

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

/// The density of a particle.
#[derive(Component, PartialEq, PartialOrd, Debug, Default, Reflect)]
#[reflect(Component, Debug)]
pub struct Density(pub u32);

/// A sequence of possible neighbors for a particle to consider as part of its movement logic.
/// The inner vectors are neighbors that should be considered as equal candidates when assessing
/// where a particle should attempt to relocate to. The order of the inner vectors can be thought
/// of as *prioritized tier* whe considering groups of movement candidates. For example, a sand
/// particle's order of movement might look like [[0, -1], [1, -1, -1, -1]]. The particle would
/// first attempt to move directly downward. If this fails, it would then look to its lower left
/// and lower right neighbors at random, considering each as equally prioritized candidates.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Neighbors(pub Vec<Vec<IVec2>>);
