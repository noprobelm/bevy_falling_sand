use bevy::prelude::*;
use bfs_assets::ParticleDefinitions;
use std::collections::HashMap;
use std::path::Path;

use crate::particle_registry::{ParticleRegistry, ParticleRegistryExt};

/// Events for particle file operations
#[derive(Event)]
pub struct SaveParticleFileEvent {
    pub path: String,
    pub entities: Vec<Entity>,
}

#[derive(Event)]
pub struct LoadParticleFileEvent {
    pub path: String,
}

#[derive(Event)]
pub struct ParticleFileSavedEvent {
    pub path: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Event)]
pub struct ParticleFileLoadedEvent {
    pub path: String,
    pub success: bool,
    pub particle_count: usize,
    pub error: Option<String>,
}

/// Utility functions for particle file I/O
pub struct ParticleFileUtils;

impl ParticleFileUtils {
    /// Save particles to a RON file
    pub fn save_particles_to_file(
        path: impl AsRef<Path>,
        particle_data: HashMap<String, bfs_assets::ParticleData>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let definitions = particle_data;
        
        let ron_string = ron::ser::to_string_pretty(&definitions, Default::default())?;
        
        std::fs::write(path, ron_string)?;
        
        Ok(())
    }

    /// Load particles from a RON file
    pub fn load_particles_from_file(
        path: impl AsRef<Path>,
    ) -> Result<HashMap<String, bfs_assets::ParticleData>, Box<dyn std::error::Error + Send + Sync>> {
        let content = std::fs::read_to_string(path)?;
        
        let definitions: ParticleDefinitions = ron::from_str(&content)?;
        
        Ok(definitions)
    }

    /// Get a suggested filename for particle exports
    pub fn get_suggested_filename() -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("particles_{}.ron", now)
    }
}

/// System to handle save particle file events
pub fn handle_save_particle_file_events(
    mut save_events: EventReader<SaveParticleFileEvent>,
    mut saved_events: EventWriter<ParticleFileSavedEvent>,
    mut registry: ResMut<ParticleRegistry>,
    world: &World,
) {
    for event in save_events.read() {
        let particle_data = registry.entities_to_data(&event.entities, world);
        
        if particle_data.is_empty() {
            saved_events.write(ParticleFileSavedEvent {
                path: event.path.clone(),
                success: false,
                error: Some("No valid particles found to save".to_string()),
            });
            continue;
        }

        let path = event.path.clone();
        
        match ParticleFileUtils::save_particles_to_file(&path, particle_data) {
            Ok(()) => {
                saved_events.write(ParticleFileSavedEvent {
                    path: event.path.clone(),
                    success: true,
                    error: None,
                });
            }
            Err(e) => {
                saved_events.write(ParticleFileSavedEvent {
                    path: event.path.clone(),
                    success: false,
                    error: Some(e.to_string()),
                });
            }
        }
    }
}

/// System to handle load particle file events
pub fn handle_load_particle_file_events(
    mut load_events: EventReader<LoadParticleFileEvent>,
    mut loaded_events: EventWriter<ParticleFileLoadedEvent>,
    mut commands: Commands,
    registry: Res<ParticleRegistry>,
) {
    for event in load_events.read() {
        let path = event.path.clone();
        
        match ParticleFileUtils::load_particles_from_file(&path) {
            Ok(particle_data) => {
                let particle_count = particle_data.len();
                
                // Spawn particles into the world
                for (_name, data) in particle_data {
                    registry.data_to_entity(&data, &mut commands);
                }
                
                loaded_events.write(ParticleFileLoadedEvent {
                    path: event.path.clone(),
                    success: true,
                    particle_count,
                    error: None,
                });
            }
            Err(e) => {
                loaded_events.write(ParticleFileLoadedEvent {
                    path: event.path.clone(),
                    success: false,
                    particle_count: 0,
                    error: Some(e.to_string()),
                });
            }
        }
    }
}

/// Plugin to add particle file I/O systems
pub struct ParticleFileIOPlugin;

impl Plugin for ParticleFileIOPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveParticleFileEvent>()
            .add_event::<LoadParticleFileEvent>()
            .add_event::<ParticleFileSavedEvent>()
            .add_event::<ParticleFileLoadedEvent>()
            .init_resource::<ParticleRegistry>()
            .add_systems(
                Update,
                (
                    handle_save_particle_file_events,
                    handle_load_particle_file_events,
                ),
            );
    }
}

/// High-level API for particle file operations
pub struct ParticleFileAPI;

impl ParticleFileAPI {
    /// Save all particles in the world to a file
    pub fn save_all_particles(
        commands: &mut Commands,
        world: &World,
        path: String,
    ) {
        let entities = world.get_particle_entities();
        commands.trigger(SaveParticleFileEvent { path, entities });
    }
    
    /// Save specific particles to a file
    pub fn save_particles(
        commands: &mut Commands,
        entities: Vec<Entity>,
        path: String,
    ) {
        commands.trigger(SaveParticleFileEvent { path, entities });
    }
    
    /// Load particles from a file
    pub fn load_particles(
        commands: &mut Commands,
        path: String,
    ) {
        commands.trigger(LoadParticleFileEvent { path });
    }
}