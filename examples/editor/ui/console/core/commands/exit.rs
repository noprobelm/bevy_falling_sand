use bevy::prelude::*;

use super::super::core::{ConsoleCommand, PrintConsoleLine};

pub struct ExitCommandPlugin;

impl Plugin for ExitCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_exit_application);
    }
}

#[derive(Event)]
pub struct ExitApplicationEvent;

#[derive(Default)]
pub struct ExitCommand;

impl ConsoleCommand for ExitCommand {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn description(&self) -> &'static str {
        "Exit the application"
    }

    fn execute_action(
        &self,
        _args: &[String],
        _console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        commands.trigger(ExitApplicationEvent);
    }
}

fn on_exit_application(
    _trigger: Trigger<ExitApplicationEvent>,
    mut app_exit: EventWriter<AppExit>,
) {
    app_exit.write(AppExit::Success);
}
