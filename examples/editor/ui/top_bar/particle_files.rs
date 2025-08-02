use std::path::PathBuf;

use crate::ui::file_browser::{FileBrowser, FileBrowserState};
use bevy::prelude::*;
use bevy_egui::egui;
use bevy_falling_sand::prelude::{
    LoadParticleDefinitionsSceneEvent, ParticleType, SaveParticleDefinitionsEvent,
};

pub struct ParticleFilesPlugin;

impl Plugin for ParticleFilesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleFileDialog>()
            .insert_resource(FileBrowserState::new(
                "assets/particles",
                "ron",
                "Particle Files",
            ))
            .add_event::<SaveParticlesSceneEvent>()
            .add_event::<LoadParticlesSceneEvent>()
            .add_systems(
                Update,
                (save_particles_scene_system, load_particles_scene_system),
            );
    }
}

#[derive(Resource, Default)]
pub struct ParticleFileDialog {
    pub last_error: Option<String>,
    pub last_success: Option<String>,
}

#[derive(Event)]
pub struct SaveParticlesSceneEvent(pub PathBuf);

#[derive(Event)]
pub struct LoadParticlesSceneEvent(pub PathBuf);

pub fn spawn_save_scene_dialog(browser_state: &mut ResMut<FileBrowserState>) {
    browser_state.show_save("Save Particle Set");
}

pub fn spawn_load_scene_dialog(browser_state: &mut ResMut<FileBrowserState>) {
    browser_state.show_load("Load Particle Set");
}

pub struct ParticleFileBrowser;

impl ParticleFileBrowser {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        browser_state: &mut ResMut<FileBrowserState>,
        ev_save_particles_scene: &mut EventWriter<SaveParticlesSceneEvent>,
        ev_load_particles_scene: &mut EventWriter<LoadParticlesSceneEvent>,
    ) {
        let file_browser = FileBrowser;

        file_browser.render_save_dialog(ui, browser_state, |path| {
            ev_save_particles_scene.write(SaveParticlesSceneEvent(path));
        });

        file_browser.render_load_dialog(ui, browser_state, |path| {
            ev_load_particles_scene.write(LoadParticlesSceneEvent(path));
        });
    }
}

// Scene-based particle definition systems
fn save_particles_scene_system(
    mut ev_save_particles_scene: EventReader<SaveParticlesSceneEvent>,
    mut ev_save_definitions: EventWriter<SaveParticleDefinitionsEvent>,
    mut dialog_state: ResMut<ParticleFileDialog>,
) {
    for SaveParticlesSceneEvent(save_path) in ev_save_particles_scene.read() {
        // Convert .ron to .particles.scn.ron for scene format
        let mut scene_path = save_path.clone();
        scene_path.set_extension("particles.scn.ron");

        ev_save_definitions.write(SaveParticleDefinitionsEvent(scene_path.clone()));

        dialog_state.last_success = Some(format!(
            "Saving particle definitions to scene format: {}",
            scene_path.display()
        ));
        dialog_state.last_error = None;
    }
}

fn load_particles_scene_system(
    mut commands: Commands,
    mut ev_load_particles_scene: EventReader<LoadParticlesSceneEvent>,
    mut ev_load_scene: EventWriter<LoadParticleDefinitionsSceneEvent>,
    mut dialog_state: ResMut<ParticleFileDialog>,
    particle_query: Query<Entity, With<ParticleType>>,
) {
    for LoadParticlesSceneEvent(path) in ev_load_particles_scene.read() {
        // First, despawn all existing particle types
        for entity in particle_query.iter() {
            commands.entity(entity).despawn();
        }

        // Then load from the scene
        ev_load_scene.write(LoadParticleDefinitionsSceneEvent(path.clone()));

        dialog_state.last_success = Some(format!(
            "Loading particle definitions from scene: {}",
            path.display()
        ));
        dialog_state.last_error = None;
    }
}
