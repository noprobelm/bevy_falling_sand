use bevy::prelude::*;

use crate::camera::{MainCamera, ZoomSpeed, ZoomTarget};

use super::super::core::{ConsoleCommand, PrintConsoleLine};

pub struct ResetCommandPlugin;

impl Plugin for ResetCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_reset_camera);
    }
}

#[derive(Event)]
pub struct ResetCameraEvent;

#[derive(Default)]
pub struct ResetCommand;

impl ConsoleCommand for ResetCommand {
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
        commands: &mut Commands,
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
                            ResetParticleCommand.execute(path, args, console_writer, commands)
                        }
                        "camera" => {
                            ResetCameraCommand.execute(path, args, console_writer, commands)
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

    fn subcommands(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![Box::new(ResetParticleCommand), Box::new(ResetCameraCommand)]
    }
}

#[derive(Default)]
pub struct ResetParticleCommand;

impl ConsoleCommand for ResetParticleCommand {
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
        commands: &mut Commands,
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
                        "wall" => {
                            ResetParticleWallCommand.execute(path, args, console_writer, commands)
                        }
                        "dynamic" => ResetParticleDynamicCommand.execute(
                            path,
                            args,
                            console_writer,
                            commands,
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

    fn subcommands(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![
            Box::new(ResetParticleWallCommand),
            Box::new(ResetParticleDynamicCommand),
        ]
    }
}

#[derive(Default)]
pub struct ResetCameraCommand;

impl ConsoleCommand for ResetCameraCommand {
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
        commands: &mut Commands,
    ) {
        println!("ResetCameraCommand::execute - triggering ResetCameraEvent");
        console_writer.write(PrintConsoleLine::new(
            "Triggering reset camera event...".to_string(),
        ));
        commands.trigger(ResetCameraEvent);
    }
}

#[derive(Default)]
pub struct ResetParticleWallCommand;

impl ConsoleCommand for ResetParticleWallCommand {
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
                    "error: 'reset particle wall' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: all".to_string(),
                ));
            }
            4 => {
                if path[3] == "all" {
                    ResetParticleWallAllCommand.execute(path, args, console_writer, commands);
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

    fn subcommands(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![Box::new(ResetParticleWallAllCommand)]
    }
}

#[derive(Default)]
pub struct ResetParticleDynamicCommand;

impl ConsoleCommand for ResetParticleDynamicCommand {
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
                    "error: 'reset particle dynamic' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: all".to_string(),
                ));
            }
            4 => {
                if path[3] == "all" {
                    ResetParticleDynamicAllCommand.execute(path, args, console_writer, commands);
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

    fn subcommands(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![Box::new(ResetParticleDynamicAllCommand)]
    }
}

#[derive(Default)]
pub struct ResetParticleWallAllCommand;

impl ConsoleCommand for ResetParticleWallAllCommand {
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
        println!("Executing: reset particle wall all");
        console_writer.write(PrintConsoleLine::new(
            "Resetting all wall particles...".to_string(),
        ));
    }
}

#[derive(Default)]
pub struct ResetParticleDynamicAllCommand;

impl ConsoleCommand for ResetParticleDynamicAllCommand {
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
        println!("Executing: reset particle dynamic all");
        console_writer.write(PrintConsoleLine::new(
            "Resetting all dynamic particles...".to_string(),
        ));
    }
}

fn on_reset_camera(
    _trigger: Trigger<ResetCameraEvent>,
    camera_query: Query<Entity, With<MainCamera>>,
    mut commands: Commands,
) -> Result {
    println!("on_reset_camera observer called!");
    let initial_scale = 0.11;
    let entity = camera_query.single()?;
    println!("Found camera entity: {:?}", entity);
    commands.entity(entity).insert((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            near: -1000.0,
            scale: initial_scale,
            ..OrthographicProjection::default_2d()
        }),
        MainCamera,
        ZoomTarget {
            target_scale: initial_scale,
            current_scale: initial_scale,
        },
        ZoomSpeed(8.0),
        Transform::default(),
    ));
    println!("Camera reset completed successfully");
    Ok(())
}
