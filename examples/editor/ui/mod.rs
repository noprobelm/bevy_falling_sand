mod console;
mod layers_panel;
mod particle_editor;
mod top_bar;

use crate::{
    app_state::InitializationState,
};
use bevy_falling_sand::prelude::{
    ParticleMaterialsParam, ParticleTypeMap, ResetParticleChildrenEvent,
};
use console::{Console, ConsolePlugin};
use console::core::{ConsoleCache, ConsoleCommandEntered, ConsoleConfiguration, ConsoleState};
use layers_panel::LayersPanel;
use particle_editor::{
    ApplyEditorChanges, ApplyEditorChangesAndReset, CreateNewParticle, CurrentEditorSelection,
    LoadParticleIntoEditor, ParticleEditorData,
};
use particle_editor::{ParticleEditor, ParticleEditorPlugin};
use top_bar::{UiTopBar, ParticleFilesPlugin};
pub use top_bar::particle_files::ParticleFileDialog;

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
            ParticleFilesPlugin,
        ))
        .add_systems(Update, console::receive_console_line)
        .add_systems(
            EguiContextPass,
            (render_ui_panels).run_if(in_state(InitializationState::Finished)),
        );
    }
}

fn render_ui_panels(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut console_state: ResMut<ConsoleState>,
    cache: Res<ConsoleCache>,
    config: Res<ConsoleConfiguration>,
    mut command_writer: EventWriter<ConsoleCommandEntered>,
    particle_materials: ParticleMaterialsParam,
    current_editor: Res<CurrentEditorSelection>,
    mut editor_data_query: Query<&mut ParticleEditorData>,
    mut load_particle_events: EventWriter<LoadParticleIntoEditor>,
    mut create_particle_events: EventWriter<CreateNewParticle>,
    mut apply_editor_events: EventWriter<ApplyEditorChanges>,
    mut apply_editor_and_reset_events: EventWriter<ApplyEditorChangesAndReset>,
    mut reset_particle_children_events: EventWriter<ResetParticleChildrenEvent>,
    particle_type_map: Res<ParticleTypeMap>,
    particle_file_dialog: Res<ParticleFileDialog>,
) {
    let ctx = contexts.ctx_mut();

    // All egui panels must be declared in the same context to coordinate layout properly
    // Order matters: Top -> Side -> Bottom to avoid overlaps
    
    // Top panel - must be declared first
    let _top_response = egui::TopBottomPanel::top("Top panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            UiTopBar.render(ui, &mut commands);
            
            // Show particle file status messages
            if let Some(ref error) = particle_file_dialog.last_error {
                ui.separator();
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            }
            
            if let Some(ref success) = particle_file_dialog.last_success {
                ui.separator();
                ui.colored_label(egui::Color32::GREEN, success);
            }
        });
    });
    
    // Side panel - must be declared before bottom panel to avoid overlap
    let _left_response = egui::SidePanel::left("Left panel")
        .resizable(false)
        .exact_width(700.0)
        .show(ctx, |ui| {
            ui.spacing_mut().indent = 0.0;
            ui.spacing_mut().button_padding = egui::Vec2::ZERO;
            ui.spacing_mut().menu_margin = egui::Margin::ZERO;
            ui.spacing_mut().indent_ends_with_horizontal_line = false;

            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, egui::Color32::from_rgb(30, 30, 30));

            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 8.0;

                let total_height = ui.available_height();
                let spacing = 8.0;
                let panel_height = (total_height - spacing) / 2.0;

                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(panel_height);
                    ParticleEditor.render(
                        ui,
                        &particle_materials,
                        &current_editor,
                        &mut editor_data_query,
                        &mut load_particle_events,
                        &mut create_particle_events,
                        &mut apply_editor_events,
                        &mut apply_editor_and_reset_events,
                        &mut reset_particle_children_events,
                        &particle_type_map,
                    );
                });

                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(panel_height);
                    LayersPanel.render(ui);
                });
            });
        });

    // Bottom panel - must be declared last
    let console_height = if console_state.expanded {
        console_state.height
    } else {
        40.0
    };

    let _console_response = egui::TopBottomPanel::bottom("Console panel")
        .exact_height(console_height)
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            Console.render(ui, &mut console_state, &cache, &config, &mut command_writer);
        });
}

