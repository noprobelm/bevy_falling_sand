use bevy::prelude::*;
use clap::Parser;

use crate::console::core::{ConsoleCommandEntered, NamedCommand};

#[derive(Parser, Resource)]
#[command(name = "exit")]
pub struct ExitCommand {
    command: Option<String>,
}

impl NamedCommand for ExitCommand {
    fn name() -> &'static str {
        "exit"
    }
}

pub fn exit_command(
    mut cmd: EventReader<ConsoleCommandEntered>,
    mut evw_app_exit: EventWriter<AppExit>,
) {
    for command_event in cmd.read() {
        if command_event.command_name == "exit" {
            evw_app_exit.write(AppExit::Success);
        }
    }
}
