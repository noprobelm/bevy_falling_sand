use bevy::prelude::*;

use crate::ui::console::core::PrintConsoleLine;

pub fn handle_reset_command(
    path: &[String],
    _args: &[String],
    writer: &mut EventWriter<PrintConsoleLine>,
) {
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
