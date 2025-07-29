use bevy::prelude::*;
use bevy_turborand::RngComponent;
use bfs_core::{
    impl_particle_rng, AttachedToParticleType, Particle, ParticleRegistrationEvent,
    ParticleRegistrationSet, ParticleRng, ParticleType,
};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::iter;
use std::slice::Iter;

use crate::{Gas, Liquid, Material, MovableSolid, Solid, Wall};

pub(super) struct ParticleDefinitionsPlugin;

impl Plugin for ParticleDefinitionsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Moved>()
            .register_type::<MovementRng>()
            .register_type::<Density>()
            .register_type::<Velocity>()
            .register_type::<Momentum>()
            .register_type::<NeighborGroup>()
            .register_type::<Movement>()
            .add_systems(
                PreUpdate,
                handle_particle_registration.after(ParticleRegistrationSet),
            );
    }
}

impl_particle_rng!(MovementRng, RngComponent);

/// Provides rng for particle movement.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct MovementRng(pub RngComponent);

/// Marker component to indicate that a particle has been moved.
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
#[reflect(Component)]
pub struct Moved(pub bool);

/// Stores the density of a particle
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

///  Stores the velocity of a particle
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Component,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct Velocity {
    /// The current velocity value.
    current: u8,
    /// The maximum velocity value.
    max: u8,
}

impl Default for Velocity {
    fn default() -> Self {
        Self::new(1, 1)
    }
}

impl Velocity {
    /// Initialize a Velocity
    #[must_use]
    pub const fn new(initial: u8, max: u8) -> Self {
        if initial < 1 {
            Self { current: 1, max }
        } else {
            Self {
                current: initial,
                max,
            }
        }
    }

    /// Get the current velocity value
    #[must_use]
    pub const fn current(&self) -> u8 {
        self.current
    }

    /// Get the current mutable velocity value
    #[must_use]
    pub const fn current_mut(&self) -> u8 {
        self.current
    }

    /// Get the max velocity value
    #[must_use]
    pub const fn max(&self) -> u8 {
        self.max
    }

    /// Get the mutable max velocity value
    pub const fn max_mut(&mut self) -> u8 {
        self.max
    }

    /// Set the velocity to a new value.
    pub const fn set_velocity(&mut self, val: u8) {
        if val < 1 {
            self.current = 1;
        } else {
            self.current = val;
        }
    }

    /// Set the velocity to a new value.
    pub const fn set_max_velocity(&mut self, val: u8) {
        if val < 1 {
            self.max = 1;
        } else {
            self.max = val;
        }
    }

    /// Increment the velocity by 1.
    pub const fn increment(&mut self) {
        if self.current < self.max {
            self.current += 1;
        }
    }

    /// Decrement the velocity by 1.
    pub const fn decrement(&mut self) {
        if self.current > 1 {
            self.current -= 1;
        }
    }
}

/// Stores the momentum for a particle.
#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct Momentum(pub IVec2);

impl Momentum {
    /// Get a [`Momentum`] with zero.
    pub const ZERO: Self = Self(IVec2::splat(0));
}

/// A `NeighborGroup` defines an ordered, hierarchial group of relative neighbors usedto evalute
/// particle movement.
///
/// The outer collection is an ordered group of prioritized tiers. The inner collection are the
/// positions of the neighbors relative to the current tier.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct NeighborGroup {
    /// The underlying neighbor group.
    pub neighbor_group: SmallVec<[IVec2; 4]>,
}

impl NeighborGroup {
    /// Initialize a new `NeighborGroup`
    #[must_use]
    pub const fn new(neighbor_group: SmallVec<[IVec2; 4]>) -> Self {
        Self { neighbor_group }
    }

    #[must_use]
    /// Initialize an empty `NeighborGroup`
    pub fn empty() -> Self {
        Self {
            neighbor_group: SmallVec::new(),
        }
    }

    /// Push a new neighbor group to the back.
    pub fn push(&mut self, neighbor: IVec2) {
        self.neighbor_group.push(neighbor);
    }

    /// Swap the position of two indices in the outer collection with one another
    ///
    /// # Errors
    ///
    /// An error is returned if either of the swap indices are out of bounds.
    pub fn swap(&mut self, index1: usize, index2: usize) -> Result<(), String> {
        if index1 < self.neighbor_group.len() && index2 < self.neighbor_group.len() {
            self.neighbor_group.swap(index1, index2);
            Ok(())
        } else {
            Err(format!(
                "Swap indices out of bounds: index1={}, index2={}, group size={}",
                index1,
                index2,
                self.neighbor_group.len()
            ))
        }
    }

    /// Returns true if the `NeighborGroup` holds no data.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.neighbor_group.is_empty()
    }

    /// Returns the length of the `NeighborGroup`.
    #[must_use]
    pub fn len(&self) -> usize {
        self.neighbor_group.len()
    }

    /// Iterates through `NeighborGroup` tiers, using `MovementRng` to randomly select a candidate
    /// in each tier and optionally `Momentum` to specify a movement preference.
    ///
    /// If `Momentum` is specified, automatically return a position in a neighbor group if it
    /// matches the passed `Momentum`. Otherwise, all neighbors in each tier are weighted equally
    /// using rng.
    fn iter_candidates<'a>(
        &'a mut self,
        rng: &mut MovementRng,
        preferred: Option<&Momentum>,
    ) -> NeighborGroupIter<'a> {
        if let Some(momentum) = preferred {
            if let Some(position) = self
                .neighbor_group
                .iter()
                .position(|&candidate| momentum.0 == candidate)
            {
                return NeighborGroupIter::Single(iter::once(&self.neighbor_group[position]));
            }
        }

        rng.shuffle(&mut self.neighbor_group);
        NeighborGroupIter::All(self.neighbor_group.iter())
    }
}

/// Enum for a `NeighborGroup` iterator.
enum NeighborGroupIter<'a> {
    Single(iter::Once<&'a IVec2>),
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

/// Specifies the order of movement priority for a particle. This is mandatory for a particle to
/// move while using [`bfs_movement`].
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Movement {
    /// The underlying groups of neighbors that define the movement priority.
    pub neighbor_groups: SmallVec<[NeighborGroup; 8]>,
}

impl Movement {
    /// Initialize a new `Movement` with the specified neighbor groups.
    #[must_use]
    pub const fn new(neighbor_groups: SmallVec<[NeighborGroup; 8]>) -> Self {
        Self { neighbor_groups }
    }

    /// Build a [`Movement`] from Vec<Vec<IVec2>>. Each inner vector represents a group of
    /// neighbors
    #[must_use]
    pub fn from(movement_priority: Vec<Vec<IVec2>>) -> Self {
        Self::new(
            movement_priority
                .into_iter()
                .map(|neighbor_group| NeighborGroup::new(SmallVec::from_vec(neighbor_group)))
                .collect::<SmallVec<[NeighborGroup; 8]>>(),
        )
    }

    /// Returns true if the [`Movement`] holds no data.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.neighbor_groups.is_empty()
    }

    /// Returns the length of the [`Movement`].
    #[must_use]
    pub fn len(&self) -> usize {
        self.neighbor_groups.len()
    }

    /// Pushes back a new group of neighbors to the [`Movement`].
    pub fn push_outer(&mut self, neighbor_group: NeighborGroup) {
        self.neighbor_groups.push(neighbor_group);
    }

    /// Pushes back a new neighbor to a specified inner group index of the [`Movement`].
    ///
    /// # Errors
    ///
    /// Returns an error if the specified inner group index is out of bounds.
    pub fn push_inner(&mut self, group_index: usize, neighbor: IVec2) -> Result<(), String> {
        self.neighbor_groups
            .get_mut(group_index)
            .map(|group| {
                group.push(neighbor);
            })
            .ok_or_else(|| format!("Group index {} out of bounds", group_index))
    }

    /// Swaps one outer group with another from two provided indices.
    ///
    /// # Errors
    ///
    /// Returns an error if either index is out of bounds for the outer groups.
    pub fn swap_outer(&mut self, index1: usize, index2: usize) -> Result<(), String> {
        if index1 < self.neighbor_groups.len() && index2 < self.neighbor_groups.len() {
            self.neighbor_groups.swap(index1, index2);
            Ok(())
        } else {
            Err("Outer indices out of bounds".to_string())
        }
    }

    /// Swaps one inner rgoup with another from two provided indices.
    ///
    /// # Errors
    ///
    /// Returns an error if either index is out of bounds for the inner groups.
    pub fn swap_inner(
        &mut self,
        group_index: usize,
        index1: usize,
        index2: usize,
    ) -> Result<(), String> {
        self.neighbor_groups
            .get_mut(group_index)
            .ok_or_else(|| format!("Group index {group_index} out of bounds"))
            .and_then(|group| {
                if index1 < group.len() && index2 < group.len() {
                    group.swap(index1, index2)?;
                    Ok(())
                } else {
                    Err("Inner indices out of bounds".to_string())
                }
            })
    }

    /// An iterator for the outer groups of the [`Movement`].
    pub fn iter(&self) -> impl Iterator<Item = &NeighborGroup> {
        self.neighbor_groups.iter()
    }

    /// Iterates through `NeighborGroup` tiers, using `MovementRng` to randomly select a candidate
    /// in each tier and optionally `Momentum` to specify a movement preference.
    ///
    /// If `Momentum` is specified, automatically return a position in a neighbor group if it
    /// matches the passed `Momentum`. Otherwise, all neighbors in each tier are weighted equally
    /// using rng.
    pub fn iter_candidates<'a>(
        &'a mut self,
        rng: &'a mut MovementRng,
        momentum: Option<&'a Momentum>,
    ) -> impl Iterator<Item = &'a IVec2> + 'a {
        self.neighbor_groups
            .iter_mut()
            .flat_map(move |neighbor_group| neighbor_group.iter_candidates(rng, momentum))
    }

    /// Mutable getter for a neighbor group at a specified index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut NeighborGroup> {
        self.neighbor_groups.get_mut(index)
    }

    /// Remove a neighbor group at a specified index and return it.
    pub fn remove(&mut self, index: usize) -> Option<NeighborGroup> {
        if index < self.neighbor_groups.len() {
            Some(self.neighbor_groups.remove(index))
        } else {
            None
        }
    }
}

impl Movement {
    /// Initialize an empty `NeighborGroup`
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            neighbor_groups: SmallVec::new_const(),
        }
    }
}

type MovementQuery<'a> = (
    Option<&'a mut Density>,
    Option<&'a mut Velocity>,
    Option<&'a mut Movement>,
    Option<&'a mut Momentum>,
    Option<&'a mut Wall>,
    Option<&'a mut Liquid>,
    Option<&'a mut Gas>,
    Option<&'a mut MovableSolid>,
    Option<&'a mut Solid>,
);

#[allow(clippy::needless_pass_by_value)]
fn handle_particle_registration(
    mut commands: Commands,
    blueprint_query: Query<MovementQuery<'_>, With<ParticleType>>,
    mut ev_particle_registered: EventReader<ParticleRegistrationEvent>,
    particle_query: Query<&AttachedToParticleType, With<Particle>>,
) {
    ev_particle_registered.read().for_each(|ev| {
        ev.entities.iter().for_each(|entity| {
            if let Ok(attached_to) = particle_query.get(*entity) {
                commands.entity(*entity).insert(MovementRng::default());
                if let Ok((
                    density,
                    velocity,
                    movement_priority,
                    momentum,
                    wall,
                    liquid,
                    gas,
                    movable_solid,
                    solid,
                )) = blueprint_query.get(attached_to.0)
                {
                    if let Some(density) = density {
                        commands.entity(*entity).insert(*density);
                    } else {
                        commands.entity(*entity).remove::<Density>();
                    }
                    if let Some(velocity) = velocity {
                        commands.entity(*entity).insert(*velocity);
                    } else {
                        commands.entity(*entity).remove::<Velocity>();
                    }
                    if let Some(momentum) = momentum {
                        commands.entity(*entity).insert(*momentum);
                    } else {
                        commands.entity(*entity).remove::<Momentum>();
                    }
                    if wall.is_some() {
                        commands.entity(*entity).insert(Wall);
                    }
                    if let Some(liquid) = liquid {
                        commands.entity(*entity).insert(liquid.clone());
                        commands.entity(*entity).insert(Moved(true));
                        commands
                            .entity(*entity)
                            .insert(liquid.to_movement_priority());
                    } else if let Some(gas) = gas {
                        commands.entity(*entity).insert(gas.clone());
                        commands.entity(*entity).insert(Moved(true));
                        commands.entity(*entity).insert(gas.to_movement_priority());
                    } else if let Some(movable_solid) = movable_solid {
                        commands.entity(*entity).insert(movable_solid.clone());
                        commands.entity(*entity).insert(Moved(true));
                        commands
                            .entity(*entity)
                            .insert(movable_solid.to_movement_priority());
                    } else if let Some(solid) = solid {
                        commands.entity(*entity).insert(solid.clone());
                        commands.entity(*entity).insert(Moved(true));
                        commands
                            .entity(*entity)
                            .insert(solid.to_movement_priority());
                    } else if let Some(movement_priority) = movement_priority {
                        commands.entity(*entity).insert(movement_priority.clone());
                    } else {
                        commands.entity(*entity).remove::<Movement>();
                    }
                }
                commands.entity(*entity).insert(Moved(true));
            }
        });
    });
}
