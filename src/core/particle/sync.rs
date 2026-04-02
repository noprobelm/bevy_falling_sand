//! Provides synchronization behaviors between [`Particle`] entities and their associated
//! [`ParticleType`].
//!
//! A `Particle` will synchronize with its parent on 2 occasions:
//! - A [`SyncParticleSignal`] has been received (sent explicitly or via
//!   [`SyncParticleTypeChildrenSignal`]).
//! - `Changed<Particle>` fires (e.g., on spawn or when [`Particle::name`] is mutated).
//!
//! The [`ParticleSyncExt`] trait provides an interface for adding your own particle sync
//! components.
use super::LocateBy;
use crate::core::{
    AttachedToParticleType, ChanceLifetime, ChunkDirtyState, ChunkIndex, GridPosition, Particle,
    ParticleMap, ParticleSystems, ParticleType, ParticleTypeRegistry, TimedLifetime,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use serde::{Deserialize, Serialize};

pub(super) struct SyncPlugin;

impl Plugin for SyncPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticlePropagators>()
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
            .add_systems(
                PreUpdate,
                msgr_sync_particle.in_set(ParticleSystems::Registration),
            );
    }
}

type PropagateFn = Box<dyn Fn(Entity, Entity, &mut Commands) + Send + Sync>;

#[derive(Resource, Default)]
pub(crate) struct ParticlePropagators(Vec<PropagateFn>);

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
/// There are two registration methods:
///
/// - [`register_particle_sync_component`](`ParticleSyncExt::register_particle_sync_component`):
///   Clones a component from the parent [`ParticleType`] onto each child [`Particle`]. If the
///   parent doesn't have the component, it is removed from the child. This covers the majority of
///   use cases.
///
/// - [`register_particle_propagator`](`ParticleSyncExt::register_particle_propagator`): Accepts
///   an arbitrary closure for cases where a simple clone isn't enough (e.g. conditional insertion,
///   deriving values from the parent, etc.).
///
/// After registration, updating the component on a [`ParticleType`] entity and sending a
/// [`SyncParticleTypeChildrenSignal`] will propagate the change to all living particles of that
/// type.
pub trait ParticleSyncExt {
    /// Registers a component type to be cloned from [`ParticleType`] to child [`Particle`]
    /// entities.
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
    ///         .register_particle_propagator(|particle, parent, commands| {
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
    fn register_particle_propagator<F>(&mut self, propagator: F) -> &mut Self
    where
        F: Fn(Entity, Entity, &mut Commands) + Send + Sync + 'static;
}

impl ParticleSyncExt for App {
    fn register_particle_sync_component<C: Component + Clone>(&mut self) -> &mut Self {
        self.register_particle_propagator(
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

    fn register_particle_propagator<F>(&mut self, propagator: F) -> &mut Self
    where
        F: Fn(Entity, Entity, &mut Commands) + Send + Sync + 'static,
    {
        self.world_mut()
            .resource_mut::<ParticlePropagators>()
            .0
            .push(Box::new(propagator));
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
    use bevy::platform::collections::HashSet;

    let mut seen = HashSet::new();
    let mut targets: Vec<(Entity, Entity, IVec2)> = Vec::new();

    for signal in params.msgr.read() {
        let entity = match &signal.locate_by {
            LocateBy::Entity(e) => Some(*e),
            LocateBy::Position(pos) => params.particle_map.get_copied(*pos).ok().flatten(),
            LocateBy::Name(_) => unreachable!(),
        };
        if let Some(entity) = entity {
            if let Ok((attached, grid_pos)) = params.particle_query.get(entity) {
                if seen.insert(entity) {
                    targets.push((entity, attached.0, grid_pos.0));
                }
            }
        }
    }

    for (entity, attached, grid_pos) in &params.changed_query {
        if seen.insert(entity) {
            targets.push((entity, attached.0, grid_pos.0));
        }
    }

    if targets.is_empty() {
        return;
    }

    for &(entity, parent, _) in &targets {
        for propagate_fn in &params.propagators.0 {
            propagate_fn(entity, parent, &mut params.commands);
        }
    }

    for &(_, _, position) in &targets {
        let chunk_coord = params.chunk_index.world_to_chunk_coord(position);
        if let Some(chunk_entity) = params.chunk_index.get(chunk_coord) {
            if let Ok(mut dirty_state) = params.chunk_dirty_query.get_mut(chunk_entity) {
                dirty_state.mark_dirty(position);
            }
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

    for propagate_fn in &propagators.0 {
        propagate_fn(entity, parent, &mut commands);
    }

    let chunk_coord = chunk_index.world_to_chunk_coord(position);
    if let Some(chunk_entity) = chunk_index.get(chunk_coord) {
        if let Ok(mut dirty_state) = chunk_dirty_query.get_mut(chunk_entity) {
            dirty_state.mark_dirty(position);
        }
    }
}

/// Signal for synchronizing a [`Particle`] entity with its [`ParticleType`] parent.
#[derive(Event, Message, Clone, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub struct SyncParticleSignal {
    locate_by: LocateBy,
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
        }
    }
}

/// Signal for syncing all [`Particle`] entities of a matching
/// [`ParticleType`]
#[derive(Event, Message, Clone, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub struct SyncParticleTypeChildrenSignal {
    locate_by: LocateBy,
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
        }
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
                msgr_sync_particle.write(SyncParticleSignal::from_entity(entity));
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
            msgr_sync_particle.write(SyncParticleSignal::from_entity(entity));
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
