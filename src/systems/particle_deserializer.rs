//! Deserializes RON strings into new particle types
use bevy::{prelude::*, utils::Duration};
use std::fs::File;

use crate::{components::material::Material, *};

/// Deserialize RON strings into new particle types and add them to the world.
pub fn deserialize_particle_types(
    mut commands: Commands,
    mut ev_particles_deserialize: EventReader<DeserializeParticleTypesEvent>,
    mut type_map: ResMut<ParticleTypeMap>,
) {
    for ev in ev_particles_deserialize.read() {
        let file = File::open(&ev.0).expect("Failed to open file for deserialization");

        let particle_types: ron::Map = ron::de::from_reader(file).expect("Failed to parse RON file");

        for (key, map) in particle_types.iter() {
            let particle_name = key.clone().into_rust::<String>().expect("Invalid particle name format");
            let entity = commands.spawn(Name::new(particle_name.clone())).id();

            type_map.insert(particle_name.clone(), entity);
            commands.entity(entity).insert((
                ParticleType {
                    name: particle_name.clone(),
                },
                SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
            ));

            let particle_data = map.clone().into_rust::<ron::Map>().expect("Config error: Expected map of particle data");

            // Deserialize each component for the particle entity
            particle_data.iter().for_each(|(component_str, component_data)| {
                if let Ok(component_str) = component_str.clone().into_rust::<String>() {
                    handle_component(&mut commands, entity, &particle_name, &component_str, component_data.clone());
                }
            });
        }
    }
}

/// Handles deserialization of a components for a given entity.
fn handle_component(
    commands: &mut Commands,
    entity: Entity,
    particle_name: &str,
    component_str: &str,
    component_data: ron::Value,
) {
    match component_str {
        "density" => insert_density(commands, entity, component_data),
        "max_velocity" => insert_max_velocity(commands, entity, component_data),
        "momentum" => insert_momentum(commands, entity, component_data),
        "colors" => insert_colors(commands, entity, component_data),
        "liquid" => insert_liquid(commands, entity, component_data),
        "movable_solid" => insert_movable_solid(commands, entity),
        "solid" => insert_solid(commands, entity),
        "gas" => insert_gas(commands, entity, component_data),
        "wall" => {commands.entity(entity).insert(Anchored);},
        "burns" => insert_burns(commands, entity, component_data),
        "fire" => insert_fire(commands, entity, component_data),
        "burning" => insert_burning(commands, entity, component_data),
        _ => warn!("Erroneous config option found for particle '{}': {}", particle_name, component_str),
    }
}

fn insert_density(commands: &mut Commands, entity: Entity, component_data: ron::Value) {
    let density = component_data.into_rust::<u32>().expect("Config error: Expected u32 for 'density'");
    commands.entity(entity).insert(Density(density));
}

fn insert_max_velocity(commands: &mut Commands, entity: Entity, component_data: ron::Value) {
    let max_velocity = component_data.into_rust::<u8>().expect("Config error: Expected u8 for 'max_velocity'");
    commands.entity(entity).insert(Velocity::new(1, max_velocity));
}

fn insert_momentum(commands: &mut Commands, entity: Entity, component_data: ron::Value) {
    component_data.into_rust::<bool>().expect("Config error: Expected 'true' or 'false' for 'momentum'");
    commands.entity(entity).insert(Momentum(IVec2::ZERO));
}

fn insert_colors(commands: &mut Commands, entity: Entity, component_data: ron::Value) {
    let colors: Vec<Color> = component_data
        .into_rust::<Vec<(f32, f32, f32, f32)>>()
        .expect("Expected array of 4 tuples holding f32 values")
        .iter()
        .map(|vals| Color::srgba(vals.0, vals.1, vals.2, vals.3))
        .collect();
    commands.entity(entity).insert(ParticleColors::new(colors));
}

fn insert_liquid(commands: &mut Commands, entity: Entity, component_data: ron::Value) {
    let fluidity = component_data.into_rust::<usize>().expect("Config error: Expected usize for 'liquid'");
    let movement_priority = Liquid::new(fluidity).into_movement_priority();
    commands.entity(entity).insert(movement_priority);
}

fn insert_movable_solid(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert(MovableSolid::new().into_movement_priority());
}

fn insert_solid(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert(Solid::new().into_movement_priority());
}

fn insert_gas(commands: &mut Commands, entity: Entity, component_data: ron::Value) {
    let fluidity = component_data.into_rust::<usize>().expect("Config error: Expected usize for 'gas'");
    let movement_priority = Gas::new(fluidity).into_movement_priority();
    commands.entity(entity).insert(movement_priority);
}

fn insert_burns(commands: &mut Commands, entity: Entity, component_data: ron::Value) {
    let (duration, tick_rate, chance_destroy_per_tick, reaction, random_colors, spreads) = parse_burns(component_data);
    let burns = Burns::new(duration, tick_rate, chance_destroy_per_tick, reaction, random_colors, spreads);
    commands.entity(entity).insert(burns);
}

fn insert_fire(commands: &mut Commands, entity: Entity, component_data: ron::Value) {
    let fire = parse_fire(component_data);
    commands.entity(entity).insert(fire);
}

fn insert_burning(commands: &mut Commands, entity: Entity, component_data: ron::Value) {
    let burning = parse_burning(component_data);
    commands.entity(entity).insert(burning);
}

fn parse_burns(component_data: ron::Value) -> (Duration, Duration, Option<f64>, Option<Reacting>, Option<RandomColors>, Option<Fire>) {
    let burn_map = component_data.into_rust::<ron::Map>().expect("Config error: Expected map for 'burns' component");

    let mut duration: Duration = Duration::from_millis(0);
    let mut tick_rate: Duration = Duration::from_millis(0);
    let mut chance_destroy_per_tick: Option<f64> = None;
    let mut reaction: Option<Reacting> = None;
    let mut random_colors: Option<RandomColors> = None;
    let mut spreads: Option<Fire> = None;

    for (burn_key, burn_value) in burn_map.iter() {
        let burn_str = burn_key.clone().into_rust::<String>().expect("Config error: Expected valid mapping for 'burns'");
        match burn_str.as_str() {
            "duration" => {
                duration = Duration::from_millis(burn_value.clone().into_rust::<u64>().expect("Config error: Expected u64 for 'duration'"));
            },
            "tick_rate" => {
                tick_rate = Duration::from_millis(burn_value.clone().into_rust::<u64>().expect("Config error: Expected u64 for 'tick_rate'"));
            },
            "chance_destroy_per_tick" => {
                chance_destroy_per_tick = Some(burn_value.clone().into_rust::<f64>().expect("Config error: Expected f64 for 'chance_destroy_per_tick'"));
            },
            "reaction" => {
                reaction = Some(parse_reaction(burn_value.clone()));
            },
            "colors" => {
                let colors: Vec<Color> = burn_value.clone()
                    .into_rust::<Vec<(f32, f32, f32, f32)>>()
                    .expect("Config error: Expected array of 4-tuples holding f32 values")
                    .iter()
                    .map(|vals| Color::srgba(vals.0, vals.1, vals.2, vals.3))
                    .collect();
                random_colors = Some(RandomColors::new(colors));
            },
            "spreads" => {
                spreads = Some(parse_fire(burn_value.clone()));
            },
            _ => {}
        }
    }

    (duration, tick_rate, chance_destroy_per_tick, reaction, random_colors, spreads)
}

fn parse_reaction(reaction_value: ron::Value) -> Reacting {
    let reaction_map = reaction_value.into_rust::<ron::Map>().expect("Config error: Expected map for 'reaction' component");

    let mut produces = String::new();
    let mut chance_to_produce: f64 = 0.0;

    for (reaction_key, reaction_value) in reaction_map.iter() {
        let reaction_str = reaction_key.clone().into_rust::<String>().expect("Config error: Expected valid mapping for 'reaction'");
        match reaction_str.as_str() {
            "produces" => {
                produces = reaction_value.clone().into_rust::<String>().expect("Config error: Expected String for 'produces'");
            },
            "chance_to_produce" => {
                chance_to_produce = reaction_value.clone().into_rust::<f64>().expect("Config error: Expected f64 for 'chance_to_produce'");
            },
            _ => {}
        }
    }

    Reacting::new(Particle::new(&produces), chance_to_produce)
}

fn parse_fire(component_data: ron::Value) -> Fire {
    let fire_map = component_data.into_rust::<ron::Map>().expect("Config error: Expected map for 'fire' component");

    let mut burn_radius: f32 = 0.0;
    let mut chance_to_spread: f64 = 0.0;
    let mut destroys_on_spread = false;

    for (fire_key, fire_value) in fire_map.iter() {
        let fire_str = fire_key.clone().into_rust::<String>().expect("Config error: Expected valid mapping for 'fire'");
        match fire_str.as_str() {
            "burn_radius" => {
                burn_radius = fire_value.clone().into_rust::<f32>().expect("Config error: Expected f32 for 'burn_radius'");
            },
            "chance_to_spread" => {
                chance_to_spread = fire_value.clone().into_rust::<f64>().expect("Config error: Expected f64 for 'chance_to_spread'");
            },
            "destroys_on_spread" => {
                destroys_on_spread = fire_value.clone().into_rust::<bool>().expect("Config error: Expected bool for 'destroys_on_spread'");
            },
            _ => {}
        }
    }

    Fire {burn_radius, chance_to_spread, destroys_on_spread}
}

fn parse_burning(component_data: ron::Value) -> Burning {
    let burning_map = component_data.into_rust::<ron::Map>().expect("Config error: Expected map for 'burning' component");

    let mut duration: Duration = Duration::from_millis(0);
    let mut tick_rate: Duration = Duration::from_millis(0);

    for (burn_key, burn_value) in burning_map.iter() {
        let burn_str = burn_key.clone().into_rust::<String>().expect("Config error: Expected valid mapping for 'burning'");
        match burn_str.as_str() {
            "duration" => {
                duration = Duration::from_millis(burn_value.clone().into_rust::<u64>().expect("Config error: Expected u64 for 'duration'"));
            },
            "tick_rate" => {
                tick_rate = Duration::from_millis(burn_value.clone().into_rust::<u64>().expect("Config error: Expected u64 for 'tick_rate'"));
            },
            _ => {}
        }
    }

    Burning::new(duration, tick_rate)
}
