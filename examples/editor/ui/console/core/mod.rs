pub mod commands;
pub mod core;

use bevy::prelude::*;
use commands::*;
pub use core::*;

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConsoleState>()
            .init_resource::<ConsoleConfiguration>()
            .init_resource::<ConsoleCache>()
            .init_resource::<CommandRegistry>()
            .add_event::<ConsoleCommandEntered>()
            .add_event::<PrintConsoleLine>()
            .add_systems(Startup, (core::init_commands, init_command_registry))
            .add_systems(
                Update,
                (help_command, clear_command, exit_command, command_handler),
            );
    }
}

fn init_command_registry(
    mut registry: ResMut<CommandRegistry>,
    mut config: ResMut<ConsoleConfiguration>,
    mut cache: ResMut<ConsoleCache>,
) {
    use commands::{camera::*, exit::ExitConsoleCommand, particles::*, physics::*};

    registry.register::<ExitConsoleCommand>();
    registry.register::<ParticlesCommand>();
    registry.register::<CameraCommand>();
    registry.register::<PhysicsCommand>();

    config.register_command::<ExitConsoleCommand>();
    config.register_command::<ParticlesCommand>();
    config.register_command::<CameraCommand>();
    config.register_command::<PhysicsCommand>();
    cache.rebuild_tries(&config);
}
