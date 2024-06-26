use bevy::prelude::*;

/// Marker component for querying for particles
#[derive(Component, Reflect, Debug, Default)]
pub struct Particle;

/// Marker component for particle parent entities. Parent particle entities are generally responsible
/// for holding common component data between particles, such as density or neighbor priority
/// selections.
#[derive(Component, Reflect, Debug, Default)]
pub struct ParticleParent;

/// This component keeps track of the coordinate of a given particle. This is primarily used for
/// movement detection from one frame to the next, so we can do things like track time last moved
/// (for hibernation)
#[derive(Component, Reflect, Clone, Default, Debug, Eq, PartialEq)]
#[reflect(Component)]
pub struct Coordinates(pub IVec2);

/// The density of a particle.
#[derive(Component, Reflect, Debug, Default, PartialEq, PartialOrd)]
#[reflect(Component, Debug)]
pub struct Density(pub u32);

/// A sequence of possible neighbors for a particle to consider as part of its movement logic.
/// The inner vectors are neighbors that should be considered as equal candidates when assessing
/// where a particle should attempt to relocate to. The order of the inner vectors can be thought
/// of as *prioritized tier* whe considering groups of movement candidates.
///
/// For example, a sand particle's order of movement might look like
/// `[[[0, -1]], [[1, -1], [-1, -1]]]`. The particle would first attempt to move directly downward.
/// If this fails, it would then look to its lower left and lower right neighbors at random,
/// considering each as equally prioritized candidates.
#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct Neighbors(pub Vec<Vec<IVec2>>);

/// This component indicates whether a particle should be considered for movement or not. Examples
/// of anchored particles might be the ground, or walls. We want anchored particles to be considered
/// as impenetrable neighbors (excluding their ability to be destroyed), but without uninstigated
/// movement.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Anchored;

/// This component manages a particle's velocity
#[derive(Component, Reflect, Debug, Default)]
pub struct Velocity {
    /// The current velocity of the particle
    pub val: u8,
    /// The maximum velocity of the particle
    pub max: u8
}

impl Velocity {
    /// Creates a new velocity component with the specified values.
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

/// This component provides optional momentum for particles. If a particle possesses the ability to
/// obtain momentum (as indicated by the presence of this component on its parent type), it will
/// attempt to relocate itself to the relative coordinate indicated by the IVec2 field *if* the
/// coordinate resides within the inner vector currently being considered in the Neighbors
/// selection strategy.
///
/// The value of the IVec2 field is influenced by the successful, unobstructed movement of the
/// particle as part of its previous step.
#[derive(Component, Reflect, Clone, Default, Debug)]
#[reflect(Component)]
pub struct Momentum(pub IVec2);

impl Momentum {
    /// Use this value for momentum if the particle is capable of gaining momentum, but currently
    /// has none.
    pub const ZERO: Self = Self(IVec2::splat(0));
}
