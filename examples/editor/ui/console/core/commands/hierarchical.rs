use bevy::prelude::*;
use super::super::core::{ConsoleCommandEntered, PrintConsoleLine};

pub fn hierarchical_command_handler(
    mut cmd: EventReader<ConsoleCommandEntered>,
    mut writer: EventWriter<PrintConsoleLine>,
) {
    for command_event in cmd.read() {
        if command_event.command_path.is_empty() {
            continue;
        }

        match command_event.command_path[0].as_str() {
            "reset" => handle_reset_command(&command_event.command_path, &command_event.args, &mut writer),
            "debug" => handle_debug_command(&command_event.command_path, &command_event.args, &mut writer),
            _ => continue,
        }
    }
}

fn handle_reset_command(path: &[String], _args: &[String], writer: &mut EventWriter<PrintConsoleLine>) {
    match path.len() {
        1 => {
            writer.write(PrintConsoleLine::new("error: 'reset' requires a subcommand".to_string()));
            writer.write(PrintConsoleLine::new("Available subcommands: particle, camera".to_string()));
        }
        2 => {
            match path[1].as_str() {
                "particle" => {
                    writer.write(PrintConsoleLine::new("error: 'reset particle' requires a subcommand".to_string()));
                    writer.write(PrintConsoleLine::new("Available subcommands: wall, dynamic".to_string()));
                }
                "camera" => {
                    println!("Executing: reset camera");
                    writer.write(PrintConsoleLine::new("Resetting camera to default position...".to_string()));
                }
                _ => {
                    writer.write(PrintConsoleLine::new(format!("error: Unknown subcommand 'reset {}'", path[1])));
                }
            }
        }
        3 => {
            if path[1] == "particle" {
                match path[2].as_str() {
                    "wall" => {
                        writer.write(PrintConsoleLine::new("error: 'reset particle wall' requires a subcommand".to_string()));
                        writer.write(PrintConsoleLine::new("Available subcommands: all".to_string()));
                    }
                    "dynamic" => {
                        writer.write(PrintConsoleLine::new("error: 'reset particle dynamic' requires a subcommand".to_string()));
                        writer.write(PrintConsoleLine::new("Available subcommands: all".to_string()));
                    }
                    _ => {
                        writer.write(PrintConsoleLine::new(format!("error: Unknown subcommand 'reset particle {}'", path[2])));
                    }
                }
            }
        }
        4 => {
            if path[1] == "particle" && path[3] == "all" {
                match path[2].as_str() {
                    "wall" => {
                        println!("Executing: reset particle wall all");
                        writer.write(PrintConsoleLine::new("Resetting all wall particles...".to_string()));
                    }
                    "dynamic" => {
                        println!("Executing: reset particle dynamic all");
                        writer.write(PrintConsoleLine::new("Resetting all dynamic particles...".to_string()));
                    }
                    _ => {
                        writer.write(PrintConsoleLine::new(format!("error: Unknown command path: {}", path.join(" "))));
                    }
                }
            }
        }
        _ => {
            writer.write(PrintConsoleLine::new(format!("error: Invalid command path: {}", path.join(" "))));
        }
    }
}

fn handle_debug_command(path: &[String], _args: &[String], writer: &mut EventWriter<PrintConsoleLine>) {
    match path.len() {
        1 => {
            writer.write(PrintConsoleLine::new("error: 'debug' requires a subcommand".to_string()));
            writer.write(PrintConsoleLine::new("Available subcommands: physics, particles".to_string()));
        }
        2 => {
            match path[1].as_str() {
                "physics" => {
                    writer.write(PrintConsoleLine::new("error: 'debug physics' requires a subcommand".to_string()));
                    writer.write(PrintConsoleLine::new("Available subcommands: enable, disable".to_string()));
                }
                "particles" => {
                    writer.write(PrintConsoleLine::new("error: 'debug particles' requires a subcommand".to_string()));
                    writer.write(PrintConsoleLine::new("Available subcommands: count".to_string()));
                }
                _ => {
                    writer.write(PrintConsoleLine::new(format!("error: Unknown subcommand 'debug {}'", path[1])));
                }
            }
        }
        3 => {
            match (path[1].as_str(), path[2].as_str()) {
                ("physics", "enable") => {
                    println!("Executing: debug physics enable");
                    writer.write(PrintConsoleLine::new("Enabling physics debug overlay...".to_string()));
                }
                ("physics", "disable") => {
                    println!("Executing: debug physics disable");
                    writer.write(PrintConsoleLine::new("Disabling physics debug overlay...".to_string()));
                }
                ("particles", "count") => {
                    println!("Executing: debug particles count");
                    writer.write(PrintConsoleLine::new("Current particle count: 1234 particles".to_string()));
                }
                _ => {
                    writer.write(PrintConsoleLine::new(format!("error: Unknown command path: {}", path.join(" "))));
                }
            }
        }
        _ => {
            writer.write(PrintConsoleLine::new(format!("error: Invalid command path: {}", path.join(" "))));
        }
    }
}