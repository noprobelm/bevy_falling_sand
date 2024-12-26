//! UI module.
use bevy::{
    input::{
        common_conditions::input_just_pressed,
        keyboard::{Key, KeyboardInput},
        mouse::MouseWheel,
    },
    prelude::*,
    utils::{Duration, Entry, HashMap},
    window::PrimaryWindow,
};
use bevy_egui::EguiContexts;
use bfs_internal::{
    reactions::{BurnsBlueprint, Fire, FireBlueprint, Reacting},
    ParticleBundle,
};
use egui::Color32;

use bevy_falling_sand::color::*;
use bevy_falling_sand::core::*;
use bevy_falling_sand::debug::{
    DebugDirtyRects, DebugHibernatingChunks, DebugParticleCount, TotalParticleCount,
};
use bevy_falling_sand::movement::*;
use bevy_falling_sand::scenes::{LoadSceneEvent, SaveSceneEvent};

use super::*;

/// UI plugin
pub(super) struct UIPlugin;

impl bevy::prelude::Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<AppState>()
            .add_systems(Update, render_ui)
            .add_systems(Update, render_particle_editor)
            .add_systems(Update, update_particle_list)
            .add_systems(Update, update_app_state.after(render_ui))
            .add_systems(
                Update,
                toggle_simulation.run_if(input_just_pressed(KeyCode::Space)),
            )
            .init_resource::<CursorCoords>()
            .init_resource::<ParticleList>()
            .init_resource::<ParticleTypeList>()
            .init_resource::<SelectedBrushParticle>()
            .init_resource::<ParticleEditorSelectedType>()
            .init_resource::<ParticleEditorName>()
            .init_resource::<ParticleEditorDensity>()
            .init_resource::<ParticleEditorMomentum>()
            .init_resource::<ParticleEditorColors>()
            .init_resource::<ParticleEditorMaxVelocity>()
            .init_resource::<ParticleEditorMovementPriority>()
            .init_resource::<ParticleEditorBurns>()
            .init_resource::<ParticleEditorFire>()
            .init_resource::<ParticleEditorWall>()
            .init_resource::<ParticleEditorSolid>()
            .init_resource::<ParticleEditorMovableSolid>()
            .init_resource::<ParticleEditorLiquid>()
            .init_resource::<ParticleEditorGas>()
            .init_state::<ParticleEditorCategoryState>()
            .add_event::<ParticleEditorSave>()
            .add_event::<ParticleEditorUpdate>()
            .add_systems(First, update_cursor_coordinates)
            .add_systems(OnEnter(AppState::Ui), show_cursor)
            .add_systems(OnEnter(AppState::Canvas), hide_cursor)
            .add_systems(Update, ev_mouse_wheel)
            .add_systems(Update, handle_search_bar_input)
            .add_systems(Update, particle_editor_save)
            .add_systems(Update, update_particle_editor_fields)
            .add_systems(
                Update,
                render_search_bar_ui.run_if(resource_exists::<ParticleSearchBar>),
            )
            .observe(on_clear_dynamic_particles)
            .observe(on_clear_wall_particles);
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

/// A list of particle types organized by material type.
#[derive(Resource, Default)]
pub struct ParticleTypeList {
    map: HashMap<String, Vec<String>>,
}

impl ParticleTypeList {
    /// Get a particle type from the list
    pub fn get(&self, name: &str) -> Option<&Vec<String>> {
        self.map.get(name)
    }

    /// Insert a list of particles into the map for a given material. If the material already exists, modify the
    /// existing list. Lists are sorted after each call to this method.
    pub fn insert_or_modify(&mut self, material: String, particles: Vec<String>) {
        match self.map.entry(material) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().extend(particles);
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

/// Provides an ordered list of particles for the UI.
#[derive(Resource, Default)]
pub struct ParticleList {
    pub particle_list: Vec<String>,
}

impl ParticleList {
    /// Adds to the ParticleList.
    pub fn push(&mut self, value: String) {
        self.particle_list.push(value);
    }

    fn iter(&self) -> impl Iterator<Item = &String> {
        self.particle_list.iter()
    }
}

/// The currently selected particle for spawning.
#[derive(Resource)]
pub struct SelectedBrushParticle(pub String);

impl Default for SelectedBrushParticle {
    fn default() -> SelectedBrushParticle {
        SelectedBrushParticle("Dirt Wall".to_string())
    }
}

/// UI for particle control mechanics.
pub struct ParticleControlUI;

impl ParticleControlUI {
    /// Renders the particle control UI
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        particle_type_list: &Res<ParticleTypeList>,
        selected_brush_particle: &mut ResMut<SelectedBrushParticle>,
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
                                    selected_brush_particle.0 = particle_name.clone();
                                    brush_state.set(BrushState::Spawn);
                                }
                            });
                        });
                }
            }

            // Existing UI elements for Remove and Despawn All Particles
            ui.horizontal_wrapped(|ui| {
                if ui.button("Remove Tool").clicked() {
                    brush_state.set(BrushState::Despawn);
                }
            });

            ui.separator();

            if ui.button("Despawn All Dynamic Particles").clicked() {
                commands.trigger(ClearDynamicParticlesEvent)
            }

            if ui.button("Despawn All Wall Particles").clicked() {
                commands.trigger(ClearWallParticlesEvent)
            }

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
        debug_hibernating_chunks: &Option<Res<DebugHibernatingChunks>>,
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
                commands.init_resource::<DebugHibernatingChunks>();
            } else {
                commands.remove_resource::<DebugHibernatingChunks>();
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
        coords.update(world_position);
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
    (debug_hibernating_chunks, debug_dirty_rects, debug_particle_count, total_particle_count): (
        Option<Res<DebugHibernatingChunks>>,
        Option<Res<DebugDirtyRects>>,
        Option<Res<DebugParticleCount>>,
        Res<TotalParticleCount>,
    ),
    (mut selected_brush_particle, particle_type_list): (ResMut<SelectedBrushParticle>, Res<ParticleTypeList>),
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
                &mut selected_brush_particle,
                &mut brush_state,
                &mut commands,
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

pub fn update_particle_list(
    new_particle_query: Query<
        (
            &ParticleType,
            Option<&WallBlueprint>,
            Option<&MovableSolidBlueprint>,
            Option<&SolidBlueprint>,
            Option<&LiquidBlueprint>,
            Option<&GasBlueprint>,
        ),
        Added<ParticleType>,
    >,
    mut particle_list: ResMut<ParticleList>,
    mut particle_type_list: ResMut<ParticleTypeList>,
) {
    new_particle_query.iter().for_each(
        |(particle_type, wall, movable_solid, solid, liquid, gas)| {
            // Add the particle type name to the particle_list
            particle_list.push(particle_type.name.clone());

            // Check for the presence of each optional component and update particle_type_list accordingly
            if wall.is_some() {
                particle_type_list
                    .insert_or_modify("Walls".to_string(), vec![particle_type.name.clone()]);
            }
            if movable_solid.is_some() {
                particle_type_list.insert_or_modify(
                    "Movable Solids".to_string(),
                    vec![particle_type.name.clone()],
                );
            }
            if solid.is_some() {
                particle_type_list
                    .insert_or_modify("Solids".to_string(), vec![particle_type.name.clone()]);
            }
            if liquid.is_some() {
                particle_type_list
                    .insert_or_modify("Liquids".to_string(), vec![particle_type.name.clone()]);
            }
            if gas.is_some() {
                particle_type_list
                    .insert_or_modify("Gases".to_string(), vec![particle_type.name.clone()]);
            }
        },
    );
}

/// Stops or starts the simulation when scheduled.
pub fn toggle_simulation(
    mut commands: Commands,
    simulation_pause: Option<Res<SimulationRun>>,
    app_state: Res<State<AppState>>,
) {
    match app_state.get() {
        AppState::Canvas => {
            if simulation_pause.is_some() {
                commands.remove_resource::<SimulationRun>();
            } else {
                commands.init_resource::<SimulationRun>();
            }
        }
        _ => {}
    }
}

/// Listens for scroll events and performs the corresponding action
pub fn ev_mouse_wheel(
    mut ev_scroll: EventReader<MouseWheel>,
    app_state: Res<State<AppState>>,
    mut camera_query: Query<&mut OrthographicProjection, With<MainCamera>>,
    mut brush_query: Query<&mut Brush>,
    max_brush_size: Res<MaxBrushSize>,
) {
    if !ev_scroll.is_empty() {
        match app_state.get() {
            AppState::Ui => {
                let mut brush = brush_query.single_mut();
                ev_scroll.read().for_each(|ev| {
                    if ev.y < 0. && brush.size - 1 >= 1 {
                        brush.size -= 1;
                    } else if ev.y > 0. && brush.size + 1 <= max_brush_size.0 {
                        brush.size += 1;
                    }
                });
            }
            AppState::Canvas => {
                let mut projection = camera_query.single_mut();
                ev_scroll.read().for_each(|ev| {
                    let zoom = -(ev.y / 100.);
                    if projection.scale + zoom > 0.01 {
                        projection.scale += zoom;
                    }
                });
            }
        };
    }
}

/// Resource to manage the state of the particle search bar.
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
    if keys.just_pressed(KeyCode::KeyN) {
        if particle_search_bar.is_none() {
            commands.insert_resource(ParticleSearchBar::default());
            return;
        }
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
                    if selected_particle.0 == *particle {
                        should_close = true;
                    } else {
                        new_selected_index = Some(i);
                        selected_particle.0 = particle.clone();
                        brush_state.set(BrushState::Spawn);
                    }
                }

                if new_selected_index.is_some() && keys.just_pressed(KeyCode::Enter) {
                    should_close = true;
                    selected_particle.0 = particle.clone();
                    new_selected_index = Some(i);
                    brush_state.set(BrushState::Spawn);
                }
            }

            particle_search_bar.selected_index = new_selected_index;
        });

    if should_close {
        commands.remove_resource::<ParticleSearchBar>();
    }
}

/// Remove all particles from the simulation.
#[derive(Event)]
pub struct ClearDynamicParticlesEvent;

/// Remove all particles from the simulation.
#[derive(Event)]
pub struct ClearWallParticlesEvent;

pub fn on_clear_dynamic_particles(
    _trigger: Trigger<ClearDynamicParticlesEvent>,
    mut commands: Commands,
    dynamic_particle_types_query: Query<&ParticleType, Without<WallBlueprint>>,
) {
    dynamic_particle_types_query
        .iter()
        .for_each(|particle_type| {
            commands.trigger(ClearParticleTypeChildrenEvent(particle_type.name.clone()))
        });
}

pub fn on_clear_wall_particles(
    _trigger: Trigger<ClearWallParticlesEvent>,
    mut commands: Commands,
    dynamic_particle_types_query: Query<&ParticleType, With<WallBlueprint>>,
) {
    dynamic_particle_types_query
        .iter()
        .for_each(|particle_type| {
            commands.trigger(ClearParticleTypeChildrenEvent(particle_type.name.clone()))
        });
}

pub fn update_particle_editor_fields(
    mut ev_particle_editor_update: EventReader<ParticleEditorUpdate>,
    mut particle_editor_selected_type: ResMut<ParticleEditorSelectedType>,
    mut particle_editor_name: ResMut<ParticleEditorName>,
    particle_type_map: Res<ParticleTypeMap>,
    particle_query: Query<
        (
            Option<&DensityBlueprint>,
            Option<&VelocityBlueprint>,
            Option<&MomentumBlueprint>,
            Option<&ParticleColorBlueprint>,
            Option<&BurnsBlueprint>,
            Option<&FireBlueprint>,
            Option<&WallBlueprint>,
            Option<&LiquidBlueprint>,
            Option<&SolidBlueprint>,
            Option<&MovableSolidBlueprint>,
            Option<&GasBlueprint>,
        ),
        With<ParticleType>,
    >,
    mut particle_density_field: ResMut<ParticleEditorDensity>,
    mut particle_max_velocity_field: ResMut<ParticleEditorMaxVelocity>,
    mut particle_momentum_field: ResMut<ParticleEditorMomentum>,
    mut particle_colors_field: ResMut<ParticleEditorColors>,
    mut particle_editor_burns_field: ResMut<ParticleEditorBurns>,
    mut particle_editor_fire_field: ResMut<ParticleEditorFire>,
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
                fire,
                wall,
                liquid,
                solid,
                movable_solid,
                gas,
            )) = particle_query.get(*entity)
            {
                particle_editor_name.0 = particle_editor_selected_type.0.name.clone();
                particle_editor_selected_type.0 =
                    ParticleType::new(particle_editor_selected_type.0.name.clone().as_str());
                if let Some(density) = density {
                    particle_density_field.blueprint = *density;
                }
                if let Some(velocity) = velocity {
                    particle_max_velocity_field.blueprint = *velocity;
                }
                if let Some(momentum) = momentum {
                    particle_momentum_field.blueprint = *momentum;
                }
                if let Some(colors) = colors {
                    particle_colors_field.blueprint = colors.clone()
                }
                if let Some(burns) = burns {
                    particle_editor_burns_field.blueprint = burns.clone();
                }
                if let Some(fire) = fire {
                    particle_editor_fire_field.blueprint = fire.clone();
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
    mut particle_editor_selected_field: ResMut<ParticleEditorSelectedType>,
    mut particle_density_field: ResMut<ParticleEditorDensity>,
    mut particle_editor_max_velocity_field: ResMut<ParticleEditorMaxVelocity>,
    mut particle_momentum_field: ResMut<ParticleEditorMomentum>,
    mut particle_editor_colors_field: ResMut<ParticleEditorColors>,
    mut particle_editor_movement_priority_field: ResMut<ParticleEditorMovementPriority>,
    mut particle_editor_burns_field: ResMut<ParticleEditorBurns>,
    mut particle_editor_fire_field: ResMut<ParticleEditorFire>,
    (mut particle_editor_liquid_field, mut particle_editor_gas_field): (
        ResMut<ParticleEditorLiquid>,
        ResMut<ParticleEditorGas>,
    ),
) {
    egui::Window::new("Particle Editor") // Title of the window
        .resizable(true) // Allow resizing
        .collapsible(true) // Allow collapsing
        .show(contexts.ctx_mut(), |ui| {
            let available_width = ui.available_width();
            let available_height = ui.available_height();

            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(available_width / 3.0, available_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        const CATEGORIES: [&str; 5] =
                            ["Walls", "Solids", "Movable Solids", "Liquids", "Gases"];

                        for &category in &CATEGORIES {
                            if let Some(particles) = particle_type_list.get(category) {
                                egui::CollapsingHeader::new(category)
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        for particle_name in particles {
                                            if ui.button(particle_name).clicked() {
                                                selected_brush_particle.0 = particle_name.clone();
                                                particle_editor_selected_field.0 =
                                                    ParticleType::new(particle_name.as_str());
                                                ev_particle_editor_update
                                                    .send(ParticleEditorUpdate);
                                            }
                                        }
                                    });
                            }
                        }

                        if ui.button("New Particle").clicked() {}
                        if ui.button("Save Particle").clicked() {
                            ev_particle_editor_save.send(ParticleEditorSave);
                        }
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(available_width * 2.0 / 3.0, available_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.text_edit_singleline(&mut particle_editor_selected_field.0.name);
                                });
                                render_state_field(
                                    ui,
                                    &current_particle_category_field,
                                    &mut next_particle_category_field,
                                );
                                match current_particle_category_field.get() {
                                    ParticleEditorCategoryState::Wall => {
                                        ui.separator();
                                        render_colors_field(ui, &mut particle_editor_colors_field);
                                    }
                                    ParticleEditorCategoryState::Solid => {
                                        ui.separator();
                                        render_colors_field(ui, &mut particle_editor_colors_field);
                                        ui.separator();
                                        render_density_field(ui, &mut particle_density_field);
                                        render_max_velocity_field(
                                            ui,
                                            &mut particle_editor_max_velocity_field,
                                        );
                                        render_burns_field(
                                            ui,
                                            &mut particle_editor_burns_field,
                                            &particle_list,
                                        );
                                        ui.separator();
                                        render_fire_field(ui, &mut particle_editor_fire_field);
                                    }
                                    ParticleEditorCategoryState::MovableSolid => {
                                        ui.separator();
                                        render_colors_field(ui, &mut particle_editor_colors_field);
                                        ui.separator();
                                        render_density_field(ui, &mut particle_density_field);
                                        render_max_velocity_field(
                                            ui,
                                            &mut particle_editor_max_velocity_field,
                                        );
                                        render_momentum_field(ui, &mut particle_momentum_field);
                                        render_burns_field(
                                            ui,
                                            &mut particle_editor_burns_field,
                                            &particle_list,
                                        );
                                        ui.separator();
                                        render_fire_field(ui, &mut particle_editor_fire_field);
                                    }
                                    ParticleEditorCategoryState::Liquid => {
                                        ui.separator();
                                        render_colors_field(ui, &mut particle_editor_colors_field);
                                        ui.separator();
                                        render_fluidity_field(
                                            ui,
                                            &mut particle_editor_liquid_field,
                                            &mut particle_editor_gas_field,
                                            &current_particle_category_field,
                                        );
                                        render_density_field(ui, &mut particle_density_field);
                                        render_max_velocity_field(
                                            ui,
                                            &mut particle_editor_max_velocity_field,
                                        );
                                        render_momentum_field(ui, &mut particle_momentum_field);
                                        render_burns_field(
                                            ui,
                                            &mut particle_editor_burns_field,
                                            &particle_list,
                                        );
                                        ui.separator();
                                        render_fire_field(ui, &mut particle_editor_fire_field);
                                    }
                                    ParticleEditorCategoryState::Gas => {
                                        ui.separator();
                                        render_colors_field(ui, &mut particle_editor_colors_field);
                                        ui.separator();
                                        render_fluidity_field(
                                            ui,
                                            &mut particle_editor_liquid_field,
                                            &mut particle_editor_gas_field,
                                            &current_particle_category_field,
                                        );
                                        render_density_field(ui, &mut particle_density_field);
                                        render_max_velocity_field(
                                            ui,
                                            &mut particle_editor_max_velocity_field,
                                        );
                                        render_momentum_field(ui, &mut particle_momentum_field);
                                        render_burns_field(
                                            ui,
                                            &mut particle_editor_burns_field,
                                            &particle_list,
                                        );
                                        ui.separator();
                                        render_fire_field(ui, &mut particle_editor_fire_field);
                                    }
                                    ParticleEditorCategoryState::Other => {
                                        ui.separator();
                                        render_colors_field(ui, &mut particle_editor_colors_field);
                                        ui.separator();
                                        render_density_field(ui, &mut particle_density_field);
                                        render_max_velocity_field(
                                            ui,
                                            &mut particle_editor_max_velocity_field,
                                        );
                                        render_momentum_field(ui, &mut particle_momentum_field);
                                        ui.separator();
                                        render_movement_priority_field(
                                            ui,
                                            &mut particle_editor_movement_priority_field,
                                        );
                                        ui.separator();
                                        render_burns_field(
                                            ui,
                                            &mut particle_editor_burns_field,
                                            &particle_list,
                                        );
                                        ui.separator();
                                        render_fire_field(ui, &mut particle_editor_fire_field);
                                    }
                                }
                            });
                        });
                    },
                );
            });
        });
}

fn render_state_field(
    ui: &mut egui::Ui,
    current_particle_category_field: &Res<State<ParticleEditorCategoryState>>,
    next_particle_category_field: &mut ResMut<NextState<ParticleEditorCategoryState>>,
) {
    ui.horizontal(|ui| {
        ui.label("State: ");
        egui::ComboBox::from_label("")
            .selected_text(current_particle_category_field.as_str())
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(
                        &mut current_particle_category_field.as_str(),
                        ParticleEditorCategoryState::Wall.as_str(),
                        "Wall",
                    )
                    .changed()
                {
                    next_particle_category_field.set(ParticleEditorCategoryState::Wall);
                }
                if ui
                    .selectable_value(
                        &mut current_particle_category_field.as_str(),
                        ParticleEditorCategoryState::Solid.as_str(),
                        "Solid",
                    )
                    .changed()
                {
                    next_particle_category_field.set(ParticleEditorCategoryState::Solid);
                }
                if ui
                    .selectable_value(
                        &mut current_particle_category_field.as_str(),
                        ParticleEditorCategoryState::MovableSolid.as_str(),
                        "Movable Solid",
                    )
                    .changed()
                {
                    next_particle_category_field.set(ParticleEditorCategoryState::MovableSolid);
                }
                if ui
                    .selectable_value(
                        &mut current_particle_category_field.as_str(),
                        ParticleEditorCategoryState::Liquid.as_str(),
                        "Liquid",
                    )
                    .changed()
                {
                    next_particle_category_field.set(ParticleEditorCategoryState::Liquid);
                }
                if ui
                    .selectable_value(
                        &mut current_particle_category_field.as_str(),
                        ParticleEditorCategoryState::Gas.as_str(),
                        "Gas",
                    )
                    .changed()
                {
                    next_particle_category_field.set(ParticleEditorCategoryState::Gas);
                }
                if ui
                    .selectable_value(
                        &mut current_particle_category_field.as_str(),
                        ParticleEditorCategoryState::Other.as_str(),
                        "Other",
                    )
                    .changed()
                {
                    next_particle_category_field.set(ParticleEditorCategoryState::Other);
                }
            });
    });
}
fn render_density_field(
    ui: &mut egui::Ui,
    particle_density_field: &mut ResMut<ParticleEditorDensity>,
) {
    ui.horizontal(|ui| {
        ui.label("Density: ");
        ui.add(egui::Slider::new(&mut particle_density_field.blueprint.0 .0, 1..=1000).step_by(1.));
    });
}

fn render_max_velocity_field(
    ui: &mut egui::Ui,
    particle_max_velocity_field: &mut ResMut<ParticleEditorMaxVelocity>,
) {
    ui.horizontal(|ui| {
        ui.label("Max Velocity: ");
        ui.add(
            egui::Slider::new(&mut particle_max_velocity_field.blueprint.0.max, 1..=5).step_by(1.),
        );
    });
}

fn render_momentum_field(
    ui: &mut egui::Ui,
    particle_momentum_field: &mut ResMut<ParticleEditorMomentum>,
) {
    ui.horizontal(|ui| {
        ui.label("Momentum"); // Add the label to the left
        ui.checkbox(&mut particle_momentum_field.enable, "");
        // Use an empty string for the checkbox text
    });
}

fn render_colors_field(
    ui: &mut egui::Ui,
    particle_colors_field: &mut ResMut<ParticleEditorColors>,
) {
    ui.horizontal(|ui| {
        ui.label("Colors");
        if ui.button("➕").clicked() {
            particle_colors_field
                .blueprint
                .0
                .palette
                .push(Color::srgba_u8(255, 255, 255, 255))
        };
    });
    let mut to_remove: Option<usize> = None;
    let mut to_change: Option<(usize, Color)> = None;
    for (i, color) in particle_colors_field.blueprint.0.palette.iter().enumerate() {
        let srgba = color.to_srgba();
        let (red, green, blue, alpha) = (
            srgba.red * 255.,
            srgba.green * 255.,
            srgba.blue * 255.,
            srgba.alpha * 255.,
        );
        let (mut red_str, mut green_str, mut blue_str, mut alpha_str) = (
            red.to_string(),
            green.to_string(),
            blue.to_string(),
            alpha.to_string(),
        );
        let mut color32 =
            Color32::from_rgba_unmultiplied(red as u8, green as u8, blue as u8, alpha as u8);
        ui.horizontal(|ui| {
            ui.label("R: ");
            let edit_red = ui.add(egui::TextEdit::singleline(&mut red_str).desired_width(25.));
            if edit_red.changed() {
                if let Ok(new_red) = red_str.parse::<u8>() {
                    to_change = Some((
                        i,
                        Color::srgba_u8(new_red as u8, green as u8, blue as u8, alpha as u8),
                    ))
                } else if red_str.is_empty() {
                    to_change = Some((i, Color::srgba_u8(0, green as u8, blue as u8, alpha as u8)))
                }
            };
            ui.label("G: ");
            let edit_green = ui.add(egui::TextEdit::singleline(&mut green_str).desired_width(25.));
            if edit_green.changed() {
                if let Ok(new_green) = green_str.parse::<u8>() {
                    to_change = Some((
                        i,
                        Color::srgba_u8(red as u8, new_green as u8, blue as u8, alpha as u8),
                    ))
                } else if green_str.is_empty() {
                    to_change = Some((i, Color::srgba_u8(red as u8, 0, blue as u8, alpha as u8)))
                }
            };
            ui.label("B: ");
            let edit_blue = ui.add(egui::TextEdit::singleline(&mut blue_str).desired_width(25.));
            if edit_blue.changed() {
                if let Ok(new_blue) = blue_str.parse::<u8>() {
                    to_change = Some((
                        i,
                        Color::srgba_u8(red as u8, green as u8, new_blue as u8, alpha as u8),
                    ))
                } else if blue_str.is_empty() {
                    to_change = Some((i, Color::srgba_u8(red as u8, green as u8, 0, alpha as u8)))
                }
            };
            ui.label("A: ");
            let edit_alpha = ui.add(egui::TextEdit::singleline(&mut alpha_str).desired_width(25.));
            if edit_alpha.changed() {
                if let Ok(new_alpha) = alpha_str.parse::<u8>() {
                    to_change = Some((
                        i,
                        Color::srgba_u8(red as u8, green as u8, blue as u8, new_alpha as u8),
                    ))
                } else if alpha_str.is_empty() {
                    to_change = Some((i, Color::srgba_u8(red as u8, green as u8, blue as u8, 0)))
                }
            };
            if ui.color_edit_button_srgba(&mut color32).changed() {
                to_change = Some((
                    i,
                    Color::srgba_u8(color32.r(), color32.g(), color32.b(), color32.a()),
                ));
            };
            if ui.button("❌").clicked() {
                to_remove = Some(i);
            };
        });
    }
    if let Some(to_remove) = to_remove {
        particle_colors_field.blueprint.0.palette.remove(to_remove);
    }
    if let Some((to_change, color)) = to_change {
        particle_colors_field.blueprint.0.palette[to_change] = color;
    }
}

fn render_movement_priority_field(
    ui: &mut egui::Ui,
    particle_movement_priority_field: &mut ResMut<ParticleEditorMovementPriority>,
) {
    ui.horizontal(|ui| {
        ui.label("Movement Priority");
        if ui.button("➕").clicked() {
            particle_movement_priority_field
                .blueprint
                .0
                .push_outer(NeighborGroup::empty());
        };
    });

    let mut to_change: Option<((usize, usize), IVec2)> = None;
    let mut inner_to_remove: Option<(usize, usize)> = None;
    let mut outer_to_remove: Option<usize> = None;
    let mut outer_to_add: Option<usize> = None;
    let mut inner_to_swap: Option<(usize, usize, usize)> = None;
    let mut outer_to_swap: Option<(usize, usize)> = None;

    for (i, neighbor_group) in particle_movement_priority_field
        .blueprint
        .0
        .iter()
        .enumerate()
    {
        ui.horizontal(|ui| {
            ui.label(format!("Group {}:", i + 1));
            if ui.button("➕").clicked() {
                outer_to_add = Some(i);
            };
            if ui.button("^").clicked() && i > 0 {
                outer_to_swap = Some((i, i - 1));
            }
            if ui.button("v").clicked()
                && i < particle_movement_priority_field.blueprint.0.len() - 1
            {
                outer_to_swap = Some((i, i + 1));
            }
            if ui.button("❌").clicked() {
                outer_to_remove = Some(i);
            };
        });
        for (j, neighbor) in neighbor_group.neighbor_group.iter().enumerate() {
            let mut x_str = neighbor.x.to_string();
            let mut y_str = neighbor.y.to_string();
            ui.horizontal(|ui| {
                ui.label("X: ");
                let edit_x = ui.add(egui::TextEdit::singleline(&mut x_str).desired_width(25.));
                if edit_x.changed() {
                    if let Ok(new_x) = x_str.parse::<i32>() {
                        to_change = Some(((i, j), IVec2::new(new_x, neighbor.y)));
                    } else if x_str.is_empty() {
                        to_change = Some(((i, j), IVec2::ZERO));
                    };
                };

                ui.label("Y: ");
                let edit_y = ui.add(egui::TextEdit::singleline(&mut y_str).desired_width(25.));
                if edit_y.changed() {
                    if let Ok(new_y) = y_str.parse::<i32>() {
                        to_change = Some(((i, j), IVec2::new(neighbor.x, new_y)));
                    } else if y_str.is_empty() {
                        to_change = Some(((i, j), IVec2::ZERO));
                    };
                };

                if ui.button("^").clicked() && j > 0 {
                    inner_to_swap = Some((i, j, j - 1));
                }
                if ui.button("v").clicked() && j < neighbor_group.len() - 1 {
                    inner_to_swap = Some((i, j, j + 1));
                }
                if ui.button("❌").clicked() {
                    inner_to_remove = Some((i, j));
                };
            });
        }
    }
    if let Some((i, j)) = inner_to_remove {
        if let Some(group) = particle_movement_priority_field.blueprint.0.get_mut(i) {
            group.neighbor_group.remove(j);
        }
    }
    if let Some((i, j1, j2)) = inner_to_swap {
        if let Some(group) = particle_movement_priority_field.blueprint.0.get_mut(i) {
            group
                .swap(j1, j2)
                .unwrap_or_else(|err| eprintln!("{}", err));
        }
    }
    if let Some((i, j)) = outer_to_swap {
        particle_movement_priority_field
            .blueprint
            .0
            .swap_outer(i, j)
            .unwrap_or_else(|err| eprintln!("{}", err));
    }
    if let Some(i) = outer_to_add {
        if let Some(group) = particle_movement_priority_field.blueprint.0.get_mut(i) {
            group.push(IVec2::ZERO);
        }
    }
    if let Some(i) = outer_to_remove {
        particle_movement_priority_field.blueprint.0.remove(i);
    }
    if let Some(((i, j), new_ivec)) = to_change {
        if let Some(group) = particle_movement_priority_field.blueprint.0.get_mut(i) {
            if let Some(neighbor) = group.neighbor_group.get_mut(j) {
                *neighbor = new_ivec;
            }
        }
    }
}

fn render_burns_field(
    ui: &mut egui::Ui,
    particle_burns_field: &mut ResMut<ParticleEditorBurns>,
    particle_list: &Res<ParticleList>,
) {
    ui.add(egui::Checkbox::new(
        &mut particle_burns_field.enable,
        "Flammable",
    ));
    if particle_burns_field.enable {
        ui.horizontal(|ui| {
            ui.label("Duration (ms): ");
            let mut duration_str = particle_burns_field
                .blueprint
                .0
                .duration
                .as_millis()
                .to_string();
            let edit_duration =
                ui.add(egui::TextEdit::singleline(&mut duration_str).desired_width(40.));
            if edit_duration.changed() {
                if let Ok(new_duration) = duration_str.parse::<u64>() {
                    particle_burns_field.blueprint.0.duration = Duration::from_millis(new_duration);
                }
            } else if duration_str.is_empty() {
                particle_burns_field.blueprint.0.duration = Duration::from_millis(1);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Tick Rate (ms)");
            let mut tick_rate_str = particle_burns_field
                .blueprint
                .0
                .tick_rate
                .as_millis()
                .to_string();
            let edit_tick_rate =
                ui.add(egui::TextEdit::singleline(&mut tick_rate_str).desired_width(40.));
            if edit_tick_rate.changed() {
                if let Ok(new_tick_rate) = tick_rate_str.parse::<u64>() {
                    particle_burns_field.blueprint.0.duration =
                        Duration::from_millis(new_tick_rate);
                } else if tick_rate_str.is_empty() {
                    particle_burns_field.blueprint.0.tick_rate = Duration::from_millis(1);
                }
            }
        });
        if ui
            .add(egui::Checkbox::new(
                &mut particle_burns_field.color_enable,
                "Change colors while burning",
            ))
            .clicked()
        {
            if particle_burns_field.color_enable {
                particle_burns_field.blueprint.0.color = Some(ParticleColor::new(
                    Color::srgba_u8(255, 255, 255, 255),
                    vec![Color::srgba_u8(255, 255, 255, 255)],
                ));
            } else {
                particle_burns_field.blueprint.0.color = None;
            }
        }

        if particle_burns_field.color_enable {
            ui.horizontal(|ui| {
                ui.label("Colors");
                if ui.button("➕").clicked() {
                    particle_burns_field
                        .blueprint
                        .0
                        .color
                        .as_mut()
                        .unwrap()
                        .palette
                        .push(Color::srgba_u8(255, 255, 255, 255))
                };
            });
            let mut to_remove: Option<usize> = None;
            let mut to_change: Option<(usize, Color)> = None;
            for (i, color) in particle_burns_field
                .blueprint
                .0
                .color
                .clone()
                .unwrap()
                .palette
                .iter()
                .enumerate()
            {
                let srgba = color.to_srgba();
                let (red, green, blue, alpha) = (
                    srgba.red * 255.,
                    srgba.green * 255.,
                    srgba.blue * 255.,
                    srgba.alpha * 255.,
                );
                let (mut red_str, mut green_str, mut blue_str, mut alpha_str) = (
                    red.to_string(),
                    green.to_string(),
                    blue.to_string(),
                    alpha.to_string(),
                );
                let mut color32 = Color32::from_rgba_unmultiplied(
                    red as u8,
                    green as u8,
                    blue as u8,
                    alpha as u8,
                );
                ui.horizontal(|ui| {
                    ui.label("R: ");
                    let edit_red =
                        ui.add(egui::TextEdit::singleline(&mut red_str).desired_width(25.));
                    if edit_red.changed() {
                        if let Ok(new_red) = red_str.parse::<u8>() {
                            to_change = Some((
                                i,
                                Color::srgba_u8(
                                    new_red as u8,
                                    green as u8,
                                    blue as u8,
                                    alpha as u8,
                                ),
                            ))
                        } else if red_str.is_empty() {
                            to_change =
                                Some((i, Color::srgba_u8(0, green as u8, blue as u8, alpha as u8)))
                        }
                    };
                    ui.label("G: ");
                    let edit_green =
                        ui.add(egui::TextEdit::singleline(&mut green_str).desired_width(25.));
                    if edit_green.changed() {
                        if let Ok(new_green) = green_str.parse::<u8>() {
                            to_change = Some((
                                i,
                                Color::srgba_u8(
                                    red as u8,
                                    new_green as u8,
                                    blue as u8,
                                    alpha as u8,
                                ),
                            ))
                        } else if green_str.is_empty() {
                            to_change =
                                Some((i, Color::srgba_u8(red as u8, 0, blue as u8, alpha as u8)))
                        }
                    };
                    ui.label("B: ");
                    let edit_blue =
                        ui.add(egui::TextEdit::singleline(&mut blue_str).desired_width(25.));
                    if edit_blue.changed() {
                        if let Ok(new_blue) = blue_str.parse::<u8>() {
                            to_change = Some((
                                i,
                                Color::srgba_u8(
                                    red as u8,
                                    green as u8,
                                    new_blue as u8,
                                    alpha as u8,
                                ),
                            ))
                        } else if blue_str.is_empty() {
                            to_change =
                                Some((i, Color::srgba_u8(red as u8, green as u8, 0, alpha as u8)))
                        }
                    };
                    ui.label("A: ");
                    let edit_alpha =
                        ui.add(egui::TextEdit::singleline(&mut alpha_str).desired_width(25.));
                    if edit_alpha.changed() {
                        if let Ok(new_alpha) = alpha_str.parse::<u8>() {
                            to_change = Some((
                                i,
                                Color::srgba_u8(
                                    red as u8,
                                    green as u8,
                                    blue as u8,
                                    new_alpha as u8,
                                ),
                            ))
                        } else if alpha_str.is_empty() {
                            to_change =
                                Some((i, Color::srgba_u8(red as u8, green as u8, blue as u8, 0)))
                        }
                    };
                    if ui.color_edit_button_srgba(&mut color32).changed() {
                        to_change = Some((
                            i,
                            Color::srgba_u8(color32.r(), color32.g(), color32.b(), color32.a()),
                        ));
                    };
                    if ui.button("❌").clicked() {
                        to_remove = Some(i);
                    };
                });
            }
            if let Some(to_remove) = to_remove {
                particle_burns_field
                    .blueprint
                    .0
                    .color
                    .as_mut()
                    .unwrap()
                    .palette
                    .remove(to_remove);
            }
            if let Some((to_change, color)) = to_change {
                particle_burns_field
                    .blueprint
                    .0
                    .color
                    .as_mut()
                    .unwrap()
                    .palette[to_change] = color;
            }
        }
        if ui
            .add(egui::Checkbox::new(
                &mut particle_burns_field.chance_destroy_enable,
                "Chance Destroy Per Tick",
            ))
            .clicked()
        {
            if particle_burns_field.chance_destroy_enable {
                particle_burns_field.blueprint.0.chance_destroy_per_tick = Some(0.);
            } else {
                particle_burns_field.blueprint.0.chance_destroy_per_tick = None;
            }
        };
        if particle_burns_field.chance_destroy_enable {
            ui.horizontal(|ui| {
                ui.label("Chance");
                let mut chance_destroy_str = particle_burns_field
                    .blueprint
                    .0
                    .chance_destroy_per_tick
                    .unwrap()
                    .to_string();
                let edit_chance_destroy =
                    ui.add(egui::TextEdit::singleline(&mut chance_destroy_str).desired_width(40.));
                if edit_chance_destroy.changed() {
                    if let Ok(new_chance_destroy) = chance_destroy_str.parse::<f64>() {
                        particle_burns_field.blueprint.0.chance_destroy_per_tick =
                            Some(new_chance_destroy);
                    } else if chance_destroy_str.is_empty() {
                        particle_burns_field.blueprint.0.tick_rate = Duration::from_millis(1);
                    }
                }
            });
        }
        if ui
            .add(egui::Checkbox::new(
                &mut particle_burns_field.reaction_enable,
                "Produces new particle while burning",
            ))
            .clicked()
        {
            if particle_burns_field.reaction_enable {
                particle_burns_field.blueprint.0.reaction =
                    Some(Reacting::new(Particle::new("Water"), 0.1));
            } else {
                particle_burns_field.blueprint.0.reaction = None;
            }
        }
        if particle_burns_field.reaction_enable {
            ui.horizontal(|ui| {
                ui.label("Particle");
                egui::ComboBox::from_id_salt("burning_reaction")
                    .selected_text(format!(
                        "{}",
                        particle_burns_field
                            .blueprint
                            .0
                            .reaction
                            .clone()
                            .unwrap()
                            .produces
                            .name
                    ))
                    .show_ui(ui, |ui| {
                        for particle in particle_list.iter() {
                            if ui
                                .selectable_value(
                                    &mut particle_burns_field
                                        .blueprint
                                        .0
                                        .reaction
                                        .as_mut()
                                        .unwrap()
                                        .produces
                                        .name,
                                    particle.clone(),
                                    particle.clone(),
                                )
                                .clicked()
                            {}
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.label("Chance to produce (per tick)");
                let mut chance_produce_str = particle_burns_field
                    .blueprint
                    .0
                    .reaction
                    .as_mut()
                    .unwrap()
                    .chance_to_produce
                    .to_string();
                let edit_chance_produce =
                    ui.add(egui::TextEdit::singleline(&mut chance_produce_str).desired_width(40.));
                if edit_chance_produce.changed() {
                    if let Ok(new_chance_produce) = chance_produce_str.parse::<f64>() {
                        particle_burns_field
                            .blueprint
                            .0
                            .reaction
                            .as_mut()
                            .unwrap()
                            .chance_to_produce = new_chance_produce;
                    } else if chance_produce_str.is_empty() {
                        particle_burns_field
                            .blueprint
                            .0
                            .reaction
                            .as_mut()
                            .unwrap()
                            .chance_to_produce = 0.01;
                    }
                }
            });

            if ui
                .add(egui::Checkbox::new(
                    &mut particle_burns_field.spreads_enable,
                    "Fire Spreads",
                ))
                .clicked()
            {
                if particle_burns_field.spreads_enable {
                    particle_burns_field.blueprint.0.spreads = Some(Fire {
                        burn_radius: 2.,
                        chance_to_spread: 0.01,
                        destroys_on_spread: false,
                    });
                } else {
                    particle_burns_field.blueprint.0.spreads = None;
                }
            }
            if particle_burns_field.spreads_enable {
                ui.horizontal(|ui| {
                    ui.label("Burn Radius");
                    let mut burn_radius_str = particle_burns_field
                        .blueprint
                        .0
                        .spreads
                        .unwrap()
                        .burn_radius
                        .to_string();
                    let edit_burn_radius =
                        ui.add(egui::TextEdit::singleline(&mut burn_radius_str).desired_width(40.));
                    if edit_burn_radius.changed() {
                        if let Ok(new_burn_radius) = burn_radius_str.parse::<f32>() {
                            particle_burns_field
                                .blueprint
                                .0
                                .spreads
                                .as_mut()
                                .unwrap()
                                .burn_radius = new_burn_radius;
                            println!(
                                "New burn radius: {:?} Particle burns field blueprint: {:?}",
                                new_burn_radius,
                                particle_burns_field
                                    .blueprint
                                    .0
                                    .spreads
                                    .as_mut()
                                    .unwrap()
                                    .burn_radius
                            );
                        } else if burn_radius_str.is_empty() {
                            particle_burns_field
                                .blueprint
                                .0
                                .spreads
                                .as_mut()
                                .unwrap()
                                .burn_radius = 2.;
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Chance to spread");
                    let mut chance_to_spread_str = particle_burns_field
                        .blueprint
                        .0
                        .spreads
                        .as_mut()
                        .unwrap()
                        .chance_to_spread
                        .to_string();
                    let edit_chance_to_spread = ui.add(
                        egui::TextEdit::singleline(&mut chance_to_spread_str).desired_width(40.),
                    );
                    if edit_chance_to_spread.changed() {
                        if let Ok(new_chance) = chance_to_spread_str.parse::<f64>() {
                            particle_burns_field
                                .blueprint
                                .0
                                .spreads
                                .as_mut()
                                .unwrap()
                                .chance_to_spread = new_chance;
                        } else {
                            particle_burns_field
                                .blueprint
                                .0
                                .spreads
                                .as_mut()
                                .unwrap()
                                .chance_to_spread = 0.01;
                        }
                    }
                });
                ui.horizontal(|ui| {
                    if ui
                        .add(egui::Checkbox::new(
                            &mut particle_burns_field
                                .blueprint
                                .0
                                .spreads
                                .as_mut()
                                .unwrap()
                                .destroys_on_spread,
                            "Destroys on spread",
                        ))
                        .clicked()
                    {
                        if particle_burns_field
                            .blueprint
                            .0
                            .spreads
                            .as_mut()
                            .unwrap()
                            .destroys_on_spread
                        {
                            particle_burns_field
                                .blueprint
                                .0
                                .spreads
                                .as_mut()
                                .unwrap()
                                .destroys_on_spread = true;
                        } else {
                            particle_burns_field
                                .blueprint
                                .0
                                .spreads
                                .as_mut()
                                .unwrap()
                                .destroys_on_spread = false;
                        }
                    }
                });
            }
        }
    }
}

fn render_fire_field(ui: &mut egui::Ui, particle_fire_field: &mut ResMut<ParticleEditorFire>) {
    ui.add(egui::Checkbox::new(
        &mut particle_fire_field.enable,
        "Fire Spreads",
    ));
    if particle_fire_field.enable {
        ui.horizontal(|ui| {
            ui.label("Burn Radius");
            let mut burn_radius_str = particle_fire_field.blueprint.0.burn_radius.to_string();
            let edit_burn_radius =
                ui.add(egui::TextEdit::singleline(&mut burn_radius_str).desired_width(40.));
            if edit_burn_radius.changed() {
                if let Ok(new_burn_radius) = burn_radius_str.parse::<f32>() {
                    particle_fire_field.blueprint.0.burn_radius = new_burn_radius;
                    println!(
                        "New burn radius: {:?} Particle burns field blueprint: {:?}",
                        new_burn_radius, particle_fire_field.blueprint.0.burn_radius
                    );
                } else if burn_radius_str.is_empty() {
                    particle_fire_field.blueprint.0.burn_radius = 2.;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Chance to spread");
            let mut chance_to_spread_str =
                particle_fire_field.blueprint.0.chance_to_spread.to_string();
            let edit_chance_to_spread =
                ui.add(egui::TextEdit::singleline(&mut chance_to_spread_str).desired_width(40.));
            if edit_chance_to_spread.changed() {
                if let Ok(new_chance) = chance_to_spread_str.parse::<f64>() {
                    particle_fire_field.blueprint.0.chance_to_spread = new_chance;
                } else {
                    particle_fire_field.blueprint.0.chance_to_spread = 0.01;
                }
            }
        });
        ui.horizontal(|ui| {
            if ui
                .add(egui::Checkbox::new(
                    &mut particle_fire_field.blueprint.0.destroys_on_spread,
                    "Destroys on spread",
                ))
                .clicked()
            {
                if particle_fire_field.blueprint.0.destroys_on_spread {
                    particle_fire_field.blueprint.0.destroys_on_spread = true;
                } else {
                    particle_fire_field.blueprint.0.destroys_on_spread = false;
                }
            }
        });
    }
}

pub fn render_fluidity_field(
    ui: &mut egui::Ui,
    particle_liquid_field: &mut ResMut<ParticleEditorLiquid>,
    particle_gas_field: &mut ResMut<ParticleEditorGas>,
    current_particle_category_field: &Res<State<ParticleEditorCategoryState>>,
) {
    ui.horizontal(|ui| {
        ui.label("Fluidity: ");
        particle_liquid_field.blueprint.0.fluidity;
        match current_particle_category_field.get() {
            ParticleEditorCategoryState::Liquid => {
                ui.add(
                    egui::Slider::new(&mut particle_liquid_field.blueprint.0.fluidity, 1..=5)
                        .step_by(1.),
                );
            }
            ParticleEditorCategoryState::Gas => {
                ui.add(
                    egui::Slider::new(&mut particle_gas_field.blueprint.0.fluidity, 1..=5)
                        .step_by(1.),
                );
            }
            _ => {}
        }
    });
}

fn particle_editor_save(
    (mut commands, mut ev_particle_editor_save): (Commands, EventReader<ParticleEditorSave>),
    particle_type_map: Res<ParticleTypeMap>,
    particle_type_query: Query<Option<&Children>, With<ParticleType>>,
    (
        current_particle_category_field,
        particle_selected_field,
        particle_density_field,
        particle_max_velocity_field,
        particle_momentum_field,
        particle_colors_field,
        particle_editor_movement_priority_field,
        particle_editor_burns_field,
        particle_editor_fire_field,
        particle_editor_wall_field,
        particle_editor_solid_field,
        particle_editor_movable_solid_field,
        particle_editor_liquid_field,
        particle_editor_gas_field,
    ): (
        Res<State<ParticleEditorCategoryState>>,
        Res<ParticleEditorSelectedType>,
        Res<ParticleEditorDensity>,
        Res<ParticleEditorMaxVelocity>,
        Res<ParticleEditorMomentum>,
        Res<ParticleEditorColors>,
        Res<ParticleEditorMovementPriority>,
        Res<ParticleEditorBurns>,
        Res<ParticleEditorFire>,
        Res<ParticleEditorWall>,
        Res<ParticleEditorSolid>,
        Res<ParticleEditorMovableSolid>,
        Res<ParticleEditorLiquid>,
        Res<ParticleEditorGas>,
    ),
) {
    ev_particle_editor_save.read().for_each(|_| {
        let entity = particle_type_map
            .get(&particle_selected_field.0.name)
            .cloned()
            .unwrap_or_else(|| {
                commands
                    .spawn(ParticleType::new(particle_selected_field.0.name.as_str()))
                    .id()
            });
        // TODO: Replace this with bundle matching so we only remove components relevant to bfs.
        commands.entity(entity).remove::<ParticleBundle>();
        match current_particle_category_field.get() {
            ParticleEditorCategoryState::Wall => {
                commands.entity(entity).insert((
                    particle_editor_wall_field.blueprint.clone(),
                    particle_colors_field.blueprint.clone(),
                ));
            }
            ParticleEditorCategoryState::Solid => {
                commands.entity(entity).insert((
                    particle_editor_solid_field.blueprint.clone(),
                    particle_colors_field.blueprint.clone(),
                    particle_density_field.blueprint,
                    particle_max_velocity_field.blueprint,
                ));
                if particle_editor_burns_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_editor_burns_field.blueprint.clone());
                }
                if particle_editor_fire_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_editor_fire_field.blueprint.clone());
                }
            }
            ParticleEditorCategoryState::MovableSolid => {
                commands.entity(entity).insert((
                    particle_editor_movable_solid_field.blueprint.clone(),
                    particle_colors_field.blueprint.clone(),
                    particle_density_field.blueprint,
                    particle_max_velocity_field.blueprint,
                ));
                if particle_momentum_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_momentum_field.blueprint.clone());
                }
                if particle_editor_burns_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_editor_burns_field.blueprint.clone());
                }
                if particle_editor_fire_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_editor_fire_field.blueprint.clone());
                }
            }
            ParticleEditorCategoryState::Liquid => {
                commands.entity(entity).insert((
                    particle_editor_liquid_field.blueprint.clone(),
                    particle_colors_field.blueprint.clone(),
                    particle_density_field.blueprint,
                    particle_max_velocity_field.blueprint,
                ));
                if particle_momentum_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_momentum_field.blueprint.clone());
                }
                if particle_editor_burns_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_editor_burns_field.blueprint.clone());
                }
                if particle_editor_fire_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_editor_fire_field.blueprint.clone());
                }
            }
            ParticleEditorCategoryState::Gas => {
                commands.entity(entity).insert((
                    particle_editor_gas_field.blueprint.clone(),
                    particle_colors_field.blueprint.clone(),
                    particle_density_field.blueprint,
                    particle_max_velocity_field.blueprint,
                ));
                if particle_momentum_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_momentum_field.blueprint.clone());
                }
                if particle_editor_burns_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_editor_burns_field.blueprint.clone());
                }
                if particle_editor_fire_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_editor_fire_field.blueprint.clone());
                }
            }
            ParticleEditorCategoryState::Other => {
                commands.entity(entity).insert((
                    particle_colors_field.blueprint.clone(),
                    particle_density_field.blueprint,
                    particle_max_velocity_field.blueprint,
                    particle_editor_movement_priority_field.blueprint.clone(),
                ));
                if particle_momentum_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_momentum_field.blueprint.clone());
                }
                if particle_editor_burns_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_editor_burns_field.blueprint.clone());
                }
                if particle_editor_fire_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_editor_fire_field.blueprint.clone());
                }
            }
        }
        if let Ok(children) = particle_type_query.get(entity) {
            if let Some(children) = children {
                children
                    .iter()
                    .for_each(|child| commands.trigger(ResetParticleEvent { entity: *child }));
            }
        }
    })
}

#[derive(Event, Clone, Debug)]
pub struct ParticleEditorUpdate;

#[derive(Event, Clone, Debug)]
pub struct ParticleEditorSave;

#[derive(Resource, Clone)]
pub struct ParticleEditorSelectedType(pub ParticleType);

#[derive(Resource, Default, Clone)]
pub struct ParticleEditorName(pub String);

impl Default for ParticleEditorSelectedType {
    fn default() -> Self {
        ParticleEditorSelectedType(ParticleType::new("Dirt Wall"))
    }
}

#[derive(Default, Resource, Clone)]
pub struct ParticleEditorDensity {
    blueprint: DensityBlueprint,
}

#[derive(Default, Resource, Clone)]
pub struct ParticleEditorMaxVelocity {
    blueprint: VelocityBlueprint,
}

#[derive(Default, Resource, Clone)]
pub struct ParticleEditorMomentum {
    enable: bool,
    blueprint: MomentumBlueprint,
}

#[derive(Resource, Clone, Debug)]
pub struct ParticleEditorColors {
    blueprint: ParticleColorBlueprint,
}

impl Default for ParticleEditorColors {
    fn default() -> Self {
        ParticleEditorColors {
            blueprint: ParticleColorBlueprint(ParticleColor::new(
                Color::srgba_u8(255, 255, 255, 255),
                vec![Color::srgba_u8(255, 255, 255, 255)],
            )),
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
    blueprint: MovementPriorityBlueprint,
}

impl Default for ParticleEditorMovementPriority {
    fn default() -> Self {
        ParticleEditorMovementPriority {
            blueprint: MovementPriorityBlueprint(MovementPriority::empty()),
        }
    }
}

#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleEditorBurns {
    enable: bool,
    chance_destroy_enable: bool,
    reaction_enable: bool,
    color_enable: bool,
    spreads_enable: bool,
    blueprint: BurnsBlueprint,
}

#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleEditorFire {
    enable: bool,
    blueprint: FireBlueprint,
}

#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleEditorWall {
    blueprint: WallBlueprint,
}

#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleEditorSolid {
    blueprint: SolidBlueprint,
}

#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleEditorMovableSolid {
    blueprint: MovableSolidBlueprint,
}

#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleEditorLiquid {
    blueprint: LiquidBlueprint,
}

#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleEditorGas {
    blueprint: GasBlueprint,
}
