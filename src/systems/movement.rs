use std::mem;

use crate::*;
use bevy::utils::HashSet;

/// Moves all qualifying particles 'v' times equal to their current velocity
#[allow(unused_mut)]
pub fn handle_particles(
    mut particle_query: Query<
        (
            Entity,
            &Parent,
            &ParticleType,
            &mut Coordinates,
            &mut Transform,
            &mut Velocity,
            &mut PhysicsRng,
        ),
        (Without<Anchored>, Without<ShouldProcessThisFrame>),
    >,
    parent_query: Query<(&Density, &Neighbors), (With<ParticleParent>, Without<Anchored>)>,
    mut map: ResMut<ChunkMap>,
) {
    // Check visited before we perform logic on a particle (particles shouldn't move more than once)
    let mut visited: HashSet<IVec2> = HashSet::default();
    unsafe {
        particle_query.iter_unsafe().for_each(
            |(_, parent, particle_type, mut coordinates, mut transform, mut velocity, mut rng)| {
                if let Ok((density, neighbors)) = parent_query.get(parent.get()) {
                    // Flag indicating whether the particle moved at all during this frame
                    let mut moved = false;
                    'velocity_loop: for _ in 0..velocity.val {
                        // Flag indicating whether the particle should be swapped. This is necessary because of the fact that we cannot have
                        // multiple shared mutable references to our ChunkMap. After we positively identify that a particle should be swapped
                        // with another, we'll set this flag to 'true' and update the ChunkMap after it's fallen out of the immutable
                        // reference in the logic that follows
                        let mut swap = false;

                        // If a particle is blocked on a certain vector, we shouldn't attempt to swap it with other particles along that
                        // same vector.
                        let mut obstructed: HashSet<IVec2> = HashSet::default();

                        for neighbor_group in &neighbors.0 {
                            let mut indices: Vec<usize> = (0..neighbor_group.len()).collect();
                            rng.shuffle(&mut indices);
                            for idx in indices {
                                let relative_coordinates = neighbor_group[idx];
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
                                            neighbor_parent,
                                            neighbor_particle_type,
                                            mut neighbor_coordinates,
                                            mut neighbor_transform,
                                            _,
                                            _,
                                        )) = particle_query.get_unchecked(*neighbor_entity)
                                        {
                                            if *particle_type == *neighbor_particle_type {
                                                continue;
                                            }
                                            if let Ok((neighbor_density, _)) =
                                                parent_query.get(neighbor_parent.get())
                                            {
                                                if density > neighbor_density {
                                                    swap_particle_positions(
                                                        &mut coordinates,
                                                        &mut transform,
                                                        &mut neighbor_coordinates,
                                                        &mut neighbor_transform,
                                                    );

                                                    swap = true;
                                                }
                                            }
                                            // We've encountered an anchored or hibernating particle. If this is a hibernating particle, it's guaranteed to
                                            // be awoken on the next frame with the logic contained in ChunkMap.reset_chunks()
                                            else {
                                                obstructed.insert(relative_coordinates.signum());
                                                velocity.decrement();

                                                continue;
                                            }
                                        }
                                        // We've encountered an anchored particle
                                        else {
                                            obstructed.insert(relative_coordinates.signum());
                                            velocity.decrement();

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

                                if swap == true {
				    // This will swap particle positions and wake up neighboring chunks if necessary
                                    map.swap(coordinates.0 - relative_coordinates, coordinates.0);
				    velocity.decrement();
				    moved = true;

                                    break 'velocity_loop;
                                }
                            }
                        }
                    }
                    if moved == true {
                        visited.insert(coordinates.0);
                    }
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
