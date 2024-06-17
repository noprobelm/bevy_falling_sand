use std::sync::{Arc, Mutex, RwLock};

use crate::*;
use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_turborand::prelude::*;

pub fn handle_particles(
    mut commands: Commands,
    particle_query: Query<(
        Entity,
        &ParticleType,
        &Parent,
        &Coordinates,
        &Velocity,
        Option<&Momentum>,
        Option<&Hibernating>,
    )>,
    parent_query: Query<(&Density, &Neighbors), (With<ParticleParent>, Without<Anchored>)>,
    chunk_groups: Res<ChunkGroups>,
    mut rng: ResMut<GlobalRng>,
    mut map: ResMut<ParticleMap>,
) {
    let rng = Arc::new(Mutex::new(rng));
    let mut visited: Arc<RwLock<HashSet<IVec2>>> = Arc::new(RwLock::new(HashSet::with_capacity(map.len())));
    let map = Arc::new(RwLock::new(map));
    for chunk_group in [
        ChunkGroup::One,
        ChunkGroup::Two,
        ChunkGroup::Three,
        ChunkGroup::Four,
    ] {
        particle_query.par_iter().for_each(
            |(entity, particle_type, parent, coordinates, velocity, momentum, hibernating)| {
                if chunk_groups.member_of(&coordinates.0) == chunk_group {
                    let mut current_coords = coordinates.0;
                    if let Ok((density, neighbors)) = parent_query.get(parent.get()) {
                        let mut obstructed: HashSet<IVec2> = HashSet::default();
                        'velocity_loop: for _ in 0..velocity.val {
                            for neighbor_group in &neighbors.0 {
                                // Momentum logic will go here
                                let mut shuffled = neighbor_group.clone();
                                rng.lock().unwrap().shuffle(&mut shuffled);
                                for relative_coords in shuffled.iter() {
                                    let neighbor_coords = current_coords + *relative_coords;
                                    if visited.read().unwrap().contains(&neighbor_coords)
                                        || obstructed.contains(&relative_coords.signum())
                                    {
                                        continue;
                                    }

                                    match map.read().unwrap().get(&neighbor_coords) {
                                        Some(neighbor_entity) => {
                                            let (_, neighbor_type, neighbor_parent, _, _, _, _) =
                                                particle_query.get(*neighbor_entity).unwrap();
                                            if particle_type == neighbor_type {
                                                continue;
                                            }
                                            if let Ok((neighbor_density, _)) =
                                                parent_query.get(neighbor_parent.get())
                                            {
                                                if density > neighbor_density {
                                                    map.write().unwrap().insert_overwrite(
                                                        current_coords,
                                                        *neighbor_entity,
                                                    );
                                                    map.write().unwrap().insert_overwrite(neighbor_coords, entity);
                                                    current_coords= neighbor_coords;
                                                    visited.write().unwrap().insert(current_coords);
                                                    break 'velocity_loop;
                                                }
                                            }
					    obstructed.insert(relative_coords.signum());
                                        }
                                        None => {
					    map.write().unwrap().remove(&current_coords);
					    map.write().unwrap().insert_overwrite(neighbor_coords, entity);
					    current_coords = neighbor_coords;
					}
                                    }
                                }
                            }
                        }
                    }
                }
            },
        );
    }
}
