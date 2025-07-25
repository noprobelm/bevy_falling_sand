//! UI module.
use bevy::platform::collections::HashSet;
use bevy::platform::collections::{hash_map::Entry, HashMap};
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::{
        common_conditions::{input_just_pressed, input_pressed},
        keyboard::{Key, KeyboardInput},
        mouse::MouseWheel,
    },
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{egui, EguiContexts};
use bevy_falling_sand::prelude::*;
use std::time::Duration;

use crate::particle_files::{ParticleFileDialog, ParticleFileManagementUI, SaveParticlesEvent, LoadParticlesEvent};
use crate::ui::file_browser::FileBrowserState;
use crate::scenes::*;
use crate::brush::{BrushState, BrushType, BrushResizeEvent, Brush, MaxBrushSize};
use crate::camera::MainCamera;

pub struct UIPlugin;

const DEFAULT_SELECTED_PARTICLE: &str = "Dirt Wall";

impl bevy::prelude::Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_state::<AppState>()
            .init_resource::<CursorCoords>()
            .init_resource::<ParticleList>()
            .init_resource::<ParticleTypeList>()
            .init_resource::<SelectedBrushParticle>()
            .init_resource::<ParticleFileDialog>()
            .init_resource::<ParticleEditorSelectedType>()
            .init_resource::<ParticleEditorName>()
            .init_resource::<ParticleEditorDensity>()
            .init_resource::<ParticleEditorMomentum>()
            .init_resource::<ParticleEditorColors>()
            .init_resource::<ParticleEditorFlowsColor>()
            .init_resource::<ParticleEditorMaxVelocity>()
            .init_resource::<ParticleEditorMovementPriority>()
            .init_resource::<ParticleEditorBurns>()
            .init_resource::<ParticleEditorWall>()
            .init_resource::<ParticleEditorSolid>()
            .init_resource::<ParticleEditorMovableSolid>()
            .init_resource::<ParticleEditorLiquid>()
            .init_resource::<ParticleEditorGas>()
            .init_state::<ParticleEditorCategoryState>()
            .add_event::<ParticleEditorSave>()
            .add_event::<ParticleEditorUpdate>()
            .add_systems(First, update_cursor_position)
            .add_systems(Update, float_dynamic_rigid_bodies)
            .add_systems(
                Update,
                (
                    render_side_panel,
                    render_particle_editor,
                    render_search_bar_ui.run_if(resource_exists::<ParticleSearchBar>),
                    update_particle_type_list,
                    update_app_state.after(render_side_panel),
                    toggle_simulation.run_if(input_just_pressed(KeyCode::Space)),
                    ev_mouse_wheel,
                    handle_search_bar_input,
                    particle_editor_save,
                    update_particle_editor_fields,
                    exit_on_key,
                ),
            )
            .add_systems(
                Update,
                ev_write_step_simulation
                    .run_if(input_just_pressed(KeyCode::KeyP))
                    .before(ParticleSimulationSet),
            )
            .add_systems(OnEnter(AppState::Ui), show_cursor)
            .add_systems(OnEnter(AppState::Canvas), hide_cursor)
            .add_systems(Update, spawn_ball.run_if(input_pressed(KeyCode::KeyB)))
            .add_systems(Update, despawn_balls.run_if(input_pressed(KeyCode::KeyV)));
    }
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Canvas,
    Ui,
}

#[derive(Default, Resource, Clone, Debug)]
pub struct CursorCoords {
    pub current: Vec2,
    pub previous: Vec2,
    pub previous_previous: Vec2,
}

impl CursorCoords {
    pub fn update(&mut self, new_coords: Vec2) {
        self.previous_previous = self.previous;
        self.previous = self.current;
        self.current = new_coords;
    }
}

#[derive(Resource, Default)]
pub struct ParticleTypeList {
    map: HashMap<String, Vec<String>>,
}

impl ParticleTypeList {
    pub fn get(&self, name: &str) -> Option<&Vec<String>> {
        self.map.get(name)
    }

    pub fn insert_or_modify(&mut self, material: String, particles: Vec<String>) {
        match self.map.entry(material) {
            Entry::Occupied(mut entry) => {
                for particle in particles {
                    if !entry.get().contains(&particle) {
                        entry.get_mut().push(particle);
                    }
                }
                entry.get_mut().sort();
            }
            Entry::Vacant(entry) => {
                let mut sorted_particles = particles;
                sorted_particles.sort();
                entry.insert(sorted_particles);
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct ParticleList {
    pub particle_list: HashSet<String>,
}

impl ParticleList {
    pub fn insert(&mut self, value: String) {
        self.particle_list.insert(value);
    }

    fn iter(&self) -> impl Iterator<Item = &String> {
        self.particle_list.iter()
    }
}

#[derive(Resource)]
pub struct SelectedBrushParticle(pub String);

impl Default for SelectedBrushParticle {
    fn default() -> SelectedBrushParticle {
        SelectedBrushParticle(DEFAULT_SELECTED_PARTICLE.to_string())
    }
}

pub struct ParticleControlUI;

impl ParticleControlUI {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        brush_state: &mut ResMut<NextState<BrushState>>,
        ev_clear_dynamic_particles: &mut EventWriter<ClearDynamicParticlesEvent>,
        ev_clear_static_particles: &mut EventWriter<ClearStaticParticlesEvent>,
        ev_clear_particle_map: &mut EventWriter<ClearParticleMapEvent>,
    ) {
        ui.vertical(|ui| {
            // Existing UI elements for Remove and Despawn All Particles
            ui.horizontal_wrapped(|ui| {
                if ui.button("Remove Tool").clicked() {
                    brush_state.set(BrushState::Despawn);
                }
            });

            ui.separator();

            if ui.button("Despawn All Dynamic Particles").clicked() {
                ev_clear_dynamic_particles.write(ClearDynamicParticlesEvent);
            }

            if ui.button("Despawn All Wall Particles").clicked() {
                ev_clear_static_particles.write(ClearStaticParticlesEvent);
            }

            if ui.button("Despawn All Particles").clicked() {
                ev_clear_particle_map.write(ClearParticleMapEvent);
            }
        });
    }
}

pub struct BrushControlUI;

impl BrushControlUI {
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
                if ui
                    .selectable_value(&mut current_brush_type, &BrushType::Cursor, "Cursor")
                    .changed()
                {
                    next_brush_type.set(BrushType::Cursor)
                };
            });
        if ui
            .add(egui::Slider::new(brush_size, 1..=max_brush_size))
            .changed()
        {
            ev_brush_resize.write(BrushResizeEvent(*brush_size));
        }
    }
}

pub struct MovementControlUI;

impl MovementControlUI {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        current_movement_source: &MovementSource,
        next_movement_source: &mut ResMut<NextState<MovementSource>>,
    ) {
        egui::ComboBox::from_label("Particle Movement Logic")
            .selected_text(format!("{:?}", current_movement_source))
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(
                        &mut current_movement_source.clone(),
                        MovementSource::Chunks,
                        "Chunks",
                    )
                    .changed()
                {
                    next_movement_source.set(MovementSource::Chunks);
                }

                if ui
                    .selectable_value(
                        &mut current_movement_source.clone(),
                        MovementSource::Particles,
                        "Particles",
                    )
                    .changed()
                {
                    next_movement_source.set(MovementSource::Particles);
                }
            });
    }
}

pub struct DebugUI;

impl DebugUI {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        debug_hibernating_chunks: &Option<Res<DebugParticleMap>>,
        debug_dirty_rects: &Option<Res<DebugDirtyRects>>,
        debug_particle_count: &Option<Res<DebugParticleCount>>,
        total_particle_count: u64,
        commands: &mut Commands,
    ) {
        let mut show_hibernating = debug_hibernating_chunks.is_some();
        let mut show_dirty_rects = debug_dirty_rects.is_some();
        let mut show_particle_count = debug_particle_count.is_some();
        if ui
            .checkbox(&mut show_hibernating, "Hibernating Chunks")
            .clicked()
        {
            if show_hibernating {
                commands.init_resource::<DebugParticleMap>();
            } else {
                commands.remove_resource::<DebugParticleMap>();
            }
        }

        if ui
            .checkbox(&mut show_dirty_rects, "Dirty Rectangles")
            .clicked()
        {
            if show_dirty_rects {
                commands.init_resource::<DebugDirtyRects>();
            } else {
                commands.remove_resource::<DebugDirtyRects>();
            }
        }

        if ui
            .checkbox(&mut show_particle_count, "Particle Count")
            .clicked()
        {
            if show_particle_count {
                commands.init_resource::<DebugParticleCount>();
            } else {
                commands.remove_resource::<DebugParticleCount>();
            }
        }

        if show_particle_count {
            ui.label(format!("Total Particles: {}", total_particle_count));
        }
    }
}

fn exit_on_key(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
    }
}

fn ev_write_step_simulation(
    app_state: Res<State<AppState>>,
    mut ev_step_simulation: EventWriter<SimulationStepEvent>,
) {
    if app_state.get() == &AppState::Canvas {
        ev_step_simulation.write(SimulationStepEvent);
    }
}

pub fn update_cursor_position(
    mut coords: ResMut<CursorCoords>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Result {
    let (camera, camera_transform) = q_camera.single()?;

    let window = q_window.single()?;
    if let Some(world_position) = window
        .cursor_position()
        .and_then(
            |cursor| -> Option<
                std::result::Result<Ray3d, bevy::render::camera::ViewportConversionError>,
            > { Some(camera.viewport_to_world(camera_transform, cursor)) },
        )
        .map(|ray| ray.unwrap().origin.truncate())
    {
        coords.update(world_position);
    }
    Ok(())
}

pub fn hide_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) -> Result {
    let mut window = primary_window.single_mut()?;
    window.cursor_options.visible = false;
    Ok(())
}

pub fn show_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) -> Result {
    let mut window = primary_window.single_mut()?;
    window.cursor_options.visible = true;
    Ok(())
}

pub fn update_app_state(
    mut contexts: EguiContexts,
    app_state: Res<State<AppState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    particle_search_bar: Option<Res<ParticleSearchBar>>,
) {
    match app_state.get() {
        AppState::Ui => {
            let ctx = contexts.ctx_mut();
            if particle_search_bar.is_none()
                && !keys.pressed(KeyCode::AltLeft)
                && !ctx.is_pointer_over_area()
            {
                next_app_state.set(AppState::Canvas);
            }
        }
        AppState::Canvas => {
            let ctx = contexts.ctx_mut();
            if particle_search_bar.is_some()
                || keys.pressed(KeyCode::AltLeft)
                || ctx.is_pointer_over_area()
            {
                next_app_state.set(AppState::Ui);
            }
        }
    }
}

pub fn render_side_panel(
    mut commands: Commands,
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
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
    (debug_hibernating_chunks, debug_dirty_rects, debug_particle_count, total_particle_count): (
        Option<Res<DebugParticleMap>>,
        Option<Res<DebugDirtyRects>>,
        Option<Res<DebugParticleCount>>,
        Res<TotalParticleCount>,
    ),
    (mut scene_selection_dialog, mut scene_browser_state, mut ev_save_scene, mut ev_load_scene): (
        ResMut<SceneSelectionDialog>,
        ResMut<SceneFileBrowserState>,
        EventWriter<SaveSceneEvent>,
        EventWriter<LoadSceneEvent>,
    ),
    (current_movement_source, mut next_movement_source): (
        Res<State<MovementSource>>,
        ResMut<NextState<MovementSource>>,
    ),
    mut particle_file_dialog: ResMut<ParticleFileDialog>,
    mut file_browser_state: ResMut<FileBrowserState>,
    mut ev_save_particles: EventWriter<SaveParticlesEvent>,
    mut ev_load_particles: EventWriter<LoadParticlesEvent>,
    mut ev_clear_dynamic_particles: EventWriter<ClearDynamicParticlesEvent>,
    mut ev_clear_static_particles: EventWriter<ClearStaticParticlesEvent>,
    mut ev_clear_particle_map: EventWriter<ClearParticleMapEvent>,
) {
    let ctx = contexts.ctx_mut();
    let brush = brush_query.single().expect("No brush found!");
    let mut brush_size = brush.size;

    egui::SidePanel::left("side_panel")
        .resizable(false)
        .show(ctx, |ui| {
            // FPS Counter
            ui.horizontal(|ui| {
                ui.label("FPS:");
                if let Some(fps) = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(|fps| fps.smoothed())
                {
                    ui.label(format!("{:.1}", fps));
                } else {
                    ui.label("--");
                }
            });
            ui.separator();

            SceneManagementUI.render(
                ui,
                &mut scene_selection_dialog,
                &mut scene_browser_state,
                &mut ev_save_scene,
                &mut ev_load_scene,
            );
            ParticleFileManagementUI.render(
                ui,
                &mut particle_file_dialog,
                &mut file_browser_state,
                &mut ev_save_particles,
                &mut ev_load_particles,
            );
            MovementControlUI.render(ui, current_movement_source.get(), &mut next_movement_source);
            BrushControlUI.render(
                ui,
                &mut brush_size,
                max_brush_size.0,
                &mut ev_brush_resize,
                current_brush_type.get(),
                &mut next_brush_type,
            );
            ParticleControlUI.render(
                ui,
                &mut brush_state,
                &mut ev_clear_dynamic_particles,
                &mut ev_clear_static_particles,
                &mut ev_clear_particle_map,
            );
            DebugUI.render(
                ui,
                &debug_hibernating_chunks,
                &debug_dirty_rects,
                &debug_particle_count,
                total_particle_count.0,
                &mut commands,
            );
        });
}

pub fn update_particle_type_list(
    new_particle_query: Query<
        (
            &ParticleType,
            Option<&Wall>,
            Option<&MovableSolid>,
            Option<&Solid>,
            Option<&Liquid>,
            Option<&Gas>,
        ),
        Added<ParticleType>,
    >,
    mut particle_list: ResMut<ParticleList>,
    mut particle_type_list: ResMut<ParticleTypeList>,
) {
    new_particle_query.iter().for_each(
        |(particle_type, wall, movable_solid, solid, liquid, gas)| {
            // Add the particle type name to the particle_list
            particle_list.insert(particle_type.name.to_string());

            // Check for the presence of each optional component and update particle_type_list accordingly
            if wall.is_some() {
                particle_type_list
                    .insert_or_modify("Walls".to_string(), vec![particle_type.name.to_string()]);
            }
            if movable_solid.is_some() {
                particle_type_list.insert_or_modify(
                    "Movable Solids".to_string(),
                    vec![particle_type.name.to_string()],
                );
            }
            if solid.is_some() {
                particle_type_list
                    .insert_or_modify("Solids".to_string(), vec![particle_type.name.to_string()]);
            }
            if liquid.is_some() {
                particle_type_list
                    .insert_or_modify("Liquids".to_string(), vec![particle_type.name.to_string()]);
            }
            if gas.is_some() {
                particle_type_list
                    .insert_or_modify("Gases".to_string(), vec![particle_type.name.to_string()]);
            }
        },
    );
}

pub fn toggle_simulation(
    mut commands: Commands,
    simulation_pause: Option<Res<ParticleSimulationRun>>,
    app_state: Res<State<AppState>>,
    mut time: ResMut<Time<Virtual>>,
) {
    if app_state.get() == &AppState::Canvas {
        if simulation_pause.is_some() {
            commands.remove_resource::<ParticleSimulationRun>();
        } else {
            commands.init_resource::<ParticleSimulationRun>();
        }
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
    }
}

pub fn ev_mouse_wheel(
    mut ev_scroll: EventReader<MouseWheel>,
    app_state: Res<State<AppState>>,
    mut camera_query: Query<&mut Projection, With<MainCamera>>,
    mut brush_query: Query<&mut Brush>,
    max_brush_size: Res<MaxBrushSize>,
) {
    const ZOOM_IN_FACTOR: f32 = 0.9;
    const ZOOM_OUT_FACTOR: f32 = 1.1;

    if !ev_scroll.is_empty() {
        match app_state.get() {
            AppState::Ui => {
                let mut brush = brush_query.single_mut().expect("No brush found!");
                ev_scroll.read().for_each(|ev| {
                    if ev.y < 0. && 1 <= brush.size.wrapping_sub(1) {
                        brush.size -= 1;
                    } else if ev.y > 0. && brush.size.wrapping_add(1) <= max_brush_size.0 {
                        brush.size += 1;
                    }
                });
            }
            AppState::Canvas => {
                let mut projection = match camera_query.single_mut() {
                    Ok(p) => p,
                    Err(_) => return,
                };
                let Projection::Orthographic(orthographic) = projection.as_mut() else {
                    return;
                };
                ev_scroll.read().for_each(|ev| {
                    if ev.y < 0. {
                        orthographic.scale *= ZOOM_OUT_FACTOR;
                    } else if ev.y > 0. {
                        orthographic.scale *= ZOOM_IN_FACTOR;
                    }
                });
            }
        };
    }
}

#[derive(Resource, Default)]
pub struct ParticleSearchBar {
    pub input: String,
    pub filtered_results: Vec<String>,
    pub selected_index: Option<usize>,
}

pub fn handle_search_bar_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut char_input_events: EventReader<KeyboardInput>,
    mut commands: Commands,
    particle_type_list: Res<ParticleTypeList>,
    particle_search_bar: Option<ResMut<ParticleSearchBar>>,
) {
    if keys.just_pressed(KeyCode::KeyN) && particle_search_bar.is_none() {
        commands.insert_resource(ParticleSearchBar::default());
        return;
    }

    let mut particle_search_bar = match particle_search_bar {
        Some(state) => state,
        None => return,
    };

    if keys.just_pressed(KeyCode::Escape) {
        commands.remove_resource::<ParticleSearchBar>();
        return;
    }

    for ev in char_input_events.read() {
        match &ev.logical_key {
            Key::Character(ch) if ev.state.is_pressed() => {
                particle_search_bar.input.push_str(ch.as_str());
            }
            Key::Backspace if ev.state.is_pressed() => {
                particle_search_bar.input.pop();
            }
            Key::Space if ev.state.is_pressed() => {
                particle_search_bar.input.push(' ');
            }
            _ => {}
        }
    }

    let old_filtered_results = particle_search_bar.filtered_results.clone();
    particle_search_bar.filtered_results = particle_type_list
        .map
        .values()
        .flat_map(|particles| particles.clone())
        .filter(|particle| {
            particle
                .to_lowercase()
                .contains(&particle_search_bar.input.to_lowercase())
        })
        .collect();

    if particle_search_bar.filtered_results != old_filtered_results {
        particle_search_bar.selected_index =
            particle_search_bar.filtered_results.first().map(|_| 0);
    }

    if keys.just_pressed(KeyCode::ArrowUp) {
        if let Some(index) = particle_search_bar.selected_index {
            if index > 0 {
                particle_search_bar.selected_index = Some(index - 1);
            }
        } else {
            particle_search_bar.selected_index =
                particle_search_bar.filtered_results.len().checked_sub(1);
        }
    }

    if keys.just_pressed(KeyCode::ArrowDown) {
        if let Some(index) = particle_search_bar.selected_index {
            if index + 1 < particle_search_bar.filtered_results.len() {
                particle_search_bar.selected_index = Some(index + 1);
            }
        } else if !particle_search_bar.filtered_results.is_empty() {
            particle_search_bar.selected_index = Some(0);
        }
    }
}

pub fn render_search_bar_ui(
    mut contexts: EguiContexts,
    mut particle_search_bar: ResMut<ParticleSearchBar>,
    mut commands: Commands,
    mut selected_particle: ResMut<SelectedBrushParticle>,
    mut brush_state: ResMut<NextState<BrushState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let ctx = contexts.ctx_mut();
    let mut should_close = false;

    egui::Window::new("Search Particles")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.text_edit_singleline(&mut particle_search_bar.input);

            let mut new_selected_index = particle_search_bar.selected_index;

            ui.separator();
            for (i, particle) in particle_search_bar.filtered_results.iter().enumerate() {
                let is_selected = Some(i) == particle_search_bar.selected_index;

                if ui.selectable_label(is_selected, particle).clicked() {
                    new_selected_index = Some(i);
                }
            }

            if keys.just_pressed(KeyCode::Enter) {
                if let Some(selected_index) = particle_search_bar.selected_index {
                    if let Some(selected_particle_name) =
                        particle_search_bar.filtered_results.get(selected_index)
                    {
                        if selected_particle.0 == *selected_particle_name {
                            should_close = true;
                        } else {
                            selected_particle.0 = selected_particle_name.clone();
                            brush_state.set(BrushState::Spawn);
                            should_close = true;
                        }
                    }
                }
            }

            particle_search_bar.selected_index = new_selected_index;
        });

    if should_close {
        commands.remove_resource::<ParticleSearchBar>();
    }
}

pub fn update_particle_editor_fields(
    mut ev_particle_editor_update: EventReader<ParticleEditorUpdate>,
    mut particle_editor_selected_type: ResMut<ParticleEditorSelectedType>,
    mut particle_editor_name: ResMut<ParticleEditorName>,
    particle_type_map: Res<ParticleTypeMap>,
    particle_query: Query<
        (
            Option<&Density>,
            Option<&Velocity>,
            Option<&Momentum>,
            Option<&ColorProfile>,
            Option<&Burns>,
            Option<&Wall>,
            Option<&Liquid>,
            Option<&Solid>,
            Option<&MovableSolid>,
            Option<&Gas>,
        ),
        With<ParticleType>,
    >,
    mut particle_density_field: ResMut<ParticleEditorDensity>,
    mut particle_max_velocity_field: ResMut<ParticleEditorMaxVelocity>,
    mut particle_momentum_field: ResMut<ParticleEditorMomentum>,
    mut particle_colors_field: ResMut<ParticleEditorColors>,
    mut particle_editor_burns_field: ResMut<ParticleEditorBurns>,
    mut next_particle_category_field: ResMut<NextState<ParticleEditorCategoryState>>,
) {
    ev_particle_editor_update.read().for_each(|_| {
        if let Some(entity) = particle_type_map.get(&particle_editor_selected_type.0.name) {
            if let Ok((
                density,
                velocity,
                momentum,
                colors,
                burns,
                wall,
                liquid,
                solid,
                movable_solid,
                gas,
            )) = particle_query.get(*entity)
            {
                particle_editor_name.0 = particle_editor_selected_type.0.name.to_string();
                particle_editor_selected_type.0 =
                    ParticleType::from_string(particle_editor_selected_type.0.name.to_string());
                if let Some(density) = density {
                    particle_density_field.blueprint = *density;
                }
                if let Some(velocity) = velocity {
                    particle_max_velocity_field.blueprint = *velocity;
                }
                if momentum.is_some() {
                    particle_momentum_field.enable = true;
                }
                if let Some(colors) = colors {
                    particle_colors_field.blueprint = colors.clone()
                }
                if let Some(burns) = burns {
                    particle_editor_burns_field.enable = true;
                    particle_editor_burns_field.chance_destroy_enable =
                        burns.chance_destroy_per_tick.map(|_| true).unwrap_or(false);
                    particle_editor_burns_field.reaction_enable =
                        burns.reaction.as_ref().map(|_| true).unwrap_or(false);
                    particle_editor_burns_field.color_enable =
                        burns.color.as_ref().map(|_| true).unwrap_or(false);
                    particle_editor_burns_field.spreads_enable =
                        burns.spreads.as_ref().map(|_| true).unwrap_or(false);
                    particle_editor_burns_field.blueprint = burns.clone();
                } else {
                    (
                        particle_editor_burns_field.enable,
                        particle_editor_burns_field.chance_destroy_enable,
                        particle_editor_burns_field.reaction_enable,
                        particle_editor_burns_field.color_enable,
                        particle_editor_burns_field.spreads_enable,
                    ) = (false, false, false, false, false);
                }
                if let Some(_) = wall {
                    next_particle_category_field.set(ParticleEditorCategoryState::Wall)
                }
                if let Some(_) = solid {
                    next_particle_category_field.set(ParticleEditorCategoryState::Solid)
                }
                if let Some(_) = movable_solid {
                    next_particle_category_field.set(ParticleEditorCategoryState::MovableSolid)
                }
                if let Some(_) = liquid {
                    next_particle_category_field.set(ParticleEditorCategoryState::Liquid)
                }
                if let Some(_) = gas {
                    next_particle_category_field.set(ParticleEditorCategoryState::Gas)
                }
            };
        }
    });
}

pub fn render_particle_editor(
    (mut ev_particle_editor_save, mut ev_particle_editor_update, mut contexts): (
        EventWriter<ParticleEditorSave>,
        EventWriter<ParticleEditorUpdate>,
        EguiContexts,
    ),
    particle_type_list: Res<ParticleTypeList>,
    particle_list: Res<ParticleList>,
    current_particle_category_field: Res<State<ParticleEditorCategoryState>>,
    mut next_particle_category_field: ResMut<NextState<ParticleEditorCategoryState>>,
    mut selected_brush_particle: ResMut<SelectedBrushParticle>,
    (
        mut particle_editor_liquid_field,
        mut particle_editor_gas_field,
        mut particle_editor_burns_field,
        mut particle_editor_movement_priority_field,
        mut particle_editor_colors_field,
        mut particle_editor_flows_color_field,
        mut particle_momentum_field,
        mut particle_editor_max_velocity_field,
        mut particle_density_field,
        mut particle_editor_selected_field,
    ): (
        ResMut<ParticleEditorLiquid>,
        ResMut<ParticleEditorGas>,
        ResMut<ParticleEditorBurns>,
        ResMut<ParticleEditorMovementPriority>,  
        ResMut<ParticleEditorColors>,
        ResMut<ParticleEditorFlowsColor>,
        ResMut<ParticleEditorMomentum>,
        ResMut<ParticleEditorMaxVelocity>,
        ResMut<ParticleEditorDensity>,
        ResMut<ParticleEditorSelectedType>,
    ),
) {
    // Full particle editor implementation would go here
    // For sandbox, we may not need the full editor, but keeping the structure
}

// Additional stub implementations for the particle editor resources
#[derive(Event, Clone, Debug)]
pub struct ParticleEditorUpdate;

#[derive(Event, Clone, Debug)]
pub struct ParticleEditorSave {
    create_new: bool,
}

#[derive(Resource, Clone)]
pub struct ParticleEditorSelectedType(pub ParticleType);

#[derive(Resource, Default, Clone)]
pub struct ParticleEditorName(pub String);

impl Default for ParticleEditorSelectedType {
    fn default() -> Self {
        ParticleEditorSelectedType(ParticleType::new(DEFAULT_SELECTED_PARTICLE))
    }
}

#[derive(Default, Resource, Clone)]
pub struct ParticleEditorDensity {
    blueprint: Density,
}

#[derive(Resource, Clone)]
pub struct ParticleEditorMaxVelocity {
    blueprint: Velocity,
}

impl Default for ParticleEditorMaxVelocity {
    fn default() -> Self {
        ParticleEditorMaxVelocity {
            blueprint: Velocity::new(1, 3),
        }
    }
}

#[derive(Default, Resource, Clone)]
pub struct ParticleEditorMomentum {
    enable: bool,
    blueprint: Momentum,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct ParticleEditorColors {
    blueprint: ColorProfile,
}

#[derive(Resource, Clone, Debug)]
pub struct ParticleEditorFlowsColor {
    enable: bool,
    blueprint: ChangesColor,
}

impl Default for ParticleEditorFlowsColor {
    fn default() -> Self {
        ParticleEditorFlowsColor {
            enable: true,
            blueprint: ChangesColor::new(0.1),
        }
    }
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ParticleEditorCategoryState {
    #[default]
    Wall,
    Solid,
    MovableSolid,
    Liquid,
    Gas,
    Other,
}

impl ParticleEditorCategoryState {
    pub fn as_str(&self) -> &str {
        match self {
            ParticleEditorCategoryState::Wall => "Wall",
            ParticleEditorCategoryState::Solid => "Solid",
            ParticleEditorCategoryState::MovableSolid => "Movable Solid",
            ParticleEditorCategoryState::Liquid => "Liquid",
            ParticleEditorCategoryState::Gas => "Gas",
            ParticleEditorCategoryState::Other => "Other",
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct ParticleEditorMovementPriority {
    blueprint: Movement,
}

impl Default for ParticleEditorMovementPriority {
    fn default() -> Self {
        ParticleEditorMovementPriority {
            blueprint: Movement::empty(),
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct ParticleEditorBurns {
    duration_str: String,
    tick_rate_str: String,
    enable: bool,
    chance_destroy_enable: bool,
    reaction_enable: bool,
    color_enable: bool,
    spreads_enable: bool,
    spawns_on_fire: bool,
    blueprint: Burns,
}

impl Default for ParticleEditorBurns {
    fn default() -> Self {
        let duration_str = Duration::default().as_millis().to_string();
        let tick_rate_str = duration_str.clone();
        ParticleEditorBurns {
            duration_str,
            tick_rate_str,
            enable: false,
            chance_destroy_enable: false,
            reaction_enable: false,
            color_enable: false,
            spreads_enable: false,
            spawns_on_fire: false,
            blueprint: Burns::default(),
        }
    }
}

#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleEditorWall {
    blueprint: Wall,
}

#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleEditorSolid {
    blueprint: Solid,
}

#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleEditorMovableSolid {
    blueprint: MovableSolid,
}

#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleEditorLiquid {
    blueprint: Liquid,
}

#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleEditorGas {
    blueprint: Gas,
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cursor_coords: Res<CursorCoords>,
    brush_query: Query<&Brush>,
) -> Result {
    let brush = brush_query.single()?;
    commands.spawn((
        RigidBody::Dynamic,
        Collider::circle(brush.size as f32),
        Transform::from_xyz(cursor_coords.current.x, cursor_coords.current.y, 0.),
        DemoBall {
            size: brush.size as f32,
        },
        TransformInterpolation,
        GravityScale(1.0),
        Mesh2d(meshes.add(Circle::new(brush.size as f32))),
        MeshMaterial2d(materials.add(Color::Srgba(Srgba::rgba_u8(246, 174, 45, 255)))),
    ));
    Ok(())
}

fn float_dynamic_rigid_bodies(
    mut rigid_body_query: Query<(
        &RigidBody,
        &Transform,
        &mut GravityScale,
        &mut LinearVelocity,
    )>,
    liquid_query: Query<&Particle, With<Liquid>>,
    chunk_map: Res<ParticleMap>,
) {
    let damping_factor = 0.95;
    rigid_body_query.iter_mut().for_each(
        |(rigid_body, transform, mut gravity_scale, mut linear_velocity)| {
            if rigid_body == &RigidBody::Dynamic {
                if let Some(entity) = chunk_map.get(&IVec2::new(
                    transform.translation.x as i32,
                    transform.translation.y as i32,
                )) {
                    if liquid_query.contains(*entity) {
                        linear_velocity.y *= damping_factor;
                        if linear_velocity.y.abs() < 0.001 {
                            linear_velocity.y = 0.0;
                        }
                        gravity_scale.0 = -1.0;
                    }
                } else {
                    gravity_scale.0 = 1.0;
                }
            }
        },
    );
}

fn despawn_balls(mut commands: Commands, ball_query: Query<Entity, With<DemoBall>>) {
    ball_query.iter().for_each(|entity| {
        commands.entity(entity).despawn();
    });
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Default, Component)]
pub struct DemoBall {
    pub size: f32,
}

fn particle_editor_save(
    mut _ev_particle_editor_save: EventReader<ParticleEditorSave>
) {
    // Particle editor save implementation would go here if needed for sandbox
}