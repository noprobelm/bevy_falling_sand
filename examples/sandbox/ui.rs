//! UI module.
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;

use super::*;
use bevy_falling_sand::{
    DebugParticles, DynamicParticleCount, LoadSceneEvent, SaveSceneEvent, TotalParticleCount,
};

/// UI plugin
pub(super) struct UIPlugin;

impl bevy::prelude::Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<AppState>()
            .add_systems(Update, render_ui)
            .add_systems(Update, update_app_state.after(render_ui))
            .init_resource::<CursorCoords>()
            .add_systems(Update, update_cursor_coordinates)
            .add_systems(OnEnter(AppState::Ui), show_cursor)
            .add_systems(OnEnter(AppState::Canvas), hide_cursor)
            .add_systems(OnEnter(AppState::Ui), show_cursor)
            .add_systems(OnEnter(AppState::Canvas), hide_cursor);
    }
}

/// When in Canvas mode, the brush renders and the cursor disappears.
/// When in Ui mode, canvas control mechanisms (zoom/pan camera) and the brush are disabled. Cursor is enabled.
#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    /// Canvas mode.
    Canvas,
    /// Ui mode.
    Ui,
}

/// Resource for tracking cursor coordinates.
#[derive(Resource, Default, Debug)]
pub struct CursorCoords(pub Vec2);

/// Updates the cursor coordinates each frame.
pub fn update_cursor_coordinates(
    mut coords: ResMut<CursorCoords>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();

    let window = q_window.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        coords.0 = world_position;
    }
}

/// Hides the cursor.
pub fn hide_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    let window = &mut primary_window.single_mut();
    window.cursor.visible = false;
}

/// Shows the cursor.
pub fn show_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    let window = &mut primary_window.single_mut();
    window.cursor.visible = true;
}

/// Updates the app state depending on whether we're focused on the GUI or the canvas.
pub fn update_app_state(
    mut contexts: EguiContexts,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let ctx = contexts.ctx_mut();
    match app_state.get() {
        AppState::Ui => {
            if !ctx.is_pointer_over_area() {
                next_app_state.set(AppState::Canvas);
            }
        }
        AppState::Canvas => {
            if ctx.is_pointer_over_area() {
                next_app_state.set(AppState::Ui);
            }
        }
    }
}

/// Bring it all together in the UI.
/// This system basically pulls types from all modules in this example and assembles them into a side panel.
pub fn render_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    (
        mut brush_state,
        brush_query,
        current_brush_type,
        mut next_brush_type,
        mut ev_brush_resize,
        max_brush_size,
    ): (
        ResMut<NextState<BrushState>>,
        Query<&Brush>,
        Res<State<BrushType>>,
        ResMut<NextState<BrushType>>,
        EventWriter<BrushResizeEvent>,
        Res<MaxBrushSize>,
    ),
    (debug_particles, dynamic_particle_count, total_particle_count): (
        Option<Res<DebugParticles>>,
        Res<DynamicParticleCount>,
        Res<TotalParticleCount>,
    ),
    (mut selected_particle, particle_type_list): (ResMut<SelectedParticle>, Res<ParticleTypeList>),
    (mut scene_selection_dialog, mut scene_path, mut ev_save_scene, mut ev_load_scene): (
        ResMut<SceneSelectionDialog>,
        ResMut<ParticleSceneFilePath>,
        EventWriter<SaveSceneEvent>,
        EventWriter<LoadSceneEvent>,
    ),
) {
    let ctx = contexts.ctx_mut();
    let brush = brush_query.single();
    let mut brush_size = brush.size;

    egui::SidePanel::left("side_panel")
        .exact_width(200.0)
        .resizable(false)
        .show(ctx, |ui| {
            SceneManagementUI.render(
                ui,
                &mut scene_selection_dialog,
                &mut scene_path,
                &mut ev_save_scene,
                &mut ev_load_scene,
            );
            BrushControlUI.render(
                ui,
                &mut brush_size,
                max_brush_size.0,
                &mut ev_brush_resize,
                &current_brush_type.get(),
                &mut next_brush_type,
            );
            ParticleControlUI.render(
                ui,
                &particle_type_list,
                &mut selected_particle,
                &mut brush_state,
                &mut commands,
            );
            DebugUI.render(
                ui,
                &debug_particles,
                dynamic_particle_count.0,
                total_particle_count.0,
                &mut commands,
            );
        });
}
