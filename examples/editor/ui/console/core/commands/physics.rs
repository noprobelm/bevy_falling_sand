use bevy::prelude::*;

use super::super::core::{ConsoleCommand, PrintConsoleLine};

#[derive(Default)]
pub struct PhysicsCommand;

impl ConsoleCommand for PhysicsCommand {
    fn name(&self) -> &'static str {
        "physics"
    }

    fn description(&self) -> &'static str {
        "Physics system operations"
    }

    fn execute(
        &self,
        path: &[String],
        args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        match path.len() {
            1 => {
                console_writer.write(PrintConsoleLine::new(
                    "error: 'physics' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: debug".to_string(),
                ));
            }
            _ => {
                if path.len() >= 2 {
                    match path[1].as_str() {
                        "debug" => {
                            PhysicsDebugCommand.execute(path, args, console_writer, commands)
                        }
                        _ => {
                            console_writer.write(PrintConsoleLine::new(format!(
                                "error: Unknown subcommand 'physics {}'",
                                path[1]
                            )));
                        }
                    }
                }
            }
        }
    }

    fn subcommands(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![Box::new(PhysicsDebugCommand)]
    }
}

#[derive(Default)]
pub struct PhysicsDebugCommand;

impl ConsoleCommand for PhysicsDebugCommand {
    fn name(&self) -> &'static str {
        "debug"
    }

    fn description(&self) -> &'static str {
        "Physics debug options"
    }

    fn execute(
        &self,
        path: &[String],
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _commands: &mut Commands,
    ) {
        match path.len() {
            2 => {
                console_writer.write(PrintConsoleLine::new(
                    "error: 'physics debug' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: enable, disable".to_string(),
                ));
            }
            3 => match path[2].as_str() {
                "enable" => {
                    println!("Executing: physics debug enable");
                    console_writer.write(PrintConsoleLine::new(
                        "Enabling physics debug overlay...".to_string(),
                    ));
                }
                "disable" => {
                    println!("Executing: physics debug disable");
                    console_writer.write(PrintConsoleLine::new(
                        "Disabling physics debug overlay...".to_string(),
                    ));
                }
                _ => {
                    console_writer.write(PrintConsoleLine::new(format!(
                        "error: Unknown command 'physics debug {}'",
                        path[2]
                    )));
                }
            },
            _ => {
                console_writer.write(PrintConsoleLine::new(format!(
                    "error: Invalid command path: {}",
                    path.join(" ")
                )));
            }
        }
    }

    fn subcommands(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![
            Box::new(PhysicsDebugEnableCommand),
            Box::new(PhysicsDebugDisableCommand),
        ]
    }
}

#[derive(Default)]
pub struct PhysicsDebugEnableCommand;

impl ConsoleCommand for PhysicsDebugEnableCommand {
    fn name(&self) -> &'static str {
        "enable"
    }

    fn description(&self) -> &'static str {
        "Enable physics debug overlay"
    }

    fn execute(
        &self,
        _path: &[String],
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _commands: &mut Commands,
    ) {
        println!("Executing: physics debug enable");
        console_writer.write(PrintConsoleLine::new(
            "Enabling physics debug overlay...".to_string(),
        ));
    }
}

#[derive(Default)]
pub struct PhysicsDebugDisableCommand;

impl ConsoleCommand for PhysicsDebugDisableCommand {
    fn name(&self) -> &'static str {
        "disable"
    }

    fn description(&self) -> &'static str {
        "Disable physics debug overlay"
    }

    fn execute(
        &self,
        _path: &[String],
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _commands: &mut Commands,
    ) {
        println!("Executing: physics debug disable");
        console_writer.write(PrintConsoleLine::new(
            "Disabling physics debug overlay...".to_string(),
        ));
    }
}