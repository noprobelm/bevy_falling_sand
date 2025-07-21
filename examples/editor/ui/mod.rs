pub mod console;
mod layers_panel;
mod particle_editor;
mod top_bar;

use crate::{
    app_state::InitializationState,
    console::{
        core::{ConsoleCache, ConsoleCommandEntered, ConsoleConfiguration, ConsoleState},
        ConsolePlugin,
    },
    particles::SelectedParticle,
};
use bevy_falling_sand::prelude::ParticleMaterialsParam;
use console::render_console;
use layers_panel::LayersPanel;
use particle_editor::{ParticleEditor, ParticleEditorPlugin};
use particle_editor::{CurrentEditorSelection, LoadParticleIntoEditor, CreateNewParticle, ParticleEditorData};
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
            ParticleEditorPlugin,
        ))
        .add_systems(Update, console::receive_console_line)
        .add_systems(
            EguiContextPass,
            render
                .before(bevy_egui::EguiPreUpdateSet::InitContexts)
                .run_if(in_state(InitializationState::Finished)),
        );
    }
}

fn render(
    mut contexts: EguiContexts,
    mut console_state: ResMut<ConsoleState>,
    cache: Res<ConsoleCache>,
    config: Res<ConsoleConfiguration>,
    mut command_writer: EventWriter<ConsoleCommandEntered>,
    particle_materials: ParticleMaterialsParam,
    selected_particle: Res<SelectedParticle>,
    current_editor: Res<CurrentEditorSelection>,
    mut editor_data_query: Query<&mut ParticleEditorData>,
    mut load_particle_events: EventWriter<LoadParticleIntoEditor>,
    mut create_particle_events: EventWriter<CreateNewParticle>,
) {
    let ctx = contexts.ctx_mut();

    let _top_response = egui::TopBottomPanel::top("Top panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            UiTopBar.render(ui);
        });
    });

    let _left_response = egui::SidePanel::left("Left panel")
        .resizable(false)
        .exact_width(550.0) // Increased to compensate for internal margins
        .show(ctx, |ui| {
            // Remove all margins and padding to use full width
            ui.spacing_mut().indent = 0.0;
            ui.spacing_mut().button_padding = egui::Vec2::ZERO;
            ui.spacing_mut().menu_margin = egui::Margin::ZERO;
            ui.spacing_mut().indent_ends_with_horizontal_line = false;

            // Fill the entire panel with the background color
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, egui::Color32::from_rgb(30, 30, 30));

            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 8.0;

                // Calculate exact 50/50 split
                let total_height = ui.available_height();
                let spacing = 8.0;
                let panel_height = (total_height - spacing) / 2.0;

                // Top half - Particle Editor (exactly 50%)
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(panel_height);
                    ParticleEditor.render(
                        ui, 
                        &particle_materials, 
                        &current_editor, 
                        &mut editor_data_query, 
                        &mut load_particle_events, 
                        &mut create_particle_events
                    );
                });

                // Bottom half - Layers Panel (exactly 50%)
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(panel_height);
                    LayersPanel.render(ui);
                });
            });
        });

    // Use a bottom panel for the console instead of taking up the entire central area
    let console_height = if console_state.expanded {
        console_state.height
    } else {
        40.0
    };

    let _console_response = egui::TopBottomPanel::bottom("Console panel")
        .exact_height(console_height)
        .frame(egui::Frame::NONE) // Let the console handle its own background
        .show(ctx, |ui| {
            render_console(ui, &mut console_state, &cache, &config, &mut command_writer);
        });

    // Don't use a central panel at all - let the canvas area be completely free of egui
}
