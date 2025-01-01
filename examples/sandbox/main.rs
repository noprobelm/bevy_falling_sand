//! A complete example on how to build an interactive particle sandbox with bevy_falling_sand
mod brush;
mod camera;
mod scenes;
mod setup;
mod ui;

use brush::*;
use camera::*;
use scenes::*;
use setup::*;
use ui::*;

use bevy::{prelude::*, window::WindowMode};
use bevy_egui::EguiPlugin;
use bevy_falling_sand::prelude::FallingSandPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Falling Sandbox".into(),
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }),
        EguiPlugin,
        FallingSandPlugin,
        // Plugins provided by the modules defined in this example.
        CameraPlugin,
        ParticleSetupPlugin,
        BrushPlugin,
        ScenesPlugin,
        UIPlugin,
    ))
    .insert_resource(ClearColor(Color::srgba(0.17, 0.16, 0.15, 1.0)))
    .run();
}
