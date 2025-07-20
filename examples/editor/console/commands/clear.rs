use bevy::prelude::*;
use clap::Parser;

use crate::console::core::{ConsoleCommandEntered, ConsoleState, NamedCommand};

#[derive(Parser, Resource)]
#[command(name = "clear", about = "Clear the console output")]
pub struct ClearCommand;

impl NamedCommand for ClearCommand {
    fn name() -> &'static str {
        "clear"
    }
}

pub fn clear_command(
    mut cmd: EventReader<ConsoleCommandEntered>,
    mut console_state: ResMut<ConsoleState>,
) {
    for command_event in cmd.read() {
        if command_event.command_name == "clear" {
            console_state.messages.clear();
            console_state.add_message("--- Bevy Falling Sand Editor Console ---".to_string());
        }
    }
}

