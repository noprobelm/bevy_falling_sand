use bevy::prelude::*;
use bevy_falling_sand::prelude::ClearDynamicParticlesEvent;

use super::super::core::{ConsoleCommand, PrintConsoleLine};

pub struct ParticlesCommandPlugin;

impl Plugin for ParticlesCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_despawn_dynamic_particles);
    }
}

#[derive(Default)]
pub struct ParticlesCommand;

impl ConsoleCommand for ParticlesCommand {
    fn name(&self) -> &'static str {
        "particles"
    }

    fn description(&self) -> &'static str {
        "Particle system operations"
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
                    "error: 'particles' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: reset, debug, despawn".to_string(),
                ));
            }
            _ => {
                if path.len() >= 2 {
                    match path[1].as_str() {
                        "reset" => {
                            ParticlesResetCommand.execute(path, args, console_writer, commands)
                        }
                        "debug" => {
                            ParticlesDebugCommand.execute(path, args, console_writer, commands)
                        }
                        "despawn" => {
                            ParticlesDespawnCommand.execute(path, args, console_writer, commands)
                        }
                        _ => {
                            console_writer.write(PrintConsoleLine::new(format!(
                                "error: Unknown subcommand 'particles {}'",
                                path[1]
                            )));
                        }
                    }
                }
            }
        }
    }

    fn subcommands(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![
            Box::new(ParticlesResetCommand),
            Box::new(ParticlesDebugCommand),
            Box::new(ParticlesDespawnCommand),
        ]
    }
}

#[derive(Default)]
pub struct ParticlesResetCommand;

impl ConsoleCommand for ParticlesResetCommand {
    fn name(&self) -> &'static str {
        "reset"
    }

    fn description(&self) -> &'static str {
        "Reset particle-related components"
    }

    fn execute(
        &self,
        path: &[String],
        args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        match path.len() {
            2 => {
                console_writer.write(PrintConsoleLine::new(
                    "error: 'particles reset' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: wall, dynamic".to_string(),
                ));
            }
            _ => {
                if path.len() >= 3 {
                    match path[2].as_str() {
                        "wall" => {
                            ParticlesResetWallCommand.execute(path, args, console_writer, commands)
                        }
                        "dynamic" => ParticlesResetDynamicCommand.execute(
                            path,
                            args,
                            console_writer,
                            commands,
                        ),
                        _ => {
                            console_writer.write(PrintConsoleLine::new(format!(
                                "error: Unknown subcommand 'particles reset {}'",
                                path[2]
                            )));
                        }
                    }
                }
            }
        }
    }

    fn subcommands(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![
            Box::new(ParticlesResetWallCommand),
            Box::new(ParticlesResetDynamicCommand),
        ]
    }
}

#[derive(Default)]
pub struct ParticlesResetWallCommand;

impl ConsoleCommand for ParticlesResetWallCommand {
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
        commands: &mut Commands,
    ) {
        match path.len() {
            3 => {
                console_writer.write(PrintConsoleLine::new(
                    "error: 'particles reset wall' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: all".to_string(),
                ));
            }
            4 => {
                if path[3] == "all" {
                    ParticlesResetWallAllCommand.execute(path, args, console_writer, commands);
                } else {
                    console_writer.write(PrintConsoleLine::new(format!(
                        "error: Unknown command 'particles reset wall {}'",
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

    fn subcommands(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![Box::new(ParticlesResetWallAllCommand)]
    }
}

#[derive(Default)]
pub struct ParticlesResetDynamicCommand;

impl ConsoleCommand for ParticlesResetDynamicCommand {
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
        commands: &mut Commands,
    ) {
        match path.len() {
            3 => {
                console_writer.write(PrintConsoleLine::new(
                    "error: 'particles reset dynamic' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: all".to_string(),
                ));
            }
            4 => {
                if path[3] == "all" {
                    ParticlesResetDynamicAllCommand.execute(path, args, console_writer, commands);
                } else {
                    console_writer.write(PrintConsoleLine::new(format!(
                        "error: Unknown command 'particles reset dynamic {}'",
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

    fn subcommands(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![Box::new(ParticlesResetDynamicAllCommand)]
    }
}

#[derive(Default)]
pub struct ParticlesResetWallAllCommand;

impl ConsoleCommand for ParticlesResetWallAllCommand {
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
        _commands: &mut Commands,
    ) {
        println!("Executing: particles reset wall all");
        console_writer.write(PrintConsoleLine::new(
            "Resetting all wall particles...".to_string(),
        ));
    }
}

#[derive(Default)]
pub struct ParticlesResetDynamicAllCommand;

impl ConsoleCommand for ParticlesResetDynamicAllCommand {
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
        _commands: &mut Commands,
    ) {
        println!("Executing: particles reset dynamic all");
        console_writer.write(PrintConsoleLine::new(
            "Resetting all dynamic particles...".to_string(),
        ));
    }
}

#[derive(Default)]
pub struct ParticlesDebugCommand;

impl ConsoleCommand for ParticlesDebugCommand {
    fn name(&self) -> &'static str {
        "debug"
    }

    fn description(&self) -> &'static str {
        "Particle debug options"
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
                    "error: 'particles debug' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: count".to_string(),
                ));
            }
            _ => {
                console_writer.write(PrintConsoleLine::new(format!(
                    "error: Invalid command path: {}",
                    path.join(" ")
                )));
            }
        }
    }
}

#[derive(Default)]
pub struct ParticlesDespawnCommand;

impl ConsoleCommand for ParticlesDespawnCommand {
    fn name(&self) -> &'static str {
        "despawn"
    }

    fn description(&self) -> &'static str {
        "Despawn particles from the world"
    }

    fn execute(
        &self,
        path: &[String],
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        match path.len() {
            2 => {
                console_writer.write(PrintConsoleLine::new(
                    "error: 'particles despawn' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: dynamic".to_string(),
                ));
            }
            3 => match path[2].as_str() {
                "dynamic" => {
                    console_writer.write(PrintConsoleLine::new(
                        "Despawning all dynamic particles from the world".to_string(),
                    ));
                    ParticlesDespawnDynamicCommand.execute(path, _args, console_writer, commands);
                }
                _ => {
                    console_writer.write(PrintConsoleLine::new(format!(
                        "error: Unknown command 'particles despawn {}'",
                        path[2]
                    )));
                }
            },
            _ => {
                console_writer.write(PrintConsoleLine::new(format!(
                    "error: Unknown command 'physics debug {}'",
                    path[2]
                )));
            }
        }
        console_writer.write(PrintConsoleLine::new(
            "Despawning particles - not yet implemented".to_string(),
        ));
    }

    fn subcommands(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![Box::new(ParticlesDespawnDynamicCommand)]
    }
}

#[derive(Default)]
pub struct ParticlesDespawnDynamicCommand;

impl ConsoleCommand for ParticlesDespawnDynamicCommand {
    fn name(&self) -> &'static str {
        "dynamic"
    }

    fn description(&self) -> &'static str {
        "Despawn dynamic particles from the world"
    }

    fn execute(
        &self,
        _path: &[String],
        _args: &[String],
        _console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        commands.trigger(ClearDynamicParticlesEvent);
    }
}

fn on_despawn_dynamic_particles(
    _trigger: Trigger<ClearDynamicParticlesEvent>,
    mut evw_clear_dynamic_particles: EventWriter<ClearDynamicParticlesEvent>,
) {
    println!("Here");
    evw_clear_dynamic_particles.write(ClearDynamicParticlesEvent);
}
