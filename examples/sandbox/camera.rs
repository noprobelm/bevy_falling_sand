use bevy::{input::mouse::MouseWheel, prelude::*};

use crate::AppState;

/// UI plugin
pub(super) struct CameraPlugin;

impl bevy::prelude::Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Camera control
        app.add_systems(Startup, setup_camera).add_systems(
            Update,
            (zoom_camera, pan_camera).run_if(in_state(AppState::Canvas)),
        );
    }
}

/// The main camera.
#[derive(Component)]
pub struct MainCamera;

/// Sets up the camera.
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            projection: OrthographicProjection {
                near: -1000.0,
                // Particles occupy only 1 pixel each, so they're really tiny. Lower zoom scales are recommended for
                // this crate in order to maximize "bang for buck". 0.1 is a good scale to start with.
                scale: 0.11,
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
            println!("{:?}", projection.scale);
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
