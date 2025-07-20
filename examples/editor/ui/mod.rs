mod console;
mod layers_panel;
mod particle_editor;
mod top_bar;

use console::{ConsoleState, render_console};
use layers_panel::LayersPanel;
use particle_editor::ParticleEditor;
use top_bar::UiTopBar;

use bevy::prelude::*;
pub(super) use bevy_egui::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        .init_resource::<ConsoleState>()
        .add_systems(EguiContextPass, render);
    }
}

fn render(mut contexts: EguiContexts, mut console_state: ResMut<ConsoleState>) {
    let ctx = contexts.ctx_mut();

    egui::TopBottomPanel::top("Top panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            UiTopBar.render(ui);
        });
    });

    egui::SidePanel::left("Left panel")
        .resizable(false)
        .show(ctx, |ui| {
            // Fill background
            ui.painter().rect_filled(
                ui.available_rect_before_wrap(),
                0.0,
                egui::Color32::from_rgb(30, 30, 30),
            );

            let available_rect = ui.available_rect_before_wrap();
            let spacing = 8.0;
            let panel_height = (available_rect.height() - spacing) / 2.0;
            let panel_bg = egui::Color32::from_rgb(46, 46, 46);

            // Top panel - Particle Editor
            let top_response = ui.allocate_response(
                egui::vec2(available_rect.width(), panel_height),
                egui::Sense::hover(),
            );

            ui.scope_builder(egui::UiBuilder::new().max_rect(top_response.rect), |ui| {
                ui.set_clip_rect(top_response.rect);
                egui::Frame::NONE
                    .fill(panel_bg)
                    .corner_radius(4.0)
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ui.set_min_height(panel_height - 16.0); // Account for margins
                        ui.set_max_height(panel_height - 16.0);
                        ParticleEditor.render(ui);
                    });
            });

            // Add spacing
            ui.add_space(spacing);

            // Bottom panel - Layers
            let bottom_response = ui.allocate_response(
                egui::vec2(available_rect.width(), panel_height),
                egui::Sense::hover(),
            );

            ui.scope_builder(
                egui::UiBuilder::new().max_rect(bottom_response.rect),
                |ui| {
                    ui.set_clip_rect(bottom_response.rect);
                    egui::Frame::NONE
                        .fill(panel_bg)
                        .corner_radius(4.0)
                        .inner_margin(egui::Margin::same(8))
                        .show(ui, |ui| {
                            ui.set_min_height(panel_height - 16.0); // Account for margins
                            ui.set_max_height(panel_height - 16.0);
                            LayersPanel.render(ui);
                        });
                },
            );
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        // Fill background to match left panel
        ui.painter().rect_filled(
            ui.available_rect_before_wrap(),
            0.0,
            egui::Color32::from_rgb(30, 30, 30),
        );
        
        let console_height = if console_state.expanded { 200.0 } else { 40.0 };
        
        egui::TopBottomPanel::bottom("Console panel")
            .resizable(false)
            .min_height(console_height)
            .max_height(console_height)
            .show_inside(ui, |ui| {
                render_console(ui, &mut console_state);
            });
    });
}
