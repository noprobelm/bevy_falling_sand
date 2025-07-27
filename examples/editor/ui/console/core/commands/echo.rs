use bevy::prelude::*;
use clap::Parser;

use super::super::core::{NamedCommand, ConsoleCommandEntered, PrintConsoleLine};

#[derive(Parser, Resource)]
#[command(name = "echo", about = "Echo some text")]
pub struct EchoCommand {
    command: Option<String>,
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
        if command_event.command_path.len() == 1 && command_event.command_path[0] == "echo" {
            let text = command_event.args.join(" ");
            println!("Executing: echo {}", text);
            writer.write(PrintConsoleLine::new(text));
        }
    }
}
