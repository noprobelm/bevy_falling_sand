//! Defines additional components for particle types to be used as blueprint data when spawning or
//! resetting particles.
//!
//! This module is a standard template that can be followed when extending particle types. Its
//! structure is as follows:
//!   - Defines new components which will be associated with particle types as blueprint information
//!     for child particles.
//!   - Adds events for each new component which manage resetting information for child particles
//!   - Adds observers for each event to specify granular logic through which a particle should have
//!     its information reset. This usually involves referencing the parent `ParticleType`.
//!
//! When a particle should have its information reset (e.g., when spawning or resetting), we can
//! trigger the events defined in this module and communicate with higher level systems that
//! something needs to happen with a given particle.

use std::iter;
use std::slice::Iter;
use smallvec::SmallVec;
use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use bfs_core::{ParticleType, Particle};

use crate::rng::PhysicsRng;

pub struct ParticleDefinitionsPlugin;

impl Plugin for ParticleDefinitionsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Density>()
            .register_type::<Velocity>()
            .register_type::<Momentum>()
            .register_type::<MovementPriority>()
            .observe(on_reset_density)
            .observe(on_reset_velocity)
            .observe(on_reset_momentum)
            .observe(on_reset_movement_priority);
    }
}

/// The density of a particle.
#[derive(
    Copy,
    Clone,
    Hash,
    Debug,
    Default,
    Eq,
    PartialEq,
    PartialOrd,
    Component,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component, Debug)]
pub struct Density(pub u32);

/// A particle's velocity.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Default,
    Component,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct Velocity {
    /// The current velocity of the particle.
    pub val: u8,
    /// The maximum velocity of the particle.
    pub max: u8,
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
#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct Momentum(pub IVec2);

impl Momentum {
    /// Use if the particle is capable of gaining momentum, but currently has none.
    pub const ZERO: Self = Self(IVec2::splat(0));
}

/// A group of neighbors representing equally prioritized candidates for particle movement.
/// Positions are relative to the particle's position.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect)]
pub struct NeighborGroup {
    /// The neighbor candidates.
    pub neighbor_group: SmallVec<[IVec2; 4]>,
}

impl NeighborGroup {
    /// Creates a new NeighborGroup instance.
    pub fn new(neighbor_group: SmallVec<[IVec2; 4]>) -> NeighborGroup {
        NeighborGroup { neighbor_group }
    }

    /// An iterator over neighbors.
    pub fn iter(&self) -> impl Iterator<Item = &IVec2> {
        self.neighbor_group.iter()
    }

    /// A mutable iterator over neighbors.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut IVec2> {
        self.neighbor_group.iter_mut()
    }

    /// An iterator over random neighbors.
    pub fn iter_candidates<'a>(
        &'a mut self,
        rng: &mut PhysicsRng,
        momentum: Option<&Momentum>,
    ) -> NeighborGroupIter<'a> {
        if let Some(momentum) = momentum {
            if let Some(position) = self
                .neighbor_group
                .iter()
                .position(|&candidate| momentum.0 == candidate)
            {
                return NeighborGroupIter::Single(iter::once(&self.neighbor_group[position]));
            }
        }

        // Shuffle the neighbors and return the iterator over all neighbors
        rng.shuffle(&mut self.neighbor_group);
        NeighborGroupIter::All(self.neighbor_group.iter())
    }
}

/// An iterator over neighbor groups
pub enum NeighborGroupIter<'a> {
    /// A single neighbor should be prioritized above all others.
    Single(iter::Once<&'a IVec2>),
    /// All neighbors should be iterated.
    All(Iter<'a, IVec2>),
}

impl<'a> Iterator for NeighborGroupIter<'a> {
    type Item = &'a IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            NeighborGroupIter::Single(iter) => iter.next(),
            NeighborGroupIter::All(iter) => iter.next(),
        }
    }
}

/// A collection of neighbor groups ordered by descending priority.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct MovementPriority {
    /// The neighbor ps.
    pub neighbor_groups: SmallVec<[NeighborGroup; 8]>,
}

impl MovementPriority {
    /// Creates a new NeighborGroup instance.
    pub fn new(neighbor_groups: SmallVec<[NeighborGroup; 8]>) -> MovementPriority {
        MovementPriority { neighbor_groups }
    }

    /// Creates a new NeighborGroup instance.
    pub fn from(movement_priority: SmallVec<[SmallVec<[IVec2; 4]>; 8]>) -> MovementPriority {
        MovementPriority::new(
            movement_priority
                .into_iter()
                .map(|neighbor_group| NeighborGroup::new(neighbor_group))
                .collect::<SmallVec<[NeighborGroup; 8]>>(),
        )
    }

    /// An iterator over neighbors.
    pub fn iter(&self) -> impl Iterator<Item = &NeighborGroup> {
        self.neighbor_groups.iter()
    }

    /// Iterates over movement candidates for a particle.
    pub fn iter_candidates<'a>(
        &'a mut self,
        rng: &'a mut PhysicsRng,
        momentum: Option<&'a Momentum>,
    ) -> impl Iterator<Item = &'a IVec2> + 'a {
        self.neighbor_groups
            .iter_mut()
            .flat_map(move |neighbor_group| neighbor_group.iter_candidates(rng, momentum))
    }
}

impl MovementPriority {
    /// Returns an empty MovementPriority
    pub const fn empty() -> MovementPriority {
        MovementPriority {
            neighbor_groups: SmallVec::new_const(),
        }
    }
}

/// Triggers a particle to reset its ParticleColor information to its parent's.
#[derive(Event)]
pub struct ResetMomentumEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

/// Triggers a particle to reset its Velocity information to its parent's.
#[derive(Event)]
pub struct ResetVelocityEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

/// Triggers a particle to reset its Density information to its parent's.
#[derive(Event)]
pub struct ResetDensityEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

/// Triggers a particle to reset its MovementPriority information to its parent's.
#[derive(Event)]
pub struct ResetMovementPriorityEvent {
    /// The entity to reset data for.
    pub entity: Entity
}

/// Observer for resetting a particle's Momentum information to its parent's.
pub fn on_reset_momentum(
    trigger: Trigger<ResetMomentumEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&Momentum>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(momentum) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(momentum.clone());
        } else {
            commands.entity(trigger.event().entity).remove::<Momentum>();
        }
    }
}

/// Observer for resetting a particle's Density information to its parent's.
pub fn on_reset_density(
    trigger: Trigger<ResetDensityEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&Density>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(density) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(density.clone());
        } else {
            commands.entity(trigger.event().entity).remove::<Density>();
        }
    }
}

/// Observer for resetting a particle's MovementPriority information to its parent's.
pub fn on_reset_movement_priority(
    trigger: Trigger<ResetMovementPriorityEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&MovementPriority>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(movement_priority) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(movement_priority.clone());
        } else {
            commands
                .entity(trigger.event().entity)
                .remove::<MovementPriority>();
        }
    }
}

/// Observer for resetting a particle's Velocity information to its parent's.
pub fn on_reset_velocity(
    trigger: Trigger<ResetVelocityEvent>,
    mut commands: Commands,
    particle_query: Query<&Parent, With<Particle>>,
    parent_query: Query<Option<&Velocity>, With<ParticleType>>,
) {
    if let Ok(parent) = particle_query.get(trigger.event().entity) {
        if let Some(velocity) = parent_query.get(parent.get()).unwrap() {
            commands
                .entity(trigger.event().entity)
                .insert(velocity.clone());
        } else {
            commands.entity(trigger.event().entity).remove::<Velocity>();
        }
    }
}
