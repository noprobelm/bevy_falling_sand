use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_turborand::RngComponent;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::iter;

use crate::{
    core::{ParticleRng, ParticleSyncExt},
    impl_particle_rng,
};

pub(super) struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_particle_sync_component::<Density>()
            .register_particle_sync_component::<Speed>()
            .register_particle_sync_component::<Momentum>()
            .register_particle_sync_component::<ParticleResistor>()
            .register_particle_sync_component::<AirResistance>()
            .register_particle_sync_component::<Movement>()
            .register_type::<MovementRng>()
            .register_type::<Density>()
            .register_type::<Speed>()
            .register_type::<Momentum>()
            .register_type::<NeighborGroup>()
            .register_type::<AirResistance>()
            .register_type::<Movement>()
            .register_type::<ParticleResistor>();
    }
}

impl_particle_rng!(MovementRng, RngComponent);

/// Provides RNG for particle movement systems.
///
/// Automatically inserted on entities that receive a [`Movement`] component.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::movement::MovementRng;
///
/// fn check_rng(query: Query<&MovementRng>) {
///     println!("Entities with movement RNG: {}", query.iter().len());
/// }
/// ```
#[derive(Component, Clone, Default, PartialEq, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct MovementRng(pub RngComponent);

/// The density of a particle, used for displacement comparisons.
///
/// When a moving particle encounters a neighbor, it will only displace the neighbor
/// if its density is strictly greater.
#[derive(
    Component,
    Copy,
    Clone,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component, Debug)]
#[type_path = "bfs_movement::particle"]
pub struct Density(pub u32);

/// Controls how many positions a particle can move per frame.
///
/// A particle starts at speed 1 and can accelerate up to [`Speed::max`]. Each consecutive
/// unobstructed move builds [`Speed::potential`]; once it reaches [`Speed::threshold`], the
/// particle's speed increments by 1 and potential resets. Swapping with another particle or
/// failing to move resets both speed and potential to their starting values.
///
/// With a threshold of `0`, a particle will increase speed each time it moves unobstructed until
/// it reaches [`Speed::max`].
///
/// # Examples
///
/// ```
/// use bevy_falling_sand::movement::Speed;
///
/// // This particle has a max speed of 10, stepping up speed after 5 unobstructed moves.
/// let speed = Speed::new(5, 10);
/// assert_eq!(speed.current(), 1u8);
/// assert_eq!(speed.max_speed(), 10u8);
/// ```
#[derive(
    Component,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component, Default)]
#[type_path = "bfs_movement::particle"]
pub struct Speed {
    current: u8,
    potential: u8,
    threshold: u8,
    max: u8,
}

impl Default for Speed {
    fn default() -> Self {
        Self::new(1, 1)
    }
}

impl Speed {
    /// Initialize a speed with the given threshold and max.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_falling_sand::movement::Speed;
    ///
    /// let speed = Speed::new(5, 10);
    /// assert_eq!(speed.threshold(), 5u8);
    /// assert_eq!(speed.max_speed(), 10u8);
    /// assert_eq!(speed.current(), 1u8);
    /// ```
    #[inline(always)]
    #[must_use]
    pub const fn new(threshold: u8, max: u8) -> Self {
        Self {
            current: 1,
            potential: 0,
            threshold,
            max,
        }
    }

    /// Get the current speed value.
    #[inline(always)]
    #[must_use]
    pub const fn current(&self) -> u8 {
        self.current
    }

    /// Get the current potential value.
    #[inline(always)]
    #[must_use]
    pub const fn potential(&self) -> u8 {
        self.potential
    }

    /// Get the speed threshold value.
    #[inline(always)]
    #[must_use]
    pub const fn threshold(&self) -> u8 {
        self.threshold
    }

    /// Get the max speed value.
    #[inline(always)]
    #[must_use]
    pub const fn max_speed(&self) -> u8 {
        self.max
    }

    /// Set the current speed.
    #[inline(always)]
    pub const fn set_speed(&mut self, val: u8) {
        self.current = val;
    }

    /// Set the speed threshold.
    #[inline(always)]
    pub const fn set_threshold(&mut self, val: u8) {
        self.threshold = val;
    }

    /// Set the maximum speed. Clamps to a minimum of 1.
    #[inline(always)]
    pub const fn set_max_speed(&mut self, val: u8) {
        if val < 1 {
            self.max = 1;
        } else {
            self.max = val;
        }
    }

    /// Increment the speed by 1 if below max and potential meets threshold.
    #[inline(always)]
    pub const fn increment(&mut self) {
        if self.current < self.max {
            self.potential += 1;
            if self.potential >= self.threshold {
                self.current += 1;
                self.potential = 0;
            }
        }
    }

    /// Decrement the speed by 1 if above 0.
    #[inline(always)]
    pub const fn decrement(&mut self) {
        if self.current > 0 {
            self.potential = 0;
            self.current -= 1;
        }
    }
}

/// Insert momentum on a particle to provide `bfs` movement systems a directional hint that biases
/// movement towards the last successful movement direction.
#[derive(
    Component, Copy, Clone, Default, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
#[type_path = "bfs_movement::particle"]
pub struct Momentum(pub IVec2);

impl Momentum {
    /// Zero momentum.
    pub const ZERO: Self = Self(IVec2::splat(0));
}

/// How much this particle resists being displaced by another particle swapping into its
/// position. A value of 0.0 means no resistance; 1.0 means the particle can never be
/// displaced.
///
/// When a moving particle attempts to swap with a neighbor that has `ParticleResistor`, the swap
/// is probabilistically skipped based on this value.
///
/// # Examples
///
/// ```
/// use bevy_falling_sand::movement::ParticleResistor;
///
/// let r = ParticleResistor(0.5);
/// assert_eq!(r.0, 0.5);
/// ```
#[derive(Component, Copy, Clone, Default, PartialEq, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
#[type_path = "bfs_movement::particle"]
pub struct ParticleResistor(pub f64);

/// Defines an ordered group of relative neighbor positions used to evaluate
/// particle movement within a single priority tier.
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use bevy_falling_sand::movement::NeighborGroup;
///
/// let group = NeighborGroup::empty();
/// assert!(group.is_empty());
/// ```
#[derive(Clone, Default, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
#[type_path = "bfs_movement::particle"]
pub struct NeighborGroup {
    /// The underlying neighbor group.
    pub neighbor_group: SmallVec<[IVec2; 4]>,
}

impl NeighborGroup {
    /// Initialize a new `NeighborGroup`.
    #[inline(always)]
    #[must_use]
    pub const fn new(neighbor_group: SmallVec<[IVec2; 4]>) -> Self {
        Self { neighbor_group }
    }

    /// Initialize an empty `NeighborGroup`.
    #[inline(always)]
    #[must_use]
    pub fn empty() -> Self {
        Self {
            neighbor_group: SmallVec::new(),
        }
    }

    /// Push a new neighbor to the back.
    #[inline(always)]
    pub fn push(&mut self, neighbor: IVec2) {
        self.neighbor_group.push(neighbor);
    }

    /// Swap the position of two indices with one another.
    ///
    /// # Errors
    ///
    /// Returns an error if either index is out of bounds.
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

    /// Returns true if empty.
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.neighbor_group.is_empty()
    }

    /// Returns the length.
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.neighbor_group.len()
    }

    pub(crate) fn iter_candidates<'a>(
        &'a self,
        rng: &mut MovementRng,
        preferred: Option<&Momentum>,
    ) -> NeighborGroupIter<'a> {
        if let Some(momentum) = preferred
            && let Some(position) = self
                .neighbor_group
                .iter()
                .position(|&candidate| momentum.0 == candidate)
        {
            return NeighborGroupIter::Single(iter::once(&self.neighbor_group[position]));
        }

        let len = self.neighbor_group.len();
        let start = if len > 1 { rng.index(0..len) } else { 0 };
        NeighborGroupIter::Offset {
            slice: &self.neighbor_group,
            start,
            count: 0,
        }
    }
}

#[allow(clippy::redundant_pub_crate)]
pub(crate) enum NeighborGroupIter<'a> {
    Single(iter::Once<&'a IVec2>),
    Offset {
        slice: &'a [IVec2],
        start: usize,
        count: usize,
    },
}

impl<'a> Iterator for NeighborGroupIter<'a> {
    type Item = &'a IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            NeighborGroupIter::Single(iter) => iter.next(),
            NeighborGroupIter::Offset {
                slice,
                start,
                count,
            } => {
                if *count >= slice.len() {
                    return None;
                }
                let idx = (*start + *count) % slice.len();
                *count += 1;
                Some(&slice[idx])
            }
        }
    }
}

/// Per-tier air resistance values (parallel to `Movement::neighbor_groups`).
///
/// When evaluating tier `i`, if `AirResistance[i] > 0.0` and a random check succeeds,
/// the movement attempt into empty space is skipped.
///
/// # Examples
///
/// ```
/// use bevy_falling_sand::movement::AirResistance;
///
/// let resistance = AirResistance::new([0.0, 0.3]);
/// assert_eq!(resistance.get(0), Some(0.0));
/// assert_eq!(resistance.get(1), Some(0.3));
/// ```
#[derive(Component, Clone, Default, PartialEq, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
#[type_path = "bfs_movement::particle"]
pub struct AirResistance {
    resistances: SmallVec<[f64; 8]>,
}

impl AirResistance {
    /// Initialize a new `AirResistance`.
    pub fn new(resistances: impl IntoIterator<Item = f64>) -> Self {
        Self {
            resistances: resistances.into_iter().collect(),
        }
    }

    /// Get the resistance value at an index.
    #[must_use]
    pub fn get(&self, idx: usize) -> Option<f64> {
        self.resistances.get(idx).copied()
    }

    /// Set the resistance value at an index.
    pub fn set(&mut self, idx: usize, value: f64) {
        if let Some(slot) = self.resistances.get_mut(idx) {
            *slot = value;
        }
    }

    /// Get the number of resistance values.
    #[must_use]
    pub fn len(&self) -> usize {
        self.resistances.len()
    }

    /// Returns `true` if there are no resistance values.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.resistances.is_empty()
    }

    /// Iterate through the resistance values.
    pub fn iter(&self) -> impl Iterator<Item = &f64> {
        self.resistances.iter()
    }

    /// Resize to len, padding with 0.0.
    pub(crate) fn resize(&mut self, len: usize) {
        self.resistances.resize(len, 0.0);
    }
}

/// `Movement` defines relative movement candidate positions for a particle. It relies on the
/// presence of 3 additional required components to function:
/// - [`Density`]
/// - [`Speed`]
/// - [`AirResistance`]
///
/// # Examples
///
/// A "Sand" particle which slowly drifts through "Water" might look like this.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_falling_sand::prelude::{ParticleType, ColorProfile, Movement, Density, Speed, AirResistance, ParticleResistor};
///
/// fn setup(mut commands: Commands) {
///     commands.spawn((
///         ParticleType::new("Sand"),
///         ColorProfile::palette(vec![
///             Color::Srgba(Srgba::hex("#FFEB8A").unwrap()),
///             Color::Srgba(Srgba::hex("#F2E06B").unwrap()),
///         ]),
///         // First tier: look directly below. Second tier: look diagonally down.
///         Movement::from(vec![
///             vec![IVec2::NEG_Y],
///             vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
///         ]),
///         // This particle will swap positions with movement candidates of lower densities.
///         Density(1250),
///         // Defines a speed increase threshold of `5`, with a max speed of `10`. When this
///         // particle moves unobstructed `5` consecutive times, it will increment its speed by
///         // `1`, creating an acceleration-like effect.
///         Speed::new(5, 10),
///         // Sets a 90% chance per frame that particles will resist moving diagonally down
///         AirResistance::new([0.0, 0.9]),
///     ));
///     commands.spawn((
///         ParticleType::new("Water"),
///         ColorProfile::palette(vec![
///             Color::Srgba(Srgba::hex("#0B80AB80").unwrap()),
///         ]),
///         // Similar to sand, but also looks as far as 5 spaces in both directions along the X
///         // axis to emulate fluidlike behavior
///         Movement::from(vec![
///             vec![IVec2::NEG_Y],
///             vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
///             vec![IVec2::new(1, 0), IVec2::new(-1, 0)],
///             vec![IVec2::new(2, 0), IVec2::new(-2, 0)],
///             vec![IVec2::new(3, 0), IVec2::new(-3, 0)],
///             vec![IVec2::new(4, 0), IVec2::new(-4, 0)],
///             vec![IVec2::new(5, 0), IVec2::new(-5, 0)],
///         ]),
///         // The density of water is less than sand, so sand will pass through it.
///         Density(750),
///         // This particle has no speed increase threshold; it will simply max out at 3 moves per
///         // frame.
///         Speed::new(0, 3),
///         // There is a 75% chance this particle will deny movement of other particles attempting
///         // to pass through it.
///         ParticleResistor(0.75)
///     ));
/// }
/// ```
#[derive(
    Component, Clone, Default, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize,
)]
#[component(on_add = Movement::on_add)]
#[require(AirResistance)]
#[require(Speed)]
#[require(Density)]
#[reflect(Component)]
#[type_path = "bfs_movement::particle"]
pub struct Movement {
    /// The underlying groups of neighbors that define the movement priority.
    pub neighbor_groups: SmallVec<[NeighborGroup; 8]>,
}

impl From<Vec<Vec<IVec2>>> for Movement {
    fn from(value: Vec<Vec<IVec2>>) -> Self {
        Self::new(
            value
                .into_iter()
                .map(|neighbor_group| NeighborGroup::new(SmallVec::from_vec(neighbor_group)))
                .collect::<SmallVec<[NeighborGroup; 8]>>(),
        )
    }
}

impl Movement {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        if !world.entity(context.entity).contains::<MovementRng>() {
            world
                .commands()
                .entity(context.entity)
                .insert(MovementRng::default());
        }
    }
}

impl Movement {
    /// Initialize a new `Movement` with the specified neighbor groups.
    #[inline(always)]
    #[must_use]
    pub const fn new(neighbor_groups: SmallVec<[NeighborGroup; 8]>) -> Self {
        Self { neighbor_groups }
    }

    /// Initialize an empty `Movement`.
    #[inline(always)]
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            neighbor_groups: SmallVec::new_const(),
        }
    }

    /// Returns true if the movement holds no neighbor groups.
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.neighbor_groups.is_empty()
    }

    /// Returns the number of neighbor groups.
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.neighbor_groups.len()
    }

    /// Push a new tier of neighbors.
    pub fn push_outer(&mut self, neighbor_group: NeighborGroup) {
        self.neighbor_groups.push(neighbor_group);
    }

    /// Push a neighbor into an existing tier.
    ///
    /// # Errors
    ///
    /// Returns an error if the group index is out of bounds.
    pub fn push_inner(&mut self, group_index: usize, neighbor: IVec2) -> Result<(), String> {
        self.neighbor_groups
            .get_mut(group_index)
            .map(|group| {
                group.push(neighbor);
            })
            .ok_or_else(|| format!("Group index {group_index} out of bounds"))
    }

    /// Returns positions within the 8-cell Moore neighborhood, in priority order.
    pub fn moore_neighbors(&self) -> impl Iterator<Item = IVec2> + '_ {
        self.neighbor_groups
            .iter()
            .flat_map(|group| group.neighbor_group.iter())
            .filter(|offset| offset.x.abs() <= 1 && offset.y.abs() <= 1)
            .copied()
    }

    /// Iterate through candidate positions across all tiers.
    pub fn iter_candidates<'a>(
        &'a self,
        rng: &'a mut MovementRng,
        momentum: Option<&'a Momentum>,
    ) -> impl Iterator<Item = (usize, &'a IVec2)> + 'a {
        self.neighbor_groups
            .iter()
            .enumerate()
            .flat_map(move |(tier, neighbor_group)| {
                neighbor_group
                    .iter_candidates(rng, momentum)
                    .map(move |pos| (tier, pos))
            })
    }

    /// Get a mutable reference to a neighbor group by index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut NeighborGroup> {
        self.neighbor_groups.get_mut(index)
    }

    /// Remove and return a neighbor group by index.
    pub fn remove(&mut self, index: usize) -> Option<NeighborGroup> {
        if index < self.neighbor_groups.len() {
            Some(self.neighbor_groups.remove(index))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod speed_tests {
        use super::*;

        #[test]
        fn new_starts_at_current_1() {
            let speed = Speed::new(3, 10);
            assert_eq!(speed.current(), 1);
            assert_eq!(speed.threshold(), 3);
            assert_eq!(speed.max_speed(), 10);
            assert_eq!(speed.potential(), 0);
        }

        #[test]
        fn default_is_1_1() {
            let speed = Speed::default();
            assert_eq!(speed.current(), 1);
            assert_eq!(speed.max_speed(), 1);
            assert_eq!(speed.threshold(), 1);
        }

        #[test]
        fn increment_accumulates_potential() {
            let mut speed = Speed::new(3, 10);
            speed.increment();
            assert_eq!(speed.current(), 1);
            assert_eq!(speed.potential(), 1);

            speed.increment();
            assert_eq!(speed.current(), 1);
            assert_eq!(speed.potential(), 2);

            speed.increment();
            assert_eq!(speed.current(), 2);
            assert_eq!(speed.potential(), 0);
        }

        #[test]
        fn increment_caps_at_max() {
            let mut speed = Speed::new(1, 2);
            speed.increment();
            assert_eq!(speed.current(), 2);

            speed.increment();
            assert_eq!(speed.current(), 2);
        }

        #[test]
        fn decrement_resets_potential() {
            let mut speed = Speed::new(3, 10);
            speed.increment();
            speed.increment();
            assert_eq!(speed.potential(), 2);

            speed.decrement();
            assert_eq!(speed.current(), 0);
            assert_eq!(speed.potential(), 0);
        }

        #[test]
        fn decrement_floors_at_zero() {
            let mut speed = Speed::new(1, 5);
            speed.decrement();
            assert_eq!(speed.current(), 0);

            speed.decrement();
            assert_eq!(speed.current(), 0);
        }

        #[test]
        fn set_speed_overrides_current() {
            let mut speed = Speed::new(1, 10);
            speed.set_speed(5);
            assert_eq!(speed.current(), 5);
        }

        #[test]
        fn set_max_speed_clamps_to_1() {
            let mut speed = Speed::new(1, 10);
            speed.set_max_speed(0);
            assert_eq!(speed.max_speed(), 1);

            speed.set_max_speed(5);
            assert_eq!(speed.max_speed(), 5);
        }
    }

    mod neighbor_group_tests {
        use super::*;

        #[test]
        fn empty_is_empty() {
            let group = NeighborGroup::empty();
            assert!(group.is_empty());
            assert_eq!(group.len(), 0);
        }

        #[test]
        fn push_adds_neighbor() {
            let mut group = NeighborGroup::empty();
            group.push(IVec2::NEG_Y);
            assert_eq!(group.len(), 1);
            assert!(!group.is_empty());
            assert_eq!(group.neighbor_group[0], IVec2::NEG_Y);
        }

        #[test]
        fn swap_within_bounds() {
            let mut group = NeighborGroup::empty();
            group.push(IVec2::NEG_Y);
            group.push(IVec2::Y);
            assert!(group.swap(0, 1).is_ok());
            assert_eq!(group.neighbor_group[0], IVec2::Y);
            assert_eq!(group.neighbor_group[1], IVec2::NEG_Y);
        }

        #[test]
        fn swap_out_of_bounds_returns_error() {
            let mut group = NeighborGroup::empty();
            group.push(IVec2::ZERO);
            assert!(group.swap(0, 5).is_err());
        }
    }

    mod air_resistance_tests {
        use super::*;

        #[test]
        fn new_from_iter() {
            let ar = AirResistance::new([0.0, 0.5, 1.0]);
            assert_eq!(ar.len(), 3);
            assert_eq!(ar.get(0), Some(0.0));
            assert_eq!(ar.get(1), Some(0.5));
            assert_eq!(ar.get(2), Some(1.0));
            assert_eq!(ar.get(3), None);
        }

        #[test]
        fn set_updates_value() {
            let mut ar = AirResistance::new([0.0, 0.0]);
            ar.set(1, 0.75);
            assert_eq!(ar.get(1), Some(0.75));
        }

        #[test]
        fn set_out_of_bounds_is_noop() {
            let mut ar = AirResistance::new([0.0]);
            ar.set(5, 1.0);
            assert_eq!(ar.len(), 1);
        }

        #[test]
        fn resize_pads_with_zero() {
            let mut ar = AirResistance::new([0.5]);
            ar.resize(3);
            assert_eq!(ar.len(), 3);
            assert_eq!(ar.get(0), Some(0.5));
            assert_eq!(ar.get(1), Some(0.0));
            assert_eq!(ar.get(2), Some(0.0));
        }

        #[test]
        fn resize_truncates() {
            let mut ar = AirResistance::new([0.1, 0.2, 0.3]);
            ar.resize(1);
            assert_eq!(ar.len(), 1);
            assert_eq!(ar.get(0), Some(0.1));
        }

        #[test]
        fn is_empty_when_no_values() {
            let ar = AirResistance::new([]);
            assert!(ar.is_empty());
        }
    }

    mod movement_tests {
        use super::*;

        #[test]
        fn empty_has_no_groups() {
            let m = Movement::empty();
            assert!(m.is_empty());
            assert_eq!(m.len(), 0);
        }

        #[test]
        fn from_vec_creates_groups() {
            let m = Movement::from(vec![
                vec![IVec2::NEG_Y],
                vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
            ]);
            assert_eq!(m.len(), 2);
            assert_eq!(m.neighbor_groups[0].len(), 1);
            assert_eq!(m.neighbor_groups[1].len(), 2);
        }

        #[test]
        fn push_outer_adds_tier() {
            let mut m = Movement::empty();
            m.push_outer(NeighborGroup::empty());
            assert_eq!(m.len(), 1);
        }

        #[test]
        fn push_inner_adds_to_tier() {
            let mut m = Movement::from(vec![vec![IVec2::NEG_Y]]);
            assert!(m.push_inner(0, IVec2::Y).is_ok());
            assert_eq!(m.neighbor_groups[0].len(), 2);
        }

        #[test]
        fn push_inner_out_of_bounds() {
            let mut m = Movement::empty();
            assert!(m.push_inner(0, IVec2::ZERO).is_err());
        }

        #[test]
        fn remove_returns_group() {
            let mut m = Movement::from(vec![vec![IVec2::NEG_Y], vec![IVec2::Y]]);
            let removed = m.remove(0);
            assert!(removed.is_some());
            assert_eq!(m.len(), 1);
        }

        #[test]
        fn remove_out_of_bounds_returns_none() {
            let mut m = Movement::empty();
            assert!(m.remove(0).is_none());
        }

        #[test]
        fn moore_neighbors_filters_to_adjacent() {
            let m = Movement::from(vec![
                vec![IVec2::new(0, -2)],
                vec![IVec2::NEG_Y],
                vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
            ]);
            let neighbors: Vec<_> = m.moore_neighbors().collect();
            assert_eq!(neighbors.len(), 3);
            assert!(!neighbors.contains(&IVec2::new(0, -2)));
        }

        #[test]
        fn get_mut_returns_group() {
            let mut m = Movement::from(vec![vec![IVec2::NEG_Y]]);
            let group = m.get_mut(0).unwrap();
            group.push(IVec2::Y);
            assert_eq!(m.neighbor_groups[0].len(), 2);
        }
    }
}
