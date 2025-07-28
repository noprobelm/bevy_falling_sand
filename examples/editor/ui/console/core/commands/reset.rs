use bevy::prelude::*;

use super::super::core::{Command, ExitCommandEvent, PrintConsoleLine};

#[derive(Default)]
pub struct ResetCommand;

impl Command for ResetCommand {
    fn name(&self) -> &'static str {
        "reset"
    }

    fn description(&self) -> &'static str {
        "Reset various system components"
    }

    fn execute(
        &self,
        path: &[String],
        args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _exit_writer: &mut EventWriter<ExitCommandEvent>,
    ) {
        match path.len() {
            1 => {
                console_writer.write(PrintConsoleLine::new(
                    "error: 'reset' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: particle, camera".to_string(),
                ));
            }
            _ => {
                if path.len() >= 2 {
                    match path[1].as_str() {
                        "particle" => {
                            ResetParticleCommand.execute(path, args, console_writer, _exit_writer)
                        }
                        "camera" => {
                            ResetCameraCommand.execute(path, args, console_writer, _exit_writer)
                        }
                        _ => {
                            console_writer.write(PrintConsoleLine::new(format!(
                                "error: Unknown subcommand 'reset {}'",
                                path[1]
                            )));
                        }
                    }
                }
            }
        }
    }

    fn subcommands(&self) -> Vec<Box<dyn Command>> {
        vec![Box::new(ResetParticleCommand), Box::new(ResetCameraCommand)]
    }
}

#[derive(Default)]
pub struct ResetParticleCommand;

impl Command for ResetParticleCommand {
    fn name(&self) -> &'static str {
        "particle"
    }

    fn description(&self) -> &'static str {
        "Reset particle-related components"
    }

    fn execute(
        &self,
        path: &[String],
        args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _exit_writer: &mut EventWriter<ExitCommandEvent>,
    ) {
        match path.len() {
            2 => {
                console_writer.write(PrintConsoleLine::new(
                    "error: 'reset particle' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: wall, dynamic".to_string(),
                ));
            }
            _ => {
                if path.len() >= 3 {
                    match path[2].as_str() {
                        "wall" => ResetParticleWallCommand.execute(
                            path,
                            args,
                            console_writer,
                            _exit_writer,
                        ),
                        "dynamic" => ResetParticleDynamicCommand.execute(
                            path,
                            args,
                            console_writer,
                            _exit_writer,
                        ),
                        _ => {
                            console_writer.write(PrintConsoleLine::new(format!(
                                "error: Unknown subcommand 'reset particle {}'",
                                path[2]
                            )));
                        }
                    }
                }
            }
        }
    }

    fn subcommands(&self) -> Vec<Box<dyn Command>> {
        vec![
            Box::new(ResetParticleWallCommand),
            Box::new(ResetParticleDynamicCommand),
        ]
    }
}

#[derive(Default)]
pub struct ResetCameraCommand;

impl Command for ResetCameraCommand {
    fn name(&self) -> &'static str {
        "camera"
    }

    fn description(&self) -> &'static str {
        "Reset camera position and zoom"
    }

    fn execute(
        &self,
        _path: &[String],
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _exit_writer: &mut EventWriter<ExitCommandEvent>,
    ) {
        println!("Executing: reset camera");
        console_writer.write(PrintConsoleLine::new(
            "Resetting camera to default position...".to_string(),
        ));
    }
}

#[derive(Default)]
pub struct ResetParticleWallCommand;

impl Command for ResetParticleWallCommand {
    fn name(&self) -> &'static str {
        "wall"
    }

    fn description(&self) -> &'static str {
        "Reset wall particles"
    }

    fn execute(
        &self,
        path: &[String],
        args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _exit_writer: &mut EventWriter<ExitCommandEvent>,
    ) {
        match path.len() {
            3 => {
                console_writer.write(PrintConsoleLine::new(
                    "error: 'reset particle wall' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: all".to_string(),
                ));
            }
            4 => {
                if path[3] == "all" {
                    ResetParticleWallAllCommand.execute(path, args, console_writer, _exit_writer);
                } else {
                    console_writer.write(PrintConsoleLine::new(format!(
                        "error: Unknown command 'reset particle wall {}'",
                        path[3]
                    )));
                }
            }
            _ => {
                console_writer.write(PrintConsoleLine::new(format!(
                    "error: Invalid command path: {}",
                    path.join(" ")
                )));
            }
        }
    }

    fn subcommands(&self) -> Vec<Box<dyn Command>> {
        vec![Box::new(ResetParticleWallAllCommand)]
    }
}

#[derive(Default)]
pub struct ResetParticleDynamicCommand;

impl Command for ResetParticleDynamicCommand {
    fn name(&self) -> &'static str {
        "dynamic"
    }

    fn description(&self) -> &'static str {
        "Reset dynamic particles"
    }

    fn execute(
        &self,
        path: &[String],
        args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _exit_writer: &mut EventWriter<ExitCommandEvent>,
    ) {
        match path.len() {
            3 => {
                console_writer.write(PrintConsoleLine::new(
                    "error: 'reset particle dynamic' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: all".to_string(),
                ));
            }
            4 => {
                if path[3] == "all" {
                    ResetParticleDynamicAllCommand.execute(
                        path,
                        args,
                        console_writer,
                        _exit_writer,
                    );
                } else {
                    console_writer.write(PrintConsoleLine::new(format!(
                        "error: Unknown command 'reset particle dynamic {}'",
                        path[3]
                    )));
                }
            }
            _ => {
                console_writer.write(PrintConsoleLine::new(format!(
                    "error: Invalid command path: {}",
                    path.join(" ")
                )));
            }
        }
    }

    fn subcommands(&self) -> Vec<Box<dyn Command>> {
        vec![Box::new(ResetParticleDynamicAllCommand)]
    }
}

#[derive(Default)]
pub struct ResetParticleWallAllCommand;

impl Command for ResetParticleWallAllCommand {
    fn name(&self) -> &'static str {
        "all"
    }

    fn description(&self) -> &'static str {
        "Reset all wall particles"
    }

    fn execute(
        &self,
        _path: &[String],
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _exit_writer: &mut EventWriter<ExitCommandEvent>,
    ) {
        println!("Executing: reset particle wall all");
        console_writer.write(PrintConsoleLine::new(
            "Resetting all wall particles...".to_string(),
        ));
    }
}

#[derive(Default)]
pub struct ResetParticleDynamicAllCommand;

impl Command for ResetParticleDynamicAllCommand {
    fn name(&self) -> &'static str {
        "all"
    }

    fn description(&self) -> &'static str {
        "Reset all dynamic particles"
    }

    fn execute(
        &self,
        _path: &[String],
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _exit_writer: &mut EventWriter<ExitCommandEvent>,
    ) {
        println!("Executing: reset particle dynamic all");
        console_writer.write(PrintConsoleLine::new(
            "Resetting all dynamic particles...".to_string(),
        ));
    }
}

// Legacy function for compatibility during transition - will be removed
pub fn handle_reset_command(
    path: &[String],
    _args: &[String],
    writer: &mut EventWriter<PrintConsoleLine>,
) {
    // For legacy compatibility, manually handle the reset command logic
    match path.len() {
        1 => {
            writer.write(PrintConsoleLine::new(
                "error: 'reset' requires a subcommand".to_string(),
            ));
            writer.write(PrintConsoleLine::new(
                "Available subcommands: particle, camera".to_string(),
            ));
        }
        2 => match path[1].as_str() {
            "particle" => {
                writer.write(PrintConsoleLine::new(
                    "error: 'reset particle' requires a subcommand".to_string(),
                ));
                writer.write(PrintConsoleLine::new(
                    "Available subcommands: wall, dynamic".to_string(),
                ));
            }
            "camera" => {
                println!("Executing: reset camera");
                writer.write(PrintConsoleLine::new(
                    "Resetting camera to default position...".to_string(),
                ));
            }
            _ => {
                writer.write(PrintConsoleLine::new(format!(
                    "error: Unknown subcommand 'reset {}'",
                    path[1]
                )));
            }
        },
        3 => {
            if path[1] == "particle" {
                match path[2].as_str() {
                    "wall" => {
                        writer.write(PrintConsoleLine::new(
                            "error: 'reset particle wall' requires a subcommand".to_string(),
                        ));
                        writer.write(PrintConsoleLine::new(
                            "Available subcommands: all".to_string(),
                        ));
                    }
                    "dynamic" => {
                        writer.write(PrintConsoleLine::new(
                            "error: 'reset particle dynamic' requires a subcommand".to_string(),
                        ));
                        writer.write(PrintConsoleLine::new(
                            "Available subcommands: all".to_string(),
                        ));
                    }
                    _ => {
                        writer.write(PrintConsoleLine::new(format!(
                            "error: Unknown subcommand 'reset particle {}'",
                            path[2]
                        )));
                    }
                }
            }
        }
        4 => {
            if path[1] == "particle" && path[3] == "all" {
                match path[2].as_str() {
                    "wall" => {
                        println!("Executing: reset particle wall all");
                        writer.write(PrintConsoleLine::new(
                            "Resetting all wall particles...".to_string(),
                        ));
                    }
                    "dynamic" => {
                        println!("Executing: reset particle dynamic all");
                        writer.write(PrintConsoleLine::new(
                            "Resetting all dynamic particles...".to_string(),
                        ));
                    }
                    _ => {
                        writer.write(PrintConsoleLine::new(format!(
                            "error: Unknown command path: {}",
                            path.join(" ")
                        )));
                    }
                }
            }
        }
        _ => {
            writer.write(PrintConsoleLine::new(format!(
                "error: Invalid command path: {}",
                path.join(" ")
            )));
        }
    }
}
