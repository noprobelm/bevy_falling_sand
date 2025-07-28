use super::{
    super::core::{ConsoleCommandEntered, PrintConsoleLine},
    handle_debug_command, handle_reset_command,
};
use bevy::prelude::*;

pub fn hierarchical_command_handler(
    mut cmd: EventReader<ConsoleCommandEntered>,
    mut writer: EventWriter<PrintConsoleLine>,
) {
    for command_event in cmd.read() {
        if command_event.command_path.is_empty() {
            continue;
        }

        match command_event.command_path[0].as_str() {
            "reset" => handle_reset_command(
                &command_event.command_path,
                &command_event.args,
                &mut writer,
            ),
            "debug" => handle_debug_command(
                &command_event.command_path,
                &command_event.args,
                &mut writer,
            ),
            _ => continue,
        }
    }
}

