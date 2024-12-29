//! This crate the capability to load particle types as assets from external sources.

use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::Duration,
};
use serde::Deserialize;
use thiserror::Error;

use bfs_core::*;
use bfs_reactions::*;
use bfs_movement::*;
use bfs_color::*;

pub struct FallingSandAssetLoadersPlugin;

impl bevy::prelude::Plugin for FallingSandAssetLoadersPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
	app.init_asset::<ParticleTypesAsset>()
        .init_asset_loader::<ParticleTypesAssetLoader>() ;
    }
}

/// Collection of particle types loaded from an asset.
#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct ParticleTypesAsset {
    /// The particle types.
    pub particle_types: ron::Map,
}

impl ParticleTypesAsset {
    /// Loads particle types from the asset into the simulation.
    pub fn load_particle_types(
        &self,
        commands: &mut Commands,
        type_map: &mut ResMut<ParticleTypeMap>,
    ) {
        for (key, map) in self.particle_types.iter() {
            let particle_name = key
                .clone()
                .into_rust::<String>()
                .expect("Invalid particle name format");
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
                .expect("Config error: Expected map of particle data");

            // Deserialize each component for the particle entity
            particle_data
                .iter()
                .for_each(|(component_str, component_data)| {
                    if let Ok(component_str) = component_str.clone().into_rust::<String>() {
                        self.handle_component(
                            commands,
                            entity,
                            &particle_name,
                            &component_str,
                            component_data.clone(),
                        );
                    }
                });
        }
    }
}
impl ParticleTypesAsset {
    /// Handles deserialization of a components for a given entity.
    fn handle_component(
        &self,
        commands: &mut Commands,
        entity: Entity,
        particle_name: &str,
        component_str: &str,
        component_data: ron::Value,
    ) {
        match component_str {
            "density" => self.insert_density(commands, entity, component_data),
            "max_velocity" => self.insert_max_velocity(commands, entity, component_data),
            "momentum" => self.insert_momentum(commands, entity, component_data),
            "colors" => self.insert_colors(commands, entity, component_data),
            "changes_colors" => self.insert_flowing_colors(commands, entity, component_data),
            "randomizes_colors" => self.insert_random_colors(commands, entity, component_data),
            "liquid" => self.insert_liquid(commands, entity, component_data),
            "movable_solid" => self.insert_movable_solid(commands, entity),
            "solid" => self.insert_solid(commands, entity),
            "gas" => self.insert_gas(commands, entity, component_data),
            "wall" => {
                commands.entity(entity).insert(Wall);
            }
            "burns" => self.insert_burns(commands, entity, component_data),
            "fire" => self.insert_fire(commands, entity, component_data),
            "burning" => self.insert_burning(commands, entity, component_data),
            _ => warn!(
                "Erroneous config option found for particle '{}': {}",
                particle_name, component_str
            ),
        }
    }
    fn insert_density(&self, commands: &mut Commands, entity: Entity, component_data: ron::Value) {
        let density = component_data
            .into_rust::<u32>()
            .expect("Config error: Expected u32 for 'density'");
        commands.entity(entity).insert(Density(density));
    }

    fn insert_max_velocity(
        &self,
        commands: &mut Commands,
        entity: Entity,
        component_data: ron::Value,
    ) {
        let max_velocity = component_data
            .into_rust::<u8>()
            .expect("Config error: Expected u8 for 'max_velocity'");
        commands
            .entity(entity)
            .insert(Velocity::new(1, max_velocity));
    }

    fn insert_momentum(&self, commands: &mut Commands, entity: Entity, component_data: ron::Value) {
        component_data
            .into_rust::<bool>()
            .expect("Config error: Expected 'true' or 'false' for 'momentum'");
        commands.entity(entity).insert(Momentum(IVec2::ZERO));
    }

    fn insert_colors(&self, commands: &mut Commands, entity: Entity, component_data: ron::Value) {
        let colors: Vec<Color> = component_data
            .into_rust::<Vec<String>>()
            .expect("Expected array of 4 tuples holding f32 values")
            .iter()
            .map(|hex_str| {
                let srgba = Srgba::hex(hex_str).unwrap();
                Color::Srgba(srgba)
            })
            .collect();
        commands
            .entity(entity)
            .insert(ParticleColor::new(*colors.get(0).unwrap(), colors));
    }

    fn insert_liquid(&self, commands: &mut Commands, entity: Entity, component_data: ron::Value) {
        let fluidity = component_data
            .into_rust::<usize>()
            .expect("Config error: Expected usize for 'liquid'");
        commands.entity(entity).insert(Liquid::new(fluidity));
    }

    fn insert_movable_solid(&self, commands: &mut Commands, entity: Entity) {
        commands.entity(entity).insert(MovableSolid::new());
    }

    fn insert_solid(&self, commands: &mut Commands, entity: Entity) {
        commands.entity(entity).insert(Solid::new());
    }

    fn insert_gas(&self, commands: &mut Commands, entity: Entity, component_data: ron::Value) {
        let fluidity = component_data
            .into_rust::<usize>()
            .expect("Config error: Expected usize for 'gas'");
        commands.entity(entity).insert(Gas::new(fluidity));
    }

    fn insert_burns(&self, commands: &mut Commands, entity: Entity, component_data: ron::Value) {
        let (duration, tick_rate, chance_destroy_per_tick, reaction, burning_colors, spreads) =
            self.parse_burns(component_data);
        let burns = Burns::new(
            duration,
            tick_rate,
            chance_destroy_per_tick,
            reaction,
            burning_colors,
            spreads,
        );
        commands.entity(entity).insert(burns);
    }

    fn insert_fire(&self, commands: &mut Commands, entity: Entity, component_data: ron::Value) {
        let fire = self.parse_fire(component_data);
        commands.entity(entity).insert(fire);
    }

    fn insert_burning(&self, commands: &mut Commands, entity: Entity, component_data: ron::Value) {
        let burning = self.parse_burning(component_data);
        commands.entity(entity).insert(burning);
    }

    fn insert_flowing_colors(
        &self,
        commands: &mut Commands,
        entity: Entity,
        component_data: ron::Value,
    ) {
        let chance = component_data.into_rust::<f64>().unwrap();
        commands.entity(entity).insert(FlowsColor::new(chance));
    }

    fn insert_random_colors(
        &self,
        commands: &mut Commands,
        entity: Entity,
        component_data: ron::Value,
    ) {
        let chance = component_data.into_rust::<f64>().unwrap();
        commands.entity(entity).insert(RandomizesColor::new(chance));
    }

    fn parse_burns(
        &self,
        component_data: ron::Value,
    ) -> (
        Duration,
        Duration,
        Option<f64>,
        Option<Reacting>,
        Option<ParticleColor>,
        Option<Fire>,
    ) {
        let burn_map = component_data
            .into_rust::<ron::Map>()
            .expect("Config error: Expected map for 'burns' component");

        let mut duration: Duration = Duration::from_millis(0);
        let mut tick_rate: Duration = Duration::from_millis(0);
        let mut chance_destroy_per_tick: Option<f64> = None;
        let mut reaction: Option<Reacting> = None;
        let mut burning_colors: Option<ParticleColor> = None;
        let mut spreads: Option<Fire> = None;

        for (burn_key, burn_value) in burn_map.iter() {
            let burn_str = burn_key
                .clone()
                .into_rust::<String>()
                .expect("Config error: Expected valid mapping for 'burns'");
            match burn_str.as_str() {
                "duration" => {
                    duration = Duration::from_millis(
                        burn_value
                            .clone()
                            .into_rust::<u64>()
                            .expect("Config error: Expected u64 for 'duration'"),
                    );
                }
                "tick_rate" => {
                    tick_rate = Duration::from_millis(
                        burn_value
                            .clone()
                            .into_rust::<u64>()
                            .expect("Config error: Expected u64 for 'tick_rate'"),
                    );
                }
                "chance_destroy_per_tick" => {
                    chance_destroy_per_tick = Some(
                        burn_value
                            .clone()
                            .into_rust::<f64>()
                            .expect("Config error: Expected f64 for 'chance_destroy_per_tick'"),
                    );
                }
                "reaction" => {
                    reaction = Some(self.parse_reaction(burn_value.clone()));
                }
                "colors" => {
                    let colors: Vec<Color> = burn_value
                        .clone()
                        .into_rust::<Vec<String>>()
                        .expect("Expected array of 4 tuples holding f32 values")
                        .iter()
                        .map(|hex_str| {
                            let srgba = Srgba::hex(hex_str).unwrap();
                            Color::Srgba(srgba)
                        })
                        .collect();
                    burning_colors = Some(ParticleColor::new(*colors.get(0).unwrap(), colors));
                }
                "spreads" => {
                    spreads = Some(self.parse_fire(burn_value.clone()));
                }
                _ => {}
            }
        }

        (
            duration,
            tick_rate,
            chance_destroy_per_tick,
            reaction,
            burning_colors,
            spreads,
        )
    }

    fn parse_reaction(&self, reaction_value: ron::Value) -> Reacting {
        let reaction_map = reaction_value
            .into_rust::<ron::Map>()
            .expect("Config error: Expected map for 'reaction' component");

        let mut produces = String::new();
        let mut chance_to_produce: f64 = 0.0;

        for (reaction_key, reaction_value) in reaction_map.iter() {
            let reaction_str = reaction_key
                .clone()
                .into_rust::<String>()
                .expect("Config error: Expected valid mapping for 'reaction'");
            match reaction_str.as_str() {
                "produces" => {
                    produces = reaction_value
                        .clone()
                        .into_rust::<String>()
                        .expect("Config error: Expected String for 'produces'");
                }
                "chance_to_produce" => {
                    chance_to_produce = reaction_value
                        .clone()
                        .into_rust::<f64>()
                        .expect("Config error: Expected f64 for 'chance_to_produce'");
                }
                _ => {}
            }
        }

        Reacting::new(Particle::new(&produces), chance_to_produce)
    }

    fn parse_fire(&self, component_data: ron::Value) -> Fire {
        let fire_map = component_data
            .into_rust::<ron::Map>()
            .expect("Config error: Expected map for 'fire' component");

        let mut burn_radius: f32 = 0.0;
        let mut chance_to_spread: f64 = 0.0;
        let mut destroys_on_spread = false;

        for (fire_key, fire_value) in fire_map.iter() {
            let fire_str = fire_key
                .clone()
                .into_rust::<String>()
                .expect("Config error: Expected valid mapping for 'fire'");
            match fire_str.as_str() {
                "burn_radius" => {
                    burn_radius = fire_value
                        .clone()
                        .into_rust::<f32>()
                        .expect("Config error: Expected f32 for 'burn_radius'");
                }
                "chance_to_spread" => {
                    chance_to_spread = fire_value
                        .clone()
                        .into_rust::<f64>()
                        .expect("Config error: Expected f64 for 'chance_to_spread'");
                }
                "destroys_on_spread" => {
                    destroys_on_spread = fire_value
                        .clone()
                        .into_rust::<bool>()
                        .expect("Config error: Expected bool for 'destroys_on_spread'");
                }
                _ => {}
            }
        }

        Fire {
            burn_radius,
            chance_to_spread,
            destroys_on_spread,
        }
    }

    fn parse_burning(&self, component_data: ron::Value) -> Burning {
        let burning_map = component_data
            .into_rust::<ron::Map>()
            .expect("Config error: Expected map for 'burning' component");

        let mut duration: Duration = Duration::from_millis(0);
        let mut tick_rate: Duration = Duration::from_millis(0);

        for (burn_key, burn_value) in burning_map.iter() {
            let burn_str = burn_key
                .clone()
                .into_rust::<String>()
                .expect("Config error: Expected valid mapping for 'burning'");
            match burn_str.as_str() {
                "duration" => {
                    duration = Duration::from_millis(
                        burn_value
                            .clone()
                            .into_rust::<u64>()
                            .expect("Config error: Expected u64 for 'duration'"),
                    );
                }
                "tick_rate" => {
                    tick_rate = Duration::from_millis(
                        burn_value
                            .clone()
                            .into_rust::<u64>()
                            .expect("Config error: Expected u64 for 'tick_rate'"),
                    );
                }
                _ => {}
            }
        }

        Burning::new(duration, tick_rate)
    }
}

/// Asset loader for particle types.
#[derive(Default)]
pub struct ParticleTypesAssetLoader;

/// Possible errors that can be produced by [`ParticleTypesAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ParticleTypesAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for ParticleTypesAssetLoader {
    type Asset = ParticleTypesAsset;
    type Settings = ();
    type Error = ParticleTypesAssetLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let particle_data = ron::de::from_bytes::<ron::Map>(&bytes)?;
        Ok(ParticleTypesAsset {
            particle_types: particle_data,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["custom"]
    }
}
