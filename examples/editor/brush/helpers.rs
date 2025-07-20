use bevy::{platform::collections::HashSet, prelude::*};
use bfs_core::Particle;

pub(crate) fn points_within_capsule(capsule: &Capsule2d, start: Vec2, end: Vec2) -> Vec<IVec2> {
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

pub(crate) fn spawn_circle(
    commands: &mut Commands,
    particle: Particle,
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

    commands.spawn_batch(points.into_iter().map(move |point| {
        (
            particle.clone(),
            Transform::from_xyz(point.x as f32, point.y as f32, 0.0),
        )
    }));
}

pub(crate) fn spawn_capsule(
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

pub(crate) fn spawn_line(
    commands: &mut Commands,
    particle: Particle,
    center: Vec2,
    brush_size: usize,
) {
    let min_x = -(brush_size as i32) / 2;
    let max_x = (brush_size as f32 / 2.0) as i32;

    commands.spawn_batch((min_x * 3..=max_x * 3).map(move |x| {
        (
            particle.clone(),
            Transform::from_xyz((center.x + x as f32).round(), center.y.round(), 0.0),
        )
    }));
}

pub(crate) fn spawn_line_interpolated(
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

    // Sample points along the interpolated line
    let num_samples = (length.ceil() as usize).max(1);
    for i in 0..=num_samples {
        let t = i as f32 / num_samples.max(1) as f32;
        let sample_point = start + direction * length * t;

        // For each sample point, spawn a line
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

pub fn spawn_cursor_interpolated(
    commands: &mut Commands,
    particle: Particle,
    start: Vec2,
    end: Vec2,
) {
    let mut points: HashSet<IVec2> = HashSet::default();
    let direction = (end - start).normalize();
    let length = (end - start).length();

    // Sample points along the interpolated path
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
