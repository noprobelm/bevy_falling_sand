use bevy::prelude::*;
use clap::Parser;

use crate::console::core::{ConsoleCommandEntered, NamedCommand, PrintConsoleLine};

#[derive(Parser, Resource)]
#[command(name = "echo")]
pub struct EchoCommand {
    message: String,
}

impl NamedCommand for EchoCommand {
    fn name() -> &'static str {
        "echo"
    }
}

pub fn echo_command(
    mut cmd: EventReader<ConsoleCommandEntered>,
    mut writer: EventWriter<PrintConsoleLine>,
) {
    for command_event in cmd.read() {
        if command_event.command_name == "echo" {
            let message = command_event.args.join(" ");
            writer.write(PrintConsoleLine::new(message));
        }
    }
}

