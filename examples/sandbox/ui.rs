//! UI module.
use bevy::{
    input::{common_conditions::input_just_pressed, mouse::MouseWheel},
    prelude::*,
    utils::{Entry, HashMap},
    window::PrimaryWindow,
};
use bevy_egui::{EguiContext, EguiContexts};

use bevy_falling_sand::core::{ClearMapEvent, ParticleType, SimulationRun};
use bevy_falling_sand::debug::{DebugParticles, TotalParticleCount};
use bevy_falling_sand::movement::*;
use bevy_falling_sand::scenes::{LoadSceneEvent, SaveSceneEvent};

use super::*;

/// UI plugin
pub(super) struct UIPlugin;

impl bevy::prelude::Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<AppState>()
            .add_systems(Update, render_ui)
            .add_systems(Update, update_particle_list)
            .add_systems(Update, update_app_state.after(render_ui))
            .add_systems(
                Update,
                toggle_simulation.run_if(input_just_pressed(KeyCode::Space)),
            )
            .init_resource::<CursorCoords>()
            .init_resource::<ParticleList>()
            .init_resource::<ParticleTypeList>()
            .init_resource::<DebugParticles>()
            .init_resource::<SelectedParticle>()
            .add_systems(First, update_cursor_coordinates)
            .add_systems(OnEnter(AppState::Ui), show_cursor)
            .add_systems(OnEnter(AppState::Canvas), hide_cursor)
            .add_systems(Update, ev_mouse_wheel)
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
}

/// The currently selected particle for spawning.
#[derive(Resource)]
pub struct SelectedParticle(pub String);

impl Default for SelectedParticle {
    fn default() -> SelectedParticle {
        SelectedParticle("Dirt Wall".to_string())
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
    keys: Res<ButtonInput<KeyCode>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let ctx = contexts.ctx_mut();
    match app_state.get() {
        AppState::Ui => {
            if !ctx.is_pointer_over_area() || keys.pressed(KeyCode::AltLeft) {
                next_app_state.set(AppState::Canvas);
            }
        }
        AppState::Canvas => {
            if ctx.is_pointer_over_area() || !keys.pressed(KeyCode::AltLeft) {
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

pub fn update_particle_list(
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
pub fn toggle_simulation(mut commands: Commands, simulation_pause: Option<Res<SimulationRun>>) {
    if simulation_pause.is_some() {
        commands.remove_resource::<SimulationRun>();
    } else {
        commands.init_resource::<SimulationRun>();
    }
}

/// Listens for scroll events and performs the corresponding action
pub fn ev_mouse_wheel(
    mut ev_scroll: EventReader<MouseWheel>,
    app_state: Res<State<AppState>>,
    mut camera_query: Query<&mut OrthographicProjection, With<MainCamera>>,
    mut brush_query: Query<&mut Brush>,
    max_brush_size: Res<MaxBrushSize>
) {
    if !ev_scroll.is_empty() {
        match app_state.get() {
            AppState::Ui => {
                let mut projection = camera_query.single_mut();
                ev_scroll.read().for_each(|ev| {
                    let zoom = -(ev.y / 100.);
                    if projection.scale + zoom > 0.01 {
                        projection.scale += zoom;
                    }
                });
            }
            AppState::Canvas => {
                let mut brush = brush_query.single_mut();
                ev_scroll.read().for_each(|ev| {
                    if ev.y < 0. && brush.size - 1 >= 1 {
                        brush.size -= 1;
                    } else if ev.y > 0. && brush.size + 1 <= max_brush_size.0 {
                        brush.size += 1;
                    }
                });
            }
        };
    }
}
