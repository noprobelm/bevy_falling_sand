use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;
use bfs_assets::{ParticleData, ParticleDefinitions};
use bfs_color::ColorProfile;
use bfs_core::{ParticleType, ParticleTypeMap};
use bfs_movement::{Density, Momentum, Velocity};
use bfs_reactions::{Burns, Fire};
use ron::ser::{to_string_pretty, PrettyConfig};
use crate::ui::file_browser::{FileBrowser, FileBrowserState};
use bevy_egui::egui;

pub struct ParticleFilesPlugin;

impl Plugin for ParticleFilesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleFileDialog>()
            .insert_resource(FileBrowserState::new("assets/particles", "ron", "Particle Files"))
            .add_event::<SaveParticlesEvent>()
            .add_event::<LoadParticlesEvent>()
            .add_systems(
                Update,
                (
                    save_particles_to_file,
                    load_particles_from_file,
                ),
            );
    }
}

#[derive(Resource, Default)]
pub struct ParticleFileDialog {
    pub last_error: Option<String>,
    pub last_success: Option<String>,
}

#[derive(Event)]
pub struct SaveParticlesEvent(pub PathBuf);

#[derive(Event)]
pub struct LoadParticlesEvent(pub PathBuf);

pub fn spawn_save_dialog(browser_state: &mut ResMut<FileBrowserState>) {
    browser_state.show_save("Save Particle Set");
}

pub fn spawn_load_dialog(browser_state: &mut ResMut<FileBrowserState>) {
    browser_state.show_load("Load Particle Set");
}

pub struct ParticleFileBrowser;

impl ParticleFileBrowser {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        browser_state: &mut ResMut<FileBrowserState>,
        ev_save_particles: &mut EventWriter<SaveParticlesEvent>,
        ev_load_particles: &mut EventWriter<LoadParticlesEvent>,
    ) {
        let file_browser = FileBrowser;
        
        file_browser.render_save_dialog(
            ui,
            browser_state,
            |path| {
                ev_save_particles.write(SaveParticlesEvent(path));
            },
        );
        
        file_browser.render_load_dialog(
            ui,
            browser_state,
            |path| {
                ev_load_particles.write(LoadParticlesEvent(path));
            },
        );
    }
}

fn save_particles_to_file(
    mut ev_save_particles: EventReader<SaveParticlesEvent>,
    _particle_type_map: Res<ParticleTypeMap>,
    mut dialog_state: ResMut<ParticleFileDialog>,
    particle_query: Query<(
        &ParticleType,
        Option<&Density>,
        Option<&Velocity>,
        Option<&Momentum>,
        Option<&ColorProfile>,
        Option<&Burns>,
        Option<&Fire>,
        Option<&bfs_movement::Wall>,
        Option<&bfs_movement::Liquid>,
        Option<&bfs_movement::Gas>,
        Option<&bfs_movement::MovableSolid>,
        Option<&bfs_movement::Solid>,
        Option<&bfs_color::ChangesColor>,
    )>,
) {
    for SaveParticlesEvent(save_path) in ev_save_particles.read() {

        let mut particle_definitions = HashMap::new();

        // Collect all particle types and convert them to ParticleData
        for (
            particle_type_id,
            density,
            velocity,
            momentum,
            color_profile,
            burns,
            fire,
            wall,
            liquid,
            gas,
            movable_solid,
            solid,
            changes_color,
        ) in particle_query.iter()
        {
            let mut particle_data = ParticleData {
                name: particle_type_id.name.to_string(),
                density: density.map(|d| d.0),
                max_velocity: velocity.map(|v| v.max()),
                momentum: momentum.map(|_| true),
                liquid: None,
                gas: None,
                movable_solid: None,
                solid: None,
                wall: None,
                colors: None,
                changes_colors: changes_color.map(|c| c.chance),
                fire: None,
                burning: None,
                burns: None,
            };

            // Set material type
            if wall.is_some() {
                particle_data.wall = Some(true);
            } else if let Some(liquid) = liquid {
                particle_data.liquid = Some(liquid.fluidity as u8);
            } else if let Some(gas) = gas {
                particle_data.gas = Some(gas.fluidity as u8);
            } else if movable_solid.is_some() {
                particle_data.movable_solid = Some(true);
            } else if solid.is_some() {
                particle_data.solid = Some(true);
            }

            // Convert colors
            if let Some(color_profile) = color_profile {
                let color_strings: Vec<String> = color_profile
                    .palette
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
                    .collect();
                particle_data.colors = Some(color_strings);
            }

            // Convert fire data
            if let Some(fire) = fire {
                particle_data.fire = Some(bfs_assets::FireData {
                    burn_radius: fire.burn_radius,
                    chance_to_spread: fire.chance_to_spread,
                    destroys_on_spread: fire.destroys_on_spread,
                });
            }

            // Convert burns data
            if let Some(burns) = burns {
                let burns_data = bfs_assets::BurnsData {
                    duration: burns.duration.as_millis() as u64,
                    tick_rate: burns.tick_rate.as_millis() as u64,
                    chance_destroy_per_tick: burns.chance_destroy_per_tick,
                    ignites_on_spawn: Some(burns.ignites_on_spawn),
                    reaction: burns.reaction.as_ref().map(|r| bfs_assets::ReactionData {
                        produces: r.produces.name.to_string(),
                        chance_to_produce: r.chance_to_produce,
                    }),
                    colors: burns.color.as_ref().map(|color_profile| {
                        color_profile
                            .palette
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
                    spreads: burns.spreads.as_ref().map(|fire| bfs_assets::FireData {
                        burn_radius: fire.burn_radius,
                        chance_to_spread: fire.chance_to_spread,
                        destroys_on_spread: fire.destroys_on_spread,
                    }),
                };
                particle_data.burns = Some(burns_data);
            }

            particle_definitions.insert(particle_type_id.name.clone(), particle_data);
        }

        // Serialize to RON and save
        match to_string_pretty(&particle_definitions, PrettyConfig::default()) {
            Ok(ron_content) => match std::fs::write(save_path, ron_content) {
                Ok(()) => {
                    dialog_state.last_success = Some(format!(
                        "Saved {} particles to {}",
                        particle_definitions.len(),
                        save_path.display()
                    ));
                    dialog_state.last_error = None;
                }
                Err(e) => {
                    dialog_state.last_error = Some(format!("Failed to write file: {}", e));
                }
            },
            Err(e) => {
                dialog_state.last_error = Some(format!("Failed to serialize particles: {}", e));
            }
        }
    }
}

fn load_particles_from_file(
    mut commands: Commands,
    mut ev_load_particles: EventReader<LoadParticlesEvent>,
    mut dialog_state: ResMut<ParticleFileDialog>,
    _particle_type_map: Res<ParticleTypeMap>,
    particle_query: Query<Entity, With<ParticleType>>,
) {
    for LoadParticlesEvent(path) in ev_load_particles.read() {
        match std::fs::read_to_string(path) {
            Ok(ron_content) => {
                match ron::from_str::<ParticleDefinitions>(&ron_content) {
                    Ok(particle_definitions) => {
                        // Clear existing particle types
                        for entity in particle_query.iter() {
                            commands.entity(entity).despawn();
                        }

                        // Spawn new particle types from loaded data
                        let mut loaded_count = 0;
                        for (_name, particle_data) in particle_definitions {
                            particle_data.spawn_particle_type(&mut commands);
                            loaded_count += 1;
                        }

                        dialog_state.last_success = Some(format!(
                            "Loaded {} particles from {}",
                            loaded_count,
                            path.display()
                        ));
                        dialog_state.last_error = None;
                    }
                    Err(e) => {
                        dialog_state.last_error = Some(format!("Failed to parse RON file: {}", e));
                    }
                }
            }
            Err(e) => {
                dialog_state.last_error = Some(format!("Failed to read file: {}", e));
            }
        }
    }
}