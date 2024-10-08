use std::mem;

use crate::*;
use bevy::utils::HashSet;

/// Moves all qualifying particles 'v' times equal to their current velocity
#[allow(unused_mut)]
pub fn handle_particles(
    mut particle_query: Query<
        (
            Entity,
            &ParticleType,
            &mut Coordinates,
            &mut Transform,
            &mut PhysicsRng,
	    &mut Velocity,
	    &Density,
	    &MovementPriority
        ),
        (Without<Hibernating>, With<Particle>),
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
		density,
		movement_priority
            )| {
                // Flag indicating whether the particle moved at all during this frame
                let mut moved = false;
                'velocity_loop: for _ in 0..velocity.val {
                    // If a particle is blocked on a certain vector, we shouldn't attempt to swap it with other particles along that
                    // same vector.
                    let mut obstructed: HashSet<IVec2> = HashSet::default();

                    for group in &movement_priority.0 {
                        let mut indices: Vec<usize> = (0..group.len()).collect();
                        rng.shuffle(&mut indices);
                        for idx in indices {
                            let relative_coordinates = group[idx];
                            let neighbor_coordinates = coordinates.0 + relative_coordinates;

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
					neighbor_density,
					_
                                    )) = particle_query.get_unchecked(*neighbor_entity)
                                    {
                                        if *particle_type == *neighbor_particle_type {
                                            continue;
                                        }
                                        if density > neighbor_density {
                                            map.swap(
						neighbor_coordinates.0,
                                                coordinates.0,
                                            );

                                            swap_particle_positions(
                                                &mut coordinates,
                                                &mut transform,
                                                &mut neighbor_coordinates,
                                                &mut neighbor_transform,
                                            );

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

                                    velocity.increment();

                                    moved = true;

                                    continue 'velocity_loop;
                                }
                            };
                        }
                    }
                }
                if moved == true {
                    visited.insert(coordinates.0);
                } else {
		    velocity.decrement();
		}
            },
        );
    }
}

pub fn swap_particle_positions(
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
