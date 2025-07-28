use bevy::prelude::*;

use crate::camera::{MainCamera, ZoomSpeed, ZoomTarget};

use super::super::core::{ConsoleCommand, PrintConsoleLine};

pub struct CameraCommandPlugin;

impl Plugin for CameraCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_reset_camera);
    }
}

#[derive(Event)]
pub struct ResetCameraEvent;

#[derive(Default)]
pub struct CameraCommand;

impl ConsoleCommand for CameraCommand {
    fn name(&self) -> &'static str {
        "camera"
    }

    fn description(&self) -> &'static str {
        "Camera system operations"
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
                    "error: 'camera' requires a subcommand".to_string(),
                ));
                console_writer.write(PrintConsoleLine::new(
                    "Available subcommands: reset".to_string(),
                ));
            }
            _ => {
                if path.len() >= 2 {
                    match path[1].as_str() {
                        "reset" => {
                            CameraResetCommand.execute(path, args, console_writer, commands)
                        }
                        _ => {
                            console_writer.write(PrintConsoleLine::new(format!(
                                "error: Unknown subcommand 'camera {}'",
                                path[1]
                            )));
                        }
                    }
                }
            }
        }
    }

    fn subcommands(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![Box::new(CameraResetCommand)]
    }
}

#[derive(Default)]
pub struct CameraResetCommand;

impl ConsoleCommand for CameraResetCommand {
    fn name(&self) -> &'static str {
        "reset"
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
        println!("CameraResetCommand::execute - triggering ResetCameraEvent");
        console_writer.write(PrintConsoleLine::new(
            "Triggering reset camera event...".to_string(),
        ));
        commands.trigger(ResetCameraEvent);
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