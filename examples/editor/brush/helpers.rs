use bevy::prelude::*;

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
