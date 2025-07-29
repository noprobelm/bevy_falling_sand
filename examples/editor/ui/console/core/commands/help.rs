use bevy::prelude::*;

use super::super::core::{
    ConsoleCommand, ConsoleCommandEntered, ConsoleConfiguration, PrintConsoleLine,
};

pub struct HelpCommandPlugin;

impl Plugin for HelpCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, help_command_system);
    }
}

#[derive(Default)]
pub struct HelpCommand;

impl ConsoleCommand for HelpCommand {
    fn name(&self) -> &'static str {
        "help"
    }

    fn description(&self) -> &'static str {
        "Display help information for commands"
    }

    fn execute_action(
        &self,
        _args: &[String],
        _console_writer: &mut EventWriter<PrintConsoleLine>,
        _commands: &mut Commands,
    ) {
        // The actual help logic is handled by the help_command_system
        // This is empty because we need access to ConsoleConfiguration
    }
}

fn help_command_system(
    mut cmd: EventReader<ConsoleCommandEntered>,
    config: Res<ConsoleConfiguration>,
    mut writer: EventWriter<PrintConsoleLine>,
) {
    for command_event in cmd.read() {
        if command_event.command_path.len() >= 1 && command_event.command_path[0] == "help" {
            if let Some(target_cmd) = command_event.args.first() {
                if let Some(root_node) = config.command_tree.get(target_cmd) {
                    show_command_tree_help(&root_node, vec![target_cmd.clone()], &mut writer);
                } else {
                    writer.write(PrintConsoleLine::new(format!(
                        "Command '{}' does not exist",
                        target_cmd
                    )));
                }
            } else {
                writer.write(PrintConsoleLine::new("Available commands:".to_string()));

                for (name, node) in &config.command_tree {
                    writer.write(PrintConsoleLine::new(format!(
                        "  {} - {}",
                        name, node.description
                    )));
                    if !node.children.is_empty() {
                        writer.write(PrintConsoleLine::new(format!(
                            "    (has subcommands: {})",
                            node.children.keys().cloned().collect::<Vec<_>>().join(", ")
                        )));
                    }
                }
            }
        }
    }
}

fn show_command_tree_help(
    node: &super::super::core::CommandNode,
    path: Vec<String>,
    writer: &mut EventWriter<PrintConsoleLine>,
) {
    writer.write(PrintConsoleLine::new(format!(
        "{} - {}",
        path.join(" "),
        node.description
    )));

    if node.is_executable {
        writer.write(PrintConsoleLine::new("  (executable command)".to_string()));
    }

    if !node.children.is_empty() {
        writer.write(PrintConsoleLine::new("  Subcommands:".to_string()));
        for (name, child) in &node.children {
            let child_path = format!("    {} - {}", name, child.description);
            writer.write(PrintConsoleLine::new(child_path));
            if child.is_executable {
                writer.write(PrintConsoleLine::new("      (executable)".to_string()));
            }
        }
    }
}
