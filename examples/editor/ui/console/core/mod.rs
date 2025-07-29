pub mod commands;
pub mod core;

use bevy::prelude::*;
use commands::ConsoleCommandsPlugin;
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
            .add_systems(Update, command_handler)
            .add_plugins(ConsoleCommandsPlugin);
    }
}
