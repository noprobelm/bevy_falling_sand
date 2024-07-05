use std::mem;
use bevy::utils::HashSet;
use crate::*;

pub fn handle_particles(
    mut particle_query: Query<
        (
            Entity,
            &Parent,
            &mut ParticleType,
            &mut Coordinates,
            &mut Transform,
            &mut Velocity,
            Option<&mut Momentum>,
            Option<&mut Hibernating>,
            &mut PhysicsRng,
        ),
        Without<Anchored>,
    >,
    parent_query: Query<(&Density, &Neighbors), (With<ParticleParent>, Without<Anchored>)>,
    mut map: ResMut<ParticleMap>,
) {
    unsafe {
        particle_query.iter_unsafe().for_each(
            |(
                entity,
                parent,
                mut particle_type,
                mut coordinates,
                mut transform,
                mut velocity,
                mut momentum,
                mut hibernating,
                mut rng,
            )| {
                let (density, neighbors) = parent_query.get(parent.get()).unwrap();
		let mut visited: HashSet<IVec2> = HashSet::default();
                'velocity_loop: for _ in 0..velocity.val {
                    let mut swapped = false;
		    let mut obstructed: HashSet<IVec2> = HashSet::default();
                    for neighbor_group in &neighbors.0 {
                        let mut shuffled = neighbor_group.clone();
                        rng.shuffle(&mut shuffled);
                        for relative_coordinates in shuffled {
                            let neighbor_coordinates = coordinates.0 + relative_coordinates;

			    if visited.contains(&neighbor_coordinates) || obstructed.contains(&relative_coordinates.signum()) {
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
                                        mut neighbor_velocity,
                                        mut neighbor_momentum,
                                        mut neighbor_hibernating,
                                        _,
                                    )) = particle_query.get_unchecked(*neighbor_entity)
                                    {
					if *particle_type == *neighbor_particle_type {
					    continue;
					}
					let (neighbor_density, _) = parent_query.get(neighbor_parent.get()).unwrap();
					if density > neighbor_density {
					    neighbor_coordinates.0 = coordinates.0;
					    neighbor_transform.translation.x = neighbor_coordinates.0.x as f32;
					    neighbor_transform.translation.y = neighbor_coordinates.0.y as f32;

					    coordinates.0 += relative_coordinates;
					    transform.translation.x = neighbor_coordinates.0.x as f32;
					    transform.translation.y = neighbor_coordinates.0.y as f32;

					    visited.insert(coordinates.0);

					    swapped = true;
					}
                                    } else {
					obstructed.insert(relative_coordinates.signum());
                                        continue;
                                    }
                                }
                                None => {
				    map.remove(&coordinates.0);
				    map.insert_overwrite(neighbor_coordinates, entity);
				    coordinates.0 = neighbor_coordinates;
				    continue 'velocity_loop
				}
                            };
			    if swapped == true {
				let neighbor_entity = map.remove(&neighbor_coordinates).unwrap();
				map.insert_overwrite(coordinates.0, neighbor_entity);
				map.insert_overwrite(neighbor_coordinates, neighbor_entity);
				break 'velocity_loop
			    }
                        }
                    }
                }
            },
        );
    }
}

#[allow(unused_mut)]
pub fn handle_particles_transform(
    mut particle_query: Query<(&mut Coordinates, &mut Transform, &mut LastMoved)>,
    map: Res<ParticleMap>,
    time: Res<Time>,
) {
    map.par_iter().for_each(|(coords, entity)| unsafe {
        let (mut coordinates, mut transform, mut last_moved) =
            particle_query.get_unchecked(*entity).unwrap();
        transform.translation.x = coords.x as f32;
        transform.translation.y = coords.y as f32;
        if coordinates.0 != *coords {
            coordinates.0 = *coords
        } else {
            last_moved.0.tick(time.delta());
        }
    });
}

pub fn handle_velocity(mut particle_query: Query<(&LastMoved, &mut Velocity)>) {
    particle_query
        .par_iter_mut()
        .for_each(|(last_moved, mut velocity)| {
            if last_moved.0.elapsed_secs() == 0. {
                velocity.increment();
            } else {
                velocity.decrement();
            }
        })
}
