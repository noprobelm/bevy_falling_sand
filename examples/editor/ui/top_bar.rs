use super::*;

pub(super) struct UiTopBar;

impl UiTopBar {
    pub fn render(&self, ui: &mut egui::Ui) {
        ui.menu_button(egui::RichText::new("File").size(16.0), |ui| {
            if ui.button("Save Particle Set").clicked() {
                ui.close_menu();
            }
            if ui.button("Load Particle Set").clicked() {
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
