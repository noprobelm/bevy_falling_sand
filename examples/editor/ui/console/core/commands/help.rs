use bevy::prelude::*;
use clap::Parser;

use super::super::core::{
    ConsoleCommandEntered, ConsoleConfiguration, NamedCommand, PrintConsoleLine,
};

#[derive(Parser, Resource)]
#[command(name = "help", about = "Display help information for commands")]
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
        if command_event.command_path.len() == 1 && command_event.command_path[0] == "help" {
            println!("Executing: help {}", command_event.args.join(" "));
            if let Some(target_cmd) = command_event.args.first() {
                
                if let Some(cmd_info) = config.commands.get(target_cmd.as_str()) {
                    let mut cloned_cmd = cmd_info.clone();
                    writer.write(PrintConsoleLine::new(
                        cloned_cmd.render_long_help().to_string(),
                    ));
                } else if let Some(root_node) = config.command_tree.get(target_cmd) {
                    
                    show_command_tree_help(&root_node, vec![target_cmd.clone()], &mut writer);
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
                
                
                for (name, node) in &config.command_tree {
                    writer.write(PrintConsoleLine::new(format!("  {} - {}", name, node.description)));
                    if !node.children.is_empty() {
                        writer.write(PrintConsoleLine::new(format!("    (has subcommands: {})", 
                            node.children.keys().cloned().collect::<Vec<_>>().join(", "))));
                    }
                }
            }
        }
    }
}

fn show_command_tree_help(node: &super::super::core::CommandNode, path: Vec<String>, writer: &mut EventWriter<PrintConsoleLine>) {
    writer.write(PrintConsoleLine::new(format!("{} - {}", path.join(" "), node.description)));
    
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

