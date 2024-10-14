//! This module demonstrates how to spawn/despawn particles from the world using a brush tool.
use bevy::{prelude::*, utils::HashSet};

use super::CursorCoords;
use bevy_falling_sand::core::{Particle, RemoveParticleEvent};

/// Brush plugin.
pub(super) struct BrushPlugin;

impl bevy::prelude::Plugin for BrushPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<MaxBrushSize>();
        app.init_state::<BrushState>().init_state::<BrushType>();
        app.init_gizmo_group::<BrushGizmos>();
        app.add_event::<BrushResizeEvent>();
        app.add_systems(Startup, setup_brush)
            .add_systems(Update, (update_brush, resize_brush_event_listener));
    }
}

/// Gizmos for rendering the brush shape to the canvas.
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BrushGizmos;

#[derive(Component)]
#[allow(dead_code)]
pub struct Brush {
    /// The brush size.
    pub size: usize,
    /// The brush color.
    pub color: Color,
}

impl Brush {
    /// Create a new Brush
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

/// State for spawning or despawning particle with the brush.
#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum BrushState {
    #[default]
    /// The spawn state.
    Spawn,
    /// The despawn state.
    Despawn,
}

/// The maximum possible brush size.
#[derive(Reflect, Resource)]
pub struct MaxBrushSize(pub usize);

impl Default for MaxBrushSize {
    fn default() -> Self {
        return MaxBrushSize(50);
    }
}

/// Resizes the brush when triggered.
#[derive(Event)]
pub struct BrushResizeEvent(pub usize);

/// State for each brush type and its functionality.
#[derive(Default, Clone, Hash, Eq, PartialEq, Debug, States)]
pub enum BrushType {
    Line,
    #[default]
    Circle,
}

impl BrushType {
    /// Updates the brush position on the canvas.
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
        }
    }

    /// Spawns the selected particle into the world.
    ///
    /// To add a particle to the simualtion, simply spawn a new entity with a `Particle` component whose `name` field
    /// can be mapped to a String in the `ParticleTypeMap` resource. Be sure to include a SpatialBundle for
    /// the particle's position.
    pub fn spawn_particles(
        &self,
        commands: &mut Commands,
        coords: Res<CursorCoords>,
        brush_size: f32,
        selected_particle: Particle,
    ) {
        let coords = coords.clone();
        let radius = brush_size;
        let half_length = (coords.current - coords.previous).length() / 2.0;
        let min_x = -(brush_size as i32) / 2;
        let max_x = (brush_size / 2.) as i32;

        match self {
            BrushType::Line => {
                let particle = selected_particle.clone();
                commands.spawn_batch((min_x * 3..=max_x * 3).map(move |x| {
                    (
                        particle.clone(), // Clone the particle for each x iteration
                        SpatialBundle::from_transform(Transform::from_xyz(
                            coords.current.x + x as f32,
                            coords.current.y,
                            0.0,
                        )),
                    )
                }));
            }
            BrushType::Circle => {
                let particle = selected_particle.clone();

		// If there's no distance between one cursor coordinate and the next, draw a circle instead.
                if (coords.current - coords.previous).length() < 1.0 {
                    let circle_center = coords.current;
                    let mut points: HashSet<IVec2> = HashSet::default();

                    let min_x = (circle_center.x - radius).floor() as i32;
                    let max_x = (circle_center.x + radius).ceil() as i32;
                    let min_y = (circle_center.y - radius).floor() as i32;
                    let max_y = (circle_center.y + radius).ceil() as i32;

                    for x in min_x..=max_x {
                        for y in min_y..=max_y {
                            let point = Vec2::new(x as f32, y as f32);
                            if (point - circle_center).length() <= radius {
                                points.insert(point.as_ivec2());
                            }
                        }
                    }

                    commands.spawn_batch(points.into_iter().map(move |point| {
                        (
                            particle.clone(),
                            SpatialBundle::from_transform(Transform::from_xyz(
                                point.x as f32,
                                point.y as f32,
                                0.0,
                            )),
                        )
                    }));
                } else {
                    let capsule = Capsule2d {
                        radius,
                        half_length,
                    };

                    let points = points_within_capsule(&capsule, coords.previous, coords.current);
                    commands.spawn_batch(points.into_iter().map(move |point| {
                        (
                            particle.clone(),
                            SpatialBundle::from_transform(Transform::from_xyz(
                                point.x as f32,
                                point.y as f32,
                                0.0,
                            )),
                        )
                    }));
                }
            }
        }
    }

    /// Removes particles under the brush from the world.
    ///
    /// To remove a particle from the simulation, send a `RemoveParticleEvent` This will guarantee that the particle
    /// will:
    ///   1. Be removed from the underlying ChunkMap.
    ///   2. Be removed as a child from the particle type parent entity.
    ///
    /// You can choose to keep the particle in the world by passing `despawn: false` to the event, though this will
    /// leave dangling entities unless you are managing them elsewhere within your application.
    pub fn remove_particles(&self, commands: &mut Commands, coords: IVec2, brush_size: f32) {
        let min_x = -(brush_size as i32) / 2;
        let max_x = (brush_size / 2.) as i32;
        let min_y = -(brush_size as i32) / 2;
        let max_y = (brush_size / 2.) as i32;

        match self {
            BrushType::Line => {
                for x in min_x * 3..=max_x * 3 {
                    let coordinates = IVec2::new(coords.x + x, coords.y);
                    commands.trigger(RemoveParticleEvent {
                        coordinates,
                        despawn: true,
                    });
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
                    commands.trigger(RemoveParticleEvent {
                        coordinates,
                        despawn: true,
                    })
                }
            }
        }
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

/// Sets up the brush.
pub fn setup_brush(
    mut commands: Commands,
    mut brush_gizmos: Gizmos<BrushGizmos>,
    cursor_coords: Res<CursorCoords>,
    brush_type: Res<State<BrushType>>,
) {
    let brush = Brush::new(2, Color::WHITE);
    let brush_size = brush.size;
    commands.spawn(brush);
    brush_type.update_brush(cursor_coords.current, brush_size as f32, &mut brush_gizmos);
}

/// Updates the brush position and size each frame.
pub fn update_brush(
    brush_query: Query<&Brush>,
    cursor_coords: Res<CursorCoords>,
    mut brush_gizmos: Gizmos<BrushGizmos>,
    brush_type: Res<State<BrushType>>,
) {
    let brush = brush_query.single();
    brush_type.update_brush(cursor_coords.current, brush.size as f32, &mut brush_gizmos);
}

/// Resizes the brush when a resize event is published.
pub fn resize_brush_event_listener(
    mut ev_brush_resize: EventReader<BrushResizeEvent>,
    mut brush_query: Query<&mut Brush>,
) {
    let mut brush = brush_query.single_mut();
    for ev in ev_brush_resize.read() {
        brush.size = ev.0;
    }
}

/// Function to get all points within a 2D capsule defined by two endpoints and a radius.
fn points_within_capsule(capsule: &Capsule2d, start: Vec2, end: Vec2) -> Vec<IVec2> {
    let mut points_inside = Vec::new();

    let min_x = (start.x.min(end.x) - capsule.radius).floor() as i32;
    let max_x = (start.x.max(end.x) + capsule.radius).ceil() as i32;
    let min_y = (start.y.min(end.y) - capsule.radius).floor() as i32;
    let max_y = (start.y.max(end.y) + capsule.radius).ceil() as i32;
    let capsule_direction = (end - start).normalize();

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            let point = Vec2::new(x as f32, y as f32);

            let to_point = point - start;
            let projected_length = to_point.dot(capsule_direction);
            let clamped_length = projected_length.clamp(-capsule.half_length, capsule.half_length);

            let closest_point = start + capsule_direction * clamped_length;
            let distance_to_line = (point - closest_point).length();

            if distance_to_line <= capsule.radius {
                points_inside.push(IVec2::new(x, y));
            }
        }
    }

    points_inside
}
