use bevy::prelude::*;

use crate::ui::console::core::CommandNode;

use super::super::core::{ConsoleCommand, ConsoleConfiguration, PrintConsoleLine};

pub struct HelpCommandPlugin;

impl Plugin for HelpCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_show_help);
    }
}

#[derive(Event)]
pub struct ShowHelpEvent {
    pub target_command: Option<String>,
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
        args: &[String],
        _console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        let target_command = args.first().cloned();
        commands.trigger(ShowHelpEvent { target_command });
    }
}

fn on_show_help(
    trigger: Trigger<ShowHelpEvent>,
    config: Res<ConsoleConfiguration>,
    mut writer: EventWriter<PrintConsoleLine>,
) {
    let event = trigger.event();

    if let Some(target_cmd) = &event.target_command {
        if let Some(root_node) = config.command_tree.get(target_cmd) {
            show_command_tree_help(root_node, vec![target_cmd.clone()], &mut writer);
        } else {
            writer.write(PrintConsoleLine::new(format!(
                "Command '{target_cmd}' does not exist"
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

fn show_command_tree_help(
    node: &CommandNode,
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

