mod console;
mod layers_panel;
mod particle_editor;
pub mod particle_search;
mod top_bar;

use bevy_falling_sand::prelude::{
    ParticleMaterialsParam, ParticleTypeMap, ResetParticleChildrenEvent,
};
use console::{Console, ConsolePlugin};
use console::core::{ConsoleCache, ConsoleCommandEntered, ConsoleConfiguration};

// Re-export for external modules
pub use console::core::ConsoleState;
use layers_panel::LayersPanel;
use particle_editor::{
    ApplyEditorChanges, ApplyEditorChangesAndReset, CreateNewParticle, CurrentEditorSelection,
    LoadParticleIntoEditor, ParticleEditorData,
};
use particle_editor::{ParticleEditor, ParticleEditorPlugin};
use particle_search::{ParticleSearch, ParticleSearchState, ParticleSearchCache, update_particle_search_cache, handle_particle_search_input};
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
        .init_resource::<ParticleSearchState>()
        .init_resource::<ParticleSearchCache>()
        .add_systems(Update, (
            console::receive_console_line,
            update_particle_search_cache,
        ))
        .add_systems(
            EguiContextPass,
            (render_ui_panels, render_particle_search, handle_particle_search_input),
        );
    }
}

type UiSystemParams<'w, 's> = (
    Commands<'w, 's>,
    EguiContexts<'w, 's>,
    ResMut<'w, console::core::ConsoleState>,
    Res<'w, ConsoleCache>,
    Res<'w, ConsoleConfiguration>,
    EventWriter<'w, ConsoleCommandEntered>,
    ParticleMaterialsParam<'w, 's>,
    Res<'w, CurrentEditorSelection>,
    Query<'w, 's, &'static mut ParticleEditorData>,
    EventWriter<'w, LoadParticleIntoEditor>,
    EventWriter<'w, CreateNewParticle>,
    EventWriter<'w, ApplyEditorChanges>,
    EventWriter<'w, ApplyEditorChangesAndReset>,
    EventWriter<'w, ResetParticleChildrenEvent>,
    Res<'w, ParticleTypeMap>,
    Res<'w, ParticleFileDialog>,
);

fn render_ui_panels(
    (
        mut commands,
        mut contexts,
        mut console_state,
        cache,
        config,
        mut command_writer,
        particle_materials,
        current_editor,
        mut editor_data_query,
        mut load_particle_events,
        mut create_particle_events,
        mut apply_editor_events,
        mut apply_editor_and_reset_events,
        mut reset_particle_children_events,
        particle_type_map,
        particle_file_dialog,
    ): UiSystemParams,
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

type ParticleSearchParams<'w, 's> = (
    EguiContexts<'w, 's>,
    ResMut<'w, ParticleSearchState>,
    Res<'w, ParticleSearchCache>,
    EventWriter<'w, LoadParticleIntoEditor>,
);

fn render_particle_search(
    (
        mut contexts,
        mut particle_search_state,
        particle_search_cache,
        mut load_particle_events,
    ): ParticleSearchParams,
) {
    let ctx = contexts.ctx_mut();
    
    // Particle search overlay (rendered after panels to appear on top)
    let mut particle_search_ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new("particle_search"),
        egui::UiBuilder::new().max_rect(ctx.screen_rect()),
    );
    
    ParticleSearch.render(
        &mut particle_search_ui,
        &mut particle_search_state,
        &particle_search_cache,
        &mut load_particle_events,
    );
}

