use bevy::prelude::*;

use crate::ui::console::core::PrintConsoleLine;

pub fn handle_debug_command(
    path: &[String],
    _args: &[String],
    writer: &mut EventWriter<PrintConsoleLine>,
) {
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
