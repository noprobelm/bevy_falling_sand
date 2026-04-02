use bevy::prelude::*;

use crate::core::{ChunkDirtyState, ChunkIndex, ChunkRegion, ChunkSystems, ParticleMap};

use super::ChunkCoord;

pub(super) struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, update_chunk_loading)
            .add_systems(
                PreUpdate,
                update_chunk_loading.in_set(ChunkSystems::Loading),
            )
            .add_systems(
                PostUpdate,
                (cleanup_unloaded_particles, process_pending_despawns)
                    .chain()
                    .in_set(ChunkSystems::Cleanup),
            );
    }
}

/// Marker component for entities that drive dynamic chunk loading.
///
/// Attach this to any entity with a [`GlobalTransform`] — typically a camera — to make the
/// loaded region follow that entity. Every frame the loading system reads the entity's world
/// position and, when it crosses a half-chunk boundary, shifts the [`ParticleMap`] and
/// [`ChunkIndex`] origins so the loader stays centered in the loaded region. Chunks that fall
/// outside the new region are unloaded and their particles cleaned up incrementally (see
/// [`ChunkLoadingConfig`]).
///
/// The [`crate::persistence`] module extends chunk loading/unload by serializing and writing
/// particle positions (and optionally color) to disk.
///
/// Only one `ChunkLoader` entity should exist at a time. If multiple are present the system
/// uses the first one returned by the query.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::prelude::*;
///
/// #[derive(Component)]
/// struct MainCamera;
///
/// fn spawn_camera(mut commands: Commands) {
///     commands.spawn(MainCamera);
/// }
///
/// fn setup_camera(
///     mut commands: Commands,
///     camera: Single<(Entity, &MainCamera)>,
/// ) {
///     commands.entity(camera.0).insert((
///         Camera2d,
///         Transform::from_xyz(0.0, 0.0, 0.0),
///         ChunkLoader,
///     ));
/// }
///
/// fn main() {
///     App::new()
///         .add_plugins((DefaultPlugins, FallingSandPlugin::default()))
///         .add_systems(Startup, (spawn_camera, setup_camera).chain())
///         .run();
/// }
/// ```
///
/// `ChunkLoader` can be inserted and removed at runtime to toggle dynamic loading. Removing
/// it freezes the loaded region in place.
#[derive(Component, Copy, Clone, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct ChunkLoader;

/// Configuration for chunk loading behavior.
///
/// When the [`ChunkLoader`] triggers an origin shift, chunks that leave the loaded region must
/// have their particles drained and despawned. This resource controls how aggressively that
/// cleanup is spread across frames to avoid frame spikes.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::prelude::*;
///
/// App::new()
///     .add_plugins((DefaultPlugins, FallingSandPlugin::default()))
///     .insert_resource(ChunkLoadingConfig {
///         max_chunk_cleanups_per_frame: 8,
///     })
///     .run();
/// ```
#[derive(Resource, Clone, Debug)]
pub struct ChunkLoadingConfig {
    /// Maximum number of chunk regions to clean up per frame. When an origin shift unloads
    /// chunks, particle cleanup is spread across frames by processing at most this many
    /// regions per frame.
    pub max_chunk_cleanups_per_frame: usize,
}

impl Default for ChunkLoadingConfig {
    fn default() -> Self {
        Self {
            max_chunk_cleanups_per_frame: 4,
        }
    }
}

/// Resource tracking chunk loading/unloading state for the current frame.
///
/// Updated by the loading system in [`ChunkSystems::Loading`] and consumed by downstream
/// systems that need to react to loading events (e.g. the
/// [`persistence`](crate::persistence) module saves chunks on unload).
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::prelude::*;
///
/// fn on_origin_shift(state: Res<ChunkLoadingState>) {
///     if state.origin_shifted {
///         info!(
///             "Origin shifted — loaded {} chunks, unloaded {}",
///             state.loaded_this_frame.len(),
///             state.unloaded_this_frame.len(),
///         );
///     }
/// }
/// ```
#[derive(Resource, Default, Debug)]
pub struct ChunkLoadingState {
    /// Chunks that were loaded this frame.
    pub loaded_this_frame: Vec<ChunkCoord>,
    /// Chunks that were unloaded this frame.
    pub unloaded_this_frame: Vec<(ChunkCoord, Entity)>,
    /// World regions from this frame's unload that need particle cleanup.
    /// Consumed by the cleanup system, which moves them into its internal queue
    /// for incremental processing.
    pub regions_pending_cleanup: Vec<IRect>,
    /// Whether an origin shift occurred this frame.
    pub origin_shifted: bool,
    /// Entities from unloaded regions awaiting `PendingDespawn` marking.
    /// The cleanup system drains all regions from the `ParticleMap` immediately
    /// but spreads the `PendingDespawn` insertion across frames at a rate of
    /// [`ChunkLoadingConfig::max_chunk_cleanups_per_frame`] chunks worth of
    /// entities per frame.
    pub entity_cleanup_queue: Vec<Entity>,
}

/// Marker component for entities pending despawn.
///
/// Entities with this component are despawned in batches to spread the cost
/// across multiple frames.
#[derive(Component, Copy, Clone, Default, Debug)]
pub struct PendingDespawn;

/// Configuration for batched entity despawning.
///
/// When chunks are unloaded, their particle entities are marked with [`PendingDespawn`] and
/// despawned incrementally. This resource controls how many entities are despawned per frame.
/// Higher values clear the backlog faster but at the cost of frame rate.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::prelude::*;
///
/// App::new()
///     .add_plugins((DefaultPlugins, FallingSandPlugin::default()))
///     .insert_resource(DespawnBatchConfig { batch_size: 1024 })
///     .run();
/// ```
#[derive(Resource, Clone, Debug)]
pub struct DespawnBatchConfig {
    /// Maximum number of entities to despawn per frame.
    pub batch_size: usize,
}

impl Default for DespawnBatchConfig {
    fn default() -> Self {
        Self { batch_size: 512 }
    }
}

/// System that manages chunk loading based on [`ChunkLoader`] entity positions.
///
/// Shifts the map origin by half a chunk width whenever the loader crosses a
/// half-chunk boundary. This loads/unloads at most one row or column of chunks
/// per shift, keeping the per-frame work small.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn update_chunk_loading(
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
    mut chunk_index: ResMut<ChunkIndex>,
    mut state: ResMut<ChunkLoadingState>,
    loader_query: Query<&GlobalTransform, With<ChunkLoader>>,
) {
    state.loaded_this_frame.clear();
    state.unloaded_this_frame.clear();
    state.regions_pending_cleanup.clear();
    state.origin_shifted = false;

    let Some(loader_transform) = loader_query.iter().next() else {
        return;
    };

    let loader_pos = loader_transform.translation().truncate();
    let loader_world_pos = IVec2::new(loader_pos.x.floor() as i32, loader_pos.y.floor() as i32);

    let cs = chunk_index.chunk_size() as i32;
    let half_chunk = cs / 2;
    let half_w = map.width() as i32 / 2;
    let half_h = map.height() as i32 / 2;
    let loaded_region = map.loaded_region();
    let region_center = IVec2::new(
        i32::midpoint(loaded_region.min.x, loaded_region.max.x),
        i32::midpoint(loaded_region.min.y, loaded_region.max.y),
    );

    let offset = loader_world_pos - region_center;

    let new_origin = if offset.x.abs() > half_w || offset.y.abs() > half_h {
        Some(IVec2::new(
            (loader_world_pos.x / cs) * cs - half_w,
            (loader_world_pos.y / cs) * cs - half_h,
        ))
    } else {
        let shift_x = if offset.x > half_chunk {
            cs
        } else if offset.x < -half_chunk {
            -cs
        } else {
            0
        };

        let shift_y = if offset.y > half_chunk {
            cs
        } else if offset.y < -half_chunk {
            -cs
        } else {
            0
        };

        if shift_x != 0 || shift_y != 0 {
            Some(map.origin() + IVec2::new(shift_x, shift_y))
        } else {
            None
        }
    };

    if let Some(new_origin) = new_origin {
        let new_region = IRect::new(
            new_origin.x,
            new_origin.y,
            new_origin.x + map.width() as i32 - 1,
            new_origin.y + map.height() as i32 - 1,
        );

        let old_region = map.loaded_region();

        #[allow(clippy::needless_collect)]
        for (coord, entity) in chunk_index.iter().collect::<Vec<_>>() {
            let chunk_region = chunk_index.chunk_coord_to_chunk_region(coord);
            if !rects_intersect(new_region, chunk_region) {
                chunk_index.remove(coord);
                commands.entity(entity).insert(PendingDespawn);
                state.unloaded_this_frame.push((coord, entity));
            }
        }

        let old_min_chunk = chunk_index.world_to_chunk_coord(old_region.min);
        let old_max_chunk = chunk_index.world_to_chunk_coord(old_region.max);
        for y in old_min_chunk.y()..=old_max_chunk.y() {
            for x in old_min_chunk.x()..=old_max_chunk.x() {
                let coord = ChunkCoord::new(x, y);
                let chunk_region_rect = chunk_index.chunk_coord_to_chunk_region(coord);
                if !rects_intersect(new_region, chunk_region_rect) {
                    state.regions_pending_cleanup.push(chunk_region_rect);
                }
            }
        }

        map.shift_origin(new_origin);
        let new_chunk_origin = chunk_index.world_to_chunk_coord(new_origin).into();
        chunk_index.shift_origin(new_chunk_origin);
        state.origin_shifted = true;
    }

    let current_min_chunk = chunk_index.world_to_chunk_coord(map.loaded_region().min);
    let current_max_chunk = chunk_index.world_to_chunk_coord(map.loaded_region().max);

    for y in current_min_chunk.y()..=current_max_chunk.y() {
        for x in current_min_chunk.x()..=current_max_chunk.x() {
            let coord = ChunkCoord::new(x, y);

            if !chunk_index.contains(coord) {
                let region = chunk_index.chunk_coord_to_chunk_region(coord);
                let entity = commands
                    .spawn((
                        ChunkRegion::new(region),
                        ChunkDirtyState::fully_dirty(region),
                    ))
                    .id();

                chunk_index.insert(coord, entity);
                state.loaded_this_frame.push(coord);
            }
        }
    }
}

/// Check if two [`IRect`] rectangles intersect.
#[inline]
const fn rects_intersect(a: IRect, b: IRect) -> bool {
    a.min.x <= b.max.x && a.max.x >= b.min.x && a.min.y <= b.max.y && a.max.y >= b.min.y
}

/// System that cleans up particles in unloaded regions.
///
/// All regions are drained from the [`ParticleMap`] immediately so that simulation
/// and rendering systems stop processing stale particles. The resulting entity
/// references are queued and marked with [`PendingDespawn`] incrementally at a rate
/// of [`ChunkLoadingConfig::max_chunk_cleanups_per_frame`] chunks worth of entities
/// per frame. This spreads the command overhead across multiple frames while keeping
/// the spatial map clean.
///
/// Uses [`ParticleMap::drain_region`] to iterate only the positions within
/// each unloaded region rather than scanning every particle in the world.
#[allow(clippy::needless_pass_by_value)]
fn cleanup_unloaded_particles(
    mut commands: Commands,
    mut map: ResMut<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    mut state: ResMut<ChunkLoadingState>,
    config: Res<ChunkLoadingConfig>,
) {
    let regions: Vec<_> = state.regions_pending_cleanup.drain(..).collect();
    for region in regions {
        state.entity_cleanup_queue.extend(map.drain_region(region));
    }

    let chunk_area = (chunk_index.chunk_size() * chunk_index.chunk_size()) as usize;
    let max_entities = config.max_chunk_cleanups_per_frame * chunk_area;
    let drain_count = max_entities.min(state.entity_cleanup_queue.len());
    for entity in state.entity_cleanup_queue.drain(..drain_count) {
        commands.entity(entity).insert(PendingDespawn);
    }
}

/// System that processes pending entity despawns in batches.
///
/// Despawns up to [`DespawnBatchConfig::batch_size`] entities marked with
/// [`PendingDespawn`] per frame to prevent frame spikes when unloading large chunks.
#[allow(clippy::needless_pass_by_value)]
fn process_pending_despawns(
    mut commands: Commands,
    config: Res<DespawnBatchConfig>,
    query: Query<Entity, With<PendingDespawn>>,
) {
    for entity in query.iter().take(config.batch_size) {
        commands.entity(entity).despawn();
    }
}
