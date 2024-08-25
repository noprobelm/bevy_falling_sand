//! A complete example on how to build an interactive particle sandbox with bevy_falling_sand
mod camera;
mod particle_setup;
mod particle_management;
mod brush;
mod scenes;
mod debug;
mod ui;

use camera::*;
use particle_setup::*;
use particle_management::*;
use brush::*;
use scenes::*;
use debug::*;
use ui::*;

use bevy::{prelude::*, window::WindowMode};

use bevy_egui::EguiPlugin;

use bevy_falling_sand::*;

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
    .run();
}
