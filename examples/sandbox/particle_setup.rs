//! Shows how to set up custom particle types for your world.
//!
//! `bevy_falling_sand` does not provide a default set of particles. See `examples/assets/particles/particles.ron` for
//! an example of how to create new particle types using RON.
//!
//! Alternatively (and for full access to particle behavior), spawn these bundles into the world to create a new
//! particle type:
//!   - `StaticParticleTypeBundle`: For particles that have no movement behavior (i.e., walls)
//!   - `DynamicParticleTypeBundle`: For particles that have movement behavior
use bevy::prelude::*;
use std::fs::File;
use std::path::Path;

use crate::particle_management::ParticleList;
use bevy_falling_sand::{material::Material, *};

/// Particle Management Plugin
pub(super) struct ParticleSetupPlugin;

impl bevy::prelude::Plugin for ParticleSetupPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Particle management systems
        app.add_systems(Startup, (setup_particle_types, setup_custom_particle));
    }
}

/// Set up particle types using a RON string.
pub fn setup_particle_types(mut commands: Commands, mut type_map: ResMut<ParticleTypeMap>) {
    let mut example_path = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    example_path.push("examples/assets/particles/particles.ron");
    let file = File::open(example_path).unwrap();
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

/// Demonstrates how to set up a custom particle with code.
pub fn setup_custom_particle(mut commands: Commands, mut particle_list: ResMut<ParticleList>) {
    // For particles that have movement, use the DynamicParticleTypeBundle
    let dynamic_particle_type = ParticleType::new("My Custom Particle");
    commands.spawn((
        DynamicParticleTypeBundle::new(
            dynamic_particle_type.clone(),
            Density(4),
            Velocity::new(1, 3),
            MovableSolid::new().into_movement_priority(),
            ParticleColors::new(vec![
                Color::srgba(0.22, 0.11, 0.16, 1.0),
                Color::srgba(0.24, 0.41, 0.56, 1.0),
                Color::srgba(0.67, 0.74, 0.55, 1.0),
                Color::srgba(0.91, 0.89, 0.71, 1.0),
                Color::srgba(0.95, 0.61, 0.43, 1.0),
            ]),
        ),
        // If momentum effects are desired, insert the marker component.
        Momentum::ZERO,
    ));

    // For particles that have no movement, use the StaticParticleTypeBundle
    let static_particle_type = ParticleType::new("My Custom Wall Particle");
    commands.spawn(StaticParticleTypeBundle::new(
        static_particle_type.clone(),
        ParticleColors::new(vec![
            Color::srgba(0.22, 0.11, 0.16, 1.0),
            Color::srgba(0.24, 0.41, 0.56, 1.0),
            Color::srgba(0.67, 0.74, 0.55, 1.0),
            Color::srgba(0.91, 0.89, 0.71, 1.0),
            Color::srgba(0.95, 0.61, 0.43, 1.0),
        ]),
    ));

    // Add the particle types to the UI for this example code.
    particle_list.push(dynamic_particle_type.name);
    particle_list.push(static_particle_type.name);
}
