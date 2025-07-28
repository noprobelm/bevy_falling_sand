use bevy::prelude::*;
use clap::Parser;

use super::super::core::{ConsoleCommand, ConsoleCommandEntered, NamedCommand, PrintConsoleLine};

pub struct ExitCommandPlugin;

impl Plugin for ExitCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_exit_application);
    }
}

#[derive(Event)]
pub struct ExitApplicationEvent;

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

impl ConsoleCommand for ExitConsoleCommand {
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
        commands: &mut Commands,
    ) {
        if path.len() == 1 && path[0] == "exit" {
            commands.trigger(ExitApplicationEvent);
        }
    }
}

impl ConsoleCommand for ExitCommand {
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
        commands: &mut Commands,
    ) {
        if path.len() == 1 && path[0] == "exit" {
            commands.trigger(ExitApplicationEvent);
        }
    }
}

fn on_exit_application(
    _trigger: Trigger<ExitApplicationEvent>,
    mut app_exit: EventWriter<AppExit>,
) {
    println!("Exit command triggered - shutting down application");
    app_exit.write(AppExit::Success);
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
