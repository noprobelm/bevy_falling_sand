pub mod particle_files;

use super::*;
use crate::scenes::{spawn_load_scene_dialog, spawn_save_scene_dialog};
use crate::ui::file_browser::FileBrowserState;
use particle_files::{
    spawn_load_scene_dialog as spawn_load_particles_scene_dialog,
    spawn_save_scene_dialog as spawn_save_particles_scene_dialog,
};

pub use particle_files::ParticleFilesPlugin;

pub(super) struct UiTopBar;

impl UiTopBar {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        commands: &mut Commands,
        particle_browser_state: &mut ResMut<FileBrowserState>,
    ) {
        ui.menu_button(egui::RichText::new("File").size(16.0), |ui| {
            if ui.button("Save Scene").clicked() {
                spawn_save_scene_dialog(commands);
                ui.close_menu();
            }
            if ui.button("Load Scene").clicked() {
                spawn_load_scene_dialog(commands);
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Save Particle Set").clicked() {
                spawn_save_particles_scene_dialog(particle_browser_state);
                ui.close_menu();
            }
            if ui.button("Load Particle Set").clicked() {
                spawn_load_particles_scene_dialog(particle_browser_state);
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Settings").clicked() {
                ui.close_menu();
            }
            if ui.button("Documentation").clicked() {
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Exit").clicked() {
                ui.close_menu();
            }
        });
    }
}
