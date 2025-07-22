use bevy::prelude::*;
use bfs_assets::ParticleData;
use bfs_color::{ChangesColor, ColorProfile};
use bfs_movement::{Density, Gas, Liquid, Momentum, MovableSolid, Solid, Velocity, Wall};
use bfs_reactions::{Burns, Fire};
use std::collections::HashMap;

/// Central registry for particle serialization and management
#[derive(Resource, Default)]
pub struct ParticleRegistry {
    /// Cache of particle data for performance
    data_cache: HashMap<Entity, ParticleData>,
}

impl ParticleRegistry {
    /// Create a new particle registry
    pub fn new() -> Self {
        Self {
            data_cache: HashMap::new(),
        }
    }

    /// Convert an entity with particle components to ParticleData
    pub fn entity_to_data(&mut self, entity: Entity, world: &World) -> Option<ParticleData> {
        // Check cache first
        if let Some(cached) = self.data_cache.get(&entity) {
            return Some(cached.clone());
        }

        let entity_ref = world.get_entity(entity).ok()?;

        // Get the particle type name (stored in ParticleType component)
        let name = if let Some(particle_type) = entity_ref.get::<bfs_core::ParticleType>() {
            particle_type.name.to_string()
        } else {
            format!("Particle_{}", entity.index())
        };

        // Extract all component data
        let density = entity_ref.get::<Density>().map(|d| d.0);
        let max_velocity = entity_ref.get::<Velocity>().map(|v| v.max());
        let momentum = entity_ref.get::<Momentum>().is_some();

        // Material state
        let wall = entity_ref.get::<Wall>().map(|_| true);
        let solid = entity_ref.get::<Solid>().map(|_| true);
        let movable_solid = entity_ref.get::<MovableSolid>().map(|_| true);
        let liquid = entity_ref.get::<Liquid>().map(|l| l.fluidity as u8);
        let gas = entity_ref.get::<Gas>().map(|g| g.fluidity as u8);

        // Color properties
        let colors = entity_ref.get::<ColorProfile>().map(|cp| {
            cp.palette
                .iter()
                .map(|color| {
                    let srgba = color.to_srgba();
                    format!(
                        "#{:02X}{:02X}{:02X}{:02X}",
                        (srgba.red * 255.0) as u8,
                        (srgba.green * 255.0) as u8,
                        (srgba.blue * 255.0) as u8,
                        (srgba.alpha * 255.0) as u8
                    )
                })
                .collect()
        });

        let changes_colors = entity_ref.get::<ChangesColor>().map(|cc| cc.chance);

        // Fire properties
        let fire = entity_ref.get::<Fire>().map(|f| bfs_assets::FireData {
            burn_radius: f.burn_radius,
            chance_to_spread: f.chance_to_spread,
            destroys_on_spread: f.destroys_on_spread,
        });

        // Burns properties
        let burns = entity_ref.get::<Burns>().map(|b| bfs_assets::BurnsData {
            duration: b.duration.as_millis() as u64,
            tick_rate: b.tick_rate.as_millis() as u64,
            chance_destroy_per_tick: b.chance_destroy_per_tick,
            reaction: b.reaction.as_ref().map(|r| bfs_assets::ReactionData {
                produces: r.produces.name.to_string(),
                chance_to_produce: r.chance_to_produce,
            }),
            colors: b.color.as_ref().map(|cp| {
                cp.palette
                    .iter()
                    .map(|color| {
                        let srgba = color.to_srgba();
                        format!(
                            "#{:02X}{:02X}{:02X}{:02X}",
                            (srgba.red * 255.0) as u8,
                            (srgba.green * 255.0) as u8,
                            (srgba.blue * 255.0) as u8,
                            (srgba.alpha * 255.0) as u8
                        )
                    })
                    .collect()
            }),
            spreads: b.spreads.as_ref().map(|f| bfs_assets::FireData {
                burn_radius: f.burn_radius,
                chance_to_spread: f.chance_to_spread,
                destroys_on_spread: f.destroys_on_spread,
            }),
            ignites_on_spawn: Some(b.ignites_on_spawn),
        });

        let particle_data = ParticleData {
            name,
            density,
            max_velocity,
            momentum: if momentum { Some(true) } else { None },
            liquid,
            gas,
            movable_solid,
            solid,
            wall,
            colors,
            changes_colors,
            fire,
            burning: None,
            burns,
        };

        // Cache the result
        self.data_cache.insert(entity, particle_data.clone());

        Some(particle_data)
    }

    /// Convert `ParticleData` to an entity with components
    pub fn data_to_entity(&self, data: &ParticleData, commands: &mut Commands) -> Entity {
        // Use the existing ParticleData::spawn_particle_type method
        data.spawn_particle_type(commands)
    }

    /// Get all particle entities from the world that have required components
    pub fn get_all_particle_entities(world: &World) -> Vec<Entity> {
        // Get all entities and filter them manually since we only have &World
        world
            .iter_entities()
            .filter(|entity_ref| entity_ref.contains::<bfs_core::ParticleType>())
            .map(|entity_ref| entity_ref.id())
            .collect()
    }

    /// Clear the internal cache
    pub fn clear_cache(&mut self) {
        self.data_cache.clear();
    }

    /// Convert multiple entities to particle data
    pub fn entities_to_data(
        &mut self,
        entities: &[Entity],
        world: &World,
    ) -> HashMap<String, ParticleData> {
        let mut result = HashMap::new();

        for &entity in entities {
            if let Some(data) = self.entity_to_data(entity, world) {
                result.insert(data.name.clone(), data);
            }
        }

        result
    }
}

/// Extension trait to add registry methods to World
pub trait ParticleRegistryExt {
    /// Convert an entity to particle data
    fn entity_to_particle_data(
        &self,
        entity: Entity,
        registry: &mut ParticleRegistry,
    ) -> Option<ParticleData>;

    /// Get all particle entities
    fn get_particle_entities(&self) -> Vec<Entity>;
}

impl ParticleRegistryExt for World {
    fn entity_to_particle_data(
        &self,
        entity: Entity,
        registry: &mut ParticleRegistry,
    ) -> Option<ParticleData> {
        registry.entity_to_data(entity, self)
    }

    fn get_particle_entities(&self) -> Vec<Entity> {
        ParticleRegistry::get_all_particle_entities(self)
    }
}
