use bevy::prelude::*;
use clap::Parser;

use crate::console::core::{ConsoleCommandEntered, NamedCommand};

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

pub fn echo_command(mut cmd: EventReader<ConsoleCommandEntered>) {}
