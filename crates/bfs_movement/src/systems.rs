use crate::PhysicsRng;
use crate::*;
use std::mem;

use bevy::{ecs::system::QueryLens, utils::HashSet};
use bfs_core::{Chunk, ChunkMap, ChunkRng, Coordinates, Particle, ParticleSimulationSet};

pub(super) struct SystemsPlugin;

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MovementSource>().add_systems(
            Update,
            handle_movement_by_chunks
                .in_set(ParticleSimulationSet)
                .run_if(in_state(MovementSource::Chunks)),
        );
    }
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum MovementSource {
    #[default]
    Chunks,
    Particles,
}

type ParticleMovementQuery<'a> = (
    Entity,
    &'a Particle,
    &'a mut Coordinates,
    &'a mut Transform,
    &'a mut PhysicsRng,
    &'a mut Velocity,
    Option<&'a mut Momentum>,
    &'a Density,
    &'a mut MovementPriority,
);

#[allow(unused_mut)]
pub fn handle_movement_by_chunks(
    mut particle_query: Query<ParticleMovementQuery>,
    mut map: ResMut<ChunkMap>,
    mut chunk_query: Query<&mut Chunk>,
    mut chunk_rng_query: Query<&mut ChunkRng>,
) {
    let chunk_query_ptr: *mut Query<&mut Chunk> = &mut chunk_query;
    let mut visited: HashSet<Entity> = HashSet::default();
    let mut coordinates_set: Vec<Entity> = Vec::with_capacity(1024);
    let mut joined: QueryLens<(&mut Chunk, &mut ChunkRng)> = chunk_rng_query.join(&mut chunk_query);

    unsafe {
        joined
            .query()
            .iter_unsafe()
            .for_each(|(mut chunk, mut chunk_rng)| {
                coordinates_set.clear();
                if let Some(dirty_rect) = chunk.prev_dirty_rect() {
                    chunk.iter().for_each(|(coordinates, entity)| {
                        if dirty_rect.contains(*coordinates) || chunk_rng.chance(0.2) {
                            coordinates_set.push(*entity);
                        }
                    });
                } else {
                    chunk.iter().for_each(|(_, entity)| {
                        if chunk_rng.chance(0.05) {
                            coordinates_set.push(*entity);
                        }
                    });
                }

                chunk_rng.shuffle(&mut coordinates_set);
                coordinates_set.iter().for_each(|entity| {
                    if visited.contains(entity) {
                        return;
                    }

                    if let Ok((
                        _,
                        particle_type,
                        mut coordinates,
                        mut transform,
                        mut rng,
                        mut velocity,
                        mut momentum,
                        density,
                        mut movement_priority,
                    )) = particle_query.get_unchecked(*entity)
                    {
                        let mut moved = false;

                        'velocity_loop: for _ in 0..velocity.val {
                            let mut obstructed: HashSet<IVec2> = HashSet::default();

                            for relative_coordinates in movement_priority
                                .iter_candidates(&mut rng, momentum.as_deref().cloned().as_ref())
                            {
                                let neighbor_coordinates = coordinates.0 + *relative_coordinates;
                                let signum = relative_coordinates.signum();

                                if obstructed.contains(&signum) {
                                    continue;
                                }

                                match map.entity(&neighbor_coordinates, &mut *chunk_query_ptr) {
                                    Some(neighbor_entity) => {
                                        if let Ok((
                                            _,
                                            neighbor_particle_type,
                                            mut neighbor_coords,
                                            mut neighbor_transform,
                                            _,
                                            _,
                                            _,
                                            neighbor_density,
                                            _,
                                        )) = particle_query.get_unchecked(neighbor_entity)
                                        {
                                            if *particle_type == *neighbor_particle_type {
                                                continue;
                                            }

                                            if density > neighbor_density {
                                                map.swap(
                                                    neighbor_coords.0,
                                                    coordinates.0,
                                                    &mut *chunk_query_ptr,
                                                );

                                                swap_particle_positions(
                                                    &mut coordinates,
                                                    &mut transform,
                                                    &mut neighbor_coords,
                                                    &mut neighbor_transform,
                                                );

                                                if let Some(ref mut m) = momentum {
                                                    m.0 = IVec2::ZERO;
                                                }

                                                velocity.decrement();
                                                moved = true;
                                                break 'velocity_loop;
                                            } else {
                                                obstructed.insert(signum);
                                                continue;
                                            }
                                        } else {
                                            obstructed.insert(signum);
                                            continue;
                                        }
                                    }
                                    None => {
                                        map.swap(
                                            coordinates.0,
                                            neighbor_coordinates,
                                            &mut *chunk_query_ptr,
                                        );
                                        coordinates.0 = neighbor_coordinates;
                                        transform.translation.x = neighbor_coordinates.x as f32;
                                        transform.translation.y = neighbor_coordinates.y as f32;

                                        if let Some(ref mut m) = momentum {
                                            m.0 = *relative_coordinates;
                                        }

                                        velocity.increment();
                                        moved = true;
                                        continue 'velocity_loop;
                                    }
                                }
                            }
                        }

                        if moved {
                            visited.insert(*entity);
                        } else {
                            if let Some(ref mut m) = momentum {
                                m.0 = IVec2::ZERO;
                            }
                            velocity.decrement();
                        }
                    }
                });
            });
    }
}

fn swap_particle_positions(
    first_coordinates: &mut Coordinates,
    first_transform: &mut Transform,
    second_coordinates: &mut Coordinates,
    second_transform: &mut Transform,
) {
    mem::swap(
        &mut first_transform.translation,
        &mut second_transform.translation,
    );
    mem::swap(&mut first_coordinates.0, &mut second_coordinates.0);
}
