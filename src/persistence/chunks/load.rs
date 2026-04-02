use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use super::resources::{
    chunk_file_path, chunk_png_path, ChunkLoadResult, ChunkPersistenceError,
    ParticlePersistenceConfig, ParticlePersistenceState, PendingChunkData, PendingLoadTasks,
    PendingSaveTasks,
};
use crate::render::ForceColor;
use crate::core::{ChunkIndex, ChunkLoadingState, Particle, ParticleTypeRegistry};
pub(super) struct LoadPlugin;

impl Plugin for LoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                spawn_chunk_load_tasks,
                poll_chunk_load_tasks,
                spawn_loaded_particles,
                handle_persistence_errors,
            )
                .chain(),
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
fn spawn_chunk_load_tasks(
    config: Res<ParticlePersistenceConfig>,
    loading_state: Res<ChunkLoadingState>,
    pending_saves: Res<PendingSaveTasks>,
    mut pending_tasks: ResMut<PendingLoadTasks>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    let chunks_to_try: Vec<crate::core::ChunkCoord> = loading_state
        .loaded_this_frame
        .iter()
        .copied()
        .chain(pending_tasks.blocked_by_save.drain(..))
        .collect();

    for coord in chunks_to_try {
        if pending_saves.has_pending_save(coord) {
            if !pending_tasks.blocked_by_save.contains(&coord) {
                pending_tasks.blocked_by_save.push(coord);
            }
            continue;
        }

        let bfs_path = chunk_file_path(&config, coord);
        let png_path = chunk_png_path(&config, coord);

        if bfs_path.exists() {
            let already_pending = pending_tasks.tasks.contains_key(&coord)
                || pending_tasks.pending_spawn.iter().any(|p| p.coord == coord);

            if !already_pending {
                let task = task_pool.spawn(load_chunk_async(coord, bfs_path, png_path));
                pending_tasks.tasks.insert(coord, task);
            }
        }
    }
}

#[allow(clippy::unused_async)]
async fn load_chunk_async(
    coord: crate::core::ChunkCoord,
    bfs_path: PathBuf,
    png_path: PathBuf,
) -> ChunkLoadResult {
    let file = match File::open(&bfs_path) {
        Ok(f) => f,
        Err(e) => {
            return ChunkLoadResult {
                coord,
                particles: Err(format!("Failed to open BFS file: {e}")),
                image: None,
            };
        }
    };

    let mut reader = BufReader::new(file);

    let particles = match crate::persistence::bfs::deserialize_from_reader(&mut reader) {
        Ok(p) => p,
        Err(e) => {
            return ChunkLoadResult {
                coord,
                particles: Err(e),
                image: None,
            };
        }
    };

    let image = if png_path.exists() {
        match image::open(&png_path) {
            Ok(img) => Some(img.to_rgba8()),
            Err(e) => {
                warn!("Failed to load PNG for chunk {:?}: {}", coord, e);
                None
            }
        }
    } else {
        None
    };

    ChunkLoadResult {
        coord,
        particles: Ok(particles),
        image,
    }
}

fn poll_chunk_load_tasks(
    mut pending_tasks: ResMut<PendingLoadTasks>,
    mut state: ResMut<ParticlePersistenceState>,
) {
    let mut loaded_chunks = Vec::new();
    let mut errors = Vec::new();

    pending_tasks.tasks.retain(|_coord, task| {
        match futures_lite::future::block_on(futures_lite::future::poll_once(task)) {
            Some(result) => {
                match result.particles {
                    Ok(particles) => {
                        loaded_chunks.push(PendingChunkData {
                            coord: result.coord,
                            particles,
                            image: result.image,
                        });
                    }
                    Err(e) => {
                        errors.push(ChunkPersistenceError::DeserializationError {
                            coord: result.coord,
                            message: e,
                        });
                    }
                }
                false
            }
            None => true,
        }
    });

    pending_tasks.pending_spawn.extend(loaded_chunks);
    state.errors.extend(errors);
}

#[allow(clippy::needless_pass_by_value)]
fn spawn_loaded_particles(
    chunk_index: Res<ChunkIndex>,
    registry: Res<ParticleTypeRegistry>,
    mut pending: ResMut<PendingLoadTasks>,
    mut commands: Commands,
) {
    let chunk_size = chunk_index.chunk_size() as i32;

    let ready_chunks: Vec<PendingChunkData> = pending
        .pending_spawn
        .extract_if(.., |chunk_data| {
            chunk_data
                .particles
                .iter()
                .all(|p| registry.contains(&p.name))
        })
        .collect();

    for chunk_data in &ready_chunks {
        let chunk_min = IVec2::new(
            chunk_data.coord.x() * chunk_size,
            chunk_data.coord.y() * chunk_size,
        );

        for particle_data in &chunk_data.particles {
            let transform = Transform::from_xyz(
                particle_data.position.x as f32,
                particle_data.position.y as f32,
                0.,
            );

            let particle = Particle::from_string(particle_data.name.clone());

            let force_color = chunk_data.image.as_ref().and_then(|img| {
                let px = (particle_data.position.x - chunk_min.x) as u32;
                let py = (chunk_size - 1 - (particle_data.position.y - chunk_min.y)) as u32;

                if px < img.width() && py < img.height() {
                    let pixel = img.get_pixel(px, py);
                    Some(ForceColor(Color::srgba(
                        f32::from(pixel[0]) / 255.0,
                        f32::from(pixel[1]) / 255.0,
                        f32::from(pixel[2]) / 255.0,
                        f32::from(pixel[3]) / 255.0,
                    )))
                } else {
                    None
                }
            });

            if let Some(color) = force_color {
                commands.spawn((particle, transform, color));
            } else {
                commands.spawn((particle, transform));
            }
        }

        debug!(
            "Restored chunk {:?} with {} particles",
            chunk_data.coord,
            chunk_data.particles.len()
        );
    }
}

fn handle_persistence_errors(mut state: ResMut<ParticlePersistenceState>) {
    for error in state.errors.drain(..) {
        match error {
            ChunkPersistenceError::IoError { coord, message } => {
                error!("Failed to persist chunk {:?}: {}", coord, message);
            }
            ChunkPersistenceError::DeserializationError { coord, message } => {
                warn!(
                    "Failed to deserialize chunk {:?}: {} - chunk will be empty",
                    coord, message
                );
            }
        }
    }
}
