mod console;
pub mod file_browser;
mod overlays;
mod particle_editor;
pub mod particle_search;
mod quick_actions;
mod statistics_panel;
mod top_bar;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy_falling_sand::prelude::{
    ActiveParticleCount, DynamicParticleCount, LoadSceneEvent, MovementSource, ParticleTypeMap,
    ParticleTypeMaterialsParam, ResetParticleChildrenEvent, SaveSceneEvent, TotalParticleCount,
    WallParticleCount,
};
use console::core::{ConsoleCache, ConsoleCommandEntered, ConsoleConfiguration};
use console::{Console, ConsolePlugin};
use quick_actions::*;

use crate::scenes::{SceneFileBrowserState, SceneManagementUI};
use crate::ui::file_browser::FileBrowserState;
pub use console::core::ConsoleState;
use overlays::OverlaysPlugin;
use particle_editor::{
    ApplyEditorChanges, ApplyEditorChangesAndReset, CreateNewParticle, CurrentEditorSelection,
    LoadParticleIntoEditor, ParticleEditorData,
};
use particle_editor::{ParticleEditor, ParticleEditorPlugin};
use particle_search::{
    handle_particle_search_input, update_particle_search_cache, ParticleSearch,
    ParticleSearchCache, ParticleSearchState,
};
use statistics_panel::StatisticsPanel;
pub use top_bar::particle_files::ParticleFileDialog;
use top_bar::particle_files::{
    LoadParticlesSceneEvent, ParticleFileBrowser, SaveParticlesSceneEvent,
};
use top_bar::{ParticleFilesPlugin, UiTopBar};

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
            FrameTimeDiagnosticsPlugin::default(),
            QuickActionsPlugin,
            OverlaysPlugin,
        ))
        .init_resource::<RenderGui>()
        .init_resource::<ParticleSearchState>()
        .init_resource::<ParticleSearchCache>()
        .init_resource::<StatisticsPanel>()
        .add_systems(
            Update,
            (console::receive_console_line, update_particle_search_cache),
        )
        .add_systems(
            EguiContextPass,
            (
                render_ui_panels,
                render_particle_search,
                handle_particle_search_input,
            )
                .run_if(resource_exists::<RenderGui>),
        );
    }
}

#[derive(Resource, Clone, Default, Debug)]
pub struct RenderGui;

type UiSystemParams<'w, 's> = (
    Commands<'w, 's>,
    EguiContexts<'w, 's>,
    ResMut<'w, console::core::ConsoleState>,
    Res<'w, ConsoleCache>,
    Res<'w, ConsoleConfiguration>,
    EventWriter<'w, ConsoleCommandEntered>,
    ParticleTypeMaterialsParam<'w, 's>,
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
    statistics_panel: Res<StatisticsPanel>,
    dynamic_particle_count: Res<DynamicParticleCount>,
    wall_particle_count: Res<WallParticleCount>,
    total_particle_count: Res<TotalParticleCount>,
    active_particle_count: Res<ActiveParticleCount>,
    diagnostics: Res<DiagnosticsStore>,
    mut scene_browser_state: ResMut<SceneFileBrowserState>,
    mut ev_save_scene: EventWriter<SaveSceneEvent>,
    mut ev_load_scene: EventWriter<LoadSceneEvent>,
    mut particle_file_browser_state: ResMut<FileBrowserState>,
    mut ev_save_particles_scene: EventWriter<SaveParticlesSceneEvent>,
    mut ev_load_particles_scene: EventWriter<LoadParticlesSceneEvent>,
    particle_movement_state_current: Res<State<MovementSource>>,
    particle_search_cache: Res<ParticleSearchCache>,
) {
    let ctx = contexts.ctx_mut();

    let _top_response = egui::TopBottomPanel::top("Top panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            UiTopBar.render(ui, &mut commands, &mut particle_file_browser_state);

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

    // Calculate responsive panel width
    let screen_width = ctx.screen_rect().width();
    let panel_width = if screen_width < 1200.0 {
        (screen_width * 0.35).max(300.0).min(500.0)
    } else if screen_width < 1920.0 {
        (screen_width * 0.4).max(400.0).min(650.0)
    } else {
        (screen_width * 0.36).max(500.0).min(800.0)
    };

    let _left_response = egui::SidePanel::left("Left panel")
        .resizable(true)
        .width_range(300.0..=panel_width * 1.2)
        .default_width(panel_width)
        .show(ctx, |ui| {
            // Use TopBottomPanel pattern within the side panel for proper layout
            egui::TopBottomPanel::bottom("Statistics panel")
                .resizable(true)
                .height_range(200.0..=ui.available_height() * 0.6)
                .default_height(ui.available_height() * 0.4)
                .show_inside(ui, |ui| {
                    ui.heading("Statistics");
                    ui.separator();
                    
                    let fps = diagnostics
                        .get(&FrameTimeDiagnosticsPlugin::FPS)
                        .and_then(|fps| fps.smoothed())
                        .unwrap_or(0.0) as f32;

                    statistics_panel.as_ref().render(
                        ui,
                        particle_movement_state_current.get(),
                        fps,
                        dynamic_particle_count.0 as u32,
                        wall_particle_count.0 as u32,
                        total_particle_count.0 as u32,
                        active_particle_count.0 as u32,
                    );
                });

            // The remaining space is automatically used for the particle editor
            egui::CentralPanel::default()
                .show_inside(ui, |ui| {
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
        });

    let screen_height = ctx.screen_rect().height();
    let console_height = if console_state.expanded {
        console_state.height.min(screen_height * 0.5).max(80.0)
    } else {
        (screen_height * 0.04).max(30.0).min(50.0)
    };

    let _console_response = egui::TopBottomPanel::bottom("Console panel")
        .exact_height(console_height)
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            Console.render(
                ui,
                &mut console_state,
                &cache,
                &config,
                &mut command_writer,
                Some(&particle_search_cache),
            );
        });

    SceneManagementUI.render(
        &mut egui::Ui::new(
            ctx.clone(),
            egui::Id::new("scene_management"),
            egui::UiBuilder::new().max_rect(ctx.screen_rect()),
        ),
        &mut scene_browser_state,
        &mut ev_save_scene,
        &mut ev_load_scene,
    );

    ParticleFileBrowser.render(
        &mut egui::Ui::new(
            ctx.clone(),
            egui::Id::new("particle_file_browser"),
            egui::UiBuilder::new().max_rect(ctx.screen_rect()),
        ),
        &mut particle_file_browser_state,
        &mut ev_save_particles_scene,
        &mut ev_load_particles_scene,
    );
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
