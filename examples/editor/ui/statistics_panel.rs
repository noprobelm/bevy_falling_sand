use bevy::prelude::*;
use bevy_egui::egui;
use bevy_falling_sand::prelude::MovementSource;

#[derive(Resource, Default)]
pub struct StatisticsPanel;

impl StatisticsPanel {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        particle_movement_state_current: &MovementSource,
        fps: f32,
        dynamic_particles: u32,
        wall_particles: u32,
        total_particles: u32,
        active_particles: u32,
        num_rigid_bodies: u32,
    ) {
        let text_color = egui::Color32::from_rgb(204, 204, 204);
        ui.visuals_mut().override_text_color = Some(text_color);

        ui.add(egui::Label::new(
            egui::RichText::new("States").heading().size(16.0),
        ));
        ui.separator();
        egui::Grid::new("states_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Particle Movement Logic:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    match particle_movement_state_current {
                        MovementSource::Particles => ui.label("Particles"),
                        MovementSource::Chunks => ui.label("Chunks"),
                    };
                });
                ui.end_row();
            });

        ui.add(egui::Label::new(
            egui::RichText::new("Performance").heading().size(16.0),
        ));
        ui.separator();
        egui::Grid::new("performance_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("FPS:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{}", fps.round() as i32));
                });
                ui.end_row();
            });

        ui.add_space(8.0);

        ui.add(egui::Label::new(
            egui::RichText::new("Particles").heading().size(16.0),
        ));
        ui.separator();
        egui::Grid::new("particles_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Total:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{}", total_particles));
                });
                ui.end_row();

                ui.label("Wall:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{}", wall_particles));
                });
                ui.end_row();

                ui.label("Dynamic:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{}", dynamic_particles));
                });
                ui.end_row();

                ui.label("Active:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{}", active_particles));
                });
                ui.end_row();
            });

        ui.add_space(8.0);

        ui.add(egui::Label::new(
            egui::RichText::new("Avian 2D").heading().size(16.0),
        ));
        ui.separator();
        egui::Grid::new("avian_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Dynamic Rigid Bodies:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{}", num_rigid_bodies));
                });
                ui.end_row();
            });
    }
}
