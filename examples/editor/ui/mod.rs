mod layers_panel;
mod particle_editor;
mod top_bar;

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
        .add_systems(EguiContextPass, render);
    }
}

fn render(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();
    
    egui::TopBottomPanel::top("Top panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            UiTopBar.render(ui);
        });
    });

    egui::SidePanel::left("left_panel")
        .resizable(false)
        .show(ctx, |ui| {
            ui.painter().rect_filled(
                ui.available_rect_before_wrap(),
                0.0,
                egui::Color32::from_rgb(30, 30, 30),
            );

            let available = ui.available_rect_before_wrap();
            let spacing = 8.0;
            let panel_height = (available.height() - spacing) / 2.0;
            
            let panel_bg = egui::Color32::from_rgb(46, 46, 46);

            let top_rect = egui::Rect::from_min_size(
                available.min,
                egui::vec2(available.width(), panel_height),
            );
            let bottom_rect = egui::Rect::from_min_size(
                available.min + egui::vec2(0.0, panel_height + spacing),
                egui::vec2(available.width(), panel_height),
            );

            ui.allocate_ui_at_rect(top_rect, |ui| {
                egui::Frame::NONE
                    .fill(panel_bg)
                    .rounding(4.0)
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ParticleEditor.render(ui);
                    });
            });

            ui.allocate_ui_at_rect(bottom_rect, |ui| {
                egui::Frame::NONE
                    .fill(panel_bg)
                    .rounding(4.0)
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        LayersPanel.render(ui);
                    });
            });
        });
}