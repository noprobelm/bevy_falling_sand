pub mod console;
mod layers_panel;
mod particle_editor;
mod top_bar;

use crate::{
    app_state::UiInteractionState,
    console::{
        core::{ConsoleCache, ConsoleCommandEntered, ConsoleConfiguration, ConsoleState},
        ConsolePlugin,
    },
};
use console::render_console;
use layers_panel::LayersPanel;
use particle_editor::ParticleEditor;
use top_bar::UiTopBar;

use bevy::prelude::*;
pub(super) use bevy_egui::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EguiPlugin {
                enable_multipass_for_primary_context: false,
            },
            ConsolePlugin,
        ))
        .add_systems(Update, console::receive_console_line)
        .add_systems(
            EguiContextPass,
            render.before(bevy_egui::EguiPreUpdateSet::InitContexts),
        );
    }
}

fn render(
    mut contexts: EguiContexts,
    mut console_state: ResMut<ConsoleState>,
    cache: Res<ConsoleCache>,
    config: Res<ConsoleConfiguration>,
    mut command_writer: EventWriter<ConsoleCommandEntered>,
    mut ui_state: ResMut<UiInteractionState>,
) {
    let ctx = contexts.ctx_mut();

    ui_state.mouse_over_ui = false;

    let top_response = egui::TopBottomPanel::top("Top panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            UiTopBar.render(ui);
        });
    });

    if top_response.response.hovered() {
        ui_state.mouse_over_ui = true;
    }

    let left_response = egui::SidePanel::left("Left panel")
        .resizable(false)
        .min_width(450.0)
        .max_width(450.0)
        .show(ctx, |ui| {
            // Fill the entire panel with the background color
            ui.painter().rect_filled(
                ui.available_rect_before_wrap(),
                0.0,
                egui::Color32::from_rgb(30, 30, 30),
            );
            
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                
                // Calculate exact 50/50 split
                let total_height = ui.available_height();
                let spacing = 8.0;
                let panel_height = (total_height - spacing) / 2.0;
                
                // Top half - Particle Editor (exactly 50%)
                ui.group(|ui| {
                    ui.set_height(panel_height);
                    ParticleEditor.render(ui);
                });

                // Bottom half - Layers Panel (exactly 50%)
                ui.group(|ui| {
                    ui.set_height(panel_height);
                    LayersPanel.render(ui);
                });
            });
        });

    if left_response.response.hovered() {
        ui_state.mouse_over_ui = true;
    }

    // Use a bottom panel for the console instead of taking up the entire central area
    let console_height = if console_state.expanded {
        console_state.height
    } else {
        40.0
    };

    let console_response = egui::TopBottomPanel::bottom("Console panel")
        .exact_height(console_height)
        .show(ctx, |ui| {
            render_console(ui, &mut console_state, &cache, &config, &mut command_writer);
        });

    if console_response.response.hovered() {
        ui_state.mouse_over_ui = true;
    }

    // The central panel is now free for the game canvas - make it transparent
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE) // No background frame
        .show(ctx, |ui| {
            // This area is now free for the Bevy game world to show through
            // We can detect mouse interaction but don't draw anything
            let rect = ui.available_rect_before_wrap();
            let response = ui.allocate_rect(rect, egui::Sense::click());
            
            // Only set mouse_over_ui to true if we're actually over UI elements, not the canvas
            if response.hovered() {
                // Don't set mouse_over_ui = true here - this area should be for the canvas
            }
        });
}
