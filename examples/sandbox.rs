use bevy::input::mouse::MouseWheel;
use bevy::utils::HashSet;
use bevy::window::PrimaryWindow;
use bevy::{prelude::*, window::WindowMode};
use bevy_egui::{EguiContexts, EguiPlugin};

use bevy::input::common_conditions::input_pressed;
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
        .init_resource::<CursorCoords>()
        .init_resource::<MaxBrushSize>();

    // States
    app.init_state::<ParticleType>()
        .init_state::<SpawnState>()
        .init_state::<BrushType>();

    // Gizmos
    app.init_gizmo_group::<BrushGizmos>();

    // Events
    app.add_event::<BrushResizeEvent>()
        .add_event::<CanvasResetEvent>();

    // Camera control
    app.add_systems(Startup, setup_camera)
        .add_systems(Update, (zoom_camera, pan_camera));

    // UI
    app.add_systems(Update, render_ui);

    // Brush systems
    app.add_systems(Startup, setup_brush).add_systems(
        Update,
        (
            update_brush,
            resize_brush_event_listener,
            hide_cursor.after(render_ui),
        ),
    );

    app.add_systems(Update, update_cursor_coordinates);

    // Particle management systems
    app.add_systems(
        Update,
        (spawn_particles
            .run_if(input_pressed(MouseButton::Left))
            .run_if(in_state(SpawnState::Add))
            .after(update_cursor_coordinates)
            .after(render_ui),),
    );
    app.add_systems(
        Update,
        despawn_particles
            .run_if(input_pressed(MouseButton::Left))
            .run_if(in_state(SpawnState::Remove))
            .before(handle_particles)
            .after(update_cursor_coordinates)
            .after(render_ui),
    );

    app.run();
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BrushGizmos;

#[derive(Component)]
#[allow(dead_code)]
pub struct Brush {
    pub size: usize,
    pub color: Color,
}

impl Brush {
    pub fn new(size: usize, color: Color) -> Self {
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
pub struct MaxBrushSize(pub usize);

#[derive(Resource, Default, Debug)]
pub struct CursorCoords(pub Vec2);

impl Default for MaxBrushSize {
    fn default() -> Self {
        return MaxBrushSize(50);
    }
}

#[derive(Default, Clone, Hash, Eq, PartialEq, Debug, States)]
pub enum BrushType {
    Line,
    #[default]
    Circle,
    Square,
}

impl BrushType {
    pub fn update_brush(
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

    pub fn spawn_particles(
        &self,
        commands: &mut Commands,
        coords: Vec2,
        brush_size: f32,
        particle_type: ParticleType,
    ) {
        let min_x = -(brush_size as i32) / 2;
        let max_x = (brush_size / 2.) as i32;
        let min_y = -(brush_size as i32) / 2;
        let max_y = (brush_size / 2.) as i32;

        match self {
            BrushType::Line => {
                commands.spawn_batch((min_x * 3..=max_x * 3).map(move |x| {
                    (
                        particle_type,
                        SpatialBundle::from_transform(Transform::from_xyz(
                            coords.x + x as f32,
                            coords.y,
                            0.0,
                        )),
                    )
                }));
            }
            BrushType::Circle => {
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
                        particle_type,
                        SpatialBundle::from_transform(Transform::from_xyz(
                            point.x as f32,
                            point.y as f32,
                            0.,
                        )),
                    )
                }));
            }
            BrushType::Square => {
                commands.spawn_batch((min_x..=max_x).flat_map(move |x| {
                    (min_y..=max_y).map(move |y| {
                        (
                            particle_type,
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

    pub fn remove_particles(
        &self,
        commands: &mut Commands,
        coords: IVec2,
        brush_size: f32,
    ) {
        let min_x = -(brush_size as i32) / 2;
        let max_x = (brush_size / 2.) as i32;
        let min_y = -(brush_size as i32) / 2;
        let max_y = (brush_size / 2.) as i32;

        match self {
            BrushType::Line => {
                for x in min_x * 3..=max_x * 3 {
                    let coordinates = IVec2::new(coords.x + x, coords.y);
		    commands.trigger(RemoveParticle {coordinates});
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
		    commands.trigger(RemoveParticle {coordinates})
                }
            }
            BrushType::Square => {
                for x in min_x..=max_x {
                    for y in min_y..=max_y {
                        let coordinates = IVec2::new(coords.x + x, coords.y + y);
			commands.trigger(RemoveParticle {coordinates})
                    }
                }
            }
        }

    }
}

#[derive(Event)]
pub struct BrushResizeEvent(pub usize);

// Triggers the removal of all particles when the corresponding UI button is clicked
#[derive(Event)]
pub struct CanvasResetEvent;

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum SpawnState {
    #[default]
    Add,
    Remove,
}

pub fn setup_camera(mut commands: Commands) {
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

pub fn zoom_camera(
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

pub fn pan_camera(
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

// The cursor looks bad with the brush. Disable it when we're not hovering over a UI element
pub fn hide_cursor(
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut contexts: EguiContexts,
) {
    let window = &mut primary_window.single_mut();
    let ctx = contexts.ctx_mut();

    if ctx.is_pointer_over_area() {
        window.cursor.visible = true;
    } else {
        window.cursor.visible = false;
    }
}

pub fn setup_brush(
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

pub fn update_brush(
    mut brush_query: Query<&Brush>,
    cursor_coords: Res<CursorCoords>,
    mut brush_gizmos: Gizmos<BrushGizmos>,
    brush_type: Res<State<BrushType>>,
) {
    let brush = brush_query.single_mut();
    brush_type.update_brush(cursor_coords.0, brush.size as f32, &mut brush_gizmos);
}

pub fn resize_brush_event_listener(
    mut ev_brush_resize: EventReader<BrushResizeEvent>,
    mut brush_query: Query<&mut Brush>,
) {
    let mut brush = brush_query.single_mut();
    for ev in ev_brush_resize.read() {
        brush.size = ev.0;
    }
}

pub fn spawn_particles(
    mut commands: Commands,
    cursor_coords: Res<CursorCoords>,
    selected: Res<State<ParticleType>>,
    brush_type: Res<State<BrushType>>,
    brush_query: Query<&Brush>,
    mut contexts: EguiContexts,
) {
    let ctx = contexts.ctx_mut();
    if ctx.is_pointer_over_area() {
        return;
    }

    let brush = brush_query.single();
    let brush_type = brush_type.get();
    brush_type.spawn_particles(
        &mut commands,
        cursor_coords.0,
        brush.size as f32,
        selected.get().clone(),
    );
}

pub fn despawn_particles(
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

    brush_type.remove_particles(
        &mut commands,
        cursor_coords.0.as_ivec2(),
        brush_size as f32,
    )
}

pub fn render_ui(
    mut commands: Commands,
    mut particle_type_state: ResMut<NextState<ParticleType>>,
    mut spawn_state: ResMut<NextState<SpawnState>>,
    brush_query: Query<&Brush>,
    current_brush_type: Res<State<BrushType>>,
    mut next_brush_type: ResMut<NextState<BrushType>>,
    mut ev_brush_resize: EventWriter<BrushResizeEvent>,
    mut contexts: EguiContexts,
    max_brush_size: Res<MaxBrushSize>,
    debug_particles: Option<Res<DebugParticles>>,
) {
    let ctx = contexts.ctx_mut();
    let brush = brush_query.single();
    let mut brush_size = brush.size;

    egui::SidePanel::left("side_panel")
        .exact_width(200.0)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("Water").clicked() {
                    particle_type_state.set(ParticleType::Water);
                    spawn_state.set(SpawnState::Add)
                } else if ui.button("Oil").clicked() {
                    particle_type_state.set(ParticleType::Oil);
                    spawn_state.set(SpawnState::Add);
                } else if ui.button("Whiskey").clicked() {
                    particle_type_state.set(ParticleType::Whiskey);
                    spawn_state.set(SpawnState::Add);
                } else if ui.button("Sand").clicked() {
                    particle_type_state.set(ParticleType::Sand);
                    spawn_state.set(SpawnState::Add);
                } else if ui.button("Steam").clicked() {
                    particle_type_state.set(ParticleType::Steam);
                    spawn_state.set(SpawnState::Add);
                } else if ui.button("Wall").clicked() {
                    particle_type_state.set(ParticleType::Wall);
                    spawn_state.set(SpawnState::Add);
                } else if ui.button("Dirt Wall").clicked() {
                    particle_type_state.set(ParticleType::DirtWall);
                    spawn_state.set(SpawnState::Add);
                } else if ui.button("Grass Wall").clicked() {
                    particle_type_state.set(ParticleType::GrassWall);
                    spawn_state.set(SpawnState::Add);
                } else if ui.button("Rock Wall").clicked() {
                    particle_type_state.set(ParticleType::RockWall);
                    spawn_state.set(SpawnState::Add);
                } else if ui.button("Dense Rock Wall").clicked() {
                    particle_type_state.set(ParticleType::DenseRockWall);
                    spawn_state.set(SpawnState::Add);
                } else if ui.button("Remove").clicked() {
                    spawn_state.set(SpawnState::Remove);
                }
            });

            if ui.button("Despawn All Particles").clicked() {
                commands.trigger(ClearChunkMap);
            }

            if ui
                .add(egui::Slider::new(&mut brush_size, 1..=max_brush_size.0))
                .changed()
            {
                ev_brush_resize.send(BrushResizeEvent(brush_size));
            };

            egui::ComboBox::from_label("Brush Type")
                .selected_text(format!("{:?}", current_brush_type.get()))
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(&mut current_brush_type.get(), &BrushType::Line, "Line")
                        .changed()
                    {
                        next_brush_type.set(BrushType::Line)
                    };
                    if ui
                        .selectable_value(
                            &mut current_brush_type.get(),
                            &BrushType::Circle,
                            "Circle",
                        )
                        .changed()
                    {
                        next_brush_type.set(BrushType::Circle)
                    };
                    if ui
                        .selectable_value(
                            &mut current_brush_type.get(),
                            &BrushType::Square,
                            "Square",
                        )
                        .changed()
                    {
                        next_brush_type.set(BrushType::Square)
                    };
                });

            ui.separator();

            let mut debugging = debug_particles.is_some();
            if ui.checkbox(&mut debugging, "Show Chunks").clicked() {
                if debugging == true {
                    commands.init_resource::<DebugParticles>();
                } else {
                    commands.remove_resource::<DebugParticles>();
                }
            }
        });
}
