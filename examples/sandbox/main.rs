//! A complete example on how to build an interactive particle sandbox with bevy_falling_sand
mod brush;
mod camera;
mod debug;
mod particle_management;
mod particle_setup;
mod scenes;
mod ui;

use brush::*;
use camera::*;
use debug::*;
use particle_management::*;
use particle_setup::*;
use scenes::*;
use ui::*;

use bevy::{prelude::*, window::WindowMode};
use bevy_egui::EguiPlugin;
use bevy_falling_sand::FallingSandPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Falling Sandbox".into(),
                mode: WindowMode::BorderlessFullscreen,
                ..default()
            }),
            ..default()
        }),
        EguiPlugin,
        FallingSandPlugin,
        // Plugins provided by the modules defined in this example.
        CameraPlugin,
        ParticleSetupPlugin,
        ParticleManagementPlugin,
        BrushPlugin,
        ScenesPlugin,
        DebugPlugin,
        UIPlugin,
    ))
    .insert_resource(ClearColor(Color::srgba(0.17, 0.16, 0.15, 1.0)))
    .run();
}
