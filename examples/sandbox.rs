use std::fs::File;
use std::path::PathBuf;

use bevy::{
    input::common_conditions::{input_just_pressed, input_pressed},
    input::mouse::MouseWheel,
    prelude::*,
    utils::HashSet,
    window::{PrimaryWindow, WindowMode},
};

use bevy_egui::{EguiContexts, EguiPlugin};

use bevy_falling_sand::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Falling Sand Playground".into(),
            mode: WindowMode::BorderlessFullscreen,
            ..default()
        }),
        ..default()
    }));

    // Plugins
    app.add_plugins(FallingSandPlugin).add_plugins(EguiPlugin);

    // Resources
    app.init_resource::<DebugParticles>()
        .init_resource::<SelectedParticle>()
        .init_resource::<ParticleTypes>()
        .init_resource::<CursorCoords>()
        .init_resource::<MaxBrushSize>()
        .init_resource::<ParticleSceneFilePath>()
        .init_resource::<SceneSelectionDialog>();

    // States
    app.init_state::<BrushState>()
        .init_state::<AppState>()
        .init_state::<BrushType>();

    // Gizmos
    app.init_gizmo_group::<BrushGizmos>();

    // Events
    app.add_event::<BrushResizeEvent>()
        .add_event::<CanvasResetEvent>();

    // Camera control
    app.add_systems(Startup, setup_camera).add_systems(
        Update,
        (zoom_camera, pan_camera).run_if(in_state(AppState::Canvas)),
    );

    // UI
    app.add_systems(Update, render_ui);
    app.add_systems(OnEnter(AppState::Ui), show_cursor);
    app.add_systems(OnEnter(AppState::Canvas), hide_cursor);

    // Brush systems
    app.add_systems(Startup, setup_brush)
        .add_systems(Update, (update_brush, resize_brush_event_listener));

    app.add_systems(Update, update_cursor_coordinates);
    app.add_systems(Update, update_app_state.after(render_ui));

    // Particle management systems
    app.add_systems(
        Update,
        toggle_simulation.run_if(input_just_pressed(KeyCode::Space)),
    );
    app.add_systems(
        Update,
        (spawn_particles
            .run_if(input_pressed(MouseButton::Left))
            .run_if(in_state(BrushState::Spawn))
            .run_if(in_state(AppState::Canvas))
            .after(update_cursor_coordinates)
            .after(render_ui),),
    );
    app.add_systems(
        Update,
        despawn_particles
            .run_if(input_pressed(MouseButton::Left))
            .run_if(in_state(BrushState::Despawn))
            .run_if(in_state(AppState::Canvas))
            .before(handle_particles)
            .after(update_cursor_coordinates)
            .after(render_ui),
    );

    // Use built-in systems for saving and loading particle scenes

    app.run();
}

/// Guarantees our particle type buttons are presented in a specific order.
#[derive(Resource)]
struct ParticleTypes {
    particle_types: Vec<String>,
}

impl Default for ParticleTypes {
    fn default() -> ParticleTypes {
        let file_path = "assets/particles/particles.ron";
        let file = File::open(file_path).unwrap();
        let particle_types_map: ron::Map = ron::de::from_reader(file).unwrap();

        let particle_types: Vec<String> = particle_types_map
            .keys()
            .map(|key| key.clone().into_rust::<String>().unwrap())
            .collect();
        ParticleTypes { particle_types }
    }
}

impl ParticleTypes {
    fn iter(&self) -> impl Iterator<Item = &String> {
        self.particle_types.iter()
    }
}

#[derive(Resource)]
struct SelectedParticle(String);

impl Default for SelectedParticle {
    fn default() -> SelectedParticle {
        SelectedParticle("Dirt Wall".to_string())
    }
}
#[derive(Component)]
struct MainCamera;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct BrushGizmos;

#[derive(Component)]
#[allow(dead_code)]
struct Brush {
    size: usize,
    color: Color,
}

impl Brush {
    fn new(size: usize, color: Color) -> Self {
        Brush { size, color }
    }
}
impl Default for Brush {
    fn default() -> Self {
        Brush {
            size: 80,
            color: Color::WHITE,
        }
    }
}

#[derive(Reflect, Resource)]
struct MaxBrushSize(usize);

#[derive(Resource, Default, Debug)]
struct CursorCoords(Vec2);

impl Default for MaxBrushSize {
    fn default() -> Self {
        return MaxBrushSize(50);
    }
}

#[derive(Default, Clone, Hash, Eq, PartialEq, Debug, States)]
enum BrushType {
    Line,
    #[default]
    Circle,
    Square,
}

impl BrushType {
    fn update_brush(
        &self,
        coords: Vec2,
        brush_size: f32,
        brush_gizmos: &mut Gizmos<BrushGizmos>,
    ) {
        match self {
            BrushType::Line => brush_gizmos.line_2d(
                Vec2::new(coords.x - brush_size * 3. / 2., coords.y),
                Vec2::new(coords.x + brush_size * 3. / 2., coords.y),
                Color::WHITE,
            ),
            BrushType::Circle => {
                brush_gizmos.circle_2d(coords, brush_size, Color::WHITE);
            }
            BrushType::Square => {
                brush_gizmos.rect_2d(coords, 0., Vec2::splat(brush_size), Color::WHITE);
            }
        }
    }

    fn spawn_particles(
        &self,
        commands: &mut Commands,
        coords: Vec2,
        brush_size: f32,
        selected_particle: ParticleType,
    ) {
        let min_x = -(brush_size as i32) / 2;
        let max_x = (brush_size / 2.) as i32;
        let min_y = -(brush_size as i32) / 2;
        let max_y = (brush_size / 2.) as i32;

        match self {
            BrushType::Line => {
                let particle = selected_particle.clone();
                commands.spawn_batch((min_x * 3..=max_x * 3).map(move |x| {
                    (
                        particle.clone(), // Clone the particle for each x iteration
                        SpatialBundle::from_transform(Transform::from_xyz(
                            coords.x + x as f32,
                            coords.y,
                            0.0,
                        )),
                    )
                }));
            }
            BrushType::Circle => {
                let particle = selected_particle.clone();
                let mut points: HashSet<IVec2> = HashSet::default();
                let circle = Circle::new(brush_size);
                for x in min_x * 2..=max_x * 2 {
                    for y in min_y * 2..=max_y * 2 {
                        let mut point = Vec2::new(x as f32, y as f32);
                        point = circle.closest_point(point);
                        points.insert((point + coords).as_ivec2());
                    }
                }
                commands.spawn_batch(points.into_iter().map(move |point| {
                    (
                        particle.clone(), // Clone the particle for each point iteration
                        SpatialBundle::from_transform(Transform::from_xyz(
                            point.x as f32,
                            point.y as f32,
                            0.,
                        )),
                    )
                }));
            }
            BrushType::Square => {
                let particle = selected_particle.clone();
                commands.spawn_batch((min_x..=max_x).flat_map(move |x| {
                    let particle = particle.clone(); // Clone the particle for each x iteration
                    (min_y..=max_y).map(move |y| {
                        (
                            particle.clone(), // Clone the particle for each y iteration
                            SpatialBundle::from_transform(Transform::from_xyz(
                                coords.x + x as f32,
                                coords.y + y as f32,
                                0.,
                            )),
                        )
                    })
                }));
            }
        }
    }

    fn remove_particles(&self, commands: &mut Commands, coords: IVec2, brush_size: f32) {
        let min_x = -(brush_size as i32) / 2;
        let max_x = (brush_size / 2.) as i32;
        let min_y = -(brush_size as i32) / 2;
        let max_y = (brush_size / 2.) as i32;

        match self {
            BrushType::Line => {
                for x in min_x * 3..=max_x * 3 {
                    let coordinates = IVec2::new(coords.x + x, coords.y);
                    commands.trigger(RemoveParticle { coordinates });
                }
            }
            BrushType::Circle => {
                let mut circle_coords: HashSet<IVec2> = HashSet::default();
                let circle = Circle::new(brush_size);
                for x in min_x * 2..=max_x * 2 {
                    for y in min_y * 2..=max_y * 2 {
                        let mut coordinates = Vec2::new(x as f32, y as f32);
                        coordinates = circle.closest_point(coordinates);
                        circle_coords.insert((coordinates + coords.as_vec2()).as_ivec2());
                    }
                }
                for coordinates in circle_coords {
                    commands.trigger(RemoveParticle { coordinates })
                }
            }
            BrushType::Square => {
                for x in min_x..=max_x {
                    for y in min_y..=max_y {
                        let coordinates = IVec2::new(coords.x + x, coords.y + y);
                        commands.trigger(RemoveParticle { coordinates })
                    }
                }
            }
        }
    }
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum BrushState {
    #[default]
    Spawn,
    Despawn,
}

#[derive(Event)]
struct BrushResizeEvent(usize);

// Triggers the removal of all particles when the corresponding UI button is clicked
#[derive(Event)]
struct CanvasResetEvent;

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    Canvas,
    Ui,
}

#[derive(Resource, Default)]
struct SceneSelectionDialog {
    show_save_dialog: bool,
    show_load_dialog: bool,
    save_input_text: String,
    load_input_text: String,
}

#[derive(Resource)]
struct ParticleSceneFilePath(PathBuf);

impl Default for ParticleSceneFilePath {
    fn default() -> ParticleSceneFilePath {
        let mut path = std::env::current_dir().unwrap();

        while path.parent().is_some() {
            if path.join("Cargo.toml").exists() {
                path.push("assets/scenes/hourglass.ron");
                break;
            }
            path.pop();
        }

        ParticleSceneFilePath(path)
    }
}

struct ParticleControlUI;
struct BrushControlUI;
struct SceneManagementUI;
struct DebugUI;

impl ParticleControlUI {
    fn render(
        &self,
        ui: &mut egui::Ui,
        particle_types: &ParticleTypes,
        selected_particle: &mut SelectedParticle,
        brush_state: &mut ResMut<NextState<BrushState>>,
        commands: &mut Commands,
    ) {
        ui.horizontal_wrapped(|ui| {
            particle_types.iter().for_each(|particle| {
                if ui.button(particle).clicked() {
                    selected_particle.0 = particle.clone();
                    brush_state.set(BrushState::Spawn);
                }
            });
            if ui.button("Remove").clicked() {
                brush_state.set(BrushState::Despawn);
            }
        });

        ui.separator();

        if ui.button("Despawn All Particles").clicked() {
            commands.trigger(ClearChunkMap);
        }
    }
}

impl BrushControlUI {
    fn render(
        &self,
        ui: &mut egui::Ui,
        brush_size: &mut usize,
        max_brush_size: usize,
        ev_brush_resize: &mut EventWriter<BrushResizeEvent>,
        mut current_brush_type: &BrushType,
        next_brush_type: &mut ResMut<NextState<BrushType>>,
    ) {
        if ui
            .add(egui::Slider::new(brush_size, 1..=max_brush_size))
            .changed()
        {
            ev_brush_resize.send(BrushResizeEvent(*brush_size));
        }

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
                    .selectable_value(&mut current_brush_type, &BrushType::Square, "Square")
                    .changed()
                {
                    next_brush_type.set(BrushType::Square)
                };
            });
    }
}

impl SceneManagementUI {
    fn render(
        &self,
        ui: &mut egui::Ui,
        dialog_state: &mut ResMut<SceneSelectionDialog>,
        scene_path: &mut ResMut<ParticleSceneFilePath>,
        ev_save_scene: &mut EventWriter<SaveSceneEvent>,
        ev_load_scene: &mut EventWriter<LoadSceneEvent>,
    ) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("SAVE SCENE").clicked() {
                dialog_state.show_save_dialog = true;
            }

            if ui.button("LOAD SCENE").clicked() {
                dialog_state.show_load_dialog = true;
            }
        });

        if dialog_state.show_save_dialog {
            egui::Window::new("Save Scene")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Enter a name to save the current scene:");
                    ui.text_edit_singleline(&mut dialog_state.save_input_text);
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Save").clicked() {
                            let mut file_name = dialog_state.save_input_text.clone();
                            if !file_name.ends_with(".ron") {
                                file_name.push_str(".ron");
                            }
                            scene_path.0.set_file_name(file_name);
                            ev_save_scene.send(SaveSceneEvent(scene_path.0.clone()));
                            dialog_state.show_save_dialog = false; // Close after saving
                        }
                        if ui.button("Cancel").clicked() {
                            dialog_state.show_save_dialog = false;
                        }
                    });
                });
        }

        if dialog_state.show_load_dialog {
            // Fetch all `.ron` files in the directory
            let ron_files: Vec<String> = std::fs::read_dir(&scene_path.0.parent().unwrap())
                .unwrap()
                .filter_map(|entry| {
                    let path = entry.unwrap().path();
                    if path.extension() == Some(std::ffi::OsStr::new("ron")) {
                        path.file_name()
                            .and_then(|name| name.to_str().map(String::from))
                    } else {
                        None
                    }
                })
                .collect();

            egui::Window::new("Load Scene")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Select the scene to load:");

                    egui::ComboBox::from_label("Available Scenes")
                        .selected_text(dialog_state.load_input_text.clone())
                        .show_ui(ui, |ui| {
                            for file_name in &ron_files {
                                let display_name =
                                    file_name.strip_suffix(".ron").unwrap_or(file_name);
                                if ui
                                    .selectable_value(
                                        &mut dialog_state.load_input_text,
                                        file_name.clone(),
                                        display_name,
                                    )
                                    .changed()
                                {
                                    // Automatically update the scene path when a file is selected
                                    scene_path.0.set_file_name(file_name.clone());
                                }
                            }
                        });

                    if ui.button("Load").clicked() {
                        ev_load_scene.send(LoadSceneEvent(scene_path.0.clone()));
                        dialog_state.show_load_dialog = false; // Close after loading
                    }
                    if ui.button("Cancel").clicked() {
                        dialog_state.show_load_dialog = false;
                    }
                });
        }
    }
}

impl DebugUI {
    fn render(
        &self,
        ui: &mut egui::Ui,
        debug_particles: &Option<Res<DebugParticles>>,
        dynamic_particle_count: u64,
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
            ui.label(format!("Dynamic Particles: {}", dynamic_particle_count));
            ui.label(format!("Total Particles: {}", total_particle_count));
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            projection: OrthographicProjection {
                near: -1000.0,
                scale: 0.1,
                ..default()
            },
            ..default()
        },
        MainCamera,
    ));
}

fn zoom_camera(
    mut scroll_evr: EventReader<MouseWheel>,
    mut camera_query: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    let mut projection = camera_query.single_mut();
    for ev in scroll_evr.read() {
        let zoom = -(ev.y / 100.);
        if projection.scale + zoom > 0.01 {
            projection.scale += zoom;
        }
    }
}

fn pan_camera(
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let mut transform = camera_query.single_mut();

    if keys.pressed(KeyCode::KeyW) {
        transform.translation.y += 2.;
    }

    if keys.pressed(KeyCode::KeyA) {
        transform.translation.x -= 2.;
    }

    if keys.pressed(KeyCode::KeyS) {
        transform.translation.y -= 2.;
    }

    if keys.pressed(KeyCode::KeyD) {
        transform.translation.x += 2.;
    }
}

fn update_cursor_coordinates(
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

fn update_app_state(
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

fn hide_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    let window = &mut primary_window.single_mut();
    window.cursor.visible = false;
}

fn show_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    let window = &mut primary_window.single_mut();
    window.cursor.visible = true;
}

fn setup_brush(
    mut commands: Commands,
    mut brush_gizmos: Gizmos<BrushGizmos>,
    cursor_coords: Res<CursorCoords>,
    brush_type: Res<State<BrushType>>,
) {
    let brush = Brush::new(2, Color::WHITE);
    let brush_size = brush.size;
    commands.spawn(brush);
    brush_type.update_brush(cursor_coords.0, brush_size as f32, &mut brush_gizmos);
}

fn update_brush(
    brush_query: Query<&Brush>,
    cursor_coords: Res<CursorCoords>,
    mut brush_gizmos: Gizmos<BrushGizmos>,
    brush_type: Res<State<BrushType>>,
) {
    let brush = brush_query.single();
    brush_type.update_brush(cursor_coords.0, brush.size as f32, &mut brush_gizmos);
}

fn resize_brush_event_listener(
    mut ev_brush_resize: EventReader<BrushResizeEvent>,
    mut brush_query: Query<&mut Brush>,
) {
    let mut brush = brush_query.single_mut();
    for ev in ev_brush_resize.read() {
        brush.size = ev.0;
    }
}

fn toggle_simulation(mut commands: Commands, simulation_pause: Option<Res<SimulationRun>>) {
    if simulation_pause.is_some() {
        commands.remove_resource::<SimulationRun>();
    } else {
        commands.init_resource::<SimulationRun>();
    }
}
fn spawn_particles(
    mut commands: Commands,
    cursor_coords: Res<CursorCoords>,
    selected: Res<SelectedParticle>,
    brush_type: Res<State<BrushType>>,
    brush_query: Query<&Brush>,
) {
    let brush = brush_query.single();
    let brush_type = brush_type.get();
    brush_type.spawn_particles(
        &mut commands,
        cursor_coords.0,
        brush.size as f32,
        ParticleType {
            name: selected.0.clone(),
        },
    );
}

fn despawn_particles(
    mut commands: Commands,
    cursor_coords: Res<CursorCoords>,
    brush_type: Res<State<BrushType>>,
    brush_query: Query<&Brush>,
    mut contexts: EguiContexts,
) {
    let ctx = contexts.ctx_mut();
    if ctx.is_pointer_over_area() {
        return;
    }

    let brush = brush_query.single();
    let brush_size = brush.size;

    brush_type.remove_particles(&mut commands, cursor_coords.0.as_ivec2(), brush_size as f32)
}

fn render_ui(
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
    (mut selected_particle, particle_types): (ResMut<SelectedParticle>, Res<ParticleTypes>),
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
            ParticleControlUI.render(
                ui,
                &particle_types,
                &mut selected_particle,
                &mut brush_state,
                &mut commands,
            );
            ui.separator();
            BrushControlUI.render(
                ui,
                &mut brush_size,
                max_brush_size.0,
                &mut ev_brush_resize,
                &current_brush_type.get(),
                &mut next_brush_type,
            );
            ui.separator();
            SceneManagementUI.render(
                ui,
                &mut scene_selection_dialog,
                &mut scene_path,
                &mut ev_save_scene,
                &mut ev_load_scene,
            );
            ui.separator();
            DebugUI.render(
                ui,
                &debug_particles,
                dynamic_particle_count.0,
                total_particle_count.0,
                &mut commands,
            );
        });
}
