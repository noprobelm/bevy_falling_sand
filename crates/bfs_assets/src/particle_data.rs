use bevy::prelude::*;
use bfs_color::{ChangesColor, ColorProfile};
use bfs_core::ParticleType;
use bfs_movement::{Density, Momentum, Velocity};
use bfs_reactions::{Burns, Fire};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A serializable representation of all components a Particle can have within [`bevy_falling_sand`].
#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct ParticleData {
    /// The unique name/identifier for this particle type.
    pub name: String,

    // Movement-related components
    /// The density of the particle, affects movement priority.
    pub density: Option<u32>,
    /// The maximum velocity the particle can achieve.
    pub max_velocity: Option<u8>,
    /// Whether the particle has momentum physics.
    pub momentum: Option<bool>,

    // Material type components (only one should be set)
    /// Liquid movement properties and viscosity.
    pub liquid: Option<u8>,
    /// Gas movement properties and buoyancy.
    pub gas: Option<u8>,
    /// Movable solid properties.
    pub movable_solid: Option<bool>,
    /// Static solid properties.
    pub solid: Option<bool>,
    /// Wall properties (immovable).
    pub wall: Option<bool>,

    // Color-related components
    /// The color palette for this particle type.
    pub colors: Option<Vec<String>>, // Hex color strings
    /// Chance for the particle to change colors.
    pub changes_colors: Option<f64>,

    // Reaction-related components
    /// Fire emission properties.
    pub fire: Option<FireData>,
    /// Burning behavior when ignited.
    pub burning: Option<BurningData>,
    /// Burn susceptibility and behavior.
    pub burns: Option<BurnsData>,
}

/// Serializable fire emission data.
#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct FireData {
    /// The radius of fire spread.
    pub burn_radius: f32,
    /// Chance to spread fire per tick.
    pub chance_to_spread: f64,
}

/// Serializable burning duration data.
#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct BurningData {
    /// How long the particle burns (in milliseconds).
    pub duration: u64,
    /// Tick rate for burning effects (in milliseconds).
    pub tick_rate: u64,
}

/// Serializable burn susceptibility data.
#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct BurnsData {
    /// How long the particle burns when ignited (in milliseconds).
    pub duration: u64,
    /// Tick rate for burning effects (in milliseconds).
    pub tick_rate: u64,
    /// Chance to be destroyed per tick while burning.
    pub chance_destroy_per_tick: Option<f64>,
    /// Reaction that occurs while burning.
    pub reaction: Option<ReactionData>,
    /// Colors to use while burning (hex strings).
    pub colors: Option<Vec<String>>,
    /// Fire spread properties while burning.
    pub spreads: Option<FireData>,
}

/// Serializable reaction data.
#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct ReactionData {
    /// What particle type to produce.
    pub produces: String,
    /// Chance to produce the reaction per tick.
    pub chance_to_produce: f64,
}

impl ParticleData {
    /// Convert a hex color string to a Bevy Color.
    fn parse_color(hex: &str) -> Result<Color, String> {
        let hex = hex.trim_start_matches('#');
        let (r, g, b, a) = if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| format!("Invalid red component: {}", &hex[0..2]))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| format!("Invalid green component: {}", &hex[2..4]))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| format!("Invalid blue component: {}", &hex[4..6]))?;
            (r, g, b, 255)
        } else if hex.len() == 8 {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| format!("Invalid red component: {}", &hex[0..2]))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| format!("Invalid green component: {}", &hex[2..4]))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| format!("Invalid blue component: {}", &hex[4..6]))?;
            let a = u8::from_str_radix(&hex[6..8], 16)
                .map_err(|_| format!("Invalid alpha component: {}", &hex[6..8]))?;
            (r, g, b, a)
        } else {
            return Err(format!("Invalid hex color length: {}", hex));
        };

        Ok(Color::srgba_u8(r, g, b, a))
    }

    /// Spawn this particle data as a [`ParticleTypeId`] entity with all appropriate components.
    pub fn spawn_particle_type(&self, commands: &mut Commands) -> Entity {
        let mut entity_commands = commands.spawn(ParticleType::from_string(self.name.clone()));

        // Add movement components
        if let Some(density) = self.density {
            entity_commands.insert(Density(density));
        }

        if let Some(max_velocity) = self.max_velocity {
            entity_commands.insert(Velocity::new(1, max_velocity));
        }

        if self.momentum.unwrap_or(false) {
            entity_commands.insert(Momentum::ZERO);
        }

        // Add material type components
        if let Some(viscosity) = self.liquid {
            entity_commands.insert(bfs_movement::Liquid::new(viscosity.into()));
        } else if let Some(buoyancy) = self.gas {
            entity_commands.insert(bfs_movement::Gas::new(buoyancy.into()));
        } else if self.movable_solid.unwrap_or(false) {
            entity_commands.insert(bfs_movement::MovableSolid);
        } else if self.solid.unwrap_or(false) {
            entity_commands.insert(bfs_movement::Solid);
        } else if self.wall.unwrap_or(false) {
            entity_commands.insert(bfs_movement::Wall);
        }

        // Color components
        if let Some(color_strings) = &self.colors {
            let mut colors = Vec::new();
            for color_str in color_strings {
                match Self::parse_color(color_str) {
                    Ok(color) => colors.push(color),
                    Err(e) => warn!(
                        "Failed to parse color '{}' for particle '{}': {}",
                        color_str, self.name, e
                    ),
                }
            }
            if !colors.is_empty() {
                entity_commands.insert(ColorProfile::new(colors));
            }
        }

        if let Some(chance) = self.changes_colors {
            entity_commands.insert(ChangesColor::new(chance));
        }

        // Reaction components
        if let Some(fire_data) = &self.fire {
            entity_commands.insert(Fire {
                burn_radius: fire_data.burn_radius,
                chance_to_spread: fire_data.chance_to_spread,
                destroys_on_spread: false,
            });
        }

        if let Some(burns_data) = &self.burns {
            let duration = std::time::Duration::from_millis(burns_data.duration);
            let tick_rate = std::time::Duration::from_millis(burns_data.tick_rate);

            let reaction = burns_data.reaction.as_ref().map(|r| {
                bfs_reactions::Reacting::new(
                    bfs_core::Particle::from_string(r.produces.clone()),
                    r.chance_to_produce,
                )
            });

            let color = if let Some(color_strings) = &burns_data.colors {
                let mut colors = Vec::new();
                for color_str in color_strings {
                    match Self::parse_color(color_str) {
                        Ok(color) => colors.push(color),
                        Err(e) => warn!(
                            "Failed to parse burn color '{}' for particle '{}': {}",
                            color_str, self.name, e
                        ),
                    }
                }
                if !colors.is_empty() {
                    Some(ColorProfile::new(colors))
                } else {
                    None
                }
            } else {
                None
            };

            let spreads = burns_data.spreads.as_ref().map(|s| Fire {
                burn_radius: s.burn_radius,
                chance_to_spread: s.chance_to_spread,
                destroys_on_spread: false,
            });

            entity_commands.insert(Burns::new(
                duration,
                tick_rate,
                burns_data.chance_destroy_per_tick,
                reaction,
                color,
                spreads,
                false, // ignites_on_spawn - could be made configurable
            ));
        }

        entity_commands.id()
    }
}

/// A collection of particle definitions loaded from a RON file.
pub type ParticleDefinitions = HashMap<String, ParticleData>;
