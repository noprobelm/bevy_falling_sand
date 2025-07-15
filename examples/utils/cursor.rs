use bevy::{prelude::*, window::PrimaryWindow};

use super::camera::MainCamera;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPosition>()
            .add_systems(Update, update_cursor_position);
    }
}

#[derive(Default, Resource, Clone, Debug)]
pub struct CursorPosition {
    pub current: Vec2,
    pub previous: Vec2,
    pub previous_previous: Vec2,
}

impl CursorPosition {
    pub fn update(&mut self, new_coords: Vec2) {
        self.previous_previous = self.previous;
        self.previous = self.current;
        self.current = new_coords;
    }
}

pub fn update_cursor_position(
    mut coords: ResMut<CursorPosition>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Result {
    let (camera, camera_transform) = q_camera.single()?;

    let window = q_window.single()?;
    if let Some(world_position) = window
        .cursor_position()
        .and_then(
            |cursor| -> Option<
                std::result::Result<Ray3d, bevy::render::camera::ViewportConversionError>,
            > { Some(camera.viewport_to_world(camera_transform, cursor)) },
        )
        .map(|ray| ray.unwrap().origin.truncate())
    {
        coords.update(world_position);
    }
    Ok(())
}
