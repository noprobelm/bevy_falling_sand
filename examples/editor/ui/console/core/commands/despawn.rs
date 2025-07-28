use bevy::prelude::*;

use super::super::core::{Command, PrintConsoleLine};

pub struct DespawnCommandPlugin;

impl Plugin for DespawnCommandPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Default)]
pub struct DespawnCommand;

impl Command for DespawnCommand {
    fn name(&self) -> &'static str {
        "despawn"
    }

    fn description(&self) -> &'static str {
        "Despawn entities from the world"
    }

    fn execute(
        &self,
        _path: &[String],
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Despawn command - not yet implemented".to_string(),
        ));
    }
}