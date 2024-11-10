//! UI module.
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{EguiContext, EguiContexts};

use super::*;
use bevy_falling_sand::core::{ParticleType, ClearMapEvent};
use bevy_falling_sand::debug::{DebugParticles, TotalParticleCount};
use bevy_falling_sand::scenes::{LoadSceneEvent, SaveSceneEvent};

/// UI plugin
pub(super) struct UIPlugin;

impl bevy::prelude::Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<AppState>()
            .add_systems(Update, render_ui)
            .add_systems(Update, update_app_state.after(render_ui))
            .init_resource::<CursorCoords>()
            .init_resource::<DebugParticles>()
            .add_systems(First, update_cursor_coordinates)
            .add_systems(OnEnter(AppState::Ui), show_cursor)
            .add_systems(OnEnter(AppState::Canvas), hide_cursor)
            .add_plugins(bevy_inspector_egui::DefaultInspectorConfigPlugin)
            .add_systems(Update, inspector_ui);
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
#[derive(Clone, Resource, Default, Debug)]
pub struct CursorCoords {
    pub previous: Vec2,
    pub current: Vec2,
}


/// UI for particle control mechanics.
pub struct ParticleControlUI;

impl ParticleControlUI {
    /// Renders the particle control UI
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        particle_type_list: &Res<ParticleTypeList>,
        selected_particle: &mut ResMut<SelectedParticle>,
        brush_state: &mut ResMut<NextState<BrushState>>,
        commands: &mut Commands,
    ) {
        ui.vertical(|ui| {
            // Define the fixed order of categories
            let categories = ["Walls", "Solids", "Movable Solids", "Liquids", "Gases"];

            // Iterate through categories in a deterministic order
            for &category in &categories {
                if let Some(particles) = particle_type_list.get(category) {
                    egui::CollapsingHeader::new(category) // Use the category as the header title
                        .default_open(false)
                        .show(ui, |ui| {
                            particles.iter().for_each(|particle_name| {
                                // Create a button for each particle name
                                if ui.button(particle_name).clicked() {
                                    selected_particle.0 = particle_name.clone();
                                    brush_state.set(BrushState::Spawn);
                                }
                            });
                        });
                }
            }

            // Existing UI elements for Remove and Despawn All Particles
            ui.horizontal_wrapped(|ui| {
                if ui.button("Remove").clicked() {
                    brush_state.set(BrushState::Despawn);
                }
            });

            if ui.button("Despawn All Particles").clicked() {
                commands.trigger(ClearMapEvent);
            }
        });
    }
}

/// UI for brush control mechanics.
pub struct BrushControlUI;

impl BrushControlUI {
    /// Renders the brush control UI
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        brush_size: &mut usize,
        max_brush_size: usize,
        ev_brush_resize: &mut EventWriter<BrushResizeEvent>,
        mut current_brush_type: &BrushType,
        next_brush_type: &mut ResMut<NextState<BrushType>>,
    ) {
        egui::ComboBox::from_label("Brush Type")
            .selected_text(format!("{:?}", current_brush_type))
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(&mut current_brush_type, &BrushType::Line, "Line")
                    .changed()
                {
                    next_brush_type.set(BrushType::Line)
                };
                if ui
                    .selectable_value(&mut current_brush_type, &BrushType::Circle, "Circle")
                    .changed()
                {
                    next_brush_type.set(BrushType::Circle)
                };
            });
        if ui
            .add(egui::Slider::new(brush_size, 1..=max_brush_size))
            .changed()
        {
            ev_brush_resize.send(BrushResizeEvent(*brush_size));
        }
    }
}


/// UI for showing `bevy_falling_sand` debug capability.
pub struct DebugUI;

impl DebugUI {
    /// Render the debug UI
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        debug_particles: &Option<Res<DebugParticles>>,
        total_particle_count: u64,
        commands: &mut Commands,
    ) {
        let mut debugging = debug_particles.is_some();
        if ui.checkbox(&mut debugging, "Debug Mode").clicked() {
            if debugging {
                commands.init_resource::<DebugParticles>();
            } else {
                commands.remove_resource::<DebugParticles>();
            }
        }

        if debug_particles.is_some() {
            ui.label(format!("Total Particles: {}", total_particle_count));
        }
    }
}

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
        coords.previous = coords.current;
        coords.current = world_position;
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
    (debug_particles, total_particle_count): (Option<Res<DebugParticles>>, Res<TotalParticleCount>),
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
            DebugUI.render(ui, &debug_particles, total_particle_count.0, &mut commands);
        });
}

fn inspector_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new("UI").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Particle Control");
            bevy_inspector_egui::bevy_inspector::ui_for_world_entities_filtered::<With<ParticleType>>(world, ui, false);
        });
    });
}
