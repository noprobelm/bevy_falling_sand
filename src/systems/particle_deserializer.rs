use bevy::{prelude::*, utils::Duration};
use std::fs::File;

use crate::{components::material::Material, *};

/// Set up particle types using a RON string.
pub fn deserialize_particle_types(
    mut commands: Commands,
    mut ev_particles_deserialize: EventReader<DeserializeParticleTypesEvent>,
    mut type_map: ResMut<ParticleTypeMap>,
) {
    for ev in ev_particles_deserialize.read() {
        let file = File::open(&ev.0).unwrap();

        let particle_types: ron::Map = ron::de::from_reader(file).unwrap();

        particle_types.iter().for_each(|(key, map)| {
            let particle_name = key.clone().into_rust::<String>().unwrap();
            let entity = commands.spawn(Name::new(particle_name.clone())).id();

            type_map.insert(particle_name.clone(), entity);
            commands.entity(entity).insert((
                ParticleType {
                    name: particle_name.clone(),
                },
                SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
            ));
            let particle_data = map
                .clone()
                .into_rust::<ron::Map>()
                .expect("Config error: Expected map of particle data for '{particle_name}'");
            particle_data
                .iter()
                .for_each(|(component_str, component_data)| {
                    if let Ok(component_str) = component_str.clone().into_rust::<String>() {
                        match component_str.as_str() {
                            "density" => {
                                let density = component_data
                                    .clone()
                                    .into_rust::<u32>()
                                    .expect("Config error: Expected u32 for 'density'");
                                commands.entity(entity).insert(Density(density));
                            }
                            "max_velocity" => {
                                let max_velocity = component_data
                                    .clone()
                                    .into_rust::<u8>()
                                    .expect("Config error: Expected u8 for 'max_velocity'");
                                commands
                                    .entity(entity)
                                    .insert(Velocity::new(1, max_velocity));
                            }
                            "momentum" => {
                                component_data.clone().into_rust::<bool>().expect(
                                    "Config error: Expected 'true' or 'false' for 'momentum'",
                                );
                                commands.entity(entity).insert(Momentum(IVec2::ZERO));
                            }
                            "colors" => {
                                let colors: Vec<Color> = component_data
                                    .clone()
                                    .into_rust::<Vec<(f32, f32, f32, f32)>>()
                                    .expect("Expected array of 4 tuples holding f32 values")
                                    .iter()
                                    .map(|vals| Color::srgba(vals.0, vals.1, vals.2, vals.3))
                                    .collect();
                                commands.entity(entity).insert(ParticleColors::new(colors));
                            }
                            "liquid" => {
                                let fluidity = component_data
                                    .clone()
                                    .into_rust::<usize>()
                                    .expect("Config error: Expected u32 for 'Liquid'");
                                let movement_priority =
                                    Liquid::new(fluidity).into_movement_priority();
                                commands.entity(entity).insert(movement_priority);
                            }
                            "movable_solid" => {
                                commands
                                    .entity(entity)
                                    .insert(MovableSolid::new().into_movement_priority());
                            }
                            "solid" => {
                                commands
                                    .entity(entity)
                                    .insert(Solid::new().into_movement_priority());
                            }
                            "gas" => {
                                let fluidity = component_data
                                    .clone()
                                    .into_rust::<usize>()
                                    .expect("Config error: Expected u32 for 'Liquid'");
                                let movement_priority = Gas::new(fluidity).into_movement_priority();
                                commands.entity(entity).insert(movement_priority);
                            }
                            "wall" => {
                                commands.entity(entity).insert(Anchored);
                            }
                            "burns" => {
				let mut duration: Duration = Duration::from_millis(1000);
				let mut tick_rate : Duration = Duration::from_millis(100);
				let mut chance_destroy_per_tick: Option<f64> = None;
				let mut reaction: Option<Reacting> = None;
				let mut random_colors: Option<RandomColors> = None;
				let mut spreads: Option<Fire> = None;
                                let burn_map =
                                    component_data.clone().into_rust::<ron::Map>().expect(
                                        "Config error: Expected burn data, found {component_str}",
                                    );
				burn_map.iter().for_each(|(burn_key, burn_value)| {
				    let burn_str = burn_key.clone().into_rust::<String>().expect("Config error: Expected valid mapping for 'burns', found {burn_key}");
				    match burn_str.as_str() {
					"duration" => {
					    duration = Duration::from_millis(burn_value.clone().into_rust::<u64>().expect("Config error: Expected milliseconds as u64 for 'duration', received {burn_value}"));
					},
					"tick_rate" => {
					    tick_rate = Duration::from_millis(burn_value.clone().into_rust::<u64>().expect("Config error: Expected milliseconds as u64 for 'tick_rate', received {burn_value}"));
					},
					"chance_destroy_per_tick" => {
					  chance_destroy_per_tick  = Some(burn_value.clone().into_rust::<f64>().expect("Config error: Expected milliseconds as u64 for 'tick_rate', received {burn_value}"));
					},
					"reaction" => {
					    let mut produces = String::new();
					    let mut chance_to_produce: f64 = 0.;
					    let reaction_map = burn_value.clone().into_rust::<ron::Map>().expect("Config error: Expected valid mapping for 'reaction', found {reaction_value}");
					    reaction_map.iter().for_each(|(reaction_key, reaction_value)| {
						let reaction_str = reaction_key.clone().into_rust::<String>().expect("Config error: Expected valid mapping for 'reaction', found {reaction_value}");
						match reaction_str.as_str() {
						    "produces" => {
							produces = reaction_value.clone().into_rust::<String>().expect("Config error: Expected string for 'produces', found {reaction_value}");
						    },
						    "chance_to_produce" => {
							chance_to_produce = reaction_value.clone().into_rust::<f64>().expect("Config error: Expected chance as f64 for 'chance_to_produce', found {reaction_value}")
						    }
						    _ => {}
						}
					    });
					    reaction = Some(Reacting::new(Particle::new(produces.as_str()), chance_to_produce));
					},
					"colors" => {
					    let colors: Vec<Color> = burn_value
						.clone()
 .into_rust::<Vec<(f32, f32, f32, f32)>>()
						.expect("Expected array of 4 tuples holding f32 values")
						.iter()
						.map(|vals| Color::srgba(vals.0, vals.1, vals.2, vals.3))
						.collect();

					    random_colors = Some(RandomColors::new(colors));
					},
					"spreads" => {
					    let mut burn_radius: f32 = 0.;
					    let mut chance_to_spread: f64 = 0.;
					    let mut destroys_on_spread = false;
					    let fire_map = burn_value.clone().into_rust::<ron::Map>().expect("Config error: Expected valid mapping for 'reaction', found {reaction_value}");
					    fire_map.iter().for_each(|(fire_key, fire_value)| {
						let reaction_str = fire_key.clone().into_rust::<String>().expect("Config error: Expected valid mapping for 'reaction', found {reaction_value}");
						match reaction_str.as_str() {
						    "burn_radius" => {
							burn_radius = fire_value.clone().into_rust::<f32>().expect("Config error: Expected f32 for 'radius', found {fire_value}");
						    },
						    "chance_to_spread" => {
							chance_to_spread = fire_value.clone().into_rust::<f64>().expect("Config error: Expected chance as f64 for 'chance_to_spread', found {fire_value}")
						    }
						    "destroys_on_spread" => {
							destroys_on_spread = fire_value.clone().into_rust::<bool>().expect("Config error: Expected bool for 'destroys_on_spread', found {fire_value}")
						    }

						    _ => {}
						}
					    });
					    spreads = Some(Fire{burn_radius, chance_to_spread, destroys_on_spread})
					}
					_ => {}
				    }
				});
				let burns = Burns::new(duration, tick_rate, chance_destroy_per_tick, reaction, random_colors, spreads);
				commands.entity(entity).insert(burns);
                            },
                            _ => {
                                warn!["Erroneous config option found: {component_str}"]
                            }
                        }
                    }
                });
        });
    }
}
