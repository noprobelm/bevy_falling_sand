use crate::{
    cursor::CursorPosition, particles::SelectedParticle, physics::DynamicRigidBodyParticle,
};
use bevy::{input::mouse::MouseWheel, platform::collections::HashSet, prelude::*};
use bfs_internal::{
    core::{DespawnParticleEvent, Particle},
    prelude::Wall,
};

pub(crate) struct BrushPlugin;

impl Plugin for BrushPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<BrushGizmos>()
            .init_resource::<MaxBrushSize>()
            .init_state::<BrushType>()
            .init_state::<BrushMode>()
            .add_sub_state::<BrushModeSpawnState>()
            .add_systems(Startup, setup)
            .add_systems(Update, update_brush_gizmos);
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BrushGizmos;

#[derive(Component)]
pub struct Brush;

#[derive(Component)]
pub struct BrushSize(pub usize);

#[derive(Component)]
struct BrushColor(pub Color);

#[derive(Reflect, Resource)]
pub struct MaxBrushSize(pub usize);

impl Default for MaxBrushSize {
    fn default() -> Self {
        MaxBrushSize(50)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect, Resource)]
pub struct DynamicRigidBodiesSpawning;

#[derive(States, Reflect, Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BrushMode {
    #[default]
    Spawn,
    Despawn,
}

#[derive(SubStates, Reflect, Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[source(BrushMode = BrushMode::Spawn)]
pub enum BrushModeSpawnState {
    #[default]
    Particles,
    DynamicRigidBodies,
}

#[derive(Default, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug, States)]
pub enum BrushType {
    Line,
    #[default]
    Circle,
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
    brush_type: Res<State<BrushType>>,
    brush_query: Query<(&BrushSize, &BrushColor), With<Brush>>,
) -> Result {
    let (size, color) = brush_query.single()?;

    match brush_type.get() {
        BrushType::Line => brush_gizmos.line_2d(
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
        BrushType::Circle => {
            brush_gizmos.circle_2d(cursor_position.current, size.0 as f32, color.0);
        }
        _ => brush_gizmos.cross_2d(cursor_position.current, 6., color.0),
    }
    Ok(())
}

pub fn update_brush_spawn_state(
    brush_spawn_state: Res<State<BrushMode>>,
    mut brush_spawn_state_next: ResMut<NextState<BrushMode>>,
) {
    match brush_spawn_state.get() {
        BrushMode::Spawn => brush_spawn_state_next.set(BrushMode::Despawn),
        BrushMode::Despawn => brush_spawn_state_next.set(BrushMode::Spawn),
    }
}

pub fn resize_brush(
    mut evr_mouse_wheel: EventReader<MouseWheel>,
    mut brush_size_query: Query<&mut BrushSize>,
    max_brush_size: Res<MaxBrushSize>,
) -> Result {
    if !evr_mouse_wheel.is_empty() {
        let mut brush_size = brush_size_query.single_mut()?;
        evr_mouse_wheel.read().for_each(|ev| {
            if ev.y < 0. && 1 <= brush_size.0.wrapping_sub(1) {
                brush_size.0 -= 1;
            } else if ev.y > 0. && brush_size.0.wrapping_add(1) <= max_brush_size.0 {
                brush_size.0 += 1;
            }
        });
    }
    Ok(())
}

pub fn update_brush_type_state(
    brush_type_state_current: Res<State<BrushType>>,
    mut brush_type_state_next: ResMut<NextState<BrushType>>,
) {
    match brush_type_state_current.get() {
        BrushType::Line => brush_type_state_next.set(BrushType::Circle),
        BrushType::Circle => brush_type_state_next.set(BrushType::Cursor),
        BrushType::Cursor => brush_type_state_next.set(BrushType::Line),
    }
}

pub fn spawn_particles(
    mut commands: Commands,
    cursor_position: Res<CursorPosition>,
    selected: Res<SelectedParticle>,
    brush_type_state: Res<State<BrushType>>,
    brush_mode_spawn_state: Option<Res<State<BrushModeSpawnState>>>,
    brush_query: Query<&BrushSize>,
) -> Result {
    let brush_size = brush_query.single()?;
    let half_length = (cursor_position.current - cursor_position.previous).length() / 2.0;
    let mut spawn_dynamic_rigid_body_particle = false;

    if let Some(brush_mode_spawn_state) = brush_mode_spawn_state {
        if brush_mode_spawn_state.get() == &BrushModeSpawnState::DynamicRigidBodies {
            spawn_dynamic_rigid_body_particle = true;
        }
    }

    match brush_type_state.get() {
        BrushType::Line => {
            let particle = selected.clone();

            if (cursor_position.previous - cursor_position.previous_previous).length() < 1.0 {
                spawn_line(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    brush_size.0,
                    spawn_dynamic_rigid_body_particle,
                );
            } else {
                spawn_line_interpolated(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    cursor_position.previous_previous,
                    brush_size.0,
                    spawn_dynamic_rigid_body_particle,
                );
            }

            if (cursor_position.current - cursor_position.previous).length() < 1.0 {
                spawn_line(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.current,
                    brush_size.0,
                    spawn_dynamic_rigid_body_particle,
                );
            } else {
                spawn_line_interpolated(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    cursor_position.current,
                    brush_size.0,
                    spawn_dynamic_rigid_body_particle,
                );
            }
        }
        BrushType::Circle => {
            let particle = selected.clone();

            if (cursor_position.previous - cursor_position.previous_previous).length() < 1.0 {
                spawn_circle(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    brush_size.0,
                    spawn_dynamic_rigid_body_particle,
                );
            } else {
                spawn_capsule(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    cursor_position.previous_previous,
                    brush_size.0,
                    half_length,
                    spawn_dynamic_rigid_body_particle,
                );
            }

            if (cursor_position.current - cursor_position.previous).length() < 1.0 {
                spawn_circle(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.current,
                    brush_size.0,
                    spawn_dynamic_rigid_body_particle,
                );
            } else {
                spawn_capsule(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    cursor_position.current,
                    brush_size.0,
                    half_length,
                    spawn_dynamic_rigid_body_particle,
                );
            }
        }
        BrushType::Cursor => {
            let particle = selected.clone();

            if (cursor_position.previous - cursor_position.previous_previous).length() >= 1.0 {
                spawn_cursor_interpolated(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous_previous,
                    cursor_position.previous,
                    spawn_dynamic_rigid_body_particle,
                );
            }

            if (cursor_position.current - cursor_position.previous).length() >= 1.0 {
                spawn_cursor_interpolated(
                    &mut commands,
                    particle.0.clone(),
                    cursor_position.previous,
                    cursor_position.current,
                    spawn_dynamic_rigid_body_particle,
                );
            } else {
                if spawn_dynamic_rigid_body_particle {
                    commands.spawn((
                        particle.0.clone(),
                        Wall,
                        Transform::from_xyz(
                            cursor_position.current.x.round(),
                            cursor_position.current.y.round(),
                            0.0,
                        ),
                        DynamicRigidBodyParticle,
                    ));
                } else {
                    commands.spawn((
                        particle.0.clone(),
                        Transform::from_xyz(
                            cursor_position.current.x.round(),
                            cursor_position.current.y.round(),
                            0.0,
                        ),
                    ));
                }
            }
        }
    }

    Ok(())
}

pub fn despawn_particles(
    mut evw_remove_particle: EventWriter<DespawnParticleEvent>,
    cursor_position: Res<CursorPosition>,
    brush_type_state: Res<State<BrushType>>,
    brush_size_query: Query<&BrushSize>,
) -> Result {
    let brush_size = brush_size_query.single()?.0;
    let half_length = (cursor_position.current - cursor_position.previous).length() / 2.0;

    match brush_type_state.get() {
        BrushType::Line => {
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
        BrushType::Circle => {
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
        BrushType::Cursor => {
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
            } else {
                evw_remove_particle.write(DespawnParticleEvent::from_position(IVec2::new(
                    cursor_position.current.x.round() as i32,
                    cursor_position.current.y.round() as i32,
                )));
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

fn spawn_circle(
    commands: &mut Commands,
    particle: Particle,
    center: Vec2,
    radius: usize,
    dynamic_rigid_body_particle: bool,
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

    if dynamic_rigid_body_particle {
        commands.spawn_batch(points.into_iter().map(move |point| {
            (
                particle.clone(),
                Wall,
                Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
                DynamicRigidBodyParticle,
            )
        }));
    } else {
        commands.spawn_batch(points.into_iter().map(move |point| {
            (
                particle.clone(),
                Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
            )
        }));
    }
}

fn spawn_capsule(
    commands: &mut Commands,
    particle: Particle,
    start: Vec2,
    end: Vec2,
    radius: usize,
    half_length: f32,
    dynamic_rigid_body_particle: bool,
) {
    let capsule = Capsule2d {
        radius: radius as f32,
        half_length,
    };

    let points = points_within_capsule(&capsule, start, end);
    if dynamic_rigid_body_particle {
        commands.spawn_batch(points.into_iter().map(move |point| {
            (
                particle.clone(),
                Wall,
                Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
                DynamicRigidBodyParticle,
            )
        }));
    } else {
        commands.spawn_batch(points.into_iter().map(move |point| {
            (
                particle.clone(),
                Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
            )
        }));
    }
}

fn spawn_line(
    commands: &mut Commands,
    particle: Particle,
    center: Vec2,
    brush_size: usize,
    dynamic_rigid_body_particle: bool,
) {
    let min_x = -(brush_size as i32) / 2;
    let max_x = (brush_size as f32 / 2.0) as i32;

    if dynamic_rigid_body_particle {
        commands.spawn_batch((min_x * 3..=max_x * 3).map(move |x| {
            (
                particle.clone(),
                Wall,
                Transform::from_xyz((center.x + x as f32).round(), center.y.round(), 0.0),
                DynamicRigidBodyParticle,
            )
        }));
    } else {
        commands.spawn_batch((min_x * 3..=max_x * 3).map(move |x| {
            (
                particle.clone(),
                Transform::from_xyz((center.x + x as f32).round(), center.y.round(), 0.0),
            )
        }));
    }
}

fn spawn_line_interpolated(
    commands: &mut Commands,
    particle: Particle,
    start: Vec2,
    end: Vec2,
    brush_size: usize,
    dynamic_rigid_body_particle: bool,
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

    if dynamic_rigid_body_particle {
        commands.spawn_batch(points.into_iter().map(move |point| {
            (
                particle.clone(),
                Wall,
                Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
                DynamicRigidBodyParticle,
            )
        }));
    } else {
        commands.spawn_batch(points.into_iter().map(move |point| {
            (
                particle.clone(),
                Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
            )
        }));
    }
}

fn spawn_cursor_interpolated(
    commands: &mut Commands,
    particle: Particle,
    start: Vec2,
    end: Vec2,
    dynamic_rigid_body_particle: bool,
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

    if dynamic_rigid_body_particle {
        commands.spawn_batch(points.into_iter().map(move |point| {
            (
                particle.clone(),
                Wall,
                Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
                DynamicRigidBodyParticle,
            )
        }));
    } else {
        commands.spawn_batch(points.into_iter().map(move |point| {
            (
                particle.clone(),
                Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
            )
        }));
    }
}

fn despawn_circle(
    evw_remove_particle: &mut EventWriter<DespawnParticleEvent>,
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
        evw_remove_particle.write(DespawnParticleEvent::from_position(position));
    }
}

fn despawn_capsule(
    evw_remove_particle: &mut EventWriter<DespawnParticleEvent>,
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
        evw_remove_particle.write(DespawnParticleEvent::from_position(position));
    }
}

fn despawn_line(
    evw_remove_particle: &mut EventWriter<DespawnParticleEvent>,
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
        evw_remove_particle.write(DespawnParticleEvent::from_position(position));
    }
}

fn despawn_line_interpolated(
    evw_remove_particle: &mut EventWriter<DespawnParticleEvent>,
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
        evw_remove_particle.write(DespawnParticleEvent::from_position(position));
    }
}

fn despawn_cursor_interpolated(
    evw_remove_particle: &mut EventWriter<DespawnParticleEvent>,
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
        evw_remove_particle.write(DespawnParticleEvent::from_position(position));
    }
}
