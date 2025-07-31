use bevy::{
    input::{
        common_conditions::{input_just_pressed, input_pressed},
        mouse::MouseWheel,
    },
    platform::collections::HashSet,
    prelude::*,
};
use bevy_falling_sand::prelude::*;

use crate::{
    app_state::{AppState, CanvasState},
    brush::{BrushModeState, BrushSize, BrushTypeState, MaxBrushSize},
    cursor::{update_cursor_position, CursorPosition},
    particles::SelectedParticle,
};

use super::{
    overlays::{draw_cursor_guide, DrawCursorGuide},
    RenderGui,
};

pub(super) struct QuickActionsPlugin;

impl Plugin for QuickActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                toggle_resource::<RenderGui>.run_if(input_just_pressed(KeyCode::KeyH)),
                toggle_resource::<DebugParticleMap>.run_if(input_just_pressed(KeyCode::F1)),
                toggle_resource::<DrawCursorGuide>.run_if(input_just_pressed(KeyCode::F2)),
                toggle_resource::<DebugDirtyRects>.run_if(input_just_pressed(KeyCode::F3)),
                toggle_particle_movement_logic.run_if(input_just_pressed(KeyCode::F4)),
                toggle_simulation_run
                    .run_if(input_just_pressed(KeyCode::Space))
                    .run_if(in_state(AppState::Canvas)),
                step_simulation.run_if(input_just_pressed(KeyCode::Enter)),
                update_brush_type_state.run_if(input_just_pressed(MouseButton::Back)),
                update_brush_spawn_state.run_if(input_just_pressed(MouseButton::Right)),
                resize_brush
                    .run_if(in_state(AppState::Canvas))
                    .run_if(in_state(CanvasState::Edit)),
                spawn_particles
                    .run_if(input_pressed(MouseButton::Left))
                    .run_if(in_state(BrushModeState::Spawn))
                    .run_if(in_state(AppState::Canvas))
                    .before(ParticleSimulationSet)
                    .after(update_cursor_position),
                despawn_particles
                    .run_if(input_pressed(MouseButton::Left))
                    .run_if(in_state(BrushModeState::Despawn))
                    .run_if(in_state(AppState::Canvas))
                    .before(ParticleSimulationSet)
                    .after(update_cursor_position),
                draw_cursor_guide
                    .run_if(resource_exists::<DrawCursorGuide>)
                    .after(ParticleDebugSet),
            ),
        );
    }
}

fn toggle_resource<T: Resource + Default>(mut commands: Commands, resource: Option<Res<T>>) {
    if resource.is_some() {
        commands.remove_resource::<T>();
    } else {
        commands.init_resource::<T>();
    }
}

fn toggle_particle_movement_logic(
    particle_movement_state_current: Res<State<MovementSource>>,
    mut particle_movement_state_next: ResMut<NextState<MovementSource>>,
) {
    match particle_movement_state_current.get() {
        MovementSource::Chunks => {
            particle_movement_state_next.set(MovementSource::Particles);
        }
        MovementSource::Particles => {
            particle_movement_state_next.set(MovementSource::Chunks);
        }
    }
}

fn toggle_simulation_run(
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

fn step_simulation(mut evw_simulation_step: EventWriter<SimulationStepEvent>) {
    evw_simulation_step.write(SimulationStepEvent);
}

fn update_brush_spawn_state(
    brush_spawn_state: Res<State<BrushModeState>>,
    mut brush_spawn_state_next: ResMut<NextState<BrushModeState>>,
) {
    match brush_spawn_state.get() {
        BrushModeState::Spawn => brush_spawn_state_next.set(BrushModeState::Despawn),
        BrushModeState::Despawn => brush_spawn_state_next.set(BrushModeState::Spawn),
    }
}

fn resize_brush(
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

fn update_brush_type_state(
    brush_type_state_current: Res<State<BrushTypeState>>,
    mut brush_type_state_next: ResMut<NextState<BrushTypeState>>,
) {
    match brush_type_state_current.get() {
        BrushTypeState::Line => brush_type_state_next.set(BrushTypeState::Circle),
        BrushTypeState::Circle => brush_type_state_next.set(BrushTypeState::Cursor),
        BrushTypeState::Cursor => brush_type_state_next.set(BrushTypeState::Line),
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
            } else {
                // Spawn particle at current cursor position when not moving
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

    Ok(())
}

fn despawn_particles(
    mut evw_remove_particle: EventWriter<DespawnParticleEvent>,
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
            } else {
                // Despawn particle at current cursor position when not moving
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
