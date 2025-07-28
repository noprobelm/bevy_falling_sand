use bevy::prelude::*;

use super::super::core::{Command, ExitCommandEvent, PrintConsoleLine};

#[derive(Default)]
pub struct DebugCommand;

impl Command for DebugCommand {
    fn name(&self) -> &'static str {
        "debug"
    }
    
    fn description(&self) -> &'static str {
        "Debug system controls"
    }
    
    fn execute(
        &self,
        path: &[String],
        args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _exit_writer: &mut EventWriter<ExitCommandEvent>,
        commands: &mut Commands,
    ) {
        match path.len() {
            1 => {
                console_writer.write(PrintConsoleLine::new(
                    "error: 'debug' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: physics, particles".to_string(),
                ));
            }
            _ => {
                // Route to subcommands
                if path.len() >= 2 {
                    match path[1].as_str() {
                        "physics" => DebugPhysicsCommand.execute(path, args, console_writer, _exit_writer, commands),
                        "particles" => DebugParticlesCommand.execute(path, args, console_writer, _exit_writer, commands),
                        _ => {
                            console_writer.write(PrintConsoleLine::new(format!(
                                "error: Unknown subcommand 'debug {}'",
                                path[1]
                            )));
                        }
                    }
                }
            }
        }
    }
    
    fn subcommands(&self) -> Vec<Box<dyn Command>> {
        vec![
            Box::new(DebugPhysicsCommand),
            Box::new(DebugParticlesCommand),
        ]
    }
}

#[derive(Default)]
pub struct DebugPhysicsCommand;

impl Command for DebugPhysicsCommand {
    fn name(&self) -> &'static str {
        "physics"
    }
    
    fn description(&self) -> &'static str {
        "Physics debug options"
    }
    
    fn execute(
        &self,
        path: &[String],
        args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _exit_writer: &mut EventWriter<ExitCommandEvent>,
        commands: &mut Commands,
    ) {
        match path.len() {
            2 => {
                console_writer.write(PrintConsoleLine::new(
                    "error: 'debug physics' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: enable, disable".to_string(),
                ));
            }
            3 => match path[2].as_str() {
                "enable" => {
                    println!("Executing: debug physics enable");
                    console_writer.write(PrintConsoleLine::new(
                        "Enabling physics debug overlay...".to_string(),
                    ));
                }
                "disable" => {
                    println!("Executing: debug physics disable");
                    console_writer.write(PrintConsoleLine::new(
                        "Disabling physics debug overlay...".to_string(),
                    ));
                }
                _ => {
                    console_writer.write(PrintConsoleLine::new(format!(
                        "error: Unknown command 'debug physics {}'",
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
    
    fn subcommands(&self) -> Vec<Box<dyn Command>> {
        vec![
            Box::new(DebugPhysicsEnableCommand),
            Box::new(DebugPhysicsDisableCommand),
        ]
    }
}

#[derive(Default)]
pub struct DebugPhysicsEnableCommand;

impl Command for DebugPhysicsEnableCommand {
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
        _exit_writer: &mut EventWriter<ExitCommandEvent>,
        _commands: &mut Commands,
    ) {
        println!("Executing: debug physics enable");
        console_writer.write(PrintConsoleLine::new(
            "Enabling physics debug overlay...".to_string(),
        ));
    }
}

#[derive(Default)]
pub struct DebugPhysicsDisableCommand;

impl Command for DebugPhysicsDisableCommand {
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
        _exit_writer: &mut EventWriter<ExitCommandEvent>,
        _commands: &mut Commands,
    ) {
        println!("Executing: debug physics disable");
        console_writer.write(PrintConsoleLine::new(
            "Disabling physics debug overlay...".to_string(),
        ));
    }
}

#[derive(Default)]
pub struct DebugParticlesCommand;

impl Command for DebugParticlesCommand {
    fn name(&self) -> &'static str {
        "particles"
    }
    
    fn description(&self) -> &'static str {
        "Particle debug options"
    }
    
    fn execute(
        &self,
        path: &[String],
        args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _exit_writer: &mut EventWriter<ExitCommandEvent>,
        commands: &mut Commands,
    ) {
        match path.len() {
            2 => {
                console_writer.write(PrintConsoleLine::new(
                    "error: 'debug particles' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: count".to_string(),
                ));
            }
            3 => match path[2].as_str() {
                "count" => DebugParticlesCountCommand.execute(path, args, console_writer, _exit_writer, commands),
                _ => {
                    console_writer.write(PrintConsoleLine::new(format!(
                        "error: Unknown command 'debug particles {}'",
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
    
    fn subcommands(&self) -> Vec<Box<dyn Command>> {
        vec![
            Box::new(DebugParticlesCountCommand),
        ]
    }
}

#[derive(Default)]
pub struct DebugParticlesCountCommand;

impl Command for DebugParticlesCountCommand {
    fn name(&self) -> &'static str {
        "count"
    }
    
    fn description(&self) -> &'static str {
        "Show particle count"
    }
    
    fn execute(
        &self,
        _path: &[String],
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _exit_writer: &mut EventWriter<ExitCommandEvent>,
        _commands: &mut Commands,
    ) {
        println!("Executing: debug particles count");
        console_writer.write(PrintConsoleLine::new(
            "Current particle count: 1234 particles".to_string(),
        ));
    }
}

// Legacy function for compatibility during transition - will be removed
pub fn handle_debug_command(
    path: &[String],
    args: &[String],
    writer: &mut EventWriter<PrintConsoleLine>,
) {
    // For legacy compatibility, manually handle the debug command logic
    match path.len() {
        1 => {
            writer.write(PrintConsoleLine::new(
                "error: 'debug' requires a subcommand".to_string(),
            ));
            writer.write(PrintConsoleLine::new(
                "Available subcommands: physics, particles".to_string(),
            ));
        }
        2 => match path[1].as_str() {
            "physics" => {
                writer.write(PrintConsoleLine::new(
                    "error: 'debug physics' requires a subcommand".to_string(),
                ));
                writer.write(PrintConsoleLine::new(
                    "Available subcommands: enable, disable".to_string(),
                ));
            }
            "particles" => {
                writer.write(PrintConsoleLine::new(
                    "error: 'debug particles' requires a subcommand".to_string(),
                ));
                writer.write(PrintConsoleLine::new(
                    "Available subcommands: count".to_string(),
                ));
            }
            _ => {
                writer.write(PrintConsoleLine::new(format!(
                    "error: Unknown subcommand 'debug {}'",
                    path[1]
                )));
            }
        },
        3 => match (path[1].as_str(), path[2].as_str()) {
            ("physics", "enable") => {
                println!("Executing: debug physics enable");
                writer.write(PrintConsoleLine::new(
                    "Enabling physics debug overlay...".to_string(),
                ));
            }
            ("physics", "disable") => {
                println!("Executing: debug physics disable");
                writer.write(PrintConsoleLine::new(
                    "Disabling physics debug overlay...".to_string(),
                ));
            }
            ("particles", "count") => {
                println!("Executing: debug particles count");
                writer.write(PrintConsoleLine::new(
                    "Current particle count: 1234 particles".to_string(),
                ));
            }
            _ => {
                writer.write(PrintConsoleLine::new(format!(
                    "error: Unknown command path: {}",
                    path.join(" ")
                )));
            }
        },
        _ => {
            writer.write(PrintConsoleLine::new(format!(
                "error: Invalid command path: {}",
                path.join(" ")
            )));
        }
    }
}
