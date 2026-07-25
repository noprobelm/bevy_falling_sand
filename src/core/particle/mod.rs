//! Components and resources central to particle behavior.

pub mod lifecycle;
pub mod schedule;
pub mod sync;

mod particle_map;
mod registry;

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use rand_core::Rng;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    ops::Bound,
    ops::RangeBounds,
    sync::atomic::{AtomicUsize, Ordering},
};

pub use lifecycle::*;
pub use particle_map::*;
pub use registry::*;
pub use schedule::*;
pub use sync::*;

use registry::RegistryPlugin;

pub(super) struct ParticlePlugin {
    pub width: u32,
    pub height: u32,
    pub origin: IVec2,
}

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ParticleMap::new(self.width, self.height, self.origin))
            .register_type::<ParticleTypeId>()
            .register_type::<ParticleType>()
            .register_type::<Particle>()
            .register_type::<GridPosition>()
            .add_plugins((
                schedule::SchedulePlugin,
                RegistryPlugin,
                LifecyclePlugin,
                SyncPlugin,
            ));
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub(crate) enum LocateBy {
    ParticleType(ParticleTypeId),
    Position(IVec2),
    Entity(Entity),
}

static NEXT_PARTICLE_TYPE_ID: AtomicUsize = AtomicUsize::new(0);

/// Unique identifier for a [`ParticleType`].
///
/// `ParticleTypeId` is the value to store in bevy resources, or anywhere else a particle type's
/// unique identiifer might be useful. It is the stable handle through which a corresponding
/// [`ParticleType`] entity can be accessed via the [`ParticleTypeRegistry`] resource.
///
/// Internally, an [`AtomicUsize`] counter is used to ensure IDs remain stable and aren't
/// duplicately assigned. As such, `ParticleTypeId::from_raw` exists to allow users to reuse IDs
/// between sessions without compromising the internal counter logic. One example of such use might
/// be to store a [`ParticleTypeId`] to particle name mapper on disk, referencing the UIDs
/// when spawning particle types during your app's initialization logic.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Reflect, Serialize)]
#[serde(transparent)]
#[reflect(Serialize, Deserialize)]
pub struct ParticleTypeId(usize);

impl ParticleTypeId {
    /// Allocate a new unique particle type identifier.
    ///
    /// Use this for ordinary runtime particle type creation. Store the returned ID wherever you
    /// will later need to spawn, mutate, despawn, or otherwise refer to the type.
    #[must_use]
    pub fn new() -> Self {
        Self(NEXT_PARTICLE_TYPE_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Return the underlying numeric value.
    ///
    /// This is useful for diagnostics, UI display, or external formats that need a primitive
    /// integer. Prefer passing [`ParticleTypeId`] itself through Rust APIs.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }

    /// Build an ID from a persisted numeric value and reserve it from future allocation.
    ///
    /// Use this for deserialization, migrations, and stable external catalogs where the numeric
    /// value is already part of a file or asset contract. For normal runtime allocation, use
    /// [`ParticleTypeId::new`] instead.
    #[must_use]
    pub fn from_raw(id: usize) -> Self {
        Self::reserve_loaded(id);
        Self(id)
    }

    fn reserve_loaded(id: usize) {
        let next = id.saturating_add(1);
        let mut current = NEXT_PARTICLE_TYPE_ID.load(Ordering::Relaxed);
        while current < next {
            match NEXT_PARTICLE_TYPE_ID.compare_exchange_weak(
                current,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }
}

impl Default for ParticleTypeId {
    fn default() -> Self {
        Self::new()
    }
}

impl<'de> Deserialize<'de> for ParticleTypeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = usize::deserialize(deserializer)?;
        Self::reserve_loaded(id);
        Ok(Self(id))
    }
}

/// Define an entity as a particle type template.
///
/// `ParticleType` is the ECS template component for particle synchronization and lifecycle
/// management routines. When a [`ParticleType`] component is inserted or changed on an entity, its
/// [`ParticleTypeId`] is synchronized with [`ParticleTypeRegistry`] and thus made available for
/// lookup.
///
/// The ID is private by design. Use [`ParticleType::id`] to read it, [`ParticleType::new`] to
/// allocate a fresh template, and [`ParticleType::from_id`] to attach an existing ID to a template
/// entity.
///
/// When new [`Particle`] entities are spawned into the world, they locate their parent template in
/// the [`ParticleTypeRegistry`] by [`ParticleTypeId`] and store the parent entity for use at future
/// synchronization points.
#[derive(
    Component,
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
#[component(on_add = ParticleType::on_add)]
#[component(on_remove = ParticleType::on_remove)]
#[reflect(Component)]
#[type_path = "bfs_core::particle"]
pub struct ParticleType {
    /// The particle type's unique identifier.
    id: ParticleTypeId,
}

impl ParticleType {
    /// Synchronize the new `ParticleType` with the [`ParticleTypeRegistry`].
    ///
    /// If a previous entity was registered under the same ID, it is despawned.
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let particle_type = world.get::<Self>(context.entity).unwrap();
        let id = particle_type.id();

        let mut type_map = world.resource_mut::<ParticleTypeRegistry>();
        let old_entity = type_map.insert(id, context.entity);

        if let Some(old) = old_entity
            && old != context.entity
        {
            world.commands().entity(old).despawn();
        }
    }

    /// Remove this `ParticleType` from the [`ParticleTypeRegistry`], but only if
    /// it is still the registered entity for its ID. This avoids clobbering a
    /// replacement that was already inserted by [`on_add`](ParticleType::on_add).
    fn on_remove(mut world: DeferredWorld, context: HookContext) {
        let particle_type = world.get::<Self>(context.entity).unwrap();
        let id = particle_type.id();
        let mut type_map = world.resource_mut::<ParticleTypeRegistry>();
        if type_map.get(id) == Some(&context.entity) {
            type_map.remove(id);
        }
    }
}

impl ParticleType {
    /// Initialize a new [`ParticleType`] with a unique ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_falling_sand::core::ParticleType;
    ///
    /// let sand = ParticleType::new();
    /// let sand_id = sand.id();
    /// let water = ParticleType::new();
    ///
    /// assert_ne!(sand.id(), water.id());
    /// assert_eq!(sand.id(), sand_id);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: ParticleTypeId::new(),
        }
    }

    /// Initialize a [`ParticleType`] from an existing ID.
    #[must_use]
    pub const fn from_id(id: ParticleTypeId) -> Self {
        Self { id }
    }

    /// Return the particle type's unique identifier.
    #[must_use]
    pub const fn id(&self) -> ParticleTypeId {
        self.id
    }
}

impl From<ParticleTypeId> for ParticleType {
    fn from(id: ParticleTypeId) -> Self {
        Self::from_id(id)
    }
}

/// Marker component for entities participating in the falling sand simulation.
///
/// `Particle` is a zero-sized component. The "type" of a particle is identified by its
/// [`AttachedToParticleType`] reference, which points at the [`ParticleType`] entity that holds
/// the canonical [`ParticleTypeId`] and shared-default behavior.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::core::{AttachedToParticleType, Particle, ParticleType};
///
/// fn read_particle_type_ids(
///     particles: Query<&AttachedToParticleType, With<Particle>>,
///     types: Query<&ParticleType>,
/// ) {
///     for attached in &particles {
///         if let Ok(particle_type) = types.get(attached.0) {
///             println!("{:?}", particle_type.id());
///         }
///     }
/// }
/// ```
///
/// To re-type a particle (change which [`ParticleType`] it belongs to), look up the new parent
/// in [`ParticleTypeRegistry`] and assign its entity directly to [`AttachedToParticleType`].
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
#[component(on_remove = Particle::on_remove)]
#[reflect(Component)]
#[type_path = "bfs_core::particle"]
pub struct Particle;

impl Particle {
    fn on_remove(mut world: DeferredWorld, context: HookContext) {
        let Some(grid_position) = world.get::<GridPosition>(context.entity) else {
            return;
        };
        let position = grid_position.0;

        let mut map = world.resource_mut::<ParticleMap>();
        if map.get_copied(position) != Ok(Some(context.entity)) {
            return;
        }
        let _ = map.remove(position);

        let chunk_index = world.resource::<crate::core::ChunkIndex>();
        let chunk_coord = chunk_index.world_to_chunk_coord(position);
        let Some(chunk_entity) = chunk_index.get(chunk_coord) else {
            return;
        };

        if let Some(mut dirty_state) = world.get_mut::<crate::core::ChunkDirtyState>(chunk_entity) {
            dirty_state.mark_dirty(position);
        }
    }
}

/// Stores the position of a particle in the simulation.
///
/// This component acts a little bit like [`Transform`], but is strictly for use in discrete 2d
/// space.
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use bevy_falling_sand::core::GridPosition;
///
/// let pos = GridPosition(IVec2::new(10, 20));
/// assert_eq!(pos.0, IVec2::new(10, 20));
/// ```
#[derive(
    Component, Copy, Clone, Default, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize,
)]
#[serde(transparent)]
#[reflect(Component)]
pub struct GridPosition(pub IVec2);

/// Component that tracks which [`ParticleType`] entity a [`Particle`] belongs to.
#[derive(Component, Copy, Clone)]
pub struct AttachedToParticleType(pub Entity);

/// A trait for common RNG utilities used in particle systems.
///
/// Use the `impl_particle_rng!` macro for a quick implementation on a newtype wrapper.
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use bevy_rand::prelude::WyRand;
/// use bevy_falling_sand::impl_particle_rng;
///
/// #[derive(Component)]
/// struct MyRng(WyRand);
///
/// impl_particle_rng!(MyRng, WyRand);
/// ```
pub trait ParticleRng: Component {
    /// The type of the internal RNG
    type InnerRng: Rng;

    /// Get mutable access to the inner RNG.
    fn inner_mut(&mut self) -> &mut Self::InnerRng;

    /// Shuffle the given slice.
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        shuffle(self.inner_mut(), slice);
    }

    /// Return true with the given probability.
    fn chance(&mut self, rate: f64) -> bool {
        chance(self.inner_mut(), rate)
    }

    /// Sample a random element from a list.
    fn sample<'a, T>(&mut self, list: &'a [T]) -> Option<&'a T> {
        (!list.is_empty()).then(|| &list[random_index(self.inner_mut(), 0..list.len())])
    }

    /// Return a random index within the given bounds.
    fn index(&mut self, bound: impl RangeBounds<usize>) -> usize {
        random_index(self.inner_mut(), bound)
    }
}

/// Shared RNG helpers for particle systems.
pub trait ParticleRngExt: Rng {
    /// Shuffle the given slice.
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        shuffle(self, slice);
    }

    /// Return true with the given probability.
    fn chance(&mut self, rate: f64) -> bool {
        chance(self, rate)
    }

    /// Return a random `u32` within the given bounds.
    fn u32(&mut self, bound: impl RangeBounds<u32>) -> u32 {
        random_u32(self, bound)
    }

    /// Return a random `u64` within the given bounds.
    fn u64(&mut self, bound: impl RangeBounds<u64>) -> u64 {
        random_u64(self, bound)
    }

    /// Return a random index within the given bounds.
    fn index(&mut self, bound: impl RangeBounds<usize>) -> usize {
        random_index(self, bound)
    }
}

impl<T: Rng + ?Sized> ParticleRngExt for T {}

fn chance(rng: &mut (impl Rng + ?Sized), rate: f64) -> bool {
    if rate <= 0.0 {
        return false;
    }
    if rate >= 1.0 {
        return true;
    }
    let sample = (rng.next_u64() as f64) / ((u64::MAX as f64) + 1.0);
    sample < rate
}

fn shuffle<T>(rng: &mut (impl Rng + ?Sized), slice: &mut [T]) {
    for i in (1..slice.len()).rev() {
        let j = random_below(rng, (i + 1) as u64) as usize;
        slice.swap(i, j);
    }
}

fn random_index(rng: &mut (impl Rng + ?Sized), bound: impl RangeBounds<usize>) -> usize {
    let (start, end) = range_bounds_to_start_end(bound, usize::MAX);
    assert!(start < end, "empty random index range");
    start + random_below(rng, (end - start) as u64) as usize
}

fn random_u32(rng: &mut (impl Rng + ?Sized), bound: impl RangeBounds<u32>) -> u32 {
    let (start, end) = range_bounds_to_start_end(bound, u32::MAX);
    assert!(start < end, "empty random u32 range");
    start + random_below(rng, u64::from(end - start)) as u32
}

fn random_u64(rng: &mut (impl Rng + ?Sized), bound: impl RangeBounds<u64>) -> u64 {
    let (start, end) = range_bounds_to_start_end(bound, u64::MAX);
    assert!(start < end, "empty random u64 range");
    start + random_below(rng, end - start)
}

fn random_below(rng: &mut (impl Rng + ?Sized), upper: u64) -> u64 {
    assert!(upper > 0, "empty random range");
    let zone = u64::MAX - (u64::MAX % upper);
    loop {
        let value = rng.next_u64();
        if value < zone {
            return value % upper;
        }
    }
}

fn range_bounds_to_start_end<T>(bound: impl RangeBounds<T>, max_exclusive: T) -> (T, T)
where
    T: Copy + From<u8> + PartialOrd + std::ops::Add<Output = T>,
{
    let start = match bound.start_bound() {
        Bound::Included(value) => *value,
        Bound::Excluded(value) => *value + T::from(1),
        Bound::Unbounded => T::from(0),
    };
    let end = match bound.end_bound() {
        Bound::Included(value) => *value + T::from(1),
        Bound::Excluded(value) => *value,
        Bound::Unbounded => max_exclusive,
    };
    (start, end)
}

/// Convenience macro for implementing [`ParticleRng`] on a component.
#[macro_export]
macro_rules! impl_particle_rng {
    ($wrapper:ident, $inner:ty) => {
        impl $crate::core::ParticleRng for $wrapper {
            type InnerRng = $inner;

            fn inner_mut(&mut self) -> &mut Self::InnerRng {
                &mut self.0
            }
        }
    };
}
