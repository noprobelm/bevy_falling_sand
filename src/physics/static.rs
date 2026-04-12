//! Static rigid body collision mesh generation for falling sand particles.
//!
//! Particles marked with [`StaticRigidBodyParticle`] contribute to per-chunk collision meshes.
//! The pipeline identifies dirty chunks, builds occupancy bitmaps, generates meshes
//! asynchronously (flood-fill, perimeter extraction, Douglas-Peucker simplification,
//! ear-cut triangulation), and attaches the resulting trimesh colliders to static rigid body
//! entities.

use avian2d::math::Vector;
use avian2d::prelude::*;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

use super::dynamic::{StaticRigidBodyParticle, SuspendedParticle};
use super::geometry::{generate_mesh_from_bitmap, MeshGenerationResult};
use crate::core::{ChunkCoord, ChunkDirtyState, ChunkIndex, ChunkRegion, ParticleMap};

pub(super) struct StaticPlugin;

impl Plugin for StaticPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StaticRigidBodyParticleMeshData>()
            .init_resource::<StaticRigidBodyParticleColliders>()
            .init_resource::<PreviousFrameDirtyChunks>()
            .init_resource::<DouglasPeuckerEpsilon>()
            .init_resource::<DirtyChunkUpdateInterval>()
            .init_resource::<ChunkLastProcessedTime>()
            .init_resource::<PendingMeshTasks>()
            .init_resource::<ChunkOccupancy>();
    }
}

/// Configures the epsilon tolerance for the Douglas-Peucker polygon simplification algorithm.
///
/// Lower values preserve more detail in collision meshes but produce more vertices.
/// Higher values simplify aggressively, improving performance at the cost of precision.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::physics::DouglasPeuckerEpsilon;
///
/// fn setup(mut commands: Commands) {
///     commands.insert_resource(DouglasPeuckerEpsilon(1.0));
/// }
/// ```
#[derive(Resource, Debug)]
pub struct DouglasPeuckerEpsilon(pub f32);

impl Default for DouglasPeuckerEpsilon {
    fn default() -> Self {
        Self(0.5)
    }
}

/// Configures how often dirty chunks recalculate their collision meshes (in seconds).
///
/// Chunks that just stopped being dirty are always processed immediately.
/// Currently dirty chunks are throttled to this interval to improve performance.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::physics::DirtyChunkUpdateInterval;
///
/// fn setup(mut commands: Commands) {
///     commands.insert_resource(DirtyChunkUpdateInterval(0.2));
/// }
/// ```
#[derive(Resource, Debug)]
pub struct DirtyChunkUpdateInterval(pub f32);

impl Default for DirtyChunkUpdateInterval {
    fn default() -> Self {
        Self(0.1)
    }
}

#[derive(Resource, Default, Debug)]
pub(super) struct PreviousFrameDirtyChunks(HashSet<ChunkCoord>);

#[derive(Resource, Default, Debug)]
pub(super) struct ChunkLastProcessedTime(HashMap<ChunkCoord, f32>);

#[derive(Resource, Default)]
pub(super) struct PendingMeshTasks {
    tasks: HashMap<ChunkCoord, Task<MeshGenerationResult>>,
}

type ChunkMeshData = (Vec<Vec<Vector>>, Vec<Vec<[u32; 3]>>);

#[derive(Resource, Default, Debug)]
pub(super) struct StaticRigidBodyParticleMeshData {
    chunks: HashMap<ChunkCoord, ChunkMeshData>,
}

#[derive(Resource, Default, Debug)]
pub(super) struct StaticRigidBodyParticleColliders(HashMap<ChunkCoord, Entity>);

#[derive(Resource, Default)]
pub(super) struct ChunkOccupancy {
    bitmaps: HashMap<ChunkCoord, Vec<bool>>,
}

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::needless_pass_by_value
)]
pub(super) fn calculate_static_rigid_bodies(
    mut commands: Commands,
    static_body_query: Query<(), (With<StaticRigidBodyParticle>, Without<SuspendedParticle>)>,
    sleeping_dynamic_bodies: Query<(Entity, &Transform), (With<RigidBody>, With<Sleeping>)>,
    mut pending_tasks: ResMut<PendingMeshTasks>,
    mut previous_dirty_chunks: ResMut<PreviousFrameDirtyChunks>,
    mut chunk_last_processed: ResMut<ChunkLastProcessedTime>,
    mut mesh_data: ResMut<StaticRigidBodyParticleMeshData>,
    mut colliders: ResMut<StaticRigidBodyParticleColliders>,
    mut occupancy: ResMut<ChunkOccupancy>,
    douglas_peucker_epsilon: Res<DouglasPeuckerEpsilon>,
    dirty_chunk_interval: Res<DirtyChunkUpdateInterval>,
    time: Res<Time>,
    map: Res<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    chunk_query: Query<(&ChunkRegion, &ChunkDirtyState)>,
) {
    let current_time = time.elapsed_secs();
    let update_interval = dirty_chunk_interval.0;
    let chunk_size = chunk_index.chunk_size() as usize;
    let bitmap_len = chunk_size * chunk_size;

    let current_dirty_chunks: HashSet<ChunkCoord> = chunk_index
        .iter()
        .filter_map(|(coord, entity)| {
            if let Ok((_, dirty_state)) = chunk_query.get(entity) {
                if dirty_state.current.is_some() {
                    return Some(coord);
                }
            }
            None
        })
        .collect();

    let just_stopped_chunks: HashSet<ChunkCoord> = previous_dirty_chunks
        .0
        .difference(&current_dirty_chunks)
        .copied()
        .collect();

    let throttled_dirty_chunks: HashSet<ChunkCoord> = current_dirty_chunks
        .iter()
        .filter(|&coord| {
            let last_processed = chunk_last_processed.0.get(coord).copied().unwrap_or(0.0);
            current_time - last_processed >= update_interval
        })
        .copied()
        .collect();

    let chunks_to_process: HashSet<ChunkCoord> = just_stopped_chunks
        .union(&throttled_dirty_chunks)
        .copied()
        .collect();

    previous_dirty_chunks.0 = current_dirty_chunks;

    if !chunks_to_process.is_empty() {
        let task_pool = AsyncComputeTaskPool::get();
        let epsilon = douglas_peucker_epsilon.0;

        for coord in &chunks_to_process {
            if pending_tasks.tasks.contains_key(coord) {
                continue;
            }

            chunk_last_processed.0.insert(*coord, current_time);

            let base_x = coord.x() * chunk_size as i32;
            let base_y = coord.y() * chunk_size as i32;

            let mut new_bitmap = vec![false; bitmap_len];
            for ly in 0..chunk_size {
                for lx in 0..chunk_size {
                    let pos = IVec2::new(base_x + lx as i32, base_y + ly as i32);
                    if let Ok(Some(entity)) = map.get(pos) {
                        if static_body_query.contains(*entity) {
                            new_bitmap[ly * chunk_size + lx] = true;
                        }
                    }
                }
            }

            let changed = occupancy
                .bitmaps
                .get(coord)
                .is_none_or(|old| *old != new_bitmap);

            if !changed {
                continue;
            }

            occupancy.bitmaps.insert(*coord, new_bitmap.clone());

            let cs = chunk_size;
            let origin = IVec2::new(base_x, base_y);
            let task = task_pool.spawn(generate_mesh_from_bitmap(
                *coord, new_bitmap, origin, cs, epsilon,
            ));
            pending_tasks.tasks.insert(*coord, task);
        }
    }

    let mut recalculated_chunks: Vec<ChunkCoord> = Vec::new();

    pending_tasks.tasks.retain(
        |_key, task| match future::block_on(future::poll_once(task)) {
            Some(result) => {
                recalculated_chunks.push(result.chunk_coord);

                mesh_data.chunks.remove(&result.chunk_coord);

                let mut merged_verts = Vec::new();
                let mut merged_indices = Vec::new();
                for (vertices, indices) in result.vertices.iter().zip(&result.indices) {
                    if vertices.is_empty() || indices.is_empty() {
                        continue;
                    }
                    let offset = merged_verts.len() as u32;
                    merged_verts.extend_from_slice(vertices);
                    merged_indices.extend(
                        indices
                            .iter()
                            .map(|[a, b, c]| [a + offset, b + offset, c + offset]),
                    );
                }

                if merged_verts.is_empty() {
                    if let Some(old_entity) = colliders.0.remove(&result.chunk_coord) {
                        commands.entity(old_entity).despawn();
                    }
                } else {
                    mesh_data.chunks.insert(
                        result.chunk_coord,
                        (result.vertices.clone(), result.indices.clone()),
                    );

                    let collider = Collider::trimesh(merged_verts, merged_indices);
                    if let Some(&existing) = colliders.0.get(&result.chunk_coord) {
                        commands.entity(existing).insert(collider);
                    } else {
                        let entity = commands.spawn((RigidBody::Static, collider)).id();
                        colliders.0.insert(result.chunk_coord, entity);
                    }
                }

                false
            }
            None => true,
        },
    );

    if !recalculated_chunks.is_empty() {
        let chunk_regions: Vec<IRect> = recalculated_chunks
            .iter()
            .filter_map(|coord| {
                chunk_index
                    .get(*coord)
                    .and_then(|e| chunk_query.get(e).ok())
                    .map(|(region, _)| region.region())
            })
            .collect();

        for (entity, transform) in sleeping_dynamic_bodies.iter() {
            let pos = IVec2::new(
                transform.translation.x.round() as i32,
                transform.translation.y.round() as i32,
            );

            for region in &chunk_regions {
                if pos.x >= region.min.x
                    && pos.x <= region.max.x
                    && pos.y >= region.min.y
                    && pos.y <= region.max.y
                {
                    commands.entity(entity).remove::<Sleeping>();
                    break;
                }
            }
        }
    }
}
