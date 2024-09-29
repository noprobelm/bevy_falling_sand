use bevy::prelude::*;
use bevy::utils::HashSet;
use std::mem;

use crate::physics_components::*;
use crate::rng::PhysicsRng;
use crate::MovementPriority;
use bfs_core::{ChunkMap, Coordinates, Hibernating, Particle};

/// Moves all qualifying particles 'v' times equal to their current velocity
#[allow(unused_mut)]
pub fn handle_particles(
    mut particle_query: Query<
        (
            Entity,
            &Particle,
            &mut Coordinates,
            &mut Transform,
            &mut PhysicsRng,
            &mut Velocity,
            Option<&mut Momentum>,
            &Density,
            &mut MovementPriority,
        ),
        Without<Hibernating>,
    >,
    mut map: ResMut<ChunkMap>,
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
            )| {
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

                        match map.entity(&neighbor_coordinates) {
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
                                    }
                                    // We've encountered an anchored or hibernating particle. If this is a hibernating particle, it's guaranteed to
                                    // be awoken on the next frame with the logic contained in ChunkMap.reset_chunks()
                                    else {
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
            },
        );
    }
}

/// Swaps the position of two particle's position information.
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
