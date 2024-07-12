use crate::*;
use bevy::utils::HashSet;

pub fn handle_particles(
    particle_query: Query<
        (
            Entity,
            &Parent,
            &ParticleType,
            &mut Coordinates,
            &mut Transform,
            &mut Velocity,
            &mut PhysicsRng,
        ),
        (Without<Anchored>, Without<Hibernating>),
    >,
    parent_query: Query<(&Density, &Neighbors), (With<ParticleParent>, Without<Anchored>)>,
    mut map: ResMut<ChunkMap>,
) {
    let mut visited: HashSet<IVec2> = HashSet::default();
    unsafe {
        particle_query.iter_unsafe().for_each(
            |(
                _,
                parent,
                particle_type,
                mut coordinates,
                mut transform,
                mut velocity,
                mut rng,
            )| {
                if let Ok((density, neighbors)) = parent_query.get(parent.get()) {
		    let mut moved = false;
                    'velocity_loop: for _ in 0..velocity.val {
                        let mut swap = false;
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

                                match map.get(&neighbor_coordinates) {
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

                                                    neighbor_transform.translation.x =
                                                        coordinates.0.x as f32;
                                                    neighbor_transform.translation.y =
                                                        coordinates.0.y as f32;

                                                    transform.translation.x =
                                                        neighbor_coordinates.0.x as f32;
                                                    transform.translation.y =
                                                        neighbor_coordinates.0.y as f32;

                                                    neighbor_coordinates.0 = coordinates.0;
                                                    coordinates.0 += relative_coordinates;

                                                    velocity.decrement();


						    moved = true;
                                                    swap = true;
                                                }
                                            } else {
                                                obstructed.insert(relative_coordinates.signum());
                                                velocity.decrement();

                                                continue;
                                            }
                                        } else {
                                            obstructed.insert(relative_coordinates.signum());

                                            continue;
                                        }
                                    }
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
				    map.swap(coordinates.0 - relative_coordinates, coordinates.0);
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

