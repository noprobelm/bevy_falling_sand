use bevy::prelude::*;
use clap::Parser;

use super::super::core::{Command, ConsoleCommandEntered, ExitCommandEvent, NamedCommand, PrintConsoleLine};

#[derive(Parser, Resource)]
#[command(name = "exit", about = "Exit the application")]
pub struct ExitCommand {
    command: Option<String>,
}

impl NamedCommand for ExitCommand {
    fn name() -> &'static str {
        "exit"
    }
}

#[derive(Default)]
pub struct ExitConsoleCommand;

impl Command for ExitConsoleCommand {
    fn name(&self) -> &'static str {
        "exit"
    }
    
    fn description(&self) -> &'static str {
        "Exit the application"
    }
    
    fn execute(
        &self,
        path: &[String],
        _args: &[String],
        _console_writer: &mut EventWriter<PrintConsoleLine>,
        exit_writer: &mut EventWriter<ExitCommandEvent>,
    ) {
        if path.len() == 1 && path[0] == "exit" {
            exit_writer.write(ExitCommandEvent);
        }
    }
}

impl Command for ExitCommand {
    fn name(&self) -> &'static str {
        "exit"
    }
    
    fn description(&self) -> &'static str {
        "Exit the application"
    }
    
    fn execute(
        &self,
        path: &[String],
        _args: &[String],
        _console_writer: &mut EventWriter<PrintConsoleLine>,
        exit_writer: &mut EventWriter<ExitCommandEvent>,
    ) {
        if path.len() == 1 && path[0] == "exit" {
            exit_writer.write(ExitCommandEvent);
        }
    }
}

pub fn exit_command(
    mut cmd: EventReader<ConsoleCommandEntered>,
    mut evw_app_exit: EventWriter<AppExit>,
) {
    for command_event in cmd.read() {
        if command_event.command_path.len() == 1 && command_event.command_path[0] == "exit" {
            evw_app_exit.write(AppExit::Success);
        }
    }
}
