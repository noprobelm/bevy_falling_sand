mod camera;
mod keybindings;
mod ui;

use camera::*;
use ui::*;

use bevy::{prelude::*, window::WindowMode};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Falling Sandbox".into(),
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                    ..default()
                }),
                ..default()
            }),
            UiPlugin,
            CameraPlugin,
        ))
        .insert_resource(ClearColor(Color::srgba(0.17, 0.16, 0.15, 1.0)))
        .run();
}
