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
use bevy_egui::{egui, egui::Color32, EguiContexts};
use bevy_falling_sand::prelude::*;

use super::*;

pub(super) struct UIPlugin;

const DEFAULT_SELECTED_PARTICLE: &str = "Dirt Wall";

impl bevy::prelude::Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<AppState>()
            .init_resource::<CursorCoords>()
            .init_resource::<ParticleList>()
            .init_resource::<ParticleTypeList>()
            .init_resource::<SelectedBrushParticle>()
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
            .add_systems(First, update_cursor_coordinates)
            .add_systems(
                Update,
                (
                    render_side_panel,
                    render_particle_editor,
                    render_search_bar_ui.run_if(resource_exists::<ParticleSearchBar>),
                    update_particle_list,
                    update_app_state.after(render_side_panel),
                    toggle_simulation.run_if(input_just_pressed(KeyCode::Space)),
                    ev_mouse_wheel,
                    handle_search_bar_input,
                    particle_editor_save,
                    update_particle_editor_fields,
                ),
            )
            .add_systems(OnEnter(AppState::Ui), show_cursor)
            .add_systems(OnEnter(AppState::Canvas), hide_cursor)
            .add_observer(on_clear_dynamic_particles)
            .add_observer(on_clear_wall_particles);
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

#[derive(Resource, Default)]
pub struct ParticleList {
    pub particle_list: Vec<String>,
}

impl ParticleList {
    pub fn push(&mut self, value: String) {
        self.particle_list.push(value);
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
        commands: &mut Commands,
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
            });
        if ui
            .add(egui::Slider::new(brush_size, 1..=max_brush_size))
            .changed()
        {
            ev_brush_resize.send(BrushResizeEvent(*brush_size));
        }
    }
}

pub struct DebugUI;

impl DebugUI {
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

pub fn update_cursor_coordinates(
    mut coords: ResMut<CursorCoords>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();

    let window = q_window.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| Some(camera.viewport_to_world(camera_transform, cursor)))
        .map(|ray| ray.unwrap().origin.truncate())
    {
        coords.update(world_position);
    }
}

pub fn hide_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    let window = &mut primary_window.single_mut();
    window.cursor_options.visible = false;
}

pub fn show_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    let window = &mut primary_window.single_mut();
    window.cursor_options.visible = true;
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
            ParticleControlUI.render(ui, &mut brush_state, &mut commands);
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

#[derive(Event)]
pub struct ClearDynamicParticlesEvent;

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
            Option<&ColorProfileBlueprint>,
            Option<&BurnsBlueprint>,
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
                particle_editor_name.0 = particle_editor_selected_type.0.name.clone();
                particle_editor_selected_type.0 =
                    ParticleType::new(particle_editor_selected_type.0.name.clone().as_str());
                if let Some(density) = density {
                    particle_density_field.blueprint = *density;
                }
                if let Some(velocity) = velocity {
                    particle_max_velocity_field.blueprint = *velocity;
                }
                if let Some(_) = momentum {
                    particle_momentum_field.enable = true;
                }
                if let Some(colors) = colors {
                    particle_colors_field.blueprint = colors.clone()
                }
                if let Some(burns) = burns {
                    particle_editor_burns_field.enable = true;
                    particle_editor_burns_field.chance_destroy_enable = burns
                        .0
                        .chance_destroy_per_tick
                        .map(|_| true)
                        .unwrap_or(false);
                    particle_editor_burns_field.reaction_enable =
                        burns.0.reaction.as_ref().map(|_| true).unwrap_or(false);
                    particle_editor_burns_field.color_enable =
                        burns.0.color.as_ref().map(|_| true).unwrap_or(false);
                    particle_editor_burns_field.spreads_enable =
                        burns.0.spreads.as_ref().map(|_| true).unwrap_or(false);
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
                                    ui.text_edit_singleline(
                                        &mut particle_editor_selected_field.0.name,
                                    );
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
                                        ui.separator();
                                        render_burns_field(
                                            ui,
                                            &mut particle_editor_burns_field,
                                            &particle_list,
                                        );
                                        ui.separator();
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
                                        render_flows_color_field(
                                            ui,
                                            &mut particle_editor_flows_color_field,
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
                                        render_burns_field(
                                            ui,
                                            &mut particle_editor_burns_field,
                                            &particle_list,
                                        );
                                        ui.separator();
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
        ui.add(
            egui::Slider::new(&mut particle_density_field.blueprint.data_mut().0, 1..=1000)
                .step_by(1.),
        );
    });
}

fn render_max_velocity_field(
    ui: &mut egui::Ui,
    particle_max_velocity_field: &mut ResMut<ParticleEditorMaxVelocity>,
) {
    ui.horizontal(|ui| {
        ui.label("Max Velocity: ");
        ui.add(
            egui::Slider::new(
                &mut particle_max_velocity_field.blueprint.data_mut().max,
                1..=5,
            )
            .step_by(1.),
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
    for (i, color) in particle_colors_field
        .blueprint
        .data()
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
        let mut color32 =
            Color32::from_rgba_unmultiplied(red as u8, green as u8, blue as u8, alpha as u8);
        ui.horizontal(|ui| {
            ui.label(format!("R: {}", red));
            ui.label(format!("G: {}", green));
            ui.label(format!("B: {}", blue));
            ui.label(format!("A: {}", alpha));
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
        particle_colors_field
            .blueprint
            .data_mut()
            .palette
            .remove(to_remove);
    }
    if let Some((to_change, color)) = to_change {
        particle_colors_field.blueprint.data_mut().palette[to_change] = color;
    }
}

fn render_flows_color_field(
    ui: &mut egui::Ui,
    particle_flows_color_field: &mut ResMut<ParticleEditorFlowsColor>,
) {
    ui.add(egui::Checkbox::new(
        &mut particle_flows_color_field.enable,
        "Flows Color",
    ));
    if particle_flows_color_field.enable {
        ui.horizontal(|ui| {
            ui.label("Rate: ");
            ui.add(egui::Slider::new(
                &mut particle_flows_color_field.blueprint.data_mut().rate,
                0.0..=1.0,
            ));
        });
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
                && i < particle_movement_priority_field.blueprint.data().len() - 1
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
        if let Some(group) = particle_movement_priority_field
            .blueprint
            .data_mut()
            .get_mut(i)
        {
            group.neighbor_group.remove(j);
        }
    }
    if let Some((i, j1, j2)) = inner_to_swap {
        if let Some(group) = particle_movement_priority_field
            .blueprint
            .data_mut()
            .get_mut(i)
        {
            group.swap(j1, j2).unwrap_or_else(|err| error!("{}", err));
        }
    }
    if let Some((i, j)) = outer_to_swap {
        particle_movement_priority_field
            .blueprint
            .0
            .swap_outer(i, j)
            .unwrap_or_else(|err| error!("{}", err));
    }
    if let Some(i) = outer_to_add {
        if let Some(group) = particle_movement_priority_field
            .blueprint
            .data_mut()
            .get_mut(i)
        {
            group.push(IVec2::ZERO);
        }
    }
    if let Some(i) = outer_to_remove {
        particle_movement_priority_field
            .blueprint
            .data_mut()
            .remove(i);
    }
    if let Some(((i, j), new_ivec)) = to_change {
        if let Some(group) = particle_movement_priority_field
            .blueprint
            .data_mut()
            .get_mut(i)
        {
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
            let edit_duration = ui.add(
                egui::TextEdit::singleline(&mut particle_burns_field.duration_str)
                    .desired_width(40.),
            );
            if edit_duration.lost_focus() {
                if let Ok(new_duration) = particle_burns_field.duration_str.parse::<u64>() {
                    particle_burns_field.blueprint.data_mut().duration =
                        Duration::from_millis(new_duration);
                    particle_burns_field.duration_str = particle_burns_field
                        .blueprint
                        .0
                        .duration
                        .as_millis()
                        .to_string();
                } else {
                    particle_burns_field.blueprint.data_mut().duration = Duration::from_millis(0);
                    particle_burns_field.duration_str = particle_burns_field
                        .blueprint
                        .0
                        .duration
                        .as_millis()
                        .to_string();
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Tick Rate (ms): ");
            let edit_duration = ui.add(
                egui::TextEdit::singleline(&mut particle_burns_field.tick_rate_str)
                    .desired_width(40.),
            );
            if edit_duration.lost_focus() {
                if let Ok(new_duration) = particle_burns_field.tick_rate_str.parse::<u64>() {
                    particle_burns_field.blueprint.data_mut().tick_rate =
                        Duration::from_millis(new_duration);
                    particle_burns_field.tick_rate_str = particle_burns_field
                        .blueprint
                        .0
                        .duration
                        .as_millis()
                        .to_string();
                } else {
                    particle_burns_field.blueprint.data_mut().tick_rate = Duration::from_millis(0);
                    particle_burns_field.tick_rate_str = particle_burns_field
                        .blueprint
                        .0
                        .duration
                        .as_millis()
                        .to_string();
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
                particle_burns_field.blueprint.data_mut().color = Some(ColorProfile::default())
            } else {
                particle_burns_field.blueprint.data_mut().color = None;
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
                let mut color32 = Color32::from_rgba_unmultiplied(
                    red as u8,
                    green as u8,
                    blue as u8,
                    alpha as u8,
                );
                ui.horizontal(|ui| {
                    ui.label(format!("R: {}", red));
                    ui.label(format!("G: {}", green));
                    ui.label(format!("B: {}", blue));
                    ui.label(format!("A: {}", alpha));
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
                particle_burns_field
                    .blueprint
                    .data_mut()
                    .chance_destroy_per_tick = Some(0.);
            } else {
                particle_burns_field
                    .blueprint
                    .data_mut()
                    .chance_destroy_per_tick = None;
            }
        };
        if particle_burns_field.chance_destroy_enable {
            ui.horizontal(|ui| {
                ui.label("Chance");
                ui.add(egui::Slider::new(
                    &mut particle_burns_field
                        .blueprint
                        .0
                        .chance_destroy_per_tick
                        .unwrap(),
                    0.0..=1.0,
                ));
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
                particle_burns_field.blueprint.data_mut().reaction =
                    Some(Reacting::new(Particle::new("Water"), 0.1));
            } else {
                particle_burns_field.blueprint.data_mut().reaction = None;
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
                ui.add(egui::Slider::new(
                    &mut particle_burns_field
                        .blueprint
                        .0
                        .reaction
                        .as_mut()
                        .unwrap()
                        .chance_to_produce,
                    0.0..=1.0,
                ));
            });
        }
        if ui
            .add(egui::Checkbox::new(
                &mut particle_burns_field.spreads_enable,
                "Fire Spreads",
            ))
            .clicked()
        {
            if particle_burns_field.spreads_enable {
                particle_burns_field.blueprint.data_mut().spreads = Some(Fire {
                    burn_radius: 2.,
                    chance_to_spread: 0.01,
                    destroys_on_spread: false,
                });
            } else {
                particle_burns_field.blueprint.data_mut().spreads = None;
            }
        }
        if particle_burns_field.spreads_enable {
            ui.horizontal(|ui| {
                ui.label("Burn Radius");
                ui.add(egui::Slider::new(
                    &mut particle_burns_field
                        .blueprint
                        .0
                        .spreads
                        .as_mut()
                        .unwrap()
                        .burn_radius,
                    1.0..=100.0,
                ));
            });
            ui.horizontal(|ui| {
                ui.label("Chance to spread");
                ui.add(egui::Slider::new(
                    &mut particle_burns_field
                        .blueprint
                        .0
                        .spreads
                        .as_mut()
                        .unwrap()
                        .chance_to_spread,
                    0.0..=1.0,
                ));
            });
            ui.add(egui::Checkbox::new(
                &mut particle_burns_field
                    .blueprint
                    .0
                    .spreads
                    .as_mut()
                    .unwrap()
                    .destroys_on_spread,
                "Destroys on spread",
            ));
            ui.add(egui::Checkbox::new(
                &mut particle_burns_field.spawns_on_fire,
                "Spawns on fire",
            ));
        }
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
        particle_liquid_field.blueprint.data().fluidity;
        match current_particle_category_field.get() {
            ParticleEditorCategoryState::Liquid => {
                ui.add(
                    egui::Slider::new(
                        &mut particle_liquid_field.blueprint.data_mut().fluidity,
                        1..=5,
                    )
                    .step_by(1.),
                );
            }
            ParticleEditorCategoryState::Gas => {
                ui.add(
                    egui::Slider::new(&mut particle_gas_field.blueprint.data_mut().fluidity, 1..=5)
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
        particle_editor_flows_color_field,
        particle_editor_movement_priority_field,
        particle_editor_burns_field,
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
        Res<ParticleEditorFlowsColor>,
        Res<ParticleEditorMovementPriority>,
        Res<ParticleEditorBurns>,
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
        commands.entity(entity).remove::<ParticleBundle>();
        match current_particle_category_field.get() {
            ParticleEditorCategoryState::Wall => {
                commands.entity(entity).insert((
                    particle_editor_wall_field.blueprint.clone(),
                    particle_colors_field.blueprint.clone(),
                ));
                if particle_editor_burns_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_editor_burns_field.blueprint.clone());
                    if particle_editor_burns_field.spawns_on_fire {
                        commands
                            .entity(entity)
                            .insert(particle_editor_burns_field.blueprint.data().to_burning());
                    }
                }
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
                    if particle_editor_burns_field.spawns_on_fire {
                        commands
                            .entity(entity)
                            .insert(particle_editor_burns_field.blueprint.data().to_burning());
                    }
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
                    if particle_editor_burns_field.spawns_on_fire {
                        commands
                            .entity(entity)
                            .insert(particle_editor_burns_field.blueprint.data().to_burning());
                    }
                }
            }
            ParticleEditorCategoryState::Liquid => {
                commands.entity(entity).insert((
                    particle_editor_liquid_field.blueprint.clone(),
                    particle_colors_field.blueprint.clone(),
                    particle_density_field.blueprint,
                    particle_max_velocity_field.blueprint,
                ));
                if particle_editor_flows_color_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_editor_flows_color_field.blueprint);
                }
                if particle_momentum_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_momentum_field.blueprint.clone());
                }
                if particle_editor_burns_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_editor_burns_field.blueprint.clone());
                    if particle_editor_burns_field.spawns_on_fire {
                        commands
                            .entity(entity)
                            .insert(particle_editor_burns_field.blueprint.data().to_burning());
                    }
                }
            }
            ParticleEditorCategoryState::Gas => {
                commands.entity(entity).insert((
                    particle_editor_gas_field.blueprint.clone(),
                    particle_colors_field.blueprint.clone(),
                    particle_density_field.blueprint,
                    particle_max_velocity_field.blueprint,
                ));
                if particle_editor_flows_color_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_editor_flows_color_field.blueprint);
                }
                if particle_editor_burns_field.enable {
                    commands
                        .entity(entity)
                        .insert(particle_editor_burns_field.blueprint.clone());
                    if particle_editor_burns_field.spawns_on_fire {
                        commands
                            .entity(entity)
                            .insert(particle_editor_burns_field.blueprint.data().to_burning());
                    }
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
                    if particle_editor_burns_field.spawns_on_fire {
                        commands
                            .entity(entity)
                            .insert(particle_editor_burns_field.blueprint.data().to_burning());
                    }
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
        ParticleEditorSelectedType(ParticleType::new(DEFAULT_SELECTED_PARTICLE))
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
    blueprint: ColorProfileBlueprint,
}

impl Default for ParticleEditorColors {
    fn default() -> Self {
        ParticleEditorColors {
            blueprint: ColorProfileBlueprint(ColorProfile::default()),
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct ParticleEditorFlowsColor {
    enable: bool,
    blueprint: ChangesColorBlueprint,
}

impl Default for ParticleEditorFlowsColor {
    fn default() -> Self {
        ParticleEditorFlowsColor {
            enable: true,
            blueprint: ChangesColorBlueprint(ChangesColor::new(0.1)),
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
    blueprint: BurnsBlueprint,
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
            blueprint: BurnsBlueprint::default(),
        }
    }
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
