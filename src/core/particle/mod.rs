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
use bevy_turborand::DelegatedRng;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, ops::RangeBounds};

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
    Name(String),
    Position(IVec2),
    Entity(Entity),
}

/// Define an entity as a `ParticleType`.
///
/// `ParticleType` is a linchpin in particle synchronization and lifecycle management routines.
/// When a [`ParticleType`] component is inserted or changed on an entity, it is synchronized
/// [`ParticleTypeRegistry`] and thus made available for lookup.
///
/// When new [`Particle`] entities are spawned into the world, they will locate their parent in
/// the [`ParticleTypeRegistry`] and store it for use at future synchronization points.
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
    pub name: Cow<'static, str>,
}

impl ParticleType {
    /// Synchronize the new `ParticleType` with the [`ParticleTypeRegistry`].
    ///
    /// If a previous entity was registered under the same name, it is despawned.
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let particle_type = world.get::<Self>(context.entity).unwrap();
        let name = particle_type.name.clone();

        let mut type_map = world.resource_mut::<ParticleTypeRegistry>();
        let old_entity = type_map.insert(name, context.entity);

        if let Some(old) = old_entity
            && old != context.entity
        {
            world.commands().entity(old).despawn();
        }
    }

    /// Remove this `ParticleType` from the [`ParticleTypeRegistry`], but only if
    /// it is still the registered entity for its name. This avoids clobbering a
    /// replacement that was already inserted by [`on_add`](ParticleType::on_add).
    fn on_remove(mut world: DeferredWorld, context: HookContext) {
        let particle_type = world.get::<Self>(context.entity).unwrap();
        let name = particle_type.name.clone();
        let mut type_map = world.resource_mut::<ParticleTypeRegistry>();
        if type_map.get(&name) == Some(&context.entity) {
            type_map.remove(&name);
        }
    }
}

impl ParticleType {
    /// Initialize a new [`ParticleType`] from a static string.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_falling_sand::core::ParticleType;
    ///
    /// let sand = ParticleType::new("Sand");
    /// assert_eq!(sand.name, "Sand");
    /// ```
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
        }
    }

    /// Initialize a new [`ParticleType`] from an owned string.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_falling_sand::core::ParticleType;
    ///
    /// let name = String::from("Water");
    /// let water = ParticleType::from_string(name);
    /// assert_eq!(water.name, "Water");
    /// ```
    #[must_use]
    pub const fn from_string(name: String) -> Self {
        Self {
            name: Cow::Owned(name),
        }
    }
}

impl From<&'static str> for ParticleType {
    fn from(name: &'static str) -> Self {
        Self::new(name)
    }
}

impl From<String> for ParticleType {
    fn from(name: String) -> Self {
        Self::from_string(name)
    }
}

impl From<Cow<'static, str>> for ParticleType {
    fn from(name: Cow<'static, str>) -> Self {
        Self { name }
    }
}

impl From<ParticleType> for Cow<'static, str> {
    fn from(val: ParticleType) -> Self {
        val.name
    }
}

/// Marker component for entities participating in the falling sand simulation.
///
/// `Particle` is a zero-sized component. The "type" of a particle is identified entirely by its
/// [`AttachedToParticleType`] reference, which points at the [`ParticleType`] entity that holds
/// the canonical name and shared-default behavior. To read a particle's type name, query its
/// parent:
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::core::{AttachedToParticleType, Particle, ParticleType};
///
/// fn read_names(
///     particles: Query<&AttachedToParticleType, With<Particle>>,
///     types: Query<&ParticleType>,
/// ) {
///     for attached in &particles {
///         if let Ok(particle_type) = types.get(attached.0) {
///             println!("{}", particle_type.name);
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
/// use bevy_turborand::prelude::*;
/// use bevy_falling_sand::impl_particle_rng;
///
/// #[derive(Component)]
/// struct MyRng(RngComponent);
///
/// impl_particle_rng!(MyRng, RngComponent);
/// ```
pub trait ParticleRng: Component {
    /// The type of the internal RNG
    type InnerRng: DelegatedRng;

    /// Get mutable access to the inner RNG.
    fn inner_mut(&mut self) -> &mut Self::InnerRng;

    /// Shuffle the given slice.
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        self.inner_mut().shuffle(slice);
    }

    /// Return true with the given probability.
    fn chance(&mut self, rate: f64) -> bool {
        self.inner_mut().chance(rate)
    }

    /// Sample a random element from a list.
    fn sample<'a, T>(&mut self, list: &'a [T]) -> Option<&'a T> {
        self.inner_mut().sample(list)
    }

    /// Return a random index within the given bounds.
    fn index(&mut self, bound: impl RangeBounds<usize>) -> usize {
        self.inner_mut().index(bound)
    }
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
