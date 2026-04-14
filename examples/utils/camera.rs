use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_falling_sand::prelude::ChunkLoader;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct ZoomTarget {
    pub target_scale: f32,
    pub current_scale: f32,
}

const PAN_SPEED_FACTOR: f32 = 10.;
const ZOOM_SPEED: f32 = 8.0;
const ZOOM_IN_FACTOR: f32 = 0.9;
const ZOOM_OUT_FACTOR: f32 = 1.1;
const MIN_SCALE: f32 = 0.01;
const MAX_SCALE: f32 = 10.0;

pub fn setup_camera(mut commands: Commands) {
    let initial_scale = 0.25;
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            near: -1000.0,
            scale: initial_scale,
            ..OrthographicProjection::default_2d()
        }),
        MainCamera,
        ChunkLoader,
        ZoomTarget {
            target_scale: initial_scale,
            current_scale: initial_scale,
        },
    ));
}

pub fn pan_camera(
    mut camera_query: Query<(&mut Transform, &ZoomTarget), With<MainCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
) -> Result {
    let (mut transform, zoom_target) = camera_query.single_mut()?;
    let speed = PAN_SPEED_FACTOR * zoom_target.current_scale;

    if keys.pressed(KeyCode::KeyW) {
        transform.translation.y += speed;
    }
    if keys.pressed(KeyCode::KeyA) {
        transform.translation.x -= speed;
    }
    if keys.pressed(KeyCode::KeyS) {
        transform.translation.y -= speed;
    }
    if keys.pressed(KeyCode::KeyD) {
        transform.translation.x += speed;
    }
    Ok(())
}

pub fn zoom_camera(
    mut ev_scroll: MessageReader<MouseWheel>,
    mut camera_query: Query<&mut ZoomTarget, With<MainCamera>>,
) {
    let Ok(mut zoom_target) = camera_query.single_mut() else {
        return;
    };

    for ev in ev_scroll.read() {
        if ev.y > 0. {
            zoom_target.target_scale = (zoom_target.target_scale * ZOOM_IN_FACTOR).max(MIN_SCALE);
        } else if ev.y < 0. {
            zoom_target.target_scale = (zoom_target.target_scale * ZOOM_OUT_FACTOR).min(MAX_SCALE);
        }
    }
}

pub fn smooth_zoom(
    mut camera_query: Query<(&mut Projection, &mut ZoomTarget), With<MainCamera>>,
    time: Res<Time>,
) {
    let Ok((mut projection, mut zoom_target)) = camera_query.single_mut() else {
        return;
    };
    let Projection::Orthographic(orthographic) = projection.as_mut() else {
        return;
    };

    let diff = zoom_target.target_scale - zoom_target.current_scale;
    if diff.abs() > 0.0001 {
        let delta = diff * ZOOM_SPEED * time.delta_secs();
        zoom_target.current_scale += delta;
        orthographic.scale = zoom_target.current_scale;
    } else {
        zoom_target.current_scale = zoom_target.target_scale;
        orthographic.scale = zoom_target.target_scale;
    }
}
