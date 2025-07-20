mod camera;
mod keybindings;
mod ui;

use camera::*;
use ui::*;

use bevy::{prelude::*, window::WindowMode};
use bevy_console::{AddConsoleCommand, ConsoleCommand, ConsoleConfiguration, ConsolePlugin, reply};
use clap::Parser;

/// Print Hello World to the console
#[derive(Parser, ConsoleCommand)]
#[command(name = "hello")]
struct HelloWorldCommand {
    /// Optional name to greet (defaults to "World")
    #[arg(short, long, default_value = "World")]
    name: String,
}

fn hello_world_command(mut log: ConsoleCommand<HelloWorldCommand>) {
    if let Some(Ok(HelloWorldCommand { name })) = log.take() {
        reply!(log, "Hello, {}!", name);
    }
}

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
            ConsolePlugin,
        ))
        .insert_resource(ConsoleConfiguration {
            ..default()
        })
        .add_console_command::<HelloWorldCommand, _>(hello_world_command)
        .insert_resource(ClearColor(Color::srgba(0.17, 0.16, 0.15, 1.0)))
        .run();
}
