//! Provides mechanisms for ergonomic spawning/despawning of particle entities from the simulation
//!
//! Each of these are responsive to both messages and triggers
//! - [`SpawnParticleSignal`]: Spawn a new particle into the simulation
//! - [`DespawnParticleSignal`]: Despawn a particle from the simulation
//! - [`DespawnAllParticlesSignal`]: Despawn all particles from the simulation
//! - [`DespawnParticleTypeChildrenSignal`]: Despawn all particle children of by name or parent
//!   handle.
//!
//! Though it is possible to safely add and remove [`Particle`] entities from the world using
//! [`Commands`] and inserting a [`Transform`] component, which often feels like the idiomatic
//! approach, it is generally preferred to use the signals provided in this module.
//!
//! <div class="warning">
//!
//! Newly spawned [`Particle`] entities with a [`Transform`] will automatically have the
//! [`Transform`] and its required components immediately removed to prevent overhead associated
//! with rebuilding dirty trees when simulating many particles. `bfs` instead uses the
//! [`GridPosition`] component for managing particle positions.
//!
//! </div>

use super::LocateBy;
use crate::core::{
    schedule::ParticleSystems, AttachedToParticleType, ChunkDirtyState, ChunkIndex, GridPosition,
    Particle, ParticleMap, ParticleType, ParticleTypeRegistry,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub(super) struct LifecyclePlugin;

impl Plugin for LifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(PreUpdate, ParticleSystems::Registration)
            .add_message::<SpawnParticleSignal>()
            .add_message::<DespawnParticleSignal>()
            .add_message::<DespawnAllParticlesSignal>()
            .add_message::<DespawnParticleTypeChildrenSignal>()
            .add_observer(on_spawn_particle)
            .add_observer(on_despawn_particle)
            .add_observer(on_despawn_particle_type_children)
            .add_observer(on_despawn_all_particles)
            .add_systems(
                PreUpdate,
                (
                    msgr_spawn_particle.before(ParticleSystems::Registration),
                    msgr_despawn_particle,
                    msgr_despawn_particle_type_children,
                    msgr_despawn_all_particles,
                ),
            )
            .add_systems(
                PreUpdate,
                (
                    mark_positionless_particles_invalid,
                    register_transform_particles,
                    despawn_orphaned_particles,
                )
                    .chain()
                    .before(ParticleSystems::Registration),
            )
            .add_systems(
                PreUpdate,
                (ApplyDeferred, despawn_invalid_particles)
                    .chain()
                    .after(ParticleSystems::Registration),
            );
    }
}

/// Callback type for custom entity setup during particle spawning routines.
pub type OnSpawnCallback = Arc<dyn Fn(&mut EntityCommands) + Send + Sync>;

/// # Spawning particles
///
/// Spawns a particle into the simulation, with extra options for behavior if an entity already
/// exists at the desired position.
///
/// - [`SpawnParticleSignal::new`] will attempt to spawn a particle at the specified position,
///   silently failing if one already exists.
/// - [`SpawnParticleSignal::overwrite_existing`] will spawn a particle at the specified position,
///   overwriting any occupying entity.
/// - [`SpawnParticleSignal::try_multiple`] accepts an ordered list of desired spawn locations,
///   short-circuiting as soon as a vacancy is found.
///
/// # Hooking custom components during spawn
///
/// Sometimes it may be desired to spawn a particle with additional behavior not managed by
/// [particle synchronization](crate::sync) routines.
///
/// The [`SpawnParticleSignal::with_on_spawn`] accepts a closure, providing the caller with
/// [`EntityCommands`] access for the particle entity being spawned.
///
/// This function only executes if the particle is spawned successfully. If unconditional execution
/// is desired, write your own message readers or observers for `SpawnParticleSignal`.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::prelude::*;
///
/// #[derive(Component)]
/// struct OnFire;
///
/// fn spawn_burning(mut writer: MessageWriter<SpawnParticleSignal>) {
///     writer.write(
///         SpawnParticleSignal::new(Particle::new("Wood"), IVec2::new(5, 5))
///             .with_on_spawn(|cmd| { cmd.insert(OnFire); }),
///     );
/// }
/// ```
#[derive(Event, Message, Clone, Reflect, Serialize, Deserialize)]
pub struct SpawnParticleSignal {
    pub(crate) particle: Particle,
    pub(crate) positions: Vec<IVec2>,
    pub(crate) overwrite_existing: bool,
    #[serde(skip)]
    #[reflect(ignore)]
    pub(crate) on_spawn: Option<OnSpawnCallback>,
}

impl std::fmt::Debug for SpawnParticleSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpawnParticleSignal")
            .field("particle", &self.particle)
            .field("positions", &self.positions)
            .field("overwrite_existing", &self.overwrite_existing)
            .field("on_spawn", &self.on_spawn.as_ref().map(|_| "..."))
            .finish()
    }
}

impl SpawnParticleSignal {
    /// Attempt to spawn a particle at position, leaving any existing particle in place.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::*;
    ///
    /// fn spawn(mut writer: MessageWriter<SpawnParticleSignal>) {
    ///     writer.write(SpawnParticleSignal::new(
    ///         Particle::new("Sand"),
    ///         IVec2::new(10, 20),
    ///     ));
    /// }
    /// ```
    #[must_use]
    pub fn new(particle: Particle, position: IVec2) -> Self {
        Self {
            particle,
            positions: vec![position],
            overwrite_existing: false,
            on_spawn: None,
        }
    }

    /// Spawn a particle at position, overwriting any existing particle at the specified position.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::*;
    ///
    /// fn replace(mut writer: MessageWriter<SpawnParticleSignal>) {
    ///     writer.write(SpawnParticleSignal::overwrite_existing(
    ///         Particle::new("Water"),
    ///         IVec2::new(10, 20),
    ///     ));
    /// }
    /// ```
    #[must_use]
    pub fn overwrite_existing(particle: Particle, position: IVec2) -> Self {
        Self {
            particle,
            positions: vec![position],
            overwrite_existing: true,
            on_spawn: None,
        }
    }

    /// Attempt to spawn a particle at `positions` until a valid position is found, at which
    /// point short-circuit occurs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::*;
    ///
    /// // If position (11, 20) is vacant, short circuit and exit early.
    /// fn spawn_fallback(mut writer: MessageWriter<SpawnParticleSignal>) {
    ///     writer.write(SpawnParticleSignal::try_multiple(
    ///         Particle::new("Sand"),
    ///         vec![IVec2::new(10, 20), IVec2::new(11, 20), IVec2::new(12, 20)],
    ///     ));
    /// }
    /// ```
    #[must_use]
    pub const fn try_multiple(particle: Particle, positions: Vec<IVec2>) -> Self {
        Self {
            particle,
            positions,
            overwrite_existing: false,
            on_spawn: None,
        }
    }

    /// Add a callback to further customize the spawned entity.
    ///
    /// This is useful if the user wants to add their own callback routines with
    /// [`EntityCommands`], which will only be executed on "valid" particle spawns. Invalid
    /// particle spawns (i.e., a position is occupied or we run out of positions to try) will
    /// skip this logic, potentially saving some ECS overhead.
    ///
    /// If more complex behaviors are desired, you can still your own message reader.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct OnFire;
    ///
    /// fn spawn_burning(mut writer: MessageWriter<SpawnParticleSignal>) {
    ///     writer.write(
    ///         SpawnParticleSignal::new(Particle::new("Wood"), IVec2::new(5, 5))
    ///             .with_on_spawn(|cmd| { cmd.insert(OnFire); }),
    ///     );
    /// }
    /// ```
    #[must_use]
    pub fn with_on_spawn<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut EntityCommands) + Send + Sync + 'static,
    {
        self.on_spawn = Some(match self.on_spawn {
            Some(existing) => Arc::new(move |cmd| {
                existing(cmd);
                callback(cmd);
            }),
            None => Arc::new(callback),
        });
        self
    }
}

/// Despawns a matching [`Particle`]
///
/// - [`DespawnParticleSignal::from_position`]: Despawn a particle from position
/// - [`DespawnParticleSignal::from_entity`]: Despawn a particle from entity ID
#[derive(Event, Message, Clone, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub struct DespawnParticleSignal {
    locate_by: LocateBy,
}

impl DespawnParticleSignal {
    /// Initialize from the particle's position.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::DespawnParticleSignal;
    ///
    /// fn despawn(mut writer: MessageWriter<DespawnParticleSignal>) {
    ///     writer.write(DespawnParticleSignal::from_position(IVec2::new(10, 20)));
    /// }
    /// ```
    #[must_use]
    pub const fn from_position(position: IVec2) -> Self {
        Self {
            locate_by: LocateBy::Position(position),
        }
    }

    /// Initialize from the [`Particle`] [`Entity`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::prelude::{DespawnParticleSignal, Particle};
    ///
    /// fn despawn(
    ///     mut writer: MessageWriter<DespawnParticleSignal>,
    ///     query: Query<Entity, With<Particle>>,
    /// ) {
    ///     for entity in &query {
    ///         writer.write(DespawnParticleSignal::from_entity(entity));
    ///     }
    /// }
    /// ```
    #[must_use]
    pub const fn from_entity(entity: Entity) -> Self {
        Self {
            locate_by: LocateBy::Entity(entity),
        }
    }
}

/// Used to despawn all particles from the simulation.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::core::DespawnAllParticlesSignal;
///
/// fn clear_all(mut writer: MessageWriter<DespawnAllParticlesSignal>) {
///     writer.write(DespawnAllParticlesSignal);
/// }
/// ```
#[derive(
    Event,
    Message,
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
pub struct DespawnAllParticlesSignal;

/// Despawns all particle children under a type
/// [`ParticleType`].
#[derive(Event, Message, Clone, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub struct DespawnParticleTypeChildrenSignal {
    locate_by: LocateBy,
}

impl DespawnParticleTypeChildrenSignal {
    /// Initialize from the [`Particle`] name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::DespawnParticleTypeChildrenSignal;
    ///
    /// fn despawn_all_sand(mut writer: MessageWriter<DespawnParticleTypeChildrenSignal>) {
    ///     writer.write(DespawnParticleTypeChildrenSignal::from_name("Sand"));
    /// }
    /// ```
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        Self {
            locate_by: LocateBy::Name(name.to_string()),
        }
    }

    /// Initialize from the [`Particle`] [`Entity`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_falling_sand::core::{DespawnParticleTypeChildrenSignal, ParticleTypeRegistry};
    ///
    /// fn despawn_by_entity(
    ///     mut writer: MessageWriter<DespawnParticleTypeChildrenSignal>,
    ///     registry: Res<ParticleTypeRegistry>,
    /// ) {
    ///     if let Some(&entity) = registry.get("Sand") {
    ///         writer.write(DespawnParticleTypeChildrenSignal::from_parent_handle(entity));
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

/// Spawns a new particle into the simulation when a [`SpawnParticleSignal`] is received.
///
/// This system ensures [`Particle`] entities are correctly synchronized with the
/// [`ParticleMap`].
///
/// After all valid spawn positions have been collected, mark each [`ChunkDirtyState`] so newly
/// spawned particlces (and their neighbors) are included in simulation systems.
#[allow(clippy::needless_pass_by_value)]
fn msgr_spawn_particle(
    mut msgr_spawn_particle: MessageReader<SpawnParticleSignal>,
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
    registry: Res<ParticleTypeRegistry>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
) {
    use bevy::platform::collections::HashMap;

    let mut pending_overwrites: HashMap<IVec2, (Particle, Entity, Option<OnSpawnCallback>)> =
        HashMap::default();
    let mut spawned_positions: Vec<IVec2> = Vec::new();

    msgr_spawn_particle.read().for_each(|msg| {
        if let Some(parent_handle) = registry.get(&msg.particle.name) {
            let on_spawn = msg.on_spawn.clone();
            for position in &msg.positions {
                if msg.overwrite_existing {
                    pending_overwrites.insert(
                        *position,
                        (msg.particle.clone(), *parent_handle, on_spawn.clone()),
                    );
                } else if map.is_position_loaded(*position) {
                    let on_spawn = on_spawn.clone();
                    if let Ok(mut entry) = map.entry(*position) {
                        if entry.insert_if_vacant_with(|| {
                            let mut entity_commands = commands.spawn((
                                msg.particle.clone(),
                                GridPosition(*position),
                                AttachedToParticleType(*parent_handle),
                            ));
                            if let Some(ref callback) = on_spawn {
                                callback(&mut entity_commands);
                            }
                            entity_commands.id()
                        }) {
                            spawned_positions.push(*position);
                            return;
                        }
                    }
                }
            }
        }
    });

    for (position, (particle, parent_handle, on_spawn)) in pending_overwrites {
        let mut entity_commands = commands.spawn((
            particle,
            GridPosition(position),
            AttachedToParticleType(parent_handle),
        ));
        if let Some(ref callback) = on_spawn {
            callback(&mut entity_commands);
        }
        let id = entity_commands.id();
        if let Ok(Some(old_entity)) = map.insert(position, id) {
            commands.entity(old_entity).try_despawn();
        }
        spawned_positions.push(position);
    }

    for position in spawned_positions {
        let chunk_coord = chunk_index.world_to_chunk_coord(position);
        if let Some(chunk_entity) = chunk_index.get(chunk_coord) {
            if let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) {
                dirty_state.mark_dirty(position);
            }
        }
    }
}

/// Spawns a new particle into the simulation when a [`SpawnParticleSignal`] is triggered.
///
/// This system ensures [`Particle`] entities are correctly synchronized with the
/// [`ParticleMap`].
///
/// After all valid spawn positions have been collected, mark each [`ChunkDirtyState`] so newly
/// spawned particlces (and their neighbors) are included in simulation systems.
#[allow(clippy::needless_pass_by_value)]
fn on_spawn_particle(
    trigger: On<SpawnParticleSignal>,
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
    registry: Res<ParticleTypeRegistry>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
) {
    use bevy::platform::collections::HashMap;

    let mut pending_overwrites: HashMap<IVec2, (Particle, Entity, Option<OnSpawnCallback>)> =
        HashMap::default();
    let mut spawned_positions: Vec<IVec2> = Vec::new();

    let event = trigger.event();
    if let Some(parent_handle) = registry.get(&event.particle.name) {
        let on_spawn = event.on_spawn.clone();
        for position in &event.positions {
            if event.overwrite_existing {
                pending_overwrites.insert(
                    *position,
                    (event.particle.clone(), *parent_handle, on_spawn.clone()),
                );
            } else if map.is_position_loaded(*position) {
                let on_spawn = on_spawn.clone();
                if let Ok(mut entry) = map.entry(*position) {
                    if entry.insert_if_vacant_with(|| {
                        let mut entity_commands = commands.spawn((
                            event.particle.clone(),
                            GridPosition(*position),
                            AttachedToParticleType(*parent_handle),
                        ));
                        if let Some(ref callback) = on_spawn {
                            callback(&mut entity_commands);
                        }
                        entity_commands.id()
                    }) {
                        spawned_positions.push(*position);
                        return;
                    }
                }
            }
        }
    }

    for (position, (particle, parent_handle, on_spawn)) in pending_overwrites {
        let mut entity_commands = commands.spawn((
            particle,
            GridPosition(position),
            AttachedToParticleType(parent_handle),
        ));
        if let Some(ref callback) = on_spawn {
            callback(&mut entity_commands);
        }
        let id = entity_commands.id();
        if let Ok(Some(old_entity)) = map.insert(position, id) {
            commands.entity(old_entity).try_despawn();
        }
        spawned_positions.push(position);
    }

    for position in spawned_positions {
        let chunk_coord = chunk_index.world_to_chunk_coord(position);
        if let Some(chunk_entity) = chunk_index.get(chunk_coord) {
            if let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) {
                dirty_state.mark_dirty(position);
            }
        }
    }
}

/// Despawns a particle from the simulation when a [`DespawnParticleSignal`] message is received.
///
/// After all despawn positions have been collected, mark each [`ChunkDirtyState`] so that
/// neighbors are included in simulation systems.
#[allow(clippy::needless_pass_by_value)]
fn msgr_despawn_particle(
    mut msgr_remove_particle: MessageReader<DespawnParticleSignal>,
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    particle_query: Query<&GridPosition, With<Particle>>,
) {
    let mut despawned_positions = if msgr_remove_particle.is_empty() {
        return;
    } else {
        Vec::new()
    };

    msgr_remove_particle
        .read()
        .for_each(|msg| match &msg.locate_by {
            LocateBy::Position(position) => {
                if let Ok(Some(entity)) = map.remove(*position) {
                    commands.entity(entity).try_despawn();
                    despawned_positions.push(*position);
                } else {
                    debug!(
                        "Attempted to despawn particle from position where none exists: {:?}",
                        position
                    );
                }
            }
            LocateBy::Entity(entity) => {
                if let Ok(grid_position) = particle_query.get(*entity) {
                    let position = grid_position.0;
                    if map.get_copied(position) == Ok(Some(*entity)) {
                        let _ = map.remove(position);
                    }
                    commands.entity(*entity).try_despawn();
                    despawned_positions.push(position);
                } else {
                    debug!(
                        "Attempted to despawn non-particle entity using DespawnParticleSignal: {:?}",
                        entity
                    );
                }
            }
            LocateBy::Name(_) => {
                unreachable!()
            }
        });

    for position in despawned_positions {
        let chunk_coord = chunk_index.world_to_chunk_coord(position);
        if let Some(chunk_entity) = chunk_index.get(chunk_coord) {
            if let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) {
                dirty_state.mark_dirty(position);
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
/// Despawns a particle from the simulation when a [`DespawnParticleSignal`] trigger is received.
///
/// After all despawn positions have been collected, mark each [`ChunkDirtyState`] so that
/// neighbors are included in simulation systems.
pub fn on_despawn_particle(
    trigger: On<DespawnParticleSignal>,
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    particle_query: Query<&GridPosition, With<Particle>>,
) {
    let mut despawned_positions: Vec<IVec2> = Vec::new();
    let event = trigger.event();

    match &event.locate_by {
        LocateBy::Position(position) => {
            if let Ok(Some(entity)) = map.remove(*position) {
                commands.entity(entity).try_despawn();
                despawned_positions.push(*position);
            } else {
                debug!(
                    "Attempted to despawn particle from position where none exists: {:?}",
                    position
                );
            }
        }
        LocateBy::Entity(entity) => {
            if let Ok(grid_position) = particle_query.get(*entity) {
                let position = grid_position.0;
                if map.get_copied(position) == Ok(Some(*entity)) {
                    let _ = map.remove(position);
                }
                commands.entity(*entity).try_despawn();
                despawned_positions.push(position);
            } else {
                debug!(
                    "Attempted to despawn non-particle entity using DespawnParticleSignal: {:?}",
                    entity
                );
            }
        }
        LocateBy::Name(_) => {
            unreachable!()
        }
    }

    for position in despawned_positions {
        let chunk_coord = chunk_index.world_to_chunk_coord(position);
        if let Some(chunk_entity) = chunk_index.get(chunk_coord) {
            if let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) {
                dirty_state.mark_dirty(position);
            }
        }
    }
}

/// Despawns all [`Particle`] entities of a given [`ParticleType`] from the simulation when a
/// [`DespawnParticleTypeChildrenSignal`] message is received.
///
/// After all despawn positions have been collected, mark each [`ChunkDirtyState`] so that
/// neighbors are included in simulation systems.
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
fn msgr_despawn_particle_type_children(
    mut msgr_clear_particle_type_children: MessageReader<DespawnParticleTypeChildrenSignal>,
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    particle_query: Query<(Entity, &GridPosition, &AttachedToParticleType), With<Particle>>,
    registry: Res<ParticleTypeRegistry>,
) {
    let mut despawned_positions = if msgr_clear_particle_type_children.is_empty() {
        return;
    } else {
        Vec::new()
    };

    msgr_clear_particle_type_children.read().for_each(|msg| {
        let parent_entity = match &msg.locate_by {
            LocateBy::Name(name) => registry.get(name.as_str()),
            LocateBy::Entity(parent_entity) => Some(parent_entity),
            LocateBy::Position(_) => {
                unreachable!()
            }
        };
        if let Some(parent_entity) = parent_entity {
            for (child_entity, position, _) in particle_query
                .iter()
                .filter(|(_, _, attached)| attached.0 == *parent_entity)
            {
                let _ = map.remove(position.0);
                commands.entity(child_entity).try_despawn();
                despawned_positions.push(position.0);
            }
        }
    });

    for position in despawned_positions {
        let chunk_coord = chunk_index.world_to_chunk_coord(position);
        if let Some(chunk_entity) = chunk_index.get(chunk_coord) {
            if let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) {
                dirty_state.mark_dirty(position);
            }
        }
    }
}

/// Despawns all [`Particle`] entities of a given [`ParticleType`] from the simulation when a
/// [`DespawnParticleTypeChildrenSignal`] trigger is received.
///
/// After all despawn positions have been collected, mark each [`ChunkDirtyState`] so that
/// neighbors are included in simulation systems.
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
fn on_despawn_particle_type_children(
    trigger: On<DespawnParticleTypeChildrenSignal>,
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    particle_query: Query<(Entity, &GridPosition, &AttachedToParticleType), With<Particle>>,
    registry: Res<ParticleTypeRegistry>,
) {
    let mut despawned_positions: Vec<IVec2> = Vec::new();

    let parent_entity = match &trigger.event().locate_by {
        LocateBy::Name(name) => registry.get(name.as_str()),
        LocateBy::Entity(parent_entity) => Some(parent_entity),
        LocateBy::Position(_) => {
            unreachable!()
        }
    };
    if let Some(parent_entity) = parent_entity {
        for (child_entity, position, _) in particle_query
            .iter()
            .filter(|(_, _, attached)| attached.0 == *parent_entity)
        {
            let _ = map.remove(position.0);
            commands.entity(child_entity).try_despawn();
            despawned_positions.push(position.0);
        }
    }
    for position in despawned_positions {
        let chunk_coord = chunk_index.world_to_chunk_coord(position);
        if let Some(chunk_entity) = chunk_index.get(chunk_coord) {
            if let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) {
                dirty_state.mark_dirty(position);
            }
        }
    }
}

/// Despawn all [`Particle`] entities from the simulation when a [`DespawnAllParticlesSignal`]
/// message is received.
///
/// After all despawn positions have been collected, mark each [`ChunkDirtyState`] so that
/// neighbors are included in simulation systems.
#[allow(clippy::needless_pass_by_value)]
fn msgr_despawn_all_particles(
    mut msgr_clear_particle_map: MessageReader<DespawnAllParticlesSignal>,
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    particle_query: Query<(Entity, &GridPosition), With<Particle>>,
) {
    msgr_clear_particle_map.read().for_each(|_| {
        for (entity, grid_position) in &particle_query {
            let position = grid_position.0;
            let _ = map.remove(position);
            commands.entity(entity).try_despawn();

            let chunk_coord = chunk_index.world_to_chunk_coord(position);
            if let Some(chunk_entity) = chunk_index.get(chunk_coord) {
                if let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) {
                    dirty_state.mark_dirty(position);
                }
            }
        }
    });
}

/// Despawn all [`Particle`] entities from the simulation when a [`DespawnAllParticlesSignal`]
/// trigger is received.
///
/// After all despawn positions have been collected, mark each [`ChunkDirtyState`] so that
/// neighbors are included in simulation systems.
#[allow(clippy::needless_pass_by_value)]
pub fn on_despawn_all_particles(
    _trigger: On<DespawnAllParticlesSignal>,
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    particle_query: Query<(Entity, &GridPosition), With<Particle>>,
) {
    for (entity, grid_position) in &particle_query {
        let position = grid_position.0;
        let _ = map.remove(position);
        commands.entity(entity).try_despawn();

        let chunk_coord = chunk_index.world_to_chunk_coord(position);
        if let Some(chunk_entity) = chunk_index.get(chunk_coord) {
            if let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) {
                dirty_state.mark_dirty(position);
            }
        }
    }
}

/// Despawns all [`Particle`] entities whose parent [`ParticleType`] has been removed.
#[allow(clippy::needless_pass_by_value)]
fn despawn_orphaned_particles(
    mut commands: Commands,
    mut removed: RemovedComponents<ParticleType>,
    mut map: ResMut<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    particle_query: Query<(Entity, &GridPosition, &AttachedToParticleType), With<Particle>>,
) {
    for removed_entity in removed.read() {
        for (child_entity, grid_pos, _) in particle_query
            .iter()
            .filter(|(_, _, attached)| attached.0 == removed_entity)
        {
            let position = grid_pos.0;
            let _ = map.remove(position);
            commands.entity(child_entity).try_despawn();

            let chunk_coord = chunk_index.world_to_chunk_coord(position);
            if let Some(chunk_entity) = chunk_index.get(chunk_coord) {
                if let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) {
                    dirty_state.mark_dirty(position);
                }
            }
        }
    }
}

/// Marker component for despawning invalid particles
#[derive(Component)]
pub struct InvalidParticle;

/// Despawn particles with the [`InvalidParticle`] component.
fn despawn_invalid_particles(mut commands: Commands, query: Query<Entity, With<InvalidParticle>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Mark derelict particles as invalid
fn mark_positionless_particles_invalid(
    mut commands: Commands,
    query: Query<Entity, (Added<Particle>, Without<Transform>, Without<GridPosition>)>,
) {
    for entity in &query {
        warn!("Particle entity {entity} spawned without position. Removing from world");
        commands.entity(entity).insert(InvalidParticle);
    }
}

/// Registers newly added [`Particle`] entities with a [`Transform`] component with the
/// [`ParticleMap`].
///
/// **Note**: [`Transform`] and its required components are removed after the particle has been
/// registered. This is to avoid overhead associated with Bevy's transform systems, which
/// becomes noticeable when the simulation is managing large numbers of particles.
#[allow(clippy::needless_pass_by_value)]
fn register_transform_particles(
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
    registry: Res<ParticleTypeRegistry>,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    mut particle_query: Query<
        (Entity, &Particle, &Transform),
        (Added<Particle>, Without<GridPosition>),
    >,
) {
    for (entity, particle, transform) in &mut particle_query {
        let position = IVec2::new(
            transform.translation.x.round() as i32,
            transform.translation.y.round() as i32,
        );

        let Some(parent_handle) = registry.get(&particle.name).copied() else {
            warn!(
                "Particle '{}' not found in registry - marking invalid",
                particle.name
            );
            commands.entity(entity).insert(InvalidParticle);
            continue;
        };

        let Ok(mut entry) = map.entry(position) else {
            commands.entity(entity).insert(InvalidParticle);
            continue;
        };

        if entry.insert_if_vacant(entity) {
            commands
                .entity(entity)
                .insert((
                    GridPosition(position),
                    AttachedToParticleType(parent_handle),
                ))
                .remove::<(Transform, GlobalTransform, TransformTreeChanged)>();

            let chunk_coord = chunk_index.world_to_chunk_coord(position);
            if let Some(chunk_entity) = chunk_index.get(chunk_coord) {
                if let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) {
                    dirty_state.mark_dirty(position);
                }
            }
        } else {
            commands.entity(entity).insert(InvalidParticle);
        }
    }
}
