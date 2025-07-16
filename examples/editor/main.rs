use std::time::Duration;

use bevy::{prelude::*, window::WindowMode};
use bevy_egui::EguiPlugin;
use bevy_falling_sand::prelude::{FallingSandDebugPlugin, FallingSandPlugin};
use bfs_assets::FallingSandAssetsPlugin;

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
            EguiPlugin {
                enable_multipass_for_primary_context: false,
            },
            FallingSandPlugin::default()
                .with_length_unit(8.0)
                .with_spatial_refresh_frequency(Duration::from_millis(50))
                .with_gravity(Vec2::NEG_Y * 50.0),
            FallingSandDebugPlugin,
            FallingSandAssetsPlugin,
            CameraPlugin,
            ParticleSetupPlugin,
            BrushPlugin,
            ScenesPlugin,
            ParticleFilesPlugin,
            UIPlugin,
        ))
        .insert_resource(ClearColor(Color::srgba(0.17, 0.16, 0.15, 1.0)))
        .run();
}
