use crate::PhysicsRng;
use crate::*;
use std::mem;

use bevy::platform::collections::HashSet;
use bevy_turborand::{DelegatedRng, GlobalRng};
use bfs_core::{Coordinates, Particle, ParticleMap, ParticleSimulationSet};

pub(super) struct SystemsPlugin;

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MovementSource>()
            .add_systems(
                Update,
                handle_movement_by_chunks
                    .in_set(ParticleSimulationSet)
                    .run_if(in_state(MovementSource::Chunks)),
            )
            .add_systems(
                Update,
                handle_movement_by_particles
                    .in_set(ParticleSimulationSet)
                    .run_if(in_state(MovementSource::Particles)),
            );
    }
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum MovementSource {
    Chunks,
    #[default]
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
    &'a mut Moved,
);

#[allow(unused_mut)]
pub fn handle_movement_by_chunks(
    mut particle_query: Query<ParticleMovementQuery>,
    mut map: ResMut<ParticleMap>,
    mut rng: ResMut<GlobalRng>,
) {
    let mut visited: HashSet<Entity> = HashSet::default();
    let mut particle_entities: Vec<Entity> = Vec::with_capacity(map.particles_per_chunk);

    unsafe {
        map.iter_chunks_mut().for_each(|mut chunk| {
            chunk.iter().for_each(|(coordinates, entity)| {
                if let Some(dirty_rect) = chunk.dirty_rect() {
                    if dirty_rect.contains(*coordinates) || rng.chance(0.05) {
                        particle_entities.push(*entity);
                    }
                } else if rng.chance(0.05) {
                    particle_entities.push(*entity);
                }
            });
        });
        rng.shuffle(&mut particle_entities);
        particle_entities.iter().for_each(|entity| {
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
                mut particle_moved,
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

                        match map.get(&neighbor_coordinates) {
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
                                    _,
                                )) = particle_query.get_unchecked(*neighbor_entity)
                                {
                                    if *particle_type == *neighbor_particle_type {
                                        continue;
                                    }

                                    if density > neighbor_density {
                                        map.swap(neighbor_coords.0, coordinates.0);

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
                                map.swap(coordinates.0, neighbor_coordinates);
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
        });
    }
}

#[allow(unused_mut)]
pub fn handle_movement_by_particles(
    mut particle_query: Query<ParticleMovementQuery>,
    mut map: ResMut<ParticleMap>,
) {
    // Check visited before we perform logic on a particle (particles shouldn't move more than once)
    let mut visited: HashSet<IVec2> = HashSet::default();
    unsafe {
        particle_query.iter_unsafe().for_each(
            |(
                _,
                particle_type,
                mut coordinates,
                mut transform,
                mut rng,
                mut velocity,
                mut momentum,
                density,
                mut movement_priority,
                mut particle_moved,
            )| {
                if let Some(chunk) = map.chunk(&coordinates.0) {
                    if let Some(dirty_rect) = chunk.dirty_rect() {
                        if !dirty_rect.contains(coordinates.0) && !rng.chance(0.2) {
                            return;
                        }
                    } else if !rng.chance(0.05) {
                        return;
                    }
                }

                // Used to determine if we should add the particle to set of visited particles.
                let mut moved = false;
                'velocity_loop: for _ in 0..velocity.val {
                    // If a particle is blocked on a certain vector, we shouldn't attempt to swap it with other particles along that
                    // same vector.
                    let mut obstructed: HashSet<IVec2> = HashSet::default();

                    for relative_coordinates in movement_priority
                        .iter_candidates(&mut rng, momentum.as_deref().cloned().as_ref())
                    {
                        let neighbor_coordinates = coordinates.0 + *relative_coordinates;

                        if visited.contains(&neighbor_coordinates)
                            || obstructed.contains(&relative_coordinates.signum())
                        {
                            continue;
                        }

                        match map.get(&neighbor_coordinates) {
                            Some(neighbor_entity) => {
                                if let Ok((
                                    _,
                                    neighbor_particle_type,
                                    mut neighbor_coordinates,
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
                                        map.swap(neighbor_coordinates.0, coordinates.0);

                                        swap_particle_positions(
                                            &mut coordinates,
                                            &mut transform,
                                            &mut neighbor_coordinates,
                                            &mut neighbor_transform,
                                        );

                                        if let Some(ref mut momentum) = momentum {
                                            momentum.0 = IVec2::ZERO; // Reset momentum after a swap
                                        }

                                        velocity.decrement();
                                        moved = true;
                                        break 'velocity_loop;
                                    } else {
                                        obstructed.insert(relative_coordinates.signum());
                                        continue;
                                    }
                                }
                                // We've encountered an anchored particle
                                else {
                                    obstructed.insert(relative_coordinates.signum());
                                    continue;
                                }
                            }
                            // We've encountered a free slot for the target particle to move to
                            None => {
                                map.swap(coordinates.0, neighbor_coordinates);
                                coordinates.0 = neighbor_coordinates;

                                transform.translation.x = neighbor_coordinates.x as f32;
                                transform.translation.y = neighbor_coordinates.y as f32;

                                if let Some(ref mut momentum) = momentum {
                                    momentum.0 = *relative_coordinates; // Set momentum relative to the current position
                                }

                                velocity.increment();

                                moved = true;

                                continue 'velocity_loop;
                            }
                        };
                    }
                }

                if moved {
                    visited.insert(coordinates.0);
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
