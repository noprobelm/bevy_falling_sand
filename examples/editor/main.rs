mod app_state;
mod brush;
mod camera;
mod console;
mod cursor;
mod keybindings;
mod particles;
mod ui;

use app_state::StatesPlugin;
use bevy_falling_sand::prelude::{FallingSandDebugPlugin, FallingSandPlugin};
use brush::*;
use camera::*;
use cursor::*;
use particles::*;
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
            FallingSandPlugin::default(),
            FallingSandDebugPlugin,
            ParticleSetupPlugin,
            CursorPlugin,
            CameraPlugin,
            BrushPlugin,
            StatesPlugin,
            UiPlugin,
        ))
        .insert_resource(ClearColor(Color::srgba(0.17, 0.16, 0.15, 1.0)))
        .run();
}
