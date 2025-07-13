use crate::{MovementRng, Velocity, Momentum, Density, Movement, Moved};
use std::mem;

use bevy::prelude::*;
use bevy::platform::collections::HashSet;
use bevy_turborand::prelude::*;
use bfs_core::{Particle, ParticleMap, ParticlePosition, ParticleSimulationSet};

type ObstructedDirections = [bool; 9];

fn direction_to_index(dir: IVec2) -> usize {
    match (dir.x, dir.y) {
        (-1, -1) => 0, // bottom-left
        (0, -1) => 1,  // bottom
        (1, -1) => 2,  // bottom-right
        (-1, 0) => 3,  // left
        (0, 0) => 4,   // center
        (1, 0) => 5,   // right
        (-1, 1) => 6,  // top-left
        (0, 1) => 7,   // top
        (1, 1) => 8,   // top-right
        _ => 4,        // fallback to center
    }
}

#[derive(Resource, Default)]
struct MovementState {
    visited_entities: HashSet<Entity>,
    visited_positions: HashSet<IVec2>,
}

pub(super) struct SystemsPlugin;

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MovementSource>()
            .init_resource::<MovementState>()
            .add_systems(
                PreUpdate,
                (
                    handle_movement_by_chunks
                        .in_set(ParticleSimulationSet)
                        .run_if(in_state(MovementSource::Chunks)),
                    handle_movement_by_particles
                        .in_set(ParticleSimulationSet)
                        .run_if(in_state(MovementSource::Particles)),
                ),
            );
    }
}

/// Controls whether particle iteration for movement is carried out per chunk or by particle query.
#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum MovementSource {
    /// The `Chunks` state.
    Chunks,
    /// The `Particles` state.
    #[default]
    Particles,
}

type ParticleMovementQuery<'a> = (
    Entity,
    &'a Particle,
    &'a mut ParticlePosition,
    &'a mut Transform,
    &'a mut MovementRng,
    &'a mut Velocity,
    Option<&'a mut Momentum>,
    &'a Density,
    &'a mut Movement,
    &'a mut Moved,
);

#[allow(unused_mut, clippy::too_many_lines)]
fn handle_movement_by_chunks(
    mut particle_query: Query<ParticleMovementQuery>,
    mut map: ResMut<ParticleMap>,
    mut movement_state: ResMut<MovementState>,
    mut global_rng: ResMut<GlobalRng>,
) {
    movement_state.visited_entities.clear();
    let visited = &mut movement_state.visited_entities;

    unsafe {
        let map_ptr = &raw mut *map;

        let mut chunks = (*map_ptr).iter_chunks_mut();
        for mut chunk in chunks {
            if let Some(dirty_rect) = chunk.dirty_rect() {
                let mut chunk_entities: Vec<_> = chunk.iter().collect();
                // Shuffle entities to prevent deterministic patterns
                global_rng.shuffle(&mut chunk_entities);

                for (position, entity) in chunk_entities {
                    if dirty_rect.contains(*position) {
                        if visited.contains(entity) {
                            continue;
                        }

                        if let Ok((
                            _,
                            particle_type,
                            mut position,
                            mut transform,
                            mut rng,
                            mut velocity,
                            mut momentum,
                            density,
                            mut movement_priority,
                            mut particle_moved,
                        )) = particle_query.get_unchecked(*entity)
                        {
                            // Early exit for particles with no velocity
                            if velocity.current() == 0 {
                                particle_moved.0 = false;
                                continue;
                            }
                            
                            let mut moved = false;

                            'velocity_loop: for _ in 0..velocity.current() {
                                let mut obstructed: ObstructedDirections = [false; 9];

                                for relative_position in movement_priority
                                    .iter_candidates(&mut rng, momentum.as_deref().copied().as_ref())
                                {
                                    let neighbor_position = position.0 + *relative_position;
                                    let signum = relative_position.signum();
                                    let obstruct_idx = direction_to_index(signum);

                                    if obstructed[obstruct_idx] {
                                        continue;
                                    }

                                    let neighbor_entity = (*map_ptr).get(&neighbor_position).copied();

                                    match neighbor_entity {
                                        Some(neighbor_entity) => {
                                            if let Ok((
                                                _,
                                                neighbor_particle_type,
                                                mut neighbor_position,
                                                mut neighbor_transform,
                                                _,
                                                _,
                                                _,
                                                neighbor_density,
                                                _,
                                                _,
                                            )) = particle_query.get_unchecked(neighbor_entity)
                                            {
                                                if *particle_type == *neighbor_particle_type {
                                                    continue;
                                                }

                                                if density > neighbor_density {
                                                    if (*map_ptr).swap(neighbor_position.0, position.0).is_ok() {
                                                        swap_particle_positions(
                                                            &mut position,
                                                            &mut transform,
                                                            &mut neighbor_position,
                                                            &mut neighbor_transform,
                                                        );

                                                        if let Some(ref mut m) = momentum {
                                                            m.0 = IVec2::ZERO;
                                                        }

                                                        velocity.decrement();
                                                        moved = true;
                                                        break 'velocity_loop;
                                                    }
                                                } else {
                                                    obstructed[obstruct_idx] = true;
                                                }
                                            } else {
                                                obstructed[obstruct_idx] = true;
                                            }
                                        }
                                        None => {
                                            if (*map_ptr).swap(position.0, neighbor_position).is_ok() {
                                                position.0 = neighbor_position;
                                                // Batch transform update
                                                transform.translation = Vec3::new(
                                                    neighbor_position.x as f32,
                                                    neighbor_position.y as f32,
                                                    transform.translation.z,
                                                );
                                                if let Some(ref mut m) = momentum {
                                                    m.0 = *relative_position;
                                                }
                                                velocity.increment();
                                                moved = true;
                                                continue 'velocity_loop;
                                            } else {
                                                obstructed[obstruct_idx] = true;
                                            }
                                        }
                                    }
                                }
                                if !moved {
                                    break 'velocity_loop;
                                }
                                particle_moved.0 = moved;
                            }

                            if moved {
                                visited.insert(*entity);
                            } else {
                                if let Some(ref mut m) = momentum {
                                    m.0 = IVec2::ZERO;
                                }
                                velocity.decrement();
                            }
                            particle_moved.0 = moved;
                        }
                    }
                }
            }
        }
    }
}



#[allow(unused_mut, clippy::too_many_lines)]
fn handle_movement_by_particles(
    mut particle_query: Query<ParticleMovementQuery>,
    mut map: ResMut<ParticleMap>,
    mut movement_state: ResMut<MovementState>,
    _global_rng: ResMut<GlobalRng>,
) {
    movement_state.visited_positions.clear();
    let visited = &mut movement_state.visited_positions;
    unsafe {
        particle_query.iter_unsafe().for_each(
            |(
                _,
                particle_type,
                mut position,
                mut transform,
                mut rng,
                mut velocity,
                mut momentum,
                density,
                mut movement_priority,
                mut particle_moved,
            )| {
                // Early exit for particles with no velocity
                if velocity.current() == 0 {
                    particle_moved.0 = false;
                    return;
                }
                
                if let Some(chunk) = map.chunk(&position.0) {
                    if let Some(dirty_rect) = chunk.dirty_rect() {
                    if !dirty_rect.contains(position.0) {
                        return;
                    }
                    } else {
                        return;
                    }
                }
                let mut moved = false;
                'velocity_loop: for _ in 0..velocity.current() {
                    let mut obstructed: ObstructedDirections = [false; 9];

                    for relative_position in movement_priority
                        .iter_candidates(&mut rng, momentum.as_deref().copied().as_ref())
                    {
                        let neighbor_position = position.0 + *relative_position;
                        let signum = relative_position.signum();
                        let obstruct_idx = direction_to_index(signum);

                        if visited.contains(&neighbor_position) || obstructed[obstruct_idx] {
                            continue;
                        }

                        match map.get(&neighbor_position) {
                            Some(neighbor_entity) => {
                                if let Ok((
                                    _,
                                    neighbor_particle_type,
                                    mut neighbor_position,
                                    mut neighbor_transform,
                                    _,
                                    _,
                                    _,
                                    neighbor_density,
                                    _,
                                    _,
                                )) = particle_query.get_unchecked(*neighbor_entity)
                                {
                                    if *particle_type == *neighbor_particle_type {
                                        continue;
                                    }
                                    if density > neighbor_density {
                                        match map.swap(neighbor_position.0, position.0) {
                                            Ok(()) => {
                                                swap_particle_positions(
                                                    &mut position,
                                                    &mut transform,
                                                    &mut neighbor_position,
                                                    &mut neighbor_transform,
                                                );
                                                if let Some(ref mut momentum) = momentum {
                                                    momentum.0 = IVec2::ZERO; 
                                                }
                                                velocity.decrement();
                                                moved = true;
                                                break 'velocity_loop;
                                            },
                                            Err(err) => {debug!("Attempted to swap particles at {:?} and {:?} but failed: {:?}", position.0, neighbor_position, err);}
                                        }
                                    } else {
                                        obstructed[obstruct_idx] = true;
                                        continue;
                                    }
                                }
                                else {
                                    obstructed[obstruct_idx] = true;
                                    continue;
                                }
                            }
                            None => {
                                match  map.swap(position.0, neighbor_position) {
                                    Ok(()) => {
                                        position.0 = neighbor_position;
                                        // Batch transform update
                                        transform.translation = Vec3::new(
                                            neighbor_position.x as f32,
                                            neighbor_position.y as f32,
                                            transform.translation.z,
                                        );
                                        if let Some(ref mut momentum) = momentum {
                                            momentum.0 = *relative_position;
                                        }
                                        velocity.increment();
                                        moved = true;
                                        continue 'velocity_loop;
                                    },
                                    Err(err) => {
                                        debug!("Attempted to swap particles at {:?} and {:?} but failed: {:?}", position.0, neighbor_position, err);
                                        obstructed[obstruct_idx] = true;
                                    }
                                }
                            }
                        };
                    }
                }

                if moved {
                    visited.insert(position.0);
                } else {
                    if let Some(ref mut momentum) = momentum {
                        momentum.0 = IVec2::ZERO;
                    }
                    velocity.decrement();
                }
                particle_moved.0 = moved;
            },
        );
    }
}

fn swap_particle_positions(
    first_position: &mut ParticlePosition,
    first_transform: &mut Transform,
    second_position: &mut ParticlePosition,
    second_transform: &mut Transform,
) {
    mem::swap(
        &mut first_transform.translation,
        &mut second_transform.translation,
    );
    mem::swap(&mut first_position.0, &mut second_position.0);
}
