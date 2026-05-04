//! Provides synchronization behaviors between [`Particle`] entities and their associated
//! [`ParticleType`].
//!
//! A `Particle` will synchronize with its parent on 2 occasions:
//! - A [`SyncParticleSignal`] has been received (sent explicitly or via
//!   [`SyncParticleTypeChildrenSignal`]).
//! - `Changed<Particle>` fires (e.g., on spawn or when [`Particle::name`] is mutated).
//!
//! The [`ParticleSyncExt`] trait provides an interface for adding your own particle sync
//! components. Each registered propagator is keyed by a [`TypeId`], enabling
//! selective synchronization through [`PropagatorFilter`]. Use
//! [`SyncParticleSignal::with`]/[`SyncParticleSignal::without`] (and the corresponding methods
//! on [`SyncParticleTypeChildrenSignal`]) to control which components are synchronized.
use super::LocateBy;
use crate::core::{
    AttachedToParticleType, ChanceLifetime, ChunkDirtyState, ChunkIndex, GridPosition, Particle,
    ParticleMap, ParticleSystems, ParticleType, ParticleTypeRegistry, TimedLifetime,
};
use bevy::{ecs::system::SystemParam, platform::collections::HashSet, prelude::*};
use bevy_turborand::{DelegatedRng, GlobalRng};
use serde::{Deserialize, Serialize};
use std::{any::TypeId, borrow::Cow, time::Duration};

pub(super) struct SyncPlugin;

impl Plugin for SyncPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticlePropagators>()
            .register_type::<ChanceMutation>()
            .add_message::<SyncParticleSignal>()
            .add_message::<SyncParticleTypeChildrenSignal>()
            .add_observer(on_sync_particle)
            .add_observer(on_sync_particle_type_children)
            .add_systems(
                PreUpdate,
                (
                    sync_particle_type_registry,
                    sync_particle_type_children_names.after(sync_particle_type_registry),
                    sync_particle_parent.after(sync_particle_type_children_names),
                    msgr_sync_particle_type_children,
                )
                    .before(ParticleSystems::Registration),
            )
            .register_particle_sync_component::<TimedLifetime>()
            .register_particle_sync_component::<ChanceLifetime>()
            .register_particle_sync_component::<ChanceMutation>()
            .add_systems(
                PreUpdate,
                msgr_sync_particle.in_set(ParticleSystems::Registration),
            )
            .add_systems(
                Update,
                handle_chance_mutations.in_set(ParticleSystems::Simulation),
            );
    }
}

/// A chance-based mutation component that has a chance to rename a [`Particle`] to a target
/// type on a per-tick basis.
///
/// When the roll succeeds, the [`Particle::name`] is updated to [`ChanceMutation::target`].
/// The change triggers normal particle synchronization, so the particle is re-attached to its
/// new [`ParticleType`] and registered components are re-propagated. If `target` does not
/// match a registered [`ParticleType`], the mutation is reverted by [`sync_particle_parent`].
#[derive(Component, Clone, PartialEq, Debug, Reflect)]
#[reflect(Component)]
#[type_path = "bfs_core::particle"]
pub struct ChanceMutation {
    /// The name of the [`ParticleType`] this particle should mutate into.
    pub target: Cow<'static, str>,
    /// The probability (0.0 to 1.0) that the particle will mutate each tick.
    pub chance: f64,
    /// Timer that controls how often the chance is evaluated.
    pub tick_timer: Timer,
}

impl Default for ChanceMutation {
    fn default() -> Self {
        Self {
            target: Cow::Borrowed(""),
            chance: 0.0,
            tick_timer: Timer::new(Duration::ZERO, TimerMode::Repeating),
        }
    }
}

impl ChanceMutation {
    /// Create a new chance-based mutation targeting `target` from a static string.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use bevy_falling_sand::core::ChanceMutation;
    ///
    /// let mutation = ChanceMutation::new("Water", 0.05, Duration::from_millis(100));
    /// assert_eq!(mutation.target, "Water");
    /// assert_eq!(mutation.chance, 0.05);
    /// ```
    #[must_use]
    pub fn new(target: &'static str, chance: f64, tick_rate: Duration) -> Self {
        Self {
            target: Cow::Borrowed(target),
            chance,
            tick_timer: Timer::new(tick_rate, TimerMode::Repeating),
        }
    }

    /// Create a new chance-based mutation targeting `target` from an owned string.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use bevy_falling_sand::core::ChanceMutation;
    ///
    /// let target = String::from("Water");
    /// let mutation = ChanceMutation::from_string(target, 0.05, Duration::from_millis(100));
    /// assert_eq!(mutation.target, "Water");
    /// ```
    #[must_use]
    pub fn from_string(target: String, chance: f64, tick_rate: Duration) -> Self {
        Self {
            target: Cow::Owned(target),
            chance,
            tick_timer: Timer::new(tick_rate, TimerMode::Repeating),
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_chance_mutations(
    mut query: Query<(&mut Particle, &mut ChanceMutation)>,
    mut rng: ResMut<GlobalRng>,
    time: Res<Time>,
) {
    for (mut particle, mut mutation) in &mut query {
        if mutation.tick_timer.tick(time.delta()).just_finished()
            && rng.chance(mutation.chance)
            && particle.name != mutation.target
        {
            particle.name.clone_from(&mutation.target);
        }
    }
}

type PropagateFn = Box<dyn Fn(Entity, Entity, &mut Commands) + Send + Sync>;

/// Controls which propagators run during particle synchronization.
///
/// Constructed implicitly via [`SyncParticleSignal::with`], [`SyncParticleSignal::without`],
/// and the corresponding methods on [`SyncParticleTypeChildrenSignal`].
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum PropagatorFilter {
    /// Run all registered propagators (the default).
    #[default]
    All,
    /// Run only the propagators whose keys are in this list.
    Only(Vec<TypeId>),
    /// Run all propagators except those whose keys are in this list.
    Except(Vec<TypeId>),
}

#[derive(Resource, Default)]
pub(crate) struct ParticlePropagators(Vec<(TypeId, PropagateFn)>);

impl ParticlePropagators {
    fn run_filtered(
        &self,
        filter: &PropagatorFilter,
        entity: Entity,
        parent: Entity,
        commands: &mut Commands,
    ) {
        match filter {
            PropagatorFilter::All => {
                for (_, f) in &self.0 {
                    f(entity, parent, commands);
                }
            }
            PropagatorFilter::Only(ids) => {
                for (key, f) in &self.0 {
                    if ids.contains(key) {
                        f(entity, parent, commands);
                    }
                }
            }
            PropagatorFilter::Except(ids) => {
                for (key, f) in &self.0 {
                    if !ids.contains(key) {
                        f(entity, parent, commands);
                    }
                }
            }
        }
    }
}

/// Extension trait for registering custom particle component propagation.
///
/// All built-in particle behavior components ([`Movement`](crate::movement::Movement),
/// [`Density`](crate::movement::Density), [`Flammable`](crate::reactions::Flammable), etc.) are
/// synchronized from [`ParticleType`] to child [`Particle`] entities using this trait. Register
/// your own components the same way.
///
/// Propagators run during component synchronization whenever a [`SyncParticleSignal`] is
/// received or [`Particle`] change detection fires (e.g. on spawn or when [`Particle::name`] is
/// mutated). Each propagator receives the particle entity, its parent [`ParticleType`] entity,
/// and [`Commands`] for deferred mutations.
///
/// Every propagator is keyed by a [`TypeId`], which enables selective
/// synchronization via [`PropagatorFilter`]. Signals can target specific propagators with
/// [`SyncParticleSignal::with`]/[`SyncParticleSignal::without`].
///
/// There are two registration methods:
///
/// - [`register_particle_sync_component`](`ParticleSyncExt::register_particle_sync_component`):
///   Clones a component from the parent [`ParticleType`] onto each child [`Particle`]. If the
///   parent doesn't have the component, it is removed from the child. The component type `C` is
///   used as the propagator key. This covers the majority of use cases.
///
/// - [`register_particle_propagator`](`ParticleSyncExt::register_particle_propagator`): Accepts
///   an arbitrary closure for cases where a simple clone isn't enough (e.g. conditional insertion,
///   deriving values from the parent, etc.). Requires an explicit marker type `K` as the
///   propagator key.
///
/// After registration, updating the component on a [`ParticleType`] entity and sending a
/// [`SyncParticleTypeChildrenSignal`] will propagate the change to all living particles of that
/// type.
pub trait ParticleSyncExt {
    /// Registers a component type to be cloned from [`ParticleType`] to child [`Particle`]
    /// entities.
    ///
    /// The component type `C` is used as both the source to clone and the propagator key for
    /// [`PropagatorFilter`]-based targeting.
    ///
    /// When a particle is spawned or synced, if its parent [`ParticleType`] has the component `C`,
    /// a clone is inserted into the particle. If the parent does not have the component, it is
    /// removed from the particle entity.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::*;
    ///
    /// #[derive(Component, Clone, Reflect)]
    /// struct Toxicity(f64);
    ///
    /// fn main() {
    ///     App::new()
    ///         .add_plugins((DefaultPlugins, FallingSandPlugin::default()))
    ///         .register_particle_sync_component::<Toxicity>()
    ///         .add_systems(Startup, setup)
    ///         .run();
    /// }
    ///
    /// fn setup(mut commands: Commands) {
    ///     commands.spawn((
    ///         ParticleType::new("Acid"),
    ///         Toxicity(0.8),
    ///     ));
    /// }
    /// ```
    fn register_particle_sync_component<C: Component + Clone>(&mut self) -> &mut Self;

    /// Registers an arbitrary propagator that runs for each particle during synchronization.
    ///
    /// The type parameter `K` is used as a key for the propagator, allowing it to be targeted
    /// by [`PropagatorFilter`]. Typically `K` is the parent component type that drives the
    /// propagation logic.
    ///
    /// The closure receives `(particle_entity, parent_entity, &mut Commands)` where
    /// `particle_entity` is the [`Particle`], `parent_entity` is its [`ParticleType`], and
    /// [`Commands`] is used for deferred mutations.
    ///
    /// Use this instead of
    /// [`register_particle_sync_component`](`ParticleSyncExt::register_particle_sync_component`)
    /// when you need logic beyond a direct clone — for example, conditionally inserting a
    /// component or deriving a child value from the parent.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::*;
    ///
    /// #[derive(Component, Clone, Reflect)]
    /// struct MaxHealth(f64);
    ///
    /// #[derive(Component, Clone, Reflect)]
    /// struct Health(f64);
    ///
    /// fn main() {
    ///     App::new()
    ///         .add_plugins((DefaultPlugins, FallingSandPlugin::default()))
    ///         .register_particle_propagator::<MaxHealth>(|particle, parent, commands| {
    ///             commands.queue(move |world: &mut World| {
    ///                 let max_hp = world.get::<MaxHealth>(parent).map(|h| h.0);
    ///                 if let Some(max_hp) = max_hp {
    ///                     if world.get::<Health>(particle).is_none() {
    ///                         world.entity_mut(particle).insert(Health(max_hp));
    ///                     }
    ///                 }
    ///             });
    ///         })
    ///         .run();
    /// }
    /// ```
    #[allow(clippy::needless_doctest_main)]
    fn register_particle_propagator<K: 'static>(
        &mut self,
        propagator: impl Fn(Entity, Entity, &mut Commands) + Send + Sync + 'static,
    ) -> &mut Self;
}

impl ParticleSyncExt for App {
    fn register_particle_sync_component<C: Component + Clone>(&mut self) -> &mut Self {
        self.register_particle_propagator::<C>(
            |entity: Entity, parent: Entity, commands: &mut Commands| {
                commands.queue(move |world: &mut World| {
                    let component = world.get::<C>(parent).cloned();
                    if let Some(component) = component {
                        world.entity_mut(entity).insert(component);
                    } else {
                        world.entity_mut(entity).remove::<C>();
                    }
                });
            },
        )
    }

    fn register_particle_propagator<K: 'static>(
        &mut self,
        propagator: impl Fn(Entity, Entity, &mut Commands) + Send + Sync + 'static,
    ) -> &mut Self {
        self.world_mut()
            .resource_mut::<ParticlePropagators>()
            .0
            .push((TypeId::of::<K>(), Box::new(propagator)));
        self
    }
}

/// Synchronizes [`Particle`] components with their [`ParticleType`] parent's components.
///
/// Targets are collected from two sources, deduplicated by entity:
/// 1. Drained [`SyncParticleSignal`] messages (sent externally or by [`sync_particle_parent`])
/// 2. `Changed<Particle>` query (catches freshly spawned particles whose deferred commands
///    were applied after `sync_particle_parent` already ran in the same frame)
#[derive(SystemParam)]
struct SyncParticleParams<'w, 's> {
    msgr: MessageReader<'w, 's, SyncParticleSignal>,
    particle_query: Query<'w, 's, (&'static AttachedToParticleType, &'static GridPosition)>,
    changed_query: Query<
        'w,
        's,
        (
            Entity,
            &'static AttachedToParticleType,
            &'static GridPosition,
        ),
        (Changed<Particle>, With<Particle>),
    >,
    particle_map: Res<'w, ParticleMap>,
    propagators: Res<'w, ParticlePropagators>,
    chunk_index: Res<'w, ChunkIndex>,
    chunk_dirty_query: Query<'w, 's, &'static mut ChunkDirtyState>,
    commands: Commands<'w, 's>,
}

#[allow(clippy::needless_pass_by_value)]
fn msgr_sync_particle(mut params: SyncParticleParams) {
    let mut seen = HashSet::new();
    let mut targets: Vec<(Entity, Entity, IVec2, PropagatorFilter)> = Vec::new();

    for signal in params.msgr.read() {
        let entity = match &signal.locate_by {
            LocateBy::Entity(e) => Some(*e),
            LocateBy::Position(pos) => params.particle_map.get_copied(*pos).ok().flatten(),
            LocateBy::Name(_) => unreachable!(),
        };
        if let Some(entity) = entity
            && let Ok((attached, grid_pos)) = params.particle_query.get(entity)
        {
            if seen.insert(entity) {
                targets.push((entity, attached.0, grid_pos.0, signal.filter.clone()));
            } else if let Some(target) = targets.iter_mut().find(|(e, _, _, _)| *e == entity) {
                target.3 = PropagatorFilter::All;
            }
        }
    }

    for (entity, attached, grid_pos) in &params.changed_query {
        if seen.insert(entity) {
            targets.push((entity, attached.0, grid_pos.0, PropagatorFilter::All));
        } else if let Some(target) = targets.iter_mut().find(|(e, _, _, _)| *e == entity) {
            target.3 = PropagatorFilter::All;
        }
    }

    if targets.is_empty() {
        return;
    }

    for (entity, parent, _, filter) in &targets {
        params
            .propagators
            .run_filtered(filter, *entity, *parent, &mut params.commands);
    }

    for &(_, _, position, _) in &targets {
        let chunk_coord = params.chunk_index.world_to_chunk_coord(position);
        if let Some(chunk_entity) = params.chunk_index.get(chunk_coord)
            && let Ok(mut dirty_state) = params.chunk_dirty_query.get_mut(chunk_entity)
        {
            dirty_state.mark_dirty(position);
        }
    }
}

/// Synchronizes a [`Particle`] entity with its [`ParticleType`] parent's components when a
/// [`SyncParticleSignal`] trigger is received.
#[allow(clippy::needless_pass_by_value)]
fn on_sync_particle(
    trigger: On<SyncParticleSignal>,
    particle_query: Query<(&AttachedToParticleType, &GridPosition)>,
    particle_map: Res<ParticleMap>,
    propagators: Res<ParticlePropagators>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_dirty_query: Query<&mut ChunkDirtyState>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let entity = match &event.locate_by {
        LocateBy::Entity(e) => Some(*e),
        LocateBy::Position(pos) => particle_map.get_copied(*pos).ok().flatten(),
        LocateBy::Name(_) => unreachable!(),
    };

    let Some(entity) = entity else { return };
    let Ok((attached, grid_pos)) = particle_query.get(entity) else {
        return;
    };
    let parent = attached.0;
    let position = grid_pos.0;

    propagators.run_filtered(&event.filter, entity, parent, &mut commands);

    let chunk_coord = chunk_index.world_to_chunk_coord(position);
    if let Some(chunk_entity) = chunk_index.get(chunk_coord)
        && let Ok(mut dirty_state) = chunk_dirty_query.get_mut(chunk_entity)
    {
        dirty_state.mark_dirty(position);
    }
}

/// Signal for synchronizing a [`Particle`] entity with its [`ParticleType`] parent.
///
/// By default, all registered propagators run. Use [`with`](Self::with) to restrict
/// synchronization to specific components, or [`without`](Self::without) to exclude specific
/// components.
#[derive(Event, Message, Clone, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub struct SyncParticleSignal {
    locate_by: LocateBy,
    #[serde(skip)]
    #[reflect(ignore)]
    filter: PropagatorFilter,
}

impl SyncParticleSignal {
    /// Initialize from the [`Entity`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{Particle, SyncParticleSignal};
    ///
    /// fn resync(
    ///     mut writer: MessageWriter<SyncParticleSignal>,
    ///     query: Query<Entity, With<Particle>>,
    /// ) {
    ///     for entity in &query {
    ///         writer.write(SyncParticleSignal::from_entity(entity));
    ///     }
    /// }
    /// ```
    #[must_use]
    pub const fn from_entity(entity: Entity) -> Self {
        Self {
            locate_by: LocateBy::Entity(entity),
            filter: PropagatorFilter::All,
        }
    }

    /// Initialize from the [`Particle`] position.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SyncParticleSignal;
    ///
    /// fn resync_at(mut writer: MessageWriter<SyncParticleSignal>) {
    ///     writer.write(SyncParticleSignal::from_position(IVec2::new(10, 20)));
    /// }
    /// ```
    #[must_use]
    pub const fn from_position(position: IVec2) -> Self {
        Self {
            locate_by: LocateBy::Position(position),
            filter: PropagatorFilter::All,
        }
    }

    /// Narrow synchronization to include the propagator keyed by `C`.
    ///
    /// When called on a signal with no filter ([`PropagatorFilter::All`]), switches to
    /// [`PropagatorFilter::Only`] containing `C`. When called on an existing
    /// [`PropagatorFilter::Only`], adds `C`. When called on [`PropagatorFilter::Except`],
    /// removes `C` from the exclusion list.
    ///
    /// Chain multiple calls to sync several components:
    ///
    /// ```ignore
    /// SyncParticleSignal::from_entity(e).with::<Density>().with::<Speed>()
    /// ```
    #[must_use]
    pub fn with<C: 'static>(mut self) -> Self {
        let id = TypeId::of::<C>();
        match &mut self.filter {
            PropagatorFilter::All => {
                self.filter = PropagatorFilter::Only(vec![id]);
            }
            PropagatorFilter::Only(ids) => {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
            PropagatorFilter::Except(ids) => {
                ids.retain(|i| *i != id);
            }
        }
        self
    }

    /// Exclude the propagator keyed by `C` from synchronization.
    ///
    /// When called on a signal with no filter ([`PropagatorFilter::All`]), switches to
    /// [`PropagatorFilter::Except`] excluding `C`. When called on an existing
    /// [`PropagatorFilter::Except`], adds `C` to the exclusion list. When called on
    /// [`PropagatorFilter::Only`], removes `C` from the inclusion list.
    ///
    /// ```ignore
    /// SyncParticleSignal::from_entity(e).without::<ColorProfile>()
    /// ```
    #[must_use]
    pub fn without<C: 'static>(mut self) -> Self {
        let id = TypeId::of::<C>();
        match &mut self.filter {
            PropagatorFilter::All => {
                self.filter = PropagatorFilter::Except(vec![id]);
            }
            PropagatorFilter::Except(ids) => {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
            PropagatorFilter::Only(ids) => {
                ids.retain(|i| *i != id);
            }
        }
        self
    }
}

/// Signal for syncing all [`Particle`] entities of a matching
/// [`ParticleType`].
///
/// By default, all registered propagators run. Use [`with`](Self::with) to restrict
/// synchronization to specific components, or [`without`](Self::without) to exclude specific
/// components. The filter is forwarded to each individual [`SyncParticleSignal`] created for
/// the children.
#[derive(Event, Message, Clone, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub struct SyncParticleTypeChildrenSignal {
    locate_by: LocateBy,
    #[serde(skip)]
    #[reflect(ignore)]
    filter: PropagatorFilter,
}

impl SyncParticleTypeChildrenSignal {
    /// Initialize from the `Particle`/`ParticleType` name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::SyncParticleTypeChildrenSignal;
    ///
    /// fn resync_all_sand(mut writer: MessageWriter<SyncParticleTypeChildrenSignal>) {
    ///     writer.write(SyncParticleTypeChildrenSignal::from_name("Sand".into()));
    /// }
    /// ```
    #[must_use]
    pub const fn from_name(name: String) -> Self {
        Self {
            locate_by: LocateBy::Name(name),
            filter: PropagatorFilter::All,
        }
    }

    /// Initialize from the parent's [`Entity`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{SyncParticleTypeChildrenSignal, ParticleTypeRegistry};
    ///
    /// fn resync_by_entity(
    ///     mut writer: MessageWriter<SyncParticleTypeChildrenSignal>,
    ///     registry: Res<ParticleTypeRegistry>,
    /// ) {
    ///     if let Some(&entity) = registry.get("Sand") {
    ///         writer.write(SyncParticleTypeChildrenSignal::from_parent_handle(entity));
    ///     }
    /// }
    /// ```
    #[must_use]
    pub const fn from_parent_handle(entity: Entity) -> Self {
        Self {
            locate_by: LocateBy::Entity(entity),
            filter: PropagatorFilter::All,
        }
    }

    /// Narrow synchronization to include the propagator keyed by `C`.
    ///
    /// See [`SyncParticleSignal::with`] for details on filter behavior.
    #[must_use]
    pub fn with<C: 'static>(mut self) -> Self {
        let id = TypeId::of::<C>();
        match &mut self.filter {
            PropagatorFilter::All => {
                self.filter = PropagatorFilter::Only(vec![id]);
            }
            PropagatorFilter::Only(ids) => {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
            PropagatorFilter::Except(ids) => {
                ids.retain(|i| *i != id);
            }
        }
        self
    }

    /// Exclude the propagator keyed by `C` from synchronization.
    ///
    /// See [`SyncParticleSignal::without`] for details on filter behavior.
    #[must_use]
    pub fn without<C: 'static>(mut self) -> Self {
        let id = TypeId::of::<C>();
        match &mut self.filter {
            PropagatorFilter::All => {
                self.filter = PropagatorFilter::Except(vec![id]);
            }
            PropagatorFilter::Except(ids) => {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
            PropagatorFilter::Only(ids) => {
                ids.retain(|i| *i != id);
            }
        }
        self
    }
}

/// Synchronize all [`Particle`] entities under a [`ParticleType`] when a
/// [`SyncParticleTypeChildrenSignal`] message is received.
#[allow(clippy::needless_pass_by_value)]
fn msgr_sync_particle_type_children(
    mut msgr_sync_particle_children: MessageReader<SyncParticleTypeChildrenSignal>,
    mut msgr_sync_particle: MessageWriter<SyncParticleSignal>,
    particle_query: Query<(Entity, &AttachedToParticleType), With<Particle>>,
    registry: Res<ParticleTypeRegistry>,
) {
    msgr_sync_particle_children.read().for_each(|msg| {
        let parent = match &msg.locate_by {
            LocateBy::Name(name) => registry.get(name.as_str()),
            LocateBy::Entity(entity) => Some(entity),
            LocateBy::Position(_) => {
                unreachable!()
            }
        };
        if let Some(parent) = parent {
            for (entity, _) in particle_query
                .iter()
                .filter(|(_, attached)| attached.0 == *parent)
            {
                let mut signal = SyncParticleSignal::from_entity(entity);
                signal.filter = msg.filter.clone();
                msgr_sync_particle.write(signal);
            }
        } else {
            warn!("No particle type found in particle type registry!");
        }
    });
}

/// Synchronize all [`Particle`] entities under a [`ParticleType`] when a
/// [`SyncParticleTypeChildrenSignal`] trigger is received.
#[allow(clippy::needless_pass_by_value)]
fn on_sync_particle_type_children(
    trigger: On<SyncParticleTypeChildrenSignal>,
    mut msgr_sync_particle: MessageWriter<SyncParticleSignal>,
    particle_query: Query<(Entity, &AttachedToParticleType), With<Particle>>,
    registry: Res<ParticleTypeRegistry>,
) {
    let event = trigger.event();
    let parent = match &event.locate_by {
        LocateBy::Name(name) => registry.get(name.as_str()),
        LocateBy::Entity(entity) => Some(entity),
        LocateBy::Position(_) => {
            unreachable!()
        }
    };
    if let Some(parent) = parent {
        for (entity, _) in particle_query
            .iter()
            .filter(|(_, attached)| attached.0 == *parent)
        {
            let mut signal = SyncParticleSignal::from_entity(entity);
            signal.filter = event.filter.clone();
            msgr_sync_particle.write(signal);
        }
    } else {
        warn!("No particle type found in particle type registry!");
    }
}

/// Synchronizes [`ParticleType`] entities with the [`ParticleTypeRegistry`] if change
/// detection occurs for a `ParticleType`
fn sync_particle_type_registry(
    query: Query<(Entity, &ParticleType), Changed<ParticleType>>,
    mut registry: ResMut<ParticleTypeRegistry>,
) {
    for (entity, particle_type) in &query {
        let old_name = registry.iter().find_map(|(name, &e)| {
            if e == entity {
                Some(name.to_owned())
            } else {
                None
            }
        });

        let Some(old_name) = old_name else {
            continue;
        };

        if old_name == particle_type.name {
            continue;
        }

        registry.remove(&old_name);
        registry.insert(particle_type.name.clone(), entity);
    }
}

/// Propagates [`ParticleType`] name changes to all living child [`Particle`] entities.
///
/// When a `ParticleType` name is updated, its children still hold the old name in their
/// [`Particle`] component. This system finds all children via [`AttachedToParticleType`] and
/// updates their names to match the parent.
fn sync_particle_type_children_names(
    parent_query: Query<(Entity, &ParticleType), Changed<ParticleType>>,
    mut child_query: Query<(&mut Particle, &AttachedToParticleType)>,
) {
    for (parent_entity, particle_type) in &parent_query {
        for (mut particle, attached) in &mut child_query {
            if attached.0 == parent_entity && particle.name != particle_type.name {
                particle.name.clone_from(&particle_type.name);
            }
        }
    }
}

/// Synchronizes a [`Particle`] with its parent [`ParticleType`] if change detection fires for
/// a `Particle`. If the new name doesn't match any registered [`ParticleType`], the change is
/// reverted and a warning is issued.
#[allow(clippy::needless_pass_by_value)]
fn sync_particle_parent(
    mut commands: Commands,
    mut particle_query: Query<
        (Entity, &mut Particle, &AttachedToParticleType),
        (Changed<Particle>, With<GridPosition>),
    >,
    registry: Res<ParticleTypeRegistry>,
    particle_type_query: Query<&ParticleType>,
) {
    for (entity, mut particle, current_parent) in &mut particle_query {
        if let Some(new_parent_handle) = registry.get(&particle.name) {
            if current_parent.0 != *new_parent_handle {
                commands
                    .entity(entity)
                    .insert(AttachedToParticleType(*new_parent_handle));
            }
        } else if let Ok(parent_type) = particle_type_query.get(current_parent.0) {
            warn!(
                "Particle name '{}' does not match any registered ParticleType, \
                 reverting to '{}'",
                particle.name, parent_type.name
            );
            particle
                .bypass_change_detection()
                .name
                .clone_from(&parent_type.name);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::{
        FallingSandMinimalPlugin,
        core::{
            AttachedToParticleType, Particle, ParticleMap, ParticleType, ParticleTypeRegistry,
            SpawnParticleSignal,
        },
    };

    #[derive(Component, Clone, Debug, PartialEq)]
    struct Marker(u32);

    #[derive(Component, Clone, Debug, PartialEq)]
    struct Marker2(u32);

    fn create_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(FallingSandMinimalPlugin::default());
        app
    }

    // ---- sync_particle_type_registry ----

    #[test]
    fn sync_particle_type_registry_on_name_mutation() {
        let mut app = create_test_app();

        let entity = app.world_mut().spawn(ParticleType::new("sand")).id();
        app.update();

        let registry = app.world().resource::<ParticleTypeRegistry>();
        assert_eq!(registry.get("sand"), Some(&entity));

        app.world_mut()
            .entity_mut(entity)
            .get_mut::<ParticleType>()
            .unwrap()
            .name = "water".into();
        app.update();

        let registry = app.world().resource::<ParticleTypeRegistry>();
        assert_eq!(registry.get("water"), Some(&entity));
        assert_eq!(registry.get("sand"), None);
    }

    // ---- sync_particle_parent ----

    #[test]
    fn sync_particle_parent_updates_attached_on_particle_mutation() {
        let mut app = create_test_app();

        let sand_pt = app.world_mut().spawn(ParticleType::new("sand")).id();
        let water_pt = app.world_mut().spawn(ParticleType::new("water")).id();
        app.update();

        app.world_mut()
            .write_message(SpawnParticleSignal::new(Particle::new("sand"), IVec2::ZERO));
        app.update();

        let particle_entity = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .next()
            .unwrap();

        let attached = app
            .world()
            .entity(particle_entity)
            .get::<AttachedToParticleType>()
            .unwrap();
        assert_eq!(attached.0, sand_pt);

        app.world_mut()
            .entity_mut(particle_entity)
            .get_mut::<Particle>()
            .unwrap()
            .name = "water".into();
        app.update();
        app.update();

        let attached = app
            .world()
            .entity(particle_entity)
            .get::<AttachedToParticleType>()
            .unwrap();
        assert_eq!(attached.0, water_pt);
    }

    #[test]
    fn sync_particle_parent_reverts_unregistered_name() {
        let mut app = create_test_app();

        let sand_pt = app.world_mut().spawn(ParticleType::new("sand")).id();
        app.update();

        app.world_mut()
            .write_message(SpawnParticleSignal::new(Particle::new("sand"), IVec2::ZERO));
        app.update();

        let particle_entity = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .next()
            .unwrap();

        app.world_mut()
            .entity_mut(particle_entity)
            .get_mut::<Particle>()
            .unwrap()
            .name = "ghost".into();
        app.update();

        let particle = app
            .world()
            .entity(particle_entity)
            .get::<Particle>()
            .unwrap();
        assert_eq!(
            particle.name, "sand",
            "Name should revert to parent ParticleType"
        );

        let attached = app
            .world()
            .entity(particle_entity)
            .get::<AttachedToParticleType>()
            .unwrap();
        assert_eq!(attached.0, sand_pt);
    }

    // ---- sync_registered_components ----

    #[test]
    fn sync_registered_components_propagates_on_spawn() {
        let mut app = create_test_app();
        app.register_particle_sync_component::<Marker>();

        app.world_mut()
            .spawn((ParticleType::new("sand"), Marker(42)));
        app.update();

        app.world_mut()
            .write_message(SpawnParticleSignal::new(Particle::new("sand"), IVec2::ZERO));
        app.update();

        let particle_entity = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .next()
            .unwrap();

        let marker = app
            .world()
            .entity(particle_entity)
            .get::<Marker>()
            .expect("Marker should have been propagated from ParticleType");
        assert_eq!(marker, &Marker(42));
    }

    #[test]
    fn sync_registered_components_removes_absent_component() {
        let mut app = create_test_app();
        app.register_particle_sync_component::<Marker>();

        app.world_mut()
            .spawn((ParticleType::new("sand"), Marker(42)));
        app.world_mut().spawn(ParticleType::new("water"));
        app.update();

        app.world_mut()
            .write_message(SpawnParticleSignal::new(Particle::new("sand"), IVec2::ZERO));
        app.update();

        let particle_entity = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .next()
            .unwrap();

        assert!(
            app.world()
                .entity(particle_entity)
                .get::<Marker>()
                .is_some()
        );

        app.world_mut()
            .entity_mut(particle_entity)
            .get_mut::<Particle>()
            .unwrap()
            .name = "water".into();
        app.update();
        app.update();

        assert_eq!(
            app.world().entity(particle_entity).get::<Marker>(),
            None,
            "Marker should be removed when switching to a ParticleType that lacks it"
        );
    }

    #[test]
    fn sync_registered_components_updates_on_particle_type_change() {
        let mut app = create_test_app();
        app.register_particle_sync_component::<Marker>();

        app.world_mut()
            .spawn((ParticleType::new("sand"), Marker(1)));
        app.world_mut()
            .spawn((ParticleType::new("water"), Marker(2)));
        app.update();

        app.world_mut()
            .write_message(SpawnParticleSignal::new(Particle::new("sand"), IVec2::ZERO));
        app.update();

        let particle_entity = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .next()
            .unwrap();

        assert_eq!(
            app.world().entity(particle_entity).get::<Marker>(),
            Some(&Marker(1))
        );

        app.world_mut()
            .entity_mut(particle_entity)
            .get_mut::<Particle>()
            .unwrap()
            .name = "water".into();
        app.update();
        app.update();

        assert_eq!(
            app.world().entity(particle_entity).get::<Marker>(),
            Some(&Marker(2)),
            "Marker should reflect the new ParticleType's value"
        );
    }

    // ---- sync_particle_signal ----

    #[test]
    fn msgr_sync_particle_triggers_change_detection_by_entity() {
        let mut app = create_test_app();
        app.register_particle_sync_component::<Marker>();

        let pt_entity = app
            .world_mut()
            .spawn((ParticleType::new("sand"), Marker(1)))
            .id();
        app.update();

        app.world_mut()
            .write_message(SpawnParticleSignal::new(Particle::new("sand"), IVec2::ZERO));
        app.update();

        let particle_entity = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .next()
            .unwrap();

        assert_eq!(
            app.world().entity(particle_entity).get::<Marker>(),
            Some(&Marker(1))
        );

        app.world_mut().entity_mut(pt_entity).insert(Marker(99));

        app.update();
        assert_eq!(
            app.world().entity(particle_entity).get::<Marker>(),
            Some(&Marker(1))
        );

        app.world_mut()
            .write_message(SyncParticleSignal::from_entity(particle_entity));
        app.update();

        assert_eq!(
            app.world().entity(particle_entity).get::<Marker>(),
            Some(&Marker(99)),
            "SyncParticleSignal should trigger re-propagation of registered components"
        );
    }

    #[test]
    fn msgr_sync_particle_triggers_change_detection_by_position() {
        let mut app = create_test_app();
        app.register_particle_sync_component::<Marker>();

        let pt_entity = app
            .world_mut()
            .spawn((ParticleType::new("sand"), Marker(1)))
            .id();
        app.update();

        let position = IVec2::new(5, 5);
        app.world_mut()
            .write_message(SpawnParticleSignal::new(Particle::new("sand"), position));
        app.update();

        let particle_entity = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .next()
            .unwrap();

        app.world_mut().entity_mut(pt_entity).insert(Marker(50));
        app.update();

        app.world_mut()
            .write_message(SyncParticleSignal::from_position(position));
        app.update();

        assert_eq!(
            app.world().entity(particle_entity).get::<Marker>(),
            Some(&Marker(50)),
        );
    }

    // ---- msgr_sync_particle_type_children ----

    #[test]
    fn msgr_sync_particle_type_children_by_name() {
        let mut app = create_test_app();
        app.register_particle_sync_component::<Marker>();

        let pt_entity = app
            .world_mut()
            .spawn((ParticleType::new("sand"), Marker(1)))
            .id();
        app.update();

        for i in 0..5 {
            app.world_mut().write_message(SpawnParticleSignal::new(
                Particle::new("sand"),
                IVec2::new(i, 0),
            ));
        }
        app.update();

        let particle_entities: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .collect();
        assert_eq!(particle_entities.len(), 5);

        app.world_mut().entity_mut(pt_entity).insert(Marker(77));
        app.update();

        for &e in &particle_entities {
            assert_eq!(app.world().entity(e).get::<Marker>(), Some(&Marker(1)));
        }

        app.world_mut()
            .write_message(SyncParticleTypeChildrenSignal::from_name("sand".into()));
        app.update();
        app.update();

        for &e in &particle_entities {
            assert_eq!(
                app.world().entity(e).get::<Marker>(),
                Some(&Marker(77)),
                "All children should be synced after SyncParticleTypeChildrenSignal"
            );
        }
    }

    #[test]
    fn msgr_sync_particle_type_children_by_entity() {
        let mut app = create_test_app();
        app.register_particle_sync_component::<Marker>();

        let pt_entity = app
            .world_mut()
            .spawn((ParticleType::new("sand"), Marker(1)))
            .id();
        app.update();

        for i in 0..3 {
            app.world_mut().write_message(SpawnParticleSignal::new(
                Particle::new("sand"),
                IVec2::new(i, 0),
            ));
        }
        app.update();

        let particle_entities: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .collect();

        app.world_mut().entity_mut(pt_entity).insert(Marker(33));
        app.update();

        app.world_mut()
            .write_message(SyncParticleTypeChildrenSignal::from_parent_handle(
                pt_entity,
            ));
        app.update();
        app.update();

        for &e in &particle_entities {
            assert_eq!(app.world().entity(e).get::<Marker>(), Some(&Marker(33)));
        }
    }

    #[test]
    fn msgr_sync_particle_type_children_only_affects_matching_type() {
        let mut app = create_test_app();
        app.register_particle_sync_component::<Marker>();

        let sand_pt = app
            .world_mut()
            .spawn((ParticleType::new("sand"), Marker(1)))
            .id();
        app.world_mut()
            .spawn((ParticleType::new("water"), Marker(2)));
        app.update();

        app.world_mut()
            .write_message(SpawnParticleSignal::new(Particle::new("sand"), IVec2::ZERO));
        app.world_mut().write_message(SpawnParticleSignal::new(
            Particle::new("water"),
            IVec2::new(1, 0),
        ));
        app.update();

        app.world_mut().entity_mut(sand_pt).insert(Marker(99));
        app.update();

        app.world_mut()
            .write_message(SyncParticleTypeChildrenSignal::from_name("sand".into()));
        app.update();
        app.update();

        let particles: Vec<(Particle, Marker)> = app
            .world_mut()
            .query::<(&Particle, &Marker)>()
            .iter(app.world())
            .map(|(p, m)| (p.clone(), m.clone()))
            .collect();

        let sand_marker = particles
            .iter()
            .find(|(p, _)| p.name == "sand")
            .map(|(_, m)| m);
        let water_marker = particles
            .iter()
            .find(|(p, _)| p.name == "water")
            .map(|(_, m)| m);

        assert_eq!(sand_marker, Some(&Marker(99)));
        assert_eq!(
            water_marker,
            Some(&Marker(2)),
            "Water particle should be unaffected"
        );
    }

    // ---- propagator_filter ----

    #[test]
    fn sync_with_filter_only_syncs_specified_component() {
        let mut app = create_test_app();
        app.register_particle_sync_component::<Marker>();
        app.register_particle_sync_component::<Marker2>();

        app.world_mut()
            .spawn((ParticleType::new("sand"), Marker(1), Marker2(2)));
        app.update();

        app.world_mut()
            .write_message(SpawnParticleSignal::new(Particle::new("sand"), IVec2::ZERO));
        app.update();

        let particle_entity = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .next()
            .unwrap();

        assert_eq!(
            app.world().entity(particle_entity).get::<Marker>(),
            Some(&Marker(1))
        );
        assert_eq!(
            app.world().entity(particle_entity).get::<Marker2>(),
            Some(&Marker2(2))
        );

        app.world_mut()
            .entity_mut(particle_entity)
            .remove::<Marker>();
        app.world_mut()
            .entity_mut(particle_entity)
            .remove::<Marker2>();

        app.world_mut()
            .write_message(SyncParticleSignal::from_entity(particle_entity).with::<Marker>());
        app.update();
        app.update();

        assert_eq!(
            app.world().entity(particle_entity).get::<Marker>(),
            Some(&Marker(1)),
            "Marker should be synced by the with::<Marker>() filter"
        );
        assert_eq!(
            app.world().entity(particle_entity).get::<Marker2>(),
            None,
            "Marker2 should NOT be synced because it was not included in the filter"
        );
    }

    #[test]
    fn sync_without_filter_excludes_specified_component() {
        let mut app = create_test_app();
        app.register_particle_sync_component::<Marker>();
        app.register_particle_sync_component::<Marker2>();

        app.world_mut()
            .spawn((ParticleType::new("sand"), Marker(1), Marker2(2)));
        app.update();

        app.world_mut()
            .write_message(SpawnParticleSignal::new(Particle::new("sand"), IVec2::ZERO));
        app.update();

        let particle_entity = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .next()
            .unwrap();

        app.world_mut()
            .entity_mut(particle_entity)
            .remove::<Marker>();
        app.world_mut()
            .entity_mut(particle_entity)
            .remove::<Marker2>();

        app.world_mut()
            .write_message(SyncParticleSignal::from_entity(particle_entity).without::<Marker>());
        app.update();
        app.update();

        assert_eq!(
            app.world().entity(particle_entity).get::<Marker>(),
            None,
            "Marker should NOT be synced because it was excluded by the filter"
        );
        assert_eq!(
            app.world().entity(particle_entity).get::<Marker2>(),
            Some(&Marker2(2)),
            "Marker2 should be synced because it was not excluded"
        );
    }

    #[test]
    fn sync_children_signal_propagates_filter() {
        let mut app = create_test_app();
        app.register_particle_sync_component::<Marker>();
        app.register_particle_sync_component::<Marker2>();

        app.world_mut()
            .spawn((ParticleType::new("sand"), Marker(10), Marker2(20)));
        app.update();

        app.world_mut()
            .write_message(SpawnParticleSignal::new(Particle::new("sand"), IVec2::ZERO));
        app.world_mut().write_message(SpawnParticleSignal::new(
            Particle::new("sand"),
            IVec2::new(1, 0),
        ));
        app.update();

        let particle_entities: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .collect();
        assert_eq!(particle_entities.len(), 2);

        for &entity in &particle_entities {
            app.world_mut().entity_mut(entity).remove::<Marker>();
            app.world_mut().entity_mut(entity).remove::<Marker2>();
        }

        app.world_mut().write_message(
            SyncParticleTypeChildrenSignal::from_name("sand".into()).with::<Marker>(),
        );
        app.update();
        app.update();

        for &entity in &particle_entities {
            assert_eq!(
                app.world().entity(entity).get::<Marker>(),
                Some(&Marker(10)),
                "Marker should be synced via children signal filter"
            );
            assert_eq!(
                app.world().entity(entity).get::<Marker2>(),
                None,
                "Marker2 should NOT be synced — excluded by children signal filter"
            );
        }
    }

    #[test]
    fn propagator_filter_default_is_all() {
        assert_eq!(PropagatorFilter::default(), PropagatorFilter::All);
    }

    // ---- chance_mutation ----

    fn spawn_particle_at(app: &mut App, name: &'static str, position: IVec2) -> Entity {
        app.world_mut()
            .write_message(SpawnParticleSignal::new(Particle::new(name), position));
        app.update();

        app.world()
            .resource::<ParticleMap>()
            .get_copied(position)
            .unwrap()
            .unwrap()
    }

    #[test]
    fn chance_mutation_default() {
        let mutation = ChanceMutation::default();
        assert_eq!(mutation.target, "");
        assert_eq!(mutation.chance, 0.0);
        assert_eq!(mutation.tick_timer.duration(), Duration::ZERO);
    }

    #[test]
    fn chance_mutation_new() {
        let mutation = ChanceMutation::new("water", 0.5, Duration::from_millis(100));
        assert_eq!(mutation.target, "water");
        assert_eq!(mutation.chance, 0.5);
        assert_eq!(mutation.tick_timer.duration(), Duration::from_millis(100));
    }

    #[test]
    fn chance_mutation_from_string() {
        let mutation =
            ChanceMutation::from_string("water".to_string(), 0.5, Duration::from_millis(100));
        assert_eq!(mutation.target, "water");
        assert_eq!(mutation.chance, 0.5);
    }

    #[test]
    fn chance_mutation_zero_never_mutates() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.world_mut().spawn(ParticleType::new("water"));
        app.update();

        let position = IVec2::ZERO;
        let entity = spawn_particle_at(&mut app, "sand", position);

        app.world_mut()
            .entity_mut(entity)
            .insert(ChanceMutation::new("water", 0.0, Duration::ZERO));

        for _ in 0..100 {
            app.update();
        }

        let particle = app.world().entity(entity).get::<Particle>().unwrap();
        assert_eq!(particle.name, "sand");
    }

    #[test]
    fn chance_mutation_one_always_mutates() {
        let mut app = create_test_app();
        let sand_pt = app.world_mut().spawn(ParticleType::new("sand")).id();
        let water_pt = app.world_mut().spawn(ParticleType::new("water")).id();
        app.update();

        let position = IVec2::ZERO;
        let entity = spawn_particle_at(&mut app, "sand", position);

        let attached = app
            .world()
            .entity(entity)
            .get::<AttachedToParticleType>()
            .unwrap();
        assert_eq!(attached.0, sand_pt);

        app.world_mut()
            .entity_mut(entity)
            .insert(ChanceMutation::new("water", 1.0, Duration::ZERO));

        app.update();
        app.update();

        let particle = app.world().entity(entity).get::<Particle>().unwrap();
        assert_eq!(particle.name, "water");

        let attached = app
            .world()
            .entity(entity)
            .get::<AttachedToParticleType>()
            .unwrap();
        assert_eq!(attached.0, water_pt);
    }

    #[test]
    fn chance_mutation_respects_tick_rate() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.world_mut().spawn(ParticleType::new("water"));
        app.update();

        let position = IVec2::ZERO;
        let entity = spawn_particle_at(&mut app, "sand", position);

        app.world_mut()
            .entity_mut(entity)
            .insert(ChanceMutation::new("water", 1.0, Duration::from_secs(999)));
        app.update();
        app.update();

        let particle = app.world().entity(entity).get::<Particle>().unwrap();
        assert_eq!(particle.name, "sand");

        *app.world_mut()
            .entity_mut(entity)
            .get_mut::<ChanceMutation>()
            .unwrap() = ChanceMutation::new("water", 1.0, Duration::ZERO);
        app.update();
        app.update();

        let particle = app.world().entity(entity).get::<Particle>().unwrap();
        assert_eq!(particle.name, "water");
    }

    #[test]
    fn chance_mutation_unregistered_target_reverts() {
        let mut app = create_test_app();
        let sand_pt = app.world_mut().spawn(ParticleType::new("sand")).id();
        app.update();

        let position = IVec2::ZERO;
        let entity = spawn_particle_at(&mut app, "sand", position);

        app.world_mut()
            .entity_mut(entity)
            .insert(ChanceMutation::new("ghost", 1.0, Duration::ZERO));

        app.update();
        app.update();

        let particle = app.world().entity(entity).get::<Particle>().unwrap();
        assert_eq!(
            particle.name, "sand",
            "Mutation to an unregistered type should be reverted"
        );

        let attached = app
            .world()
            .entity(entity)
            .get::<AttachedToParticleType>()
            .unwrap();
        assert_eq!(attached.0, sand_pt);
    }

    #[test]
    fn chance_mutation_propagates_from_particle_type() {
        let mut app = create_test_app();
        app.world_mut().spawn((
            ParticleType::new("sand"),
            ChanceMutation::new("water", 1.0, Duration::ZERO),
        ));
        app.world_mut().spawn(ParticleType::new("water"));
        app.update();

        let position = IVec2::ZERO;
        let entity = spawn_particle_at(&mut app, "sand", position);

        assert!(
            app.world().entity(entity).get::<ChanceMutation>().is_some(),
            "ChanceMutation should be propagated from ParticleType to child Particle"
        );

        app.update();
        app.update();

        let particle = app.world().entity(entity).get::<Particle>().unwrap();
        assert_eq!(particle.name, "water");
    }
}
