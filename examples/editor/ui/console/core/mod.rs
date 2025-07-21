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
            .add_event::<ConsoleCommandEntered>()
            .add_event::<PrintConsoleLine>()
            .add_systems(Startup, core::init_commands)
            .add_systems(
                Update,
                (help_command, clear_command, exit_command, echo_command),
            );
    }
}
