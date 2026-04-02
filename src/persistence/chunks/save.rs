use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use image::{ImageBuffer, RgbaImage};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::resources::{
    chunk_file_path, chunk_png_path, ChunkImageData, ChunkSaveResult, ParticlePersistenceConfig,
    PendingSaveTasks,
};
use crate::core::{
    ChunkIndex, ChunkLoadingState, ChunkSystems, GridPosition, Particle, ParticleMap,
};
use crate::persistence::bfs::ParticleData as BfsParticleData;
use crate::render::{extract_chunk_image, ChunkRenderingConfig, ParticleColor};

pub(super) struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PersistChunksSignal>()
            .add_systems(
                PreUpdate,
                persist_unloaded_chunks.after(ChunkSystems::Loading),
            )
            .add_systems(PostUpdate, poll_pending_save_tasks)
            .add_systems(PostUpdate, msgr_save_all_chunks)
            .add_observer(on_save_all_chunks);
    }
}

/// Signal that triggers saving all loaded chunks to disk.
///
/// Use this before exiting the application to ensure all loaded chunks are persisted.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::persistence::PersistChunksSignal;
///
/// fn save_on_exit(mut writer: MessageWriter<PersistChunksSignal>) {
///     writer.write(PersistChunksSignal);
/// }
/// ```
#[derive(Event, Message, Clone, Eq, PartialEq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub struct PersistChunksSignal;

#[allow(clippy::needless_pass_by_value)]
fn persist_unloaded_chunks(
    config: Res<ParticlePersistenceConfig>,
    rendering_config: Res<ChunkRenderingConfig>,
    loading_state: Res<ChunkLoadingState>,
    chunk_index: Res<ChunkIndex>,
    mut pending_tasks: ResMut<PendingSaveTasks>,
    particle_query: Query<(Entity, &Particle, &GridPosition)>,
    color_query: Query<&ParticleColor, With<Particle>>,
) {
    if loading_state.unloaded_this_frame.is_empty() {
        return;
    }

    if let Err(e) = std::fs::create_dir_all(&config.save_path) {
        error!("Failed to create chunk save directory: {}", e);
        return;
    }

    let task_pool = AsyncComputeTaskPool::get();
    let chunk_size = chunk_index.chunk_size() as i32;

    let unloaded_coords: bevy::platform::collections::HashSet<crate::core::ChunkCoord> =
        loading_state
            .unloaded_this_frame
            .iter()
            .map(|(c, _)| *c)
            .collect();

    let mut particles_by_chunk: bevy::platform::collections::HashMap<
        crate::core::ChunkCoord,
        Vec<BfsParticleData>,
    > = bevy::platform::collections::HashMap::default();
    let mut colors_by_position: bevy::platform::collections::HashMap<IVec2, Color> =
        bevy::platform::collections::HashMap::default();

    for &(coord, _) in &loading_state.unloaded_this_frame {
        particles_by_chunk.entry(coord).or_default();
    }

    for (entity, particle, grid_pos) in particle_query.iter() {
        let coord = crate::core::ChunkCoord::new(
            grid_pos.0.x.div_euclid(chunk_size),
            grid_pos.0.y.div_euclid(chunk_size),
        );
        if unloaded_coords.contains(&coord) {
            particles_by_chunk
                .entry(coord)
                .or_default()
                .push(BfsParticleData {
                    name: particle.name.to_string(),
                    position: grid_pos.0,
                });
            if let Ok(pc) = color_query.get(entity) {
                colors_by_position.insert(grid_pos.0, pc.0);
            }
        }
    }

    for (coord, particles_to_save) in particles_by_chunk {
        let bg = rendering_config.background_color.to_srgba();
        let bg_bytes = [
            (bg.red * 255.0) as u8,
            (bg.green * 255.0) as u8,
            (bg.blue * 255.0) as u8,
            (bg.alpha * 255.0) as u8,
        ];
        let cs = chunk_size as usize;
        let chunk_min = IVec2::new(coord.x() * chunk_size, coord.y() * chunk_size);
        let chunk_max = chunk_min + IVec2::splat(chunk_size - 1);
        let mut img_data = vec![0u8; cs * cs * 4];
        for pixel in img_data.chunks_exact_mut(4) {
            pixel.copy_from_slice(&bg_bytes);
        }
        for y in chunk_min.y..=chunk_max.y {
            for x in chunk_min.x..=chunk_max.x {
                let pos = IVec2::new(x, y);
                if let Some(color) = colors_by_position.get(&pos) {
                    let c = color.to_srgba();
                    let px = (x - chunk_min.x) as usize;
                    let py = (chunk_max.y - y) as usize;
                    let pi = (py * cs + px) * 4;
                    img_data[pi] = (c.red * 255.0) as u8;
                    img_data[pi + 1] = (c.green * 255.0) as u8;
                    img_data[pi + 2] = (c.blue * 255.0) as u8;
                    img_data[pi + 3] = (c.alpha * 255.0) as u8;
                }
            }
        }
        let image_data = Some(ChunkImageData {
            data: img_data,
            width: cs as u32,
            height: cs as u32,
        });

        let bfs_path = chunk_file_path(&config, coord);
        let png_path = chunk_png_path(&config, coord);
        let task = task_pool.spawn(save_chunk_async(
            coord,
            particles_to_save,
            bfs_path,
            png_path,
            image_data,
        ));
        pending_tasks.tasks.insert(coord, task);
    }
}

#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
fn msgr_save_all_chunks(
    mut reader: MessageReader<PersistChunksSignal>,
    config: Res<ParticlePersistenceConfig>,
    rendering_config: Res<ChunkRenderingConfig>,
    map: Res<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    pending_tasks: ResMut<PendingSaveTasks>,
    particle_query: Query<(&Particle, &GridPosition)>,
    color_query: Query<&ParticleColor, With<Particle>>,
) {
    if reader.read().next().is_none() {
        return;
    }
    save_all_chunks_impl(
        &config,
        &rendering_config,
        &map,
        &chunk_index,
        pending_tasks,
        &particle_query,
        &color_query,
    );
}

#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
fn on_save_all_chunks(
    _trigger: On<PersistChunksSignal>,
    config: Res<ParticlePersistenceConfig>,
    rendering_config: Res<ChunkRenderingConfig>,
    map: Res<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    pending_tasks: ResMut<PendingSaveTasks>,
    particle_query: Query<(&Particle, &GridPosition)>,
    color_query: Query<&ParticleColor, With<Particle>>,
) {
    save_all_chunks_impl(
        &config,
        &rendering_config,
        &map,
        &chunk_index,
        pending_tasks,
        &particle_query,
        &color_query,
    );
}

#[allow(clippy::too_many_arguments)]
fn save_all_chunks_impl(
    config: &ParticlePersistenceConfig,
    rendering_config: &ChunkRenderingConfig,
    map: &ParticleMap,
    chunk_index: &ChunkIndex,
    mut pending_tasks: ResMut<PendingSaveTasks>,
    particle_query: &Query<(&Particle, &GridPosition)>,
    color_query: &Query<&ParticleColor, With<Particle>>,
) {
    debug!(
        "PersistChunksSignal triggered - saving {} loaded chunks",
        chunk_index.len()
    );

    if let Err(e) = std::fs::create_dir_all(&config.save_path) {
        error!("Failed to create chunk save directory on exit: {}", e);
        return;
    }

    let task_pool = AsyncComputeTaskPool::get();
    let chunk_size = chunk_index.chunk_size() as i32;

    for (coord, _chunk_entity) in chunk_index.iter() {
        let mut particles_to_save = Vec::new();

        let min = IVec2::new(coord.x() * chunk_size, coord.y() * chunk_size);
        let max = min + IVec2::splat(chunk_size - 1);

        for y in min.y..=max.y {
            for x in min.x..=max.x {
                let pos = IVec2::new(x, y);
                if let Ok(Some(entity)) = map.get_copied(pos) {
                    if let Ok((particle, grid_pos)) = particle_query.get(entity) {
                        particles_to_save.push(BfsParticleData {
                            name: particle.name.to_string(),
                            position: grid_pos.0,
                        });
                    }
                }
            }
        }

        let (data, width, height) =
            extract_chunk_image(map, chunk_index, rendering_config, coord, |entity| {
                color_query.get(entity).ok().map(|pc| pc.0)
            });
        let image_data = Some(ChunkImageData {
            data,
            width,
            height,
        });

        let bfs_path = chunk_file_path(config, coord);
        let png_path = chunk_png_path(config, coord);
        let task = task_pool.spawn(save_chunk_async(
            coord,
            particles_to_save,
            bfs_path,
            png_path,
            image_data,
        ));
        pending_tasks.tasks.insert(coord, task);
    }

    debug!(
        "Spawned {} async save tasks for chunks",
        pending_tasks.tasks.len()
    );
}

#[allow(clippy::unused_async)]
pub(super) async fn save_chunk_async(
    coord: crate::core::ChunkCoord,
    particles: Vec<BfsParticleData>,
    bfs_path: PathBuf,
    png_path: PathBuf,
    image_data: Option<ChunkImageData>,
) -> ChunkSaveResult {
    let particle_count = particles.len();
    let bytes = crate::persistence::bfs::serialize_to_bytes(&particles);

    if let Err(e) = std::fs::write(&bfs_path, bytes) {
        return ChunkSaveResult {
            coord,
            particle_count: 0,
            error: Some(format!("Failed to write BFS: {e}")),
        };
    }

    if let Some(img_data) = image_data {
        let img: RgbaImage = ImageBuffer::from_raw(img_data.width, img_data.height, img_data.data)
            .expect("Image data should be valid");

        if let Err(e) = img.save(&png_path) {
            return ChunkSaveResult {
                coord,
                particle_count,
                error: Some(format!("Failed to write PNG: {e}")),
            };
        }
    }

    ChunkSaveResult {
        coord,
        particle_count,
        error: None,
    }
}

fn poll_pending_save_tasks(mut pending_tasks: ResMut<PendingSaveTasks>) {
    let mut total_saved = 0;
    let mut chunks_saved = 0;
    let mut had_errors = false;

    pending_tasks.tasks.retain(|_coord, task| {
        match futures_lite::future::block_on(futures_lite::future::poll_once(task)) {
            Some(result) => {
                if let Some(err) = result.error {
                    error!("Failed to save chunk {:?}: {}", result.coord, err);
                    had_errors = true;
                } else {
                    total_saved += result.particle_count;
                    chunks_saved += 1;
                }
                false
            }
            None => true,
        }
    });

    if chunks_saved > 0 && !had_errors {
        info!(
            "Saved {} particles across {} chunks",
            total_saved, chunks_saved
        );
    }
}
