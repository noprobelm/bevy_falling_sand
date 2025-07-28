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
                (
                    help_command,
                    clear_command,
                    exit_command,
                    echo_command,
                    command_handler,
                ),
            );
    }
}

fn init_command_registry(
    mut registry: ResMut<CommandRegistry>,
    mut config: ResMut<ConsoleConfiguration>,
) {
    use commands::{debug::*, exit::ExitConsoleCommand, reset::*};

    // Register commands
    registry.register::<ExitConsoleCommand>();
    registry.register::<DebugCommand>();
    registry.register::<ResetCommand>();

    // Register in configuration for completion
    config.register_command::<ExitConsoleCommand>();
    config.register_command::<DebugCommand>();
    config.register_command::<ResetCommand>();
}
