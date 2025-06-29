use bevy::prelude::*;
use bevy_turborand::RngComponent;
use bfs_core::{
    impl_particle_blueprint, impl_particle_rng, Particle, ParticleComponent,
    ParticleRegistrationEvent, ParticleRng, ParticleType,
};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::iter;
use std::slice::Iter;

use crate::{
    Gas, GasBlueprint, Liquid, LiquidBlueprint, MovableSolid, MovableSolidBlueprint, Solid,
    SolidBlueprint, Wall, WallBlueprint,
};

pub(super) struct ParticleDefinitionsPlugin;

impl Plugin for ParticleDefinitionsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Density>()
            .register_type::<Velocity>()
            .register_type::<Momentum>()
            .register_type::<MovementPriority>()
            .add_systems(Update, handle_particle_registration);
    }
}

impl_particle_rng!(MovementRng, RngComponent);
impl_particle_blueprint!(DensityBlueprint, Density);
impl_particle_blueprint!(VelocityBlueprint, Velocity);
impl_particle_blueprint!(MomentumBlueprint, Momentum);
impl_particle_blueprint!(MovementPriorityBlueprint, MovementPriority);

/// Provides rng for particle movement.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct MovementRng(pub RngComponent);

/// Marker comopenponent to indicate that a particle has been moved.
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

/// Blueprint for a [`Density`]
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
pub struct DensityBlueprint(pub Density);

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
    Default,
    Component,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct Velocity {
    /// The current velocity value.
    pub val: u8,
    /// The maximum velocity value.
    pub max: u8,
}

impl Velocity {
    /// Initialize a Velocity
    #[must_use]
    pub const fn new(val: u8, max: u8) -> Self {
        Self { val, max }
    }

    /// Increment the velocity by 1.
    pub const fn increment(&mut self) {
        if self.val < self.max {
            self.val += 1;
        }
    }

    /// Decrement the velocity by 1.
    pub const fn decrement(&mut self) {
        if self.val > 1 {
            self.val -= 1;
        }
    }
}

/// Blueprint for a [`Velocity`].
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
pub struct VelocityBlueprint(pub Velocity);

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

/// Blueprint for a [`Momentum`]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct MomentumBlueprint(pub Momentum);

impl Default for MomentumBlueprint {
    fn default() -> Self {
        Self(Momentum::ZERO)
    }
}

/// A `NeighborGroup` defines an ordered, hierarchial group of relative neighbors usedto evalute
/// particle movement.
///
/// The outer collection is an ordered group of prioritized tiers. The inner collection are the
/// positions of the neighbors relative to the current tier.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect)]
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
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct MovementPriority {
    /// The underlying groups of neighbors that define the movement priority.
    pub neighbor_groups: SmallVec<[NeighborGroup; 8]>,
}

impl MovementPriority {
    /// Initialize a new `MovementPriority` with the specified neighbor groups.
    #[must_use]
    pub const fn new(neighbor_groups: SmallVec<[NeighborGroup; 8]>) -> Self {
        Self { neighbor_groups }
    }

    /// Build a [`MovementPriority`] from Vec<Vec<IVec2>>. Each inner vector represents a group of
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

    /// Returns true if the [`MovementPriority`] holds no data.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.neighbor_groups.is_empty()
    }

    /// Returns the length of the [`MovementPriority`].
    #[must_use]
    pub fn len(&self) -> usize {
        self.neighbor_groups.len()
    }

    /// Pushes back a new group of neighbors to the [`MovementPriority`].
    pub fn push_outer(&mut self, neighbor_group: NeighborGroup) {
        self.neighbor_groups.push(neighbor_group);
    }

    /// Pushes back a new neighbor to a specified inner group index of the [`MovementPriority`].
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

    /// An iterator for the outer groups of the [`MovementPriority`].
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

impl MovementPriority {
    /// Initialize an empty `NeighborGroup`
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            neighbor_groups: SmallVec::new_const(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
/// Blueprint for a [`MovementPriority`]
pub struct MovementPriorityBlueprint(pub MovementPriority);

type BlueprintQuery<'a> = (
    Option<&'a mut DensityBlueprint>,
    Option<&'a mut VelocityBlueprint>,
    Option<&'a mut MovementPriorityBlueprint>,
    Option<&'a mut MomentumBlueprint>,
    Option<&'a mut WallBlueprint>,
    Option<&'a mut LiquidBlueprint>,
    Option<&'a mut GasBlueprint>,
    Option<&'a mut MovableSolidBlueprint>,
    Option<&'a mut SolidBlueprint>,
);

#[allow(clippy::needless_pass_by_value)]
fn handle_particle_registration(
    mut commands: Commands,
    blueprint_query: Query<BlueprintQuery<'_>, With<ParticleType>>,
    mut ev_particle_registered: EventReader<ParticleRegistrationEvent>,
    particle_query: Query<&ChildOf, With<Particle>>,
) {
    ev_particle_registered.read().for_each(|ev| {
        ev.entities.iter().for_each(|entity| {
            if let Ok(child_of) = particle_query.get(*entity) {
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
                )) = blueprint_query.get(child_of.parent())
                {
                    if let Some(density) = density {
                        commands.entity(*entity).insert(density.0);
                    } else {
                        commands.entity(*entity).remove::<Density>();
                    }
                    if let Some(velocity) = velocity {
                        commands.entity(*entity).insert(velocity.0);
                    } else {
                        commands.entity(*entity).remove::<Velocity>();
                    }
                    if let Some(movement_priority) = movement_priority {
                        commands.entity(*entity).insert(movement_priority.0.clone());
                    } else {
                        commands.entity(*entity).remove::<MovementPriority>();
                    }
                    if let Some(momentum) = momentum {
                        commands.entity(*entity).insert(momentum.0);
                    } else {
                        commands.entity(*entity).remove::<Momentum>();
                    }
                    if wall.is_some() {
                        commands.entity(*entity).insert(Wall);
                        commands.entity(*entity).insert(Moved(false));
                    } else {
                        commands.entity(*entity).remove::<Wall>();
                        commands.entity(*entity).remove::<Moved>();
                    }
                    if let Some(liquid) = liquid {
                        commands.entity(*entity).insert(liquid.0.clone());
                        commands.entity(*entity).insert(Moved(true));
                    } else {
                        commands.entity(*entity).remove::<Liquid>();
                        commands.entity(*entity).remove::<Moved>();
                    }
                    if let Some(gas) = gas {
                        commands.entity(*entity).insert(gas.0.clone());
                        commands.entity(*entity).insert(Moved(true));
                    } else {
                        commands.entity(*entity).remove::<Gas>();
                        commands.entity(*entity).remove::<Moved>();
                    }
                    if let Some(movable_solid) = movable_solid {
                        commands.entity(*entity).insert(movable_solid.0.clone());
                        commands.entity(*entity).insert(Moved(true));
                    } else {
                        commands.entity(*entity).remove::<MovableSolid>();
                        commands.entity(*entity).remove::<Moved>();
                    }
                    if let Some(solid) = solid {
                        commands.entity(*entity).insert(solid.0.clone());
                        commands.entity(*entity).insert(Moved(true));
                    } else {
                        commands.entity(*entity).remove::<Solid>();
                        commands.entity(*entity).remove::<Moved>();
                    }
                }
                commands.entity(*entity).insert(Moved(true));
            }
        });
    });
}
