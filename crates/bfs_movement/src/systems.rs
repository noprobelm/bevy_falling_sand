use crate::{MovementRng, Velocity, Momentum, Density, MovementPriority, Moved};
use std::mem;

use bevy::prelude::*;
use bevy::platform::collections::HashSet;
use bevy_turborand::{DelegatedRng, GlobalRng};
use bfs_core::{Particle, ParticleMap, ParticlePosition, ParticleSimulationSet};

pub(super) struct SystemsPlugin;

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MovementSource>()
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
    &'a mut MovementPriority,
    &'a mut Moved,
);

#[allow(unused_mut, clippy::too_many_lines)]
fn handle_movement_by_chunks(
    mut particle_query: Query<ParticleMovementQuery>,
    mut map: ResMut<ParticleMap>,
    mut rng: ResMut<GlobalRng>,
) {
    let mut visited: HashSet<Entity> = HashSet::default();
    let mut particle_entities: Vec<Entity> = Vec::with_capacity(map.particles_per_chunk);

    unsafe {
        map.iter_chunks_mut().for_each(|mut chunk| {
            if let Some(dirty_rect) = chunk.dirty_rect() {
                chunk.iter().for_each(|(position, entity)| {
                    if dirty_rect.contains(*position) {
                        particle_entities.push(*entity);
                    }
                });
            }
        });
        rng.shuffle(&mut particle_entities);
        particle_entities.iter().for_each(|entity| {
            if visited.contains(entity) {
                return;
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
                let mut moved = false;

                'velocity_loop: for _ in 0..velocity.val {
                    let mut obstructed: HashSet<IVec2> = HashSet::default();

                    for relative_position in movement_priority
                        .iter_candidates(&mut rng, momentum.as_deref().cloned().as_ref())
                    {
                        let neighbor_position = position.0 + *relative_position;
                        let signum = relative_position.signum();

                        if obstructed.contains(&signum) {
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

                                                if let Some(ref mut m) = momentum {
                                                    m.0 = IVec2::ZERO;
                                                }

                                                velocity.decrement();
                                                moved = true;
                                                break 'velocity_loop;
                                            }
                                            Err(err) => {
                                                debug!("Attempted to swap particles at {:?} and {:?} but failed: {:?}", position.0, neighbor_position, err);
                                            }
                                        }
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
                                match map.swap(position.0, neighbor_position) {
                                    Ok(()) => {
                                        position.0 = neighbor_position;
                                        transform.translation.x = neighbor_position.x as f32;
                                        transform.translation.y = neighbor_position.y as f32;
                                        if let Some(ref mut m) = momentum {
                                            m.0 = *relative_position;
                                        }
                                        velocity.increment();
                                        moved = true;
                                        continue 'velocity_loop;
                                    },
                                    Err(err) => {debug!("Attempted to swap particles at {:?} and {:?} but failed: {:?}", position.0, neighbor_position, err);}
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
        });
    }
}

#[allow(unused_mut, clippy::too_many_lines)]
fn handle_movement_by_particles(
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
                mut position,
                mut transform,
                mut rng,
                mut velocity,
                mut momentum,
                density,
                mut movement_priority,
                mut particle_moved,
            )| {
                if let Some(chunk) = map.chunk(&position.0) {
                    if let Some(dirty_rect) = chunk.dirty_rect() {
                    if !dirty_rect.contains(position.0) {
                        return;
                    }
                    } else {
                        return;
                    }
                }
                // Used to determine if we should add the particle to set of visited particles.
                let mut moved = false;
                'velocity_loop: for _ in 0..velocity.val {
                    // If a particle is blocked on a certain vector, we shouldn't attempt to swap it with other particles along that
                    // same vector.
                    let mut obstructed: HashSet<IVec2> = HashSet::default();

                    for relative_position in movement_priority
                        .iter_candidates(&mut rng, momentum.as_deref().copied().as_ref())
                    {
                        let neighbor_position = position.0 + *relative_position;

                        if visited.contains(&neighbor_position)
                            || obstructed.contains(&relative_position.signum())
                        {
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
                                        obstructed.insert(relative_position.signum());
                                        continue;
                                    }
                                }
                                // We've encountered an anchored particle
                                else {
                                    obstructed.insert(relative_position.signum());
                                    continue;
                                }
                            }
                            // We've encountered a free slot for the target particle to move to
                            None => {
                                match  map.swap(position.0, neighbor_position) {
                                    Ok(()) => {
                                        position.0 = neighbor_position;
                                        transform.translation.x = neighbor_position.x as f32;
                                        transform.translation.y = neighbor_position.y as f32;
                                        if let Some(ref mut momentum) = momentum {
                                            momentum.0 = *relative_position; // Set momentum relative to the current position
                                        }
                                        velocity.increment();
                                        moved = true;
                                        continue 'velocity_loop;
                                    },
                                    Err(err) => {
                                        debug!("Attempted to swap particles at {:?} and {:?} but failed: {:?}", position.0, neighbor_position, err);
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
