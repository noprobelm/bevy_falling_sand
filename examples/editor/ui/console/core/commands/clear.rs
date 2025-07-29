use bevy::prelude::*;

use super::super::core::{ConsoleCommand, ConsoleState, PrintConsoleLine};

pub struct ClearCommandPlugin;

impl Plugin for ClearCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_clear_console);
    }
}

#[derive(Event)]
pub struct ClearConsoleEvent;

#[derive(Default)]
pub struct ClearCommand;

impl ConsoleCommand for ClearCommand {
    fn name(&self) -> &'static str {
        "clear"
    }

    fn description(&self) -> &'static str {
        "Clear the console output"
    }

    fn execute_action(
        &self,
        _args: &[String],
        _console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        commands.trigger(ClearConsoleEvent);
    }
}

fn on_clear_console(_trigger: Trigger<ClearConsoleEvent>, mut console_state: ResMut<ConsoleState>) {
    console_state.messages.clear();
    console_state.add_message("--- Bevy Falling Sand Editor Console ---".to_string());
}
