use bevy::prelude::*;
use clap::Parser;

use crate::console::core::{
    ConsoleCommandEntered, ConsoleConfiguration, NamedCommand, PrintConsoleLine,
};

#[derive(Parser, Resource)]
#[command(name = "help")]
pub struct HelpCommand {
    command: Option<String>,
}

impl NamedCommand for HelpCommand {
    fn name() -> &'static str {
        "help"
    }
}

pub fn help_command(
    mut cmd: EventReader<ConsoleCommandEntered>,
    config: Res<ConsoleConfiguration>,
    mut writer: EventWriter<PrintConsoleLine>,
) {
    for command_event in cmd.read() {
        if command_event.command_name == "help" {
            if let Some(target_cmd) = command_event.args.first() {
                if let Some(cmd_info) = config.commands.get(target_cmd.as_str()) {
                    let mut cloned_cmd = cmd_info.clone();
                    writer.write(PrintConsoleLine::new(
                        cloned_cmd.render_long_help().to_string(),
                    ));
                } else {
                    writer.write(PrintConsoleLine::new(format!(
                        "Command '{}' does not exist",
                        target_cmd
                    )));
                }
            } else {
                writer.write(PrintConsoleLine::new("Available commands:".to_string()));
                for (name, cmd) in &config.commands {
                    let help_text = cmd.get_about().map(|s| s.to_string()).unwrap_or_default();
                    writer.write(PrintConsoleLine::new(format!("  {} - {}", name, help_text)));
                }
            }
        }
    }
}

