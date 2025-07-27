use crate::{
    app_state::AppState,
    cursor::{update_cursor_position, CursorPosition},
    particles::SelectedParticle,
};
use bevy::{
    input::common_conditions::{input_just_pressed, input_pressed},
    platform::collections::HashSet,
    prelude::*,
};
use bevy_falling_sand::prelude::{Particle, ParticleSimulationSet, RemoveParticleEvent};

pub(crate) struct BrushPlugin;

impl Plugin for BrushPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<BrushGizmos>()
            .init_resource::<MaxBrushSize>()
            .init_state::<BrushTypeState>()
            .init_state::<BrushSpawnState>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    update_brush_gizmos,
                    toggle_brush_spawn_state.run_if(input_just_pressed(MouseButton::Right)),
                    spawn_particles
                        .run_if(input_pressed(MouseButton::Left))
                        .run_if(in_state(BrushSpawnState::Spawn))
                        .run_if(in_state(AppState::Canvas))
                        .before(ParticleSimulationSet)
                        .after(update_cursor_position),
                    despawn_particles
                        .run_if(input_pressed(MouseButton::Left))
                        .run_if(in_state(BrushSpawnState::Despawn))
                        .run_if(in_state(AppState::Canvas))
                        .before(ParticleSimulationSet)
                        .after(update_cursor_position),
                ),
            );
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BrushGizmos;

#[derive(Component)]
#[allow(dead_code)]
struct Brush;

#[derive(Component)]
struct BrushSize(usize);

#[derive(Component)]
struct BrushColor(Color);

#[derive(Reflect, Resource)]
struct MaxBrushSize(pub usize);

impl Default for MaxBrushSize {
    fn default() -> Self {
        MaxBrushSize(50)
    }
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum BrushSpawnState {
    #[default]
    Spawn,
    Despawn,
}

#[derive(Default, Clone, Hash, Eq, PartialEq, Debug, States)]
enum BrushTypeState {
    #[allow(unused)]
    Line,
    #[allow(unused)]
    #[default]
    Circle,
    #[allow(unused)]
    Cursor,
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Brush,
        BrushSize(2),
        BrushColor(Color::Srgba(Srgba::new(1., 1., 1., 0.3))),
    ));
}

fn update_brush_gizmos(
    cursor_position: Res<CursorPosition>,
    mut brush_gizmos: Gizmos<BrushGizmos>,
    brush_type: Res<State<BrushTypeState>>,
    brush_query: Query<(&BrushSize, &BrushColor), With<Brush>>,
) -> Result {
    let (size, color) = brush_query.single()?;

    match brush_type.get() {
        BrushTypeState::Line => brush_gizmos.line_2d(
            Vec2::new(
                cursor_position.current.x - size.0 as f32 * 3. / 2.,
                cursor_position.current.y,
            ),
            Vec2::new(
                cursor_position.current.x + size.0 as f32 * 3. / 2.,
                cursor_position.current.y,
            ),
            color.0,
        ),
        BrushTypeState::Circle => {
            brush_gizmos.circle_2d(cursor_position.current, size.0 as f32, color.0);
        }
        _ => brush_gizmos.cross_2d(cursor_position.current, 6., color.0),
    }
    Ok(())
}

fn toggle_brush_spawn_state(
    brush_spawn_state: Res<State<BrushSpawnState>>,
    mut brush_spawn_state_next: ResMut<NextState<BrushSpawnState>>,
) {
    match brush_spawn_state.get() {
        BrushSpawnState::Spawn => brush_spawn_state_next.set(BrushSpawnState::Despawn),
        BrushSpawnState::Despawn => brush_spawn_state_next.set(BrushSpawnState::Spawn),
    }
}

fn spawn_particles(
    mut commands: Commands,
    cursor_position: Res<CursorPosition>,
    selected: Res<SelectedParticle>,
    brush_type_state: Res<State<BrushTypeState>>,
    brush_query: Query<&BrushSize>,
) -> Result {
    let brush_size = brush_query.single()?;
    let half_length = (cursor_position.current - cursor_position.previous).length() / 2.0;

    match brush_type_state.get() {
        BrushTypeState::Line => {
            let particle = selected.clone();

            if (cursor_position.previous - cursor_position.previous_previous).length() < 1.0 {
                spawn_line(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    brush_size.0,
                );
            } else {
                spawn_line_interpolated(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    cursor_position.previous_previous,
                    brush_size.0,
                );
            }

            if (cursor_position.current - cursor_position.previous).length() < 1.0 {
                spawn_line(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.current,
                    brush_size.0,
                );
            } else {
                spawn_line_interpolated(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    cursor_position.current,
                    brush_size.0,
                );
            }
        }
        BrushTypeState::Circle => {
            let particle = selected.clone();

            if (cursor_position.previous - cursor_position.previous_previous).length() < 1.0 {
                spawn_circle(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    brush_size.0,
                );
            } else {
                spawn_capsule(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    cursor_position.previous_previous,
                    brush_size.0,
                    half_length,
                );
            }

            if (cursor_position.current - cursor_position.previous).length() < 1.0 {
                spawn_circle(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.current,
                    brush_size.0,
                );
            } else {
                spawn_capsule(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    cursor_position.current,
                    brush_size.0,
                    half_length,
                );
            }
        }
        BrushTypeState::Cursor => {
            let particle = selected.clone();

            if (cursor_position.previous - cursor_position.previous_previous).length() >= 1.0 {
                spawn_cursor_interpolated(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous_previous,
                    cursor_position.previous,
                );
            }

            if (cursor_position.current - cursor_position.previous).length() >= 1.0 {
                spawn_cursor_interpolated(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    cursor_position.current,
                );
            }
        }
    }

    Ok(())
}

fn despawn_particles(
    mut evw_remove_particle: EventWriter<RemoveParticleEvent>,
    cursor_position: Res<CursorPosition>,
    brush_type_state: Res<State<BrushTypeState>>,
    brush_size_query: Query<&BrushSize>,
) -> Result {
    let brush_size = brush_size_query.single()?.0;
    let half_length = (cursor_position.current - cursor_position.previous).length() / 2.0;

    match brush_type_state.get() {
        BrushTypeState::Line => {
            if (cursor_position.previous - cursor_position.previous_previous).length() < 1.0 {
                despawn_line(
                    &mut evw_remove_particle,
                    cursor_position.previous,
                    brush_size,
                );
            } else {
                despawn_line_interpolated(
                    &mut evw_remove_particle,
                    cursor_position.previous,
                    cursor_position.previous_previous,
                    brush_size,
                );
            }

            if (cursor_position.current - cursor_position.previous).length() < 1.0 {
                despawn_line(
                    &mut evw_remove_particle,
                    cursor_position.current,
                    brush_size,
                );
            } else {
                despawn_line_interpolated(
                    &mut evw_remove_particle,
                    cursor_position.previous,
                    cursor_position.current,
                    brush_size,
                );
            }
        }
        BrushTypeState::Circle => {
            if (cursor_position.previous - cursor_position.previous_previous).length() < 1.0 {
                despawn_circle(
                    &mut evw_remove_particle,
                    cursor_position.previous,
                    brush_size,
                );
            } else {
                despawn_capsule(
                    &mut evw_remove_particle,
                    cursor_position.previous,
                    cursor_position.previous_previous,
                    brush_size,
                    half_length,
                );
            }

            if (cursor_position.current - cursor_position.previous).length() < 1.0 {
                despawn_circle(
                    &mut evw_remove_particle,
                    cursor_position.current,
                    brush_size,
                );
            } else {
                despawn_capsule(
                    &mut evw_remove_particle,
                    cursor_position.previous,
                    cursor_position.current,
                    brush_size,
                    half_length,
                );
            }
        }
        BrushTypeState::Cursor => {
            if (cursor_position.previous - cursor_position.previous_previous).length() >= 1.0 {
                despawn_cursor_interpolated(
                    &mut evw_remove_particle,
                    cursor_position.previous_previous,
                    cursor_position.previous,
                );
            }

            if (cursor_position.current - cursor_position.previous).length() >= 1.0 {
                despawn_cursor_interpolated(
                    &mut evw_remove_particle,
                    cursor_position.previous,
                    cursor_position.current,
                );
            }
        }
    }

    Ok(())
}

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

fn spawn_circle(commands: &mut Commands, particle: Particle, center: Vec2, radius: usize) {
    let mut points: HashSet<IVec2> = HashSet::default();

    let min_x = (center.x - radius as f32).floor() as i32;
    let max_x = (center.x + radius as f32).ceil() as i32;
    let min_y = (center.y - radius as f32).floor() as i32;
    let max_y = (center.y + radius as f32).ceil() as i32;

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            let point = Vec2::new(x as f32, y as f32);
            if (point - center).length() <= radius as f32 {
                points.insert(point.as_ivec2());
            }
        }
    }

    commands.spawn_batch(points.into_iter().map(move |point| {
        (
            particle.clone(),
            Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
        )
    }));
}

fn spawn_capsule(
    commands: &mut Commands,
    particle: Particle,
    start: Vec2,
    end: Vec2,
    radius: usize,
    half_length: f32,
) {
    let capsule = Capsule2d {
        radius: radius as f32,
        half_length,
    };

    let points = points_within_capsule(&capsule, start, end);
    commands.spawn_batch(points.into_iter().map(move |point| {
        (
            particle.clone(),
            Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
        )
    }));
}

fn spawn_line(commands: &mut Commands, particle: Particle, center: Vec2, brush_size: usize) {
    let min_x = -(brush_size as i32) / 2;
    let max_x = (brush_size as f32 / 2.0) as i32;

    commands.spawn_batch((min_x * 3..=max_x * 3).map(move |x| {
        (
            particle.clone(),
            Transform::from_xyz((center.x + x as f32).round(), center.y.round(), 0.0),
        )
    }));
}

fn spawn_line_interpolated(
    commands: &mut Commands,
    particle: Particle,
    start: Vec2,
    end: Vec2,
    brush_size: usize,
) {
    let mut points: HashSet<IVec2> = HashSet::default();
    let direction = (end - start).normalize();
    let length = (end - start).length();
    let min_x = -(brush_size as i32) / 2;
    let max_x = (brush_size as f32 / 2.0) as i32;

    let num_samples = (length.ceil() as usize).max(1);
    for i in 0..=num_samples {
        let t = i as f32 / num_samples.max(1) as f32;
        let sample_point = start + direction * length * t;

        for x in min_x * 3..=max_x * 3 {
            let position = Vec2::new((sample_point.x + x as f32).round(), sample_point.y.round());
            points.insert(position.as_ivec2());
        }
    }

    commands.spawn_batch(points.into_iter().map(move |point| {
        (
            particle.clone(),
            Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
        )
    }));
}

fn spawn_cursor_interpolated(commands: &mut Commands, particle: Particle, start: Vec2, end: Vec2) {
    let mut points: HashSet<IVec2> = HashSet::default();
    let direction = (end - start).normalize();
    let length = (end - start).length();

    let num_samples = (length.ceil() as usize).max(1);
    for i in 0..=num_samples {
        let t = i as f32 / num_samples.max(1) as f32;
        let sample_point = start + direction * length * t;

        points.insert(IVec2::new(
            sample_point.x.round() as i32,
            sample_point.y.round() as i32,
        ));
    }

    commands.spawn_batch(points.into_iter().map(move |point| {
        (
            particle.clone(),
            Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
        )
    }));
}

fn despawn_circle(
    evw_remove_particle: &mut EventWriter<RemoveParticleEvent>,
    center: Vec2,
    radius: usize,
) {
    let mut points: HashSet<IVec2> = HashSet::default();

    let min_x = (center.x - radius as f32).floor() as i32;
    let max_x = (center.x + radius as f32).ceil() as i32;
    let min_y = (center.y - radius as f32).floor() as i32;
    let max_y = (center.y + radius as f32).ceil() as i32;

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            let point = Vec2::new(x as f32, y as f32);
            if (point - center).length() <= radius as f32 {
                points.insert(point.as_ivec2());
            }
        }
    }

    for position in points {
        evw_remove_particle.write(RemoveParticleEvent {
            position,
            despawn: true,
        });
    }
}

fn despawn_capsule(
    evw_remove_particle: &mut EventWriter<RemoveParticleEvent>,
    start: Vec2,
    end: Vec2,
    radius: usize,
    half_length: f32,
) {
    let capsule = Capsule2d {
        radius: radius as f32,
        half_length,
    };

    let points = points_within_capsule(&capsule, start, end);
    for position in points {
        evw_remove_particle.write(RemoveParticleEvent {
            position,
            despawn: true,
        });
    }
}

fn despawn_line(
    evw_remove_particle: &mut EventWriter<RemoveParticleEvent>,
    center: Vec2,
    brush_size: usize,
) {
    let min_x = -(brush_size as i32) / 2;
    let max_x = (brush_size as f32 / 2.0) as i32;

    for x in min_x * 3..=max_x * 3 {
        let position = IVec2::new(
            (center.x + x as f32).round() as i32,
            center.y.round() as i32,
        );
        evw_remove_particle.write(RemoveParticleEvent {
            position,
            despawn: true,
        });
    }
}

fn despawn_line_interpolated(
    evw_remove_particle: &mut EventWriter<RemoveParticleEvent>,
    start: Vec2,
    end: Vec2,
    brush_size: usize,
) {
    let mut points: HashSet<IVec2> = HashSet::default();
    let direction = (end - start).normalize();
    let length = (end - start).length();
    let min_x = -(brush_size as i32) / 2;
    let max_x = (brush_size as f32 / 2.0) as i32;

    let num_samples = (length.ceil() as usize).max(1);
    for i in 0..=num_samples {
        let t = i as f32 / num_samples.max(1) as f32;
        let sample_point = start + direction * length * t;

        for x in min_x * 3..=max_x * 3 {
            let position = Vec2::new((sample_point.x + x as f32).round(), sample_point.y.round());
            points.insert(position.as_ivec2());
        }
    }

    for position in points {
        evw_remove_particle.write(RemoveParticleEvent {
            position,
            despawn: true,
        });
    }
}

fn despawn_cursor_interpolated(
    evw_remove_particle: &mut EventWriter<RemoveParticleEvent>,
    start: Vec2,
    end: Vec2,
) {
    let mut points: HashSet<IVec2> = HashSet::default();
    let direction = (end - start).normalize();
    let length = (end - start).length();

    let num_samples = (length.ceil() as usize).max(1);
    for i in 0..=num_samples {
        let t = i as f32 / num_samples.max(1) as f32;
        let sample_point = start + direction * length * t;

        points.insert(IVec2::new(
            sample_point.x.round() as i32,
            sample_point.y.round() as i32,
        ));
    }

    for position in points {
        evw_remove_particle.write(RemoveParticleEvent {
            position,
            despawn: true,
        });
    }
}
