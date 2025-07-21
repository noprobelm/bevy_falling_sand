pub mod particle_files;

use super::*;
use particle_files::{spawn_save_dialog, spawn_load_dialog};

pub use particle_files::ParticleFilesPlugin;

pub(super) struct UiTopBar;

impl UiTopBar {
    pub fn render(&self, ui: &mut egui::Ui, commands: &mut Commands) {
        ui.menu_button(egui::RichText::new("File").size(16.0), |ui| {
            if ui.button("Save Particle Set").clicked() {
                spawn_save_dialog(commands);
                ui.close_menu();
            }
            if ui.button("Load Particle Set").clicked() {
                spawn_load_dialog(commands);
                ui.close_menu();
            }
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