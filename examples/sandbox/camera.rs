use bevy::prelude::*;

use crate::AppState;

/// UI plugin
pub(super) struct CameraPlugin;

impl bevy::prelude::Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Camera control
        app.add_systems(Startup, setup_camera).add_systems(
            Update,
            pan_camera.run_if(in_state(AppState::Canvas)),
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
                scale: 0.11,
                ..default()
            },
            ..default()
        },
        MainCamera,
    ));
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
