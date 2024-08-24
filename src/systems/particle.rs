use std::fs::File;

use bevy::prelude::*;

use crate::components::Material;
use crate::*;

/// Sets up particle types predefined in a .ron file from the assets path.
pub fn setup_particle_types(mut commands: Commands, mut type_map: ResMut<ParticleTypeMap>) {
    let file_path = "assets/particles/particles.ron";
    let file = File::open(file_path).unwrap();
    let particle_types: ron::Map = ron::de::from_reader(file).unwrap();

    particle_types.iter().for_each(|(key, map)| {
        let particle_name = key.clone().into_rust::<String>().unwrap();
        let entity = commands.spawn(Name::new(particle_name.clone())).id();

        type_map.insert(particle_name.clone(), entity);
        commands.entity(entity).insert((
            ParticleType{name: particle_name.clone()},
            SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.))),
        );
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
                            component_data
                                .clone()
                                .into_rust::<bool>()
                                .expect("Config error: Expected 'true' or 'false' for 'momentum'");
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
                            let movement_priority = Liquid::new(fluidity).into_movement_priority();
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
                        _ => {
                            warn!["Erroneous config option found: {component_str}"]
                        }
                    }
                }
            });
    });
}
