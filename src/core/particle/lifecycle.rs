//! Provides mechanisms for ergonomic spawning/despawning of particle entities from the simulation.
//!
//! Each of these are responsive to both messages and triggers:
//! - [`SpawnParticleSignal`]: Spawn a new particle into the simulation
//! - [`DespawnParticleSignal`]: Despawn a particle from the simulation
//! - [`DespawnAllParticlesSignal`]: Despawn all particles from the simulation
//! - [`DespawnParticleTypeChildrenSignal`]: Despawn all particle children by name or parent
//!   handle.
//!
//! [`SpawnParticleSignal`] is the **only** supported way to introduce a particle into the
//! simulation. Direct `commands.spawn(...)` of a [`Particle`] is not currently supported —
//! the simulation relies on internal bookkeeping (`ParticleMap`, chunk dirty rects, parent
//! resolution, sync propagation) that only the signal handlers wire up.

use super::LocateBy;
use crate::core::{
    AttachedToParticleType, ChunkDirtyState, ChunkIndex, GridPosition, Particle, ParticleMap,
    ParticleType, ParticleTypeRegistry, schedule::ParticleSystems,
};
use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

pub(super) struct LifecyclePlugin;

impl Plugin for LifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(PreUpdate, ParticleSystems::Registration)
            .register_type::<TimedLifetime>()
            .register_type::<ChanceLifetime>()
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
                despawn_orphaned_particles.before(ParticleSystems::Registration),
            )
            .add_systems(
                Update,
                (handle_timed_lifetimes, handle_chance_lifetimes)
                    .in_set(ParticleSystems::Simulation),
            );
    }
}

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
        if lifetime.tick_timer.tick(time.delta()).just_finished() && rng.chance(lifetime.chance) {
            msgw_despawn.write(DespawnParticleSignal::from_entity(entity));
        }
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
/// The signal carries a [`ParticleType`] value which is resolved to the matching parent entity via
/// [`ParticleTypeRegistry`] at handle time. Spawned entities receive only the [`Particle`]
/// marker plus an [`AttachedToParticleType`] reference. Therefore, when looking up a particle
/// entity's type, the user should also query for [`ParticleType`] entities and look up the subject
/// particle entity's [`AttachedToParticleType`].
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
///         SpawnParticleSignal::new("Wood", IVec2::new(5, 5))
///             .with_on_spawn(|cmd| { cmd.insert(OnFire); }),
///     );
/// }
/// ```
#[derive(Event, Message, Clone, Reflect, Serialize, Deserialize)]
pub struct SpawnParticleSignal {
    pub(crate) particle_type: ParticleType,
    pub(crate) positions: Vec<IVec2>,
    pub(crate) overwrite_existing: bool,
    #[serde(skip)]
    #[reflect(ignore)]
    pub(crate) on_spawn: Option<OnSpawnCallback>,
}

impl std::fmt::Debug for SpawnParticleSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpawnParticleSignal")
            .field("particle_type", &self.particle_type)
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
    ///         "Sand",
    ///         IVec2::new(10, 20),
    ///     ));
    /// }
    /// ```
    #[must_use]
    pub fn new(particle_type: impl Into<ParticleType>, position: IVec2) -> Self {
        Self {
            particle_type: particle_type.into(),
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
    ///         "Water",
    ///         IVec2::new(10, 20),
    ///     ));
    /// }
    /// ```
    #[must_use]
    pub fn overwrite_existing(particle_type: impl Into<ParticleType>, position: IVec2) -> Self {
        Self {
            particle_type: particle_type.into(),
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
    ///         "Sand",
    ///         vec![IVec2::new(10, 20), IVec2::new(11, 20), IVec2::new(12, 20)],
    ///     ));
    /// }
    /// ```
    #[must_use]
    pub fn try_multiple(particle_type: impl Into<ParticleType>, positions: Vec<IVec2>) -> Self {
        Self {
            particle_type: particle_type.into(),
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
    ///         SpawnParticleSignal::new("Wood", IVec2::new(5, 5))
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

    let mut pending_overwrites: HashMap<IVec2, (Entity, Option<OnSpawnCallback>)> =
        HashMap::default();
    let mut spawned_positions: Vec<IVec2> = Vec::new();

    msgr_spawn_particle.read().for_each(|msg| {
        if let Some(parent_handle) = registry.get(&msg.particle_type.name) {
            let on_spawn = msg.on_spawn.clone();
            for position in &msg.positions {
                if msg.overwrite_existing {
                    pending_overwrites.insert(*position, (*parent_handle, on_spawn.clone()));
                } else if map.is_position_loaded(*position) {
                    let on_spawn = on_spawn.clone();
                    if let Ok(mut entry) = map.entry(*position)
                        && entry.insert_if_vacant_with(|| {
                            let mut entity_commands = commands.spawn((
                                Particle,
                                GridPosition(*position),
                                AttachedToParticleType(*parent_handle),
                            ));
                            if let Some(ref callback) = on_spawn {
                                callback(&mut entity_commands);
                            }
                            entity_commands.id()
                        })
                    {
                        spawned_positions.push(*position);
                        return;
                    }
                }
            }
        }
    });

    for (position, (parent_handle, on_spawn)) in pending_overwrites {
        let mut entity_commands = commands.spawn((
            Particle,
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
        if let Some(chunk_entity) = chunk_index.get(chunk_coord)
            && let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity)
        {
            dirty_state.mark_dirty(position);
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

    let mut pending_overwrites: HashMap<IVec2, (Entity, Option<OnSpawnCallback>)> =
        HashMap::default();
    let mut spawned_positions: Vec<IVec2> = Vec::new();

    let event = trigger.event();
    if let Some(parent_handle) = registry.get(&event.particle_type.name) {
        let on_spawn = event.on_spawn.clone();
        for position in &event.positions {
            if event.overwrite_existing {
                pending_overwrites.insert(*position, (*parent_handle, on_spawn.clone()));
            } else if map.is_position_loaded(*position) {
                let on_spawn = on_spawn.clone();
                if let Ok(mut entry) = map.entry(*position)
                    && entry.insert_if_vacant_with(|| {
                        let mut entity_commands = commands.spawn((
                            Particle,
                            GridPosition(*position),
                            AttachedToParticleType(*parent_handle),
                        ));
                        if let Some(ref callback) = on_spawn {
                            callback(&mut entity_commands);
                        }
                        entity_commands.id()
                    })
                {
                    spawned_positions.push(*position);
                    return;
                }
            }
        }
    }

    for (position, (parent_handle, on_spawn)) in pending_overwrites {
        let mut entity_commands = commands.spawn((
            Particle,
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
        if let Some(chunk_entity) = chunk_index.get(chunk_coord)
            && let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity)
        {
            dirty_state.mark_dirty(position);
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
        if let Some(chunk_entity) = chunk_index.get(chunk_coord)
            && let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity)
        {
            dirty_state.mark_dirty(position);
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
        if let Some(chunk_entity) = chunk_index.get(chunk_coord)
            && let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity)
        {
            dirty_state.mark_dirty(position);
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
        if let Some(chunk_entity) = chunk_index.get(chunk_coord)
            && let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity)
        {
            dirty_state.mark_dirty(position);
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
        if let Some(chunk_entity) = chunk_index.get(chunk_coord)
            && let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity)
        {
            dirty_state.mark_dirty(position);
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
            if let Some(chunk_entity) = chunk_index.get(chunk_coord)
                && let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity)
            {
                dirty_state.mark_dirty(position);
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
        if let Some(chunk_entity) = chunk_index.get(chunk_coord)
            && let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity)
        {
            dirty_state.mark_dirty(position);
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
            if let Some(chunk_entity) = chunk_index.get(chunk_coord)
                && let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity)
            {
                dirty_state.mark_dirty(position);
            }
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
            AttachedToParticleType, ChanceLifetime, GridPosition, Particle, ParticleMap,
            ParticleType, ParticleTypeRegistry, TimedLifetime,
        },
    };

    #[derive(Component, Clone, Debug, PartialEq)]
    struct Marker(u32);

    fn create_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(FallingSandMinimalPlugin::default());
        app
    }

    fn name_of(app: &App, entity: Entity) -> String {
        let attached = app
            .world()
            .entity(entity)
            .get::<AttachedToParticleType>()
            .unwrap();
        app.world()
            .entity(attached.0)
            .get::<ParticleType>()
            .unwrap()
            .name
            .to_string()
    }

    // ---- particle_type hooks ----

    #[test]
    fn hook_on_add_particle_type() {
        let mut app = create_test_app();

        let _entity = app.world_mut().spawn(ParticleType::new("sand")).id();
        app.update();

        let registry = app.world().resource::<ParticleTypeRegistry>();
        assert!(registry.contains("sand"));
    }

    #[test]
    fn hook_on_remove_particle_type() {
        let mut app = create_test_app();

        let entity = app.world_mut().spawn(ParticleType::new("sand")).id();
        app.update();

        app.world_mut().despawn(entity);
        app.update();

        let registry = app.world().resource::<ParticleTypeRegistry>();
        assert!(!registry.contains("sand"));
    }

    #[test]
    fn hook_on_add_duplicate_particle_type_despawns_old_entity() {
        let mut app = create_test_app();

        let old_entity = app.world_mut().spawn(ParticleType::new("sand")).id();
        app.update();

        let new_entity = app.world_mut().spawn(ParticleType::new("sand")).id();
        app.update();

        let registry = app.world().resource::<ParticleTypeRegistry>();
        assert_eq!(registry.get("sand"), Some(&new_entity));

        assert!(
            app.world().get_entity(old_entity).is_err(),
            "Old ParticleType entity should be despawned when a duplicate name is registered"
        );
    }

    // ---- msgr_spawn_particle ----

    #[test]
    fn msgr_spawn_particle_at_position() {
        let mut app = create_test_app();

        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let position = IVec2::new(3, 4);
        app.world_mut()
            .write_message(SpawnParticleSignal::new("sand", position));
        app.update();

        let map = app.world().resource::<ParticleMap>();
        let entity = map
            .get_copied(position)
            .unwrap()
            .expect("Particle should exist in map");

        assert_eq!(name_of(&app, entity), "sand");

        let grid_pos = app.world().entity(entity).get::<GridPosition>().unwrap();
        assert_eq!(grid_pos.0, position);
    }

    #[test]
    fn msgr_spawn_particle_does_not_overwrite_existing() {
        let mut app = create_test_app();

        app.world_mut().spawn(ParticleType::new("sand"));
        app.world_mut().spawn(ParticleType::new("water"));
        app.update();

        let position = IVec2::ZERO;

        app.world_mut()
            .write_message(SpawnParticleSignal::new("sand", position));
        app.update();

        let first_entity = app
            .world()
            .resource::<ParticleMap>()
            .get_copied(position)
            .unwrap()
            .unwrap();

        app.world_mut()
            .write_message(SpawnParticleSignal::new("water", position));
        app.update();

        let entity = app
            .world()
            .resource::<ParticleMap>()
            .get_copied(position)
            .unwrap()
            .unwrap();
        assert_eq!(entity, first_entity);

        assert_eq!(name_of(&app, entity), "sand");
    }

    #[test]
    fn msgr_spawn_particle_overwrite_existing() {
        let mut app = create_test_app();

        app.world_mut().spawn(ParticleType::new("sand"));
        app.world_mut().spawn(ParticleType::new("water"));
        app.update();

        let position = IVec2::ZERO;

        app.world_mut()
            .write_message(SpawnParticleSignal::new("sand", position));
        app.update();

        let old_entity = app
            .world()
            .resource::<ParticleMap>()
            .get_copied(position)
            .unwrap()
            .unwrap();

        app.world_mut()
            .write_message(SpawnParticleSignal::overwrite_existing("water", position));
        app.update();

        let new_entity = app
            .world()
            .resource::<ParticleMap>()
            .get_copied(position)
            .unwrap()
            .unwrap();

        assert_ne!(old_entity, new_entity);
        assert!(!app.world().entities().contains(old_entity));

        assert_eq!(name_of(&app, new_entity), "water");
    }

    #[test]
    fn msgr_spawn_particle_try_multiple_skips_occupied() {
        let mut app = create_test_app();

        app.world_mut().spawn(ParticleType::new("sand"));
        app.world_mut().spawn(ParticleType::new("water"));
        app.update();

        let pos_a = IVec2::ZERO;
        let pos_b = IVec2::new(1, 0);

        app.world_mut()
            .write_message(SpawnParticleSignal::new("sand", pos_a));
        app.update();

        app.world_mut()
            .write_message(SpawnParticleSignal::try_multiple(
                "water",
                vec![pos_a, pos_b],
            ));
        app.update();

        let map = app.world().resource::<ParticleMap>();

        let entity_a = map.get_copied(pos_a).unwrap().unwrap();
        assert_eq!(name_of(&app, entity_a), "sand");

        let entity_b = map.get_copied(pos_b).unwrap().unwrap();
        assert_eq!(name_of(&app, entity_b), "water");
    }

    #[test]
    fn msgr_spawn_particle_with_on_spawn_callback() {
        let mut app = create_test_app();

        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        app.world_mut()
            .write_message(
                SpawnParticleSignal::new("sand", IVec2::ZERO).with_on_spawn(|cmd| {
                    cmd.insert(Marker(99));
                }),
            );
        app.update();

        let particle_entity = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .next()
            .unwrap();

        assert_eq!(
            app.world().entity(particle_entity).get::<Marker>(),
            Some(&Marker(99)),
        );
    }

    #[test]
    fn msgr_spawn_particle_ignores_unregistered_type() {
        let mut app = create_test_app();
        app.update();

        app.world_mut()
            .write_message(SpawnParticleSignal::new("ghost", IVec2::ZERO));
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "No particle should spawn for unregistered type");
    }

    // ---- msgr_despawn_particle ----

    #[test]
    fn msgr_despawn_particle_by_position() {
        let mut app = create_test_app();

        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let position = IVec2::ZERO;
        app.world_mut()
            .write_message(SpawnParticleSignal::new("sand", position));
        app.update();

        let entity = app
            .world()
            .resource::<ParticleMap>()
            .get_copied(position)
            .unwrap()
            .unwrap();

        app.world_mut()
            .write_message(DespawnParticleSignal::from_position(position));
        app.update();

        let map = app.world().resource::<ParticleMap>();
        assert_eq!(map.get_copied(position).ok().flatten(), None);
        assert!(!app.world().entities().contains(entity));
    }

    #[test]
    fn msgr_despawn_particle_by_entity() {
        let mut app = create_test_app();

        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let position = IVec2::ZERO;
        app.world_mut()
            .write_message(SpawnParticleSignal::new("sand", position));
        app.update();

        let entity = app
            .world()
            .resource::<ParticleMap>()
            .get_copied(position)
            .unwrap()
            .unwrap();

        app.world_mut()
            .write_message(DespawnParticleSignal::from_entity(entity));
        app.update();

        let map = app.world().resource::<ParticleMap>();
        assert_eq!(map.get_copied(position).ok().flatten(), None);
        assert!(!app.world().entities().contains(entity));
    }

    // ---- hook_on_remove_particle ----

    #[test]
    fn hook_on_remove_particle_cleans_up_map() {
        let mut app = create_test_app();

        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let position = IVec2::ZERO;
        app.world_mut()
            .write_message(SpawnParticleSignal::new("sand", position));
        app.update();

        let entity = app
            .world()
            .resource::<ParticleMap>()
            .get_copied(position)
            .unwrap()
            .unwrap();

        app.world_mut().despawn(entity);
        app.update();

        let map = app.world().resource::<ParticleMap>();
        assert_eq!(map.get_copied(position).ok().flatten(), None);
        assert!(!app.world().entities().contains(entity));
    }

    // ---- msgr_despawn_particle_type_children ----

    #[test]
    fn msgr_despawn_particle_type_children_by_name() {
        let mut app = create_test_app();

        app.world_mut().spawn(ParticleType::new("sand"));
        app.world_mut().spawn(ParticleType::new("water"));
        app.update();

        for i in 0..5 {
            app.world_mut()
                .write_message(SpawnParticleSignal::new("sand", IVec2::new(i, 0)));
        }
        app.world_mut()
            .write_message(SpawnParticleSignal::new("water", IVec2::new(10, 0)));
        app.update();

        app.world_mut()
            .write_message(DespawnParticleTypeChildrenSignal::from_name("sand"));
        app.update();

        let entities: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .collect();
        let remaining: Vec<String> = entities.iter().map(|&e| name_of(&app, e)).collect();

        assert_eq!(remaining, vec!["water"]);
    }

    #[test]
    fn msgr_despawn_particle_type_children_by_entity() {
        let mut app = create_test_app();

        let sand_pt = app.world_mut().spawn(ParticleType::new("sand")).id();
        app.world_mut().spawn(ParticleType::new("water"));
        app.update();

        for i in 0..3 {
            app.world_mut()
                .write_message(SpawnParticleSignal::new("sand", IVec2::new(i, 0)));
        }
        app.world_mut()
            .write_message(SpawnParticleSignal::new("water", IVec2::new(10, 0)));
        app.update();

        app.world_mut()
            .write_message(DespawnParticleTypeChildrenSignal::from_parent_handle(
                sand_pt,
            ));
        app.update();

        let entities: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .collect();
        let remaining: Vec<String> = entities.iter().map(|&e| name_of(&app, e)).collect();

        assert_eq!(remaining, vec!["water"]);
    }

    // ---- msgr_despawn_all_particles ----

    #[test]
    fn msgr_despawn_all_particles() {
        let mut app = create_test_app();

        app.world_mut().spawn(ParticleType::new("sand"));
        app.world_mut().spawn(ParticleType::new("water"));
        app.update();

        let mut entities = vec![];
        for i in 0..5 {
            app.world_mut()
                .write_message(SpawnParticleSignal::new("sand", IVec2::new(i, 0)));
        }
        app.world_mut()
            .write_message(SpawnParticleSignal::new("water", IVec2::new(10, 0)));
        app.update();

        entities.extend(
            app.world_mut()
                .query_filtered::<Entity, With<Particle>>()
                .iter(app.world()),
        );
        assert_eq!(entities.len(), 6);

        app.world_mut().write_message(DespawnAllParticlesSignal);
        app.update();

        let map = app.world().resource::<ParticleMap>();
        assert!(map.is_empty());
        for entity in entities {
            assert!(!app.world().entities().contains(entity));
        }
    }

    // ---- despawn_orphaned_particles ----

    #[test]
    fn despawn_orphaned_particles_on_parent_despawn() {
        let mut app = create_test_app();

        let pt_entity = app.world_mut().spawn(ParticleType::new("sand")).id();
        app.update();

        let mut particle_entities = vec![];
        for i in 0..5 {
            app.world_mut()
                .write_message(SpawnParticleSignal::new("sand", IVec2::new(i, 0)));
        }
        app.update();

        particle_entities.extend(
            app.world_mut()
                .query_filtered::<Entity, With<Particle>>()
                .iter(app.world()),
        );
        assert_eq!(particle_entities.len(), 5);

        app.world_mut().despawn(pt_entity);
        app.update();

        let map = app.world().resource::<ParticleMap>();
        for &entity in &particle_entities {
            assert!(
                !app.world().entities().contains(entity),
                "Child particle should be despawned when parent ParticleType is removed"
            );
        }
        for i in 0..5 {
            assert_eq!(
                map.get_copied(IVec2::new(i, 0)).ok().flatten(),
                None,
                "Map should be cleared for orphaned particle positions"
            );
        }
    }

    #[test]
    fn despawn_orphaned_particles_only_affects_children_of_removed_type() {
        let mut app = create_test_app();

        let sand_pt = app.world_mut().spawn(ParticleType::new("sand")).id();
        app.world_mut().spawn(ParticleType::new("water"));
        app.update();

        for i in 0..3 {
            app.world_mut()
                .write_message(SpawnParticleSignal::new("sand", IVec2::new(i, 0)));
        }
        app.world_mut()
            .write_message(SpawnParticleSignal::new("water", IVec2::new(10, 0)));
        app.update();

        let water_entity = app
            .world()
            .resource::<ParticleMap>()
            .get_copied(IVec2::new(10, 0))
            .unwrap()
            .unwrap();

        app.world_mut().despawn(sand_pt);
        app.update();

        assert!(app.world().entities().contains(water_entity));
        assert_eq!(name_of(&app, water_entity), "water");

        let map = app.world().resource::<ParticleMap>();
        for i in 0..3 {
            assert_eq!(map.get_copied(IVec2::new(i, 0)).ok().flatten(), None);
        }
    }

    // ---- timed_lifetime ----

    fn spawn_particle_at(app: &mut App, name: &'static str, position: IVec2) -> Entity {
        app.world_mut()
            .write_message(SpawnParticleSignal::new(name, position));
        app.update();

        app.world()
            .resource::<ParticleMap>()
            .get_copied(position)
            .unwrap()
            .unwrap()
    }

    #[test]
    fn timed_lifetime_despawns_after_duration() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let position = IVec2::ZERO;
        let entity = spawn_particle_at(&mut app, "sand", position);

        let mut lifetime = TimedLifetime::new(Duration::from_millis(100));
        lifetime.tick(Duration::from_millis(150));
        app.world_mut().entity_mut(entity).insert(lifetime);
        app.update();
        app.update();

        assert!(!app.world().entities().contains(entity));
        let map = app.world().resource::<ParticleMap>();
        assert_eq!(map.get_copied(position).ok().flatten(), None);
    }

    #[test]
    fn timed_lifetime_does_not_despawn_before_duration() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let position = IVec2::ZERO;
        let entity = spawn_particle_at(&mut app, "sand", position);

        let mut lifetime = TimedLifetime::new(Duration::from_millis(100));
        lifetime.tick(Duration::from_millis(50));
        app.world_mut().entity_mut(entity).insert(lifetime);
        app.update();
        app.update();

        assert!(app.world().entities().contains(entity));
        let map = app.world().resource::<ParticleMap>();
        assert!(map.get_copied(position).ok().flatten().is_some());
    }

    #[test]
    fn timed_lifetime_new_sets_duration() {
        let lifetime = TimedLifetime::new(Duration::from_secs(5));
        assert_eq!(lifetime.duration(), Duration::from_secs(5));
        assert!(!lifetime.finished());
    }

    #[test]
    fn timed_lifetime_tick_advances_timer() {
        let mut lifetime = TimedLifetime::new(Duration::from_millis(100));
        lifetime.tick(Duration::from_millis(50));
        assert!(!lifetime.finished());
        lifetime.tick(Duration::from_millis(60));
        assert!(lifetime.finished());
    }

    // ---- chance_lifetime ----

    #[test]
    fn chance_lifetime_default() {
        let lifetime = ChanceLifetime::default();
        assert_eq!(lifetime.chance, 0.0);
        assert_eq!(lifetime.tick_timer.duration(), Duration::ZERO);
    }

    #[test]
    fn chance_lifetime_new() {
        let lifetime = ChanceLifetime::new(0.5, Duration::ZERO);
        assert_eq!(lifetime.chance, 0.5);
        assert_eq!(lifetime.tick_timer.duration(), Duration::ZERO);
    }

    #[test]
    fn chance_lifetime_with_tick_rate() {
        let lifetime = ChanceLifetime::with_tick_rate(0.75, Duration::from_millis(200));
        assert_eq!(lifetime.chance, 0.75);
        assert_eq!(lifetime.tick_timer.duration(), Duration::from_millis(200));
    }

    #[test]
    fn chance_lifetime_zero_never_despawns() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let position = IVec2::ZERO;
        let entity = spawn_particle_at(&mut app, "sand", position);

        app.world_mut()
            .entity_mut(entity)
            .insert(ChanceLifetime::new(0.0, Duration::ZERO));

        for _ in 0..100 {
            app.update();
        }

        assert!(app.world().entities().contains(entity));
    }

    #[test]
    fn chance_lifetime_one_always_despawns() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let position = IVec2::ZERO;
        let entity = spawn_particle_at(&mut app, "sand", position);

        app.world_mut()
            .entity_mut(entity)
            .insert(ChanceLifetime::new(1.0, Duration::ZERO));

        app.update();
        app.update();

        assert!(!app.world().entities().contains(entity));
        let map = app.world().resource::<ParticleMap>();
        assert_eq!(map.get_copied(position).ok().flatten(), None);
    }

    #[test]
    fn chance_lifetime_respects_tick_rate() {
        let mut app = create_test_app();
        app.world_mut().spawn(ParticleType::new("sand"));
        app.update();

        let position = IVec2::ZERO;
        let entity = spawn_particle_at(&mut app, "sand", position);

        app.world_mut()
            .entity_mut(entity)
            .insert(ChanceLifetime::with_tick_rate(
                1.0,
                Duration::from_secs(999),
            ));
        app.update();
        app.update();

        assert!(app.world().entities().contains(entity));

        *app.world_mut()
            .entity_mut(entity)
            .get_mut::<ChanceLifetime>()
            .unwrap() = ChanceLifetime::new(1.0, Duration::ZERO);
        app.update();
        app.update();

        assert!(!app.world().entities().contains(entity));
    }
}
