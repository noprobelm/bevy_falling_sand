use bevy::{input::mouse::MouseWheel, prelude::*};

use crate::app_state::{AppState, AppStateDetectionSet, CanvasState};

pub(super) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_systems(
            Update,
            (pan_camera, zoom_camera, smooth_zoom)
                .chain()
                .run_if(in_state(AppState::Canvas))
                .run_if(in_state(CanvasState::Interact))
                .before(AppStateDetectionSet),
        );
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
struct ZoomTarget {
    target_scale: f32,
    current_scale: f32,
}

#[derive(Component)]
struct ZoomSpeed(f32);

fn setup_camera(mut commands: Commands) {
    let initial_scale = 0.11;
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            near: -1000.0,
            scale: initial_scale,
            ..OrthographicProjection::default_2d()
        }),
        MainCamera,
        ZoomTarget {
            target_scale: initial_scale,
            current_scale: initial_scale,
        },
        ZoomSpeed(8.0),
    ));
}

fn pan_camera(
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
) -> Result {
    let mut transform = camera_query.single_mut()?;

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
    Ok(())
}

fn zoom_camera(
    mut ev_scroll: EventReader<MouseWheel>,
    mut camera_query: Query<&mut ZoomTarget, With<MainCamera>>,
) {
    const ZOOM_IN_FACTOR: f32 = 0.9;
    const ZOOM_OUT_FACTOR: f32 = 1.1;
    const MIN_SCALE: f32 = 0.01;
    const MAX_SCALE: f32 = 10.0;

    if !ev_scroll.is_empty() {
        let mut zoom_target = match camera_query.single_mut() {
            Ok(z) => z,
            Err(_) => return,
        };

        ev_scroll.read().for_each(|ev| {
            if ev.y < 0. {
                zoom_target.target_scale =
                    (zoom_target.target_scale * ZOOM_OUT_FACTOR).min(MAX_SCALE);
            } else if ev.y > 0. {
                zoom_target.target_scale =
                    (zoom_target.target_scale * ZOOM_IN_FACTOR).max(MIN_SCALE);
            }
        });
    }
}

fn smooth_zoom(
    mut camera_query: Query<(&mut Projection, &mut ZoomTarget, &ZoomSpeed), With<MainCamera>>,
    time: Res<Time>,
) {
    let (mut projection, mut zoom_target, zoom_speed) = match camera_query.single_mut() {
        Ok(q) => q,
        Err(_) => return,
    };

    let Projection::Orthographic(orthographic) = projection.as_mut() else {
        return;
    };

    let diff = zoom_target.target_scale - zoom_target.current_scale;
    if diff.abs() > 0.0001 {
        let delta = diff * zoom_speed.0 * time.delta_secs();
        zoom_target.current_scale += delta;
        orthographic.scale = zoom_target.current_scale;
    } else {
        zoom_target.current_scale = zoom_target.target_scale;
        orthographic.scale = zoom_target.target_scale;
    }
}
