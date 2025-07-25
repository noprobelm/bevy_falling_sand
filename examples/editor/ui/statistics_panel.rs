use bevy::prelude::*;
use bevy_egui::egui;

#[derive(Resource)]
pub struct StatisticsPanel {
    pub fps: f32,
    pub total_particles: u32,
    pub dynamic_particles: u32,
    pub wall_particles: u32,
    pub active_particles: u32,
    pub dynamic_rigid_bodies: u32,
}

impl Default for StatisticsPanel {
    fn default() -> Self {
        Self {
            fps: 0.0,
            total_particles: 0,
            dynamic_particles: 0,
            wall_particles: 0,
            active_particles: 0,
            dynamic_rigid_bodies: 0,
        }
    }
}

impl StatisticsPanel {
    pub fn render(&self, ui: &mut egui::Ui) {
        let text_color = egui::Color32::from_rgb(204, 204, 204);
        ui.visuals_mut().override_text_color = Some(text_color);

        ui.heading("Statistics");
        ui.separator();
        ui.add_space(8.0);

        // Create a nice grid layout for the statistics
        egui::Grid::new("statistics_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .striped(false)
            .show(ui, |ui| {
                // FPS
                ui.label("FPS:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{:.1}", self.fps));
                });
                ui.end_row();

                // Total particles
                ui.label("Total Particles:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{}", self.total_particles));
                });
                ui.end_row();

                // Dynamic particles
                ui.label("Dynamic Particles:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{}", self.dynamic_particles));
                });
                ui.end_row();

                // Wall particles
                ui.label("Wall Particles:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{}", self.wall_particles));
                });
                ui.end_row();

                // Active particles
                ui.label("Active Particles:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{}", self.active_particles));
                });
                ui.end_row();

                // Dynamic rigid bodies
                ui.label("Dynamic Rigid Bodies:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{}", self.dynamic_rigid_bodies));
                });
                ui.end_row();
            });
    }
}