//! Components and resources central to particle behavior.

pub mod lifecycle;
pub mod schedule;
pub mod sync;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use crate::core::components::*;
pub use crate::core::resources::*;
pub use lifecycle::*;
pub use schedule::*;
pub use sync::*;
use systems::SystemsPlugin;

pub(super) struct ParticlePlugin {
    pub width: u32,
    pub height: u32,
    pub origin: IVec2,
}

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ParticleMap::new(self.width, self.height, self.origin))
            .add_plugins((
                schedule::SchedulePlugin,
                ComponentsPlugin,
                ResourcesPlugin,
                SystemsPlugin,
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

pub(super) mod components {
    use bevy::{
        ecs::{lifecycle::HookContext, world::DeferredWorld},
        prelude::*,
    };
    use bevy_turborand::DelegatedRng;
    use serde::{Deserialize, Serialize};
    use std::{borrow::Cow, ops::RangeBounds, time::Duration};

    use super::resources::ParticleMap;
    use crate::core::{ChunkDirtyState, ChunkIndex, ParticleTypeRegistry};

    pub(super) struct ComponentsPlugin;

    impl Plugin for ComponentsPlugin {
        fn build(&self, app: &mut App) {
            app.register_type::<ParticleType>()
                .register_type::<Particle>()
                .register_type::<GridPosition>();
        }
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
        /// replacement that was already inserted by [`on_add`].
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

    /// Define an entity as a `Particle`.
    ///
    /// `Particle` acts as a marker comopnent for particles which are actively being simulated.
    ///
    /// When a `Particle` changes its name, it will automatically synchronize with an associated
    /// [`ParticleType`] entity.
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
    #[component(on_remove = Particle::on_remove)]
    #[reflect(Component)]
    #[type_path = "bfs_core::particle"]
    pub struct Particle {
        /// The name of the particle, which corresponds to its [`ParticleType`] and can be used as an
        /// index in the  [`ParticleTypeRegistry`][crate::ParticleTypeRegistry] resource.
        pub name: Cow<'static, str>,
    }

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

            let chunk_index = world.resource::<ChunkIndex>();
            let chunk_coord = chunk_index.world_to_chunk_coord(position);
            let Some(chunk_entity) = chunk_index.get(chunk_coord) else {
                return;
            };

            if let Some(mut dirty_state) = world.get_mut::<ChunkDirtyState>(chunk_entity) {
                dirty_state.mark_dirty(position);
            }
        }

        /// Initialize a new `Particle` from a static string.
        ///
        /// # Examples
        ///
        /// ```
        /// use bevy_falling_sand::core::Particle;
        ///
        /// let sand = Particle::new("Sand");
        /// assert_eq!(sand.name, "Sand");
        /// ```
        #[must_use]
        pub const fn new(name: &'static str) -> Self {
            Self {
                name: Cow::Borrowed(name),
            }
        }

        /// Initialize a new `Particle` from an owned string.
        ///
        /// # Examples
        ///
        /// ```
        /// use bevy_falling_sand::core::Particle;
        ///
        /// let name = String::from("Water");
        /// let water = Particle::from_string(name);
        /// assert_eq!(water.name, "Water");
        /// ```
        #[must_use]
        pub const fn from_string(name: String) -> Self {
            Self {
                name: Cow::Owned(name),
            }
        }
    }

    impl From<&'static str> for Particle {
        fn from(name: &'static str) -> Self {
            Self::new(name)
        }
    }

    impl From<String> for Particle {
        fn from(name: String) -> Self {
            Self::from_string(name)
        }
    }

    impl From<Cow<'static, str>> for Particle {
        fn from(name: Cow<'static, str>) -> Self {
            Self { name }
        }
    }

    impl From<Particle> for Cow<'static, str> {
        fn from(value: Particle) -> Self {
            value.name
        }
    }

    impl From<ParticleType> for Particle {
        fn from(value: ParticleType) -> Self {
            Self { name: value.name }
        }
    }

    impl From<Particle> for ParticleType {
        fn from(value: Particle) -> Self {
            Self { name: value.name }
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

    /// A timed lifetime component that despawns the particle after a specified duration.
    #[derive(Component, Clone, Default, Eq, PartialEq, Debug, Reflect)]
    #[reflect(Component)]
    #[type_path = "bfs_core::particle"]
    pub struct TimedLifetime(pub Timer);

    impl TimedLifetime {
        /// Initialize a new lifetime with the given duration.
        ///
        /// # Examples
        ///
        /// ```
        /// use std::time::Duration;
        /// use bevy_falling_sand::core::TimedLifetime;
        ///
        /// let lifetime = TimedLifetime::new(Duration::from_secs(5));
        /// assert_eq!(lifetime.duration(), Duration::from_secs(5));
        /// assert!(!lifetime.finished());
        /// ```
        #[must_use]
        pub fn new(duration: Duration) -> Self {
            Self(Timer::new(duration, TimerMode::Once))
        }

        pub(crate) fn tick(&mut self, delta: Duration) {
            self.0.tick(delta);
        }

        /// Returns the duration of the lifetime timer.
        ///
        /// # Examples
        ///
        /// ```
        /// use std::time::Duration;
        /// use bevy_falling_sand::core::TimedLifetime;
        ///
        /// let lifetime = TimedLifetime::new(Duration::from_secs(5));
        /// assert_eq!(lifetime.duration(), Duration::from_secs(5));
        /// ```
        #[must_use]
        pub fn duration(&self) -> Duration {
            self.0.duration()
        }

        /// Returns true if the lifetime has expired.
        ///
        /// # Examples
        ///
        /// ```
        /// use std::time::Duration;
        /// use bevy_falling_sand::core::TimedLifetime;
        ///
        /// let lifetime = TimedLifetime::new(Duration::from_secs(5));
        /// assert!(!lifetime.finished());
        /// ```
        #[must_use]
        pub fn finished(&self) -> bool {
            self.0.is_finished()
        }
    }

    /// A chance-based lifetime component that has a chance to despawn the entity on a per-tick
    /// basis.
    #[derive(Component, Clone, PartialEq, Debug, Reflect)]
    #[reflect(Component)]
    #[type_path = "bfs_core::particle"]
    pub struct ChanceLifetime {
        /// The probability (0.0 to 1.0) that the particle will despawn each tick.
        pub chance: f64,
        /// Timer that controls how often the chance is evaluated.
        pub tick_timer: Timer,
    }

    impl Default for ChanceLifetime {
        fn default() -> Self {
            Self {
                chance: 0.0,
                tick_timer: Timer::new(Duration::ZERO, TimerMode::Repeating),
            }
        }
    }

    impl ChanceLifetime {
        /// Create a new chance-based lifetime with the given probability, evaluated every frame.
        ///
        /// # Examples
        ///
        /// ```
        /// use std::time::Duration;
        /// use bevy_falling_sand::core::ChanceLifetime;
        ///
        /// let lifetime = ChanceLifetime::new(0.05, Duration::from_millis(100));
        /// assert_eq!(lifetime.chance, 0.05);
        /// ```
        #[must_use]
        pub fn new(chance: f64, tick_rate: Duration) -> Self {
            Self {
                chance,
                tick_timer: Timer::new(tick_rate, TimerMode::Repeating),
            }
        }

        /// Create a new chance-based lifetime with the given probability and tick rate.
        ///
        /// # Examples
        ///
        /// ```
        /// use std::time::Duration;
        /// use bevy_falling_sand::core::ChanceLifetime;
        ///
        /// let lifetime = ChanceLifetime::with_tick_rate(0.05, Duration::from_millis(100));
        /// assert_eq!(lifetime.chance, 0.05);
        /// ```
        #[must_use]
        pub fn with_tick_rate(chance: f64, tick_rate: Duration) -> Self {
            Self {
                chance,
                tick_timer: Timer::new(tick_rate, TimerMode::Repeating),
            }
        }
    }

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
}

pub(super) mod systems {
    use bevy::prelude::*;
    use bevy_turborand::{DelegatedRng, GlobalRng};

    use crate::core::{
        ChanceLifetime, DespawnParticleSignal, Particle, ParticleSystems, TimedLifetime,
    };
    pub(super) struct SystemsPlugin;

    impl Plugin for SystemsPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(
                Update,
                (handle_timed_lifetimes, handle_chance_lifetimes)
                    .in_set(ParticleSystems::Simulation),
            );
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn handle_timed_lifetimes(
        mut msgw_despawn: MessageWriter<DespawnParticleSignal>,
        mut query: Query<(Entity, &mut TimedLifetime), With<Particle>>,
        time: Res<Time>,
    ) {
        for (entity, mut lifetime) in &mut query {
            lifetime.tick(time.delta());
            if lifetime.finished() {
                msgw_despawn.write(DespawnParticleSignal::from_entity(entity));
            }
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn handle_chance_lifetimes(
        mut msgw_despawn: MessageWriter<DespawnParticleSignal>,
        mut query: Query<(Entity, &mut ChanceLifetime), With<Particle>>,
        mut rng: ResMut<GlobalRng>,
        time: Res<Time>,
    ) {
        for (entity, mut lifetime) in &mut query {
            if lifetime.tick_timer.tick(time.delta()).just_finished() && rng.chance(lifetime.chance)
            {
                msgw_despawn.write(DespawnParticleSignal::from_entity(entity));
            }
        }
    }
}

pub(super) mod resources {
    use bevy::{platform::collections::HashMap, prelude::*};
    use std::borrow::Cow;

    use crate::core::SpatialMap;

    pub(super) struct ResourcesPlugin;

    impl Plugin for ResourcesPlugin {
        fn build(&self, app: &mut App) {
            app.init_resource::<ParticleTypeRegistry>();
        }
    }

    /// Alias for the particle spatial map.
    ///
    /// This map is intended to store [`Particle`](crate::Particle) entities IDs
    pub type ParticleMap = SpatialMap<Entity>;

    impl ParticleMap {
        /// Find all particles within radius that have line-of-sight to center.
        ///
        /// Particles that are blocked by entities matching the `blocker_query` are excluded.
        ///
        /// # Examples
        ///
        /// ```no_run
        /// use bevy::prelude::*;
        /// use bevy_falling_sand::core::ParticleMap;
        ///
        /// #[derive(Component)]
        /// struct Wall;
        ///
        /// fn find_visible(map: Res<ParticleMap>, walls: Query<(), With<Wall>>) {
        ///     let visible = map.within_radius_los(IVec2::new(10, 10), 5.0, &walls);
        ///     println!("Visible particles: {}", visible.len());
        /// }
        /// ```
        #[inline(always)]
        #[must_use]
        pub fn within_radius_los(
            &self,
            center: IVec2,
            radius: f32,
            blocker_query: &Query<(), impl bevy::ecs::query::QueryFilter>,
        ) -> Vec<(IVec2, Entity)> {
            self.within_radius_los_by(center, radius, |entity| blocker_query.get(entity).is_ok())
        }

        /// Check if there's a clear line of sight between two positions.
        ///
        /// Returns `true` if no entity matching the `blocker_query` exists along the path.
        ///
        /// # Examples
        ///
        /// ```no_run
        /// use bevy::prelude::*;
        /// use bevy_falling_sand::core::ParticleMap;
        ///
        /// #[derive(Component)]
        /// struct Wall;
        ///
        /// fn check_los(map: Res<ParticleMap>, walls: Query<(), With<Wall>>) {
        ///     let can_see = map.has_line_of_sight(
        ///         IVec2::new(0, 0),
        ///         IVec2::new(10, 10),
        ///         &walls,
        ///     );
        ///     println!("Line of sight: {can_see}");
        /// }
        /// ```
        #[inline(always)]
        #[must_use]
        pub fn has_line_of_sight(
            &self,
            from: IVec2,
            to: IVec2,
            blocker_query: &Query<(), impl bevy::ecs::query::QueryFilter>,
        ) -> bool {
            self.has_line_of_sight_by(from, to, &|entity| blocker_query.get(entity).is_ok())
        }

        /// Cast a ray from `from` toward `to`, returning the first entity that matches
        /// the `blocker_query` along with its position.
        ///
        /// # Examples
        ///
        /// ```no_run
        /// use bevy::prelude::*;
        /// use bevy_falling_sand::core::ParticleMap;
        ///
        /// #[derive(Component)]
        /// struct Wall;
        ///
        /// fn find_wall(map: Res<ParticleMap>, walls: Query<(), With<Wall>>) {
        ///     if let Some((pos, entity)) = map.raycast_query(
        ///         IVec2::new(0, 0),
        ///         IVec2::new(10, 10),
        ///         &walls,
        ///     ) {
        ///         println!("Hit wall {entity:?} at {pos}");
        ///     }
        /// }
        /// ```
        #[inline(always)]
        #[must_use]
        pub fn raycast_query(
            &self,
            from: IVec2,
            to: IVec2,
            blocker_query: &Query<(), impl bevy::ecs::query::QueryFilter>,
        ) -> Option<(IVec2, Entity)> {
            self.raycast(from, to, &|entity| blocker_query.get(entity).is_ok())
        }
    }

    /// Associates unique [`ParticleType`](crate::prelude::ParticleType) with their entity ID
    #[derive(Resource, Clone, Default, Eq, PartialEq, Debug, Reflect)]
    #[reflect(Resource)]
    pub struct ParticleTypeRegistry {
        map: HashMap<Cow<'static, str>, Entity>,
    }

    impl ParticleTypeRegistry {
        /// Returns true if the designated key exists in the map.
        ///
        /// # Examples
        ///
        /// ```no_run
        /// use bevy::prelude::*;
        /// use bevy_falling_sand::core::ParticleTypeRegistry;
        ///
        /// fn check_type(registry: Res<ParticleTypeRegistry>) {
        ///     if registry.contains("Sand") {
        ///         println!("Sand type is registered");
        ///     }
        /// }
        /// ```
        #[must_use]
        #[inline(always)]
        pub fn contains(&self, name: &str) -> bool {
            self.map.contains_key(name)
        }

        /// Iterate over key value pairs in the map.
        ///
        /// # Examples
        ///
        /// ```no_run
        /// use bevy::prelude::*;
        /// use bevy_falling_sand::core::ParticleTypeRegistry;
        ///
        /// fn list_types(registry: Res<ParticleTypeRegistry>) {
        ///     for (name, entity) in registry.iter() {
        ///         println!("{name}: {entity:?}");
        ///     }
        /// }
        /// ```
        pub fn iter(&self) -> impl Iterator<Item = (&str, &Entity)> {
            self.map.iter().map(|(k, v)| (k.as_ref(), v))
        }

        /// Iterate over particle names in the map.
        ///
        /// # Examples
        ///
        /// ```no_run
        /// use bevy::prelude::*;
        /// use bevy_falling_sand::core::ParticleTypeRegistry;
        ///
        /// fn list_names(registry: Res<ParticleTypeRegistry>) {
        ///     for name in registry.names() {
        ///         println!("Particle type: {name}");
        ///     }
        /// }
        /// ```
        pub fn names(&self) -> impl Iterator<Item = &str> + use<'_> {
            self.map.keys().map(AsRef::as_ref)
        }

        /// Iterate over entities in the map.
        ///
        /// # Examples
        ///
        /// ```no_run
        /// use bevy::prelude::*;
        /// use bevy_falling_sand::core::ParticleTypeRegistry;
        ///
        /// fn list_entities(registry: Res<ParticleTypeRegistry>) {
        ///     for entity in registry.entities() {
        ///         println!("ParticleType entity: {entity:?}");
        ///     }
        /// }
        /// ```
        pub fn entities(&self) -> impl Iterator<Item = &Entity> + use<'_> {
            self.map.values()
        }

        /// Get a particle type from the map if it exists.
        ///
        /// # Examples
        ///
        /// ```no_run
        /// use bevy::prelude::*;
        /// use bevy_falling_sand::core::ParticleTypeRegistry;
        ///
        /// fn get_sand(registry: Res<ParticleTypeRegistry>) {
        ///     if let Some(entity) = registry.get("Sand") {
        ///         println!("Sand type entity: {entity:?}");
        ///     }
        /// }
        /// ```
        #[must_use]
        #[inline(always)]
        pub fn get(&self, name: &str) -> Option<&Entity> {
            self.map.get(name)
        }

        /// Returns `true` if the particle type map is empty.
        ///
        /// # Examples
        ///
        /// ```no_run
        /// use bevy::prelude::*;
        /// use bevy_falling_sand::core::ParticleTypeRegistry;
        ///
        /// fn check_registry(registry: Res<ParticleTypeRegistry>) {
        ///     if registry.is_empty() {
        ///         println!("No particle types registered");
        ///     }
        /// }
        /// ```
        #[must_use]
        pub fn is_empty(&self) -> bool {
            self.map.is_empty()
        }

        /// Insert a new [`ParticleType`] and entity.
        #[inline(always)]
        pub(crate) fn insert(
            &mut self,
            name: impl Into<Cow<'static, str>>,
            entity: Entity,
        ) -> Option<Entity> {
            self.map.insert(name.into(), entity)
        }

        /// Remove a particle type from the map
        pub(crate) fn remove(&mut self, name: &str) -> Option<Entity> {
            self.map.remove(name)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{ChunkCoord, ChunkIndex, ParticleMap};
    use bevy::prelude::*;

    fn create_map() -> ParticleMap {
        ParticleMap::new(64, 64, IVec2::ZERO)
    }

    fn create_chunk_index() -> ChunkIndex {
        ChunkIndex::new(2, 2, 32, IVec2::ZERO)
    }

    mod constructor {
        use super::*;

        #[test]
        fn new_creates_map_with_correct_dimensions() {
            let map = create_map();

            assert_eq!(map.width(), 64);
            assert_eq!(map.height(), 64);
            assert_eq!(map.origin(), IVec2::ZERO);
        }

        #[test]
        fn new_accepts_power_of_two() {
            for size in [32, 64, 128, 256, 512, 1024] {
                let map = ParticleMap::new(size, size, IVec2::ZERO);
                assert_eq!(map.width(), size);
            }
        }

        #[test]
        #[should_panic(expected = "power of 2")]
        fn new_panics_on_non_power_of_two_width() {
            let _ = ParticleMap::new(33, 64, IVec2::ZERO);
        }

        #[test]
        #[should_panic(expected = "power of 2")]
        fn new_panics_on_non_power_of_two_height() {
            let _ = ParticleMap::new(64, 33, IVec2::ZERO);
        }
    }

    mod loaded_region {
        use super::*;

        #[test]
        fn loaded_region_at_origin() {
            let map = create_map();
            let region = map.loaded_region();

            assert_eq!(region.min, IVec2::ZERO);
            assert_eq!(region.max, IVec2::new(63, 63));
        }

        #[test]
        fn loaded_region_with_offset_origin() {
            let map = ParticleMap::new(64, 64, IVec2::new(100, 200));
            let region = map.loaded_region();

            assert_eq!(region.min, IVec2::new(100, 200));
            assert_eq!(region.max, IVec2::new(163, 263));
        }

        #[test]
        fn is_position_loaded_inside() {
            let map = create_map();

            assert!(map.is_position_loaded(IVec2::new(0, 0)));
            assert!(map.is_position_loaded(IVec2::new(32, 32)));
            assert!(map.is_position_loaded(IVec2::new(63, 63)));
        }

        #[test]
        fn is_position_loaded_outside() {
            let map = create_map();

            assert!(!map.is_position_loaded(IVec2::new(-1, 0)));
            assert!(!map.is_position_loaded(IVec2::new(0, -1)));
            assert!(!map.is_position_loaded(IVec2::new(64, 0)));
            assert!(!map.is_position_loaded(IVec2::new(0, 64)));
        }
    }

    mod coordinate_conversion {
        use super::*;

        #[test]
        fn world_to_chunk_coord_positive() {
            let index = create_chunk_index();

            assert_eq!(
                index.world_to_chunk_coord(IVec2::new(0, 0)),
                ChunkCoord::new(0, 0)
            );
            assert_eq!(
                index.world_to_chunk_coord(IVec2::new(31, 31)),
                ChunkCoord::new(0, 0)
            );
            assert_eq!(
                index.world_to_chunk_coord(IVec2::new(32, 32)),
                ChunkCoord::new(1, 1)
            );
        }

        #[test]
        fn world_to_chunk_coord_negative() {
            let index = create_chunk_index();

            assert_eq!(
                index.world_to_chunk_coord(IVec2::new(-1, -1)),
                ChunkCoord::new(-1, -1)
            );
            assert_eq!(
                index.world_to_chunk_coord(IVec2::new(-32, -32)),
                ChunkCoord::new(-1, -1)
            );
        }

        #[test]
        fn chunk_coord_to_region() {
            let index = create_chunk_index();

            let region = index.chunk_coord_to_chunk_region(ChunkCoord::new(0, 0));
            assert_eq!(region.min, IVec2::new(0, 0));
            assert_eq!(region.max, IVec2::new(31, 31));

            let region = index.chunk_coord_to_chunk_region(ChunkCoord::new(1, 1));
            assert_eq!(region.min, IVec2::new(32, 32));
            assert_eq!(region.max, IVec2::new(63, 63));
        }
    }

    mod entity_operations {
        use super::*;

        #[test]
        fn insert_and_get() {
            let mut map = create_map();
            let entity = Entity::from_bits(42);
            let pos = IVec2::new(5, 5);

            let _ = map.insert(pos, entity);

            assert_eq!(map.get_copied(pos), Ok(Some(entity)));
            assert_eq!(map.contains(pos), Ok(true));
        }

        #[test]
        fn insert_returns_previous() {
            let mut map = create_map();
            let e1 = Entity::from_bits(1);
            let e2 = Entity::from_bits(2);
            let pos = IVec2::new(5, 5);

            let _ = map.insert(pos, e1);
            let prev = map.insert(pos, e2);

            assert_eq!(prev, Ok(Some(e1)));
            assert_eq!(map.get_copied(pos), Ok(Some(e2)));
        }

        #[test]
        fn insert_outside_region_fails() {
            let mut map = create_map();
            let entity = Entity::from_bits(42);

            let result = map.insert(IVec2::new(100, 100), entity);

            assert!(result.is_err());
        }

        #[test]
        fn remove_returns_entity() {
            let mut map = create_map();
            let entity = Entity::from_bits(42);
            let pos = IVec2::new(5, 5);

            let _ = map.insert(pos, entity);
            let removed = map.remove(pos);

            assert_eq!(removed, Ok(Some(entity)));
            assert_eq!(map.contains(pos), Ok(false));
        }

        #[test]
        fn remove_empty_returns_none() {
            let mut map = create_map();
            let removed = map.remove(IVec2::new(5, 5));

            assert_eq!(removed, Ok(None));
        }

        #[test]
        fn is_empty_on_new_map() {
            let map = create_map();
            assert!(map.is_empty());
        }

        #[test]
        fn is_empty_after_insert() {
            let mut map = create_map();
            let _ = map.insert(IVec2::new(5, 5), Entity::from_bits(1));

            assert!(!map.is_empty());
        }

        #[test]
        fn swap_positions() {
            let mut map = create_map();
            let e1 = Entity::from_bits(1);
            let e2 = Entity::from_bits(2);
            let pos1 = IVec2::new(5, 5);
            let pos2 = IVec2::new(10, 10);

            let _ = map.insert(pos1, e1);
            let _ = map.insert(pos2, e2);

            let result = map.swap(pos1, pos2);

            assert!(result.is_ok());
            assert_eq!(map.get_copied(pos1), Ok(Some(e2)));
            assert_eq!(map.get_copied(pos2), Ok(Some(e1)));
        }

        #[test]
        fn swap_with_empty() {
            let mut map = create_map();
            let entity = Entity::from_bits(1);
            let pos1 = IVec2::new(5, 5);
            let pos2 = IVec2::new(10, 10);

            let _ = map.insert(pos1, entity);

            let result = map.swap(pos1, pos2);

            assert!(result.is_ok());
            assert_eq!(map.get_copied(pos1), Ok(None));
            assert_eq!(map.get_copied(pos2), Ok(Some(entity)));
        }
    }

    mod chunk_grouping {
        use super::*;

        #[test]
        fn chunk_group_even_even() {
            assert_eq!(ChunkCoord::new(0, 0).group(), 0);
            assert_eq!(ChunkCoord::new(2, 2).group(), 0);
        }

        #[test]
        fn chunk_group_odd_even() {
            assert_eq!(ChunkCoord::new(1, 0).group(), 1);
            assert_eq!(ChunkCoord::new(3, 2).group(), 1);
        }

        #[test]
        fn chunk_group_even_odd() {
            assert_eq!(ChunkCoord::new(0, 1).group(), 2);
            assert_eq!(ChunkCoord::new(2, 3).group(), 2);
        }

        #[test]
        fn chunk_group_odd_odd() {
            assert_eq!(ChunkCoord::new(1, 1).group(), 3);
            assert_eq!(ChunkCoord::new(3, 3).group(), 3);
        }

        #[test]
        fn loaded_chunk_coords() {
            let index = create_chunk_index();
            let coords = index.loaded_chunk_coords();

            assert_eq!(coords.len(), 4);
            assert!(coords.contains(&ChunkCoord::new(0, 0)));
            assert!(coords.contains(&ChunkCoord::new(1, 0)));
            assert!(coords.contains(&ChunkCoord::new(0, 1)));
            assert!(coords.contains(&ChunkCoord::new(1, 1)));
        }

        #[test]
        fn partition_chunks_by_group() {
            let index = create_chunk_index();
            let groups = index.partition_chunks_by_group();

            assert!(groups[0].contains(&ChunkCoord::new(0, 0)));
            assert!(groups[1].contains(&ChunkCoord::new(1, 0)));
            assert!(groups[2].contains(&ChunkCoord::new(0, 1)));
            assert!(groups[3].contains(&ChunkCoord::new(1, 1)));
        }
    }

    mod toroidal_indexing {
        use super::*;

        #[test]
        fn shift_origin_updates_region() {
            let mut map = create_map();

            let (old_region, new_region) = map.shift_origin(IVec2::new(32, 32));

            assert_eq!(old_region.min, IVec2::ZERO);
            assert_eq!(new_region.min, IVec2::new(32, 32));
            assert_eq!(map.origin(), IVec2::new(32, 32));
        }

        #[test]
        fn data_persists_after_origin_shift() {
            let mut map = create_map();
            let entity = Entity::from_bits(42);

            // Insert at (40,40) with origin (0,0)
            let _ = map.insert(IVec2::new(40, 40), entity);

            map.shift_origin(IVec2::new(32, 32));

            // (40,40) is still in the new loaded region [32,96) x [32,96)
            assert_eq!(map.get_copied(IVec2::new(40, 40)), Ok(Some(entity)));
        }

        #[test]
        fn position_outside_after_shift() {
            let mut map = create_map();
            let entity = Entity::from_bits(42);

            let _ = map.insert(IVec2::new(5, 5), entity);

            map.shift_origin(IVec2::new(32, 32));

            assert!(!map.is_position_loaded(IVec2::new(5, 5)));
        }
    }

    mod spatial_queries {
        use super::*;

        #[test]
        fn within_radius_finds_entities() {
            let mut map = create_map();

            let e1 = Entity::from_bits(1);
            let e2 = Entity::from_bits(2);
            let e3 = Entity::from_bits(3);

            let _ = map.insert(IVec2::new(10, 10), e1);
            let _ = map.insert(IVec2::new(12, 10), e2);
            let _ = map.insert(IVec2::new(20, 20), e3);

            let center = IVec2::new(10, 10);
            assert_eq!(map.within_radius(center, 5.0).count(), 2);
        }
    }

    mod entry_api {
        use super::*;

        #[test]
        fn entry_insert_if_vacant_when_empty() {
            let mut map = create_map();
            let entity = Entity::from_bits(42);
            let pos = IVec2::new(5, 5);

            let mut entry = map.entry(pos).unwrap();
            let inserted = entry.insert_if_vacant(entity);

            assert!(inserted);
            assert_eq!(map.get_copied(pos), Ok(Some(entity)));
        }

        #[test]
        fn entry_insert_if_vacant_when_occupied() {
            let mut map = create_map();
            let e1 = Entity::from_bits(1);
            let e2 = Entity::from_bits(2);
            let pos = IVec2::new(5, 5);

            let _ = map.insert(pos, e1);

            let mut entry = map.entry(pos).unwrap();
            let inserted = entry.insert_if_vacant(e2);

            assert!(!inserted);
            assert_eq!(map.get_copied(pos), Ok(Some(e1)));
        }
    }
}
