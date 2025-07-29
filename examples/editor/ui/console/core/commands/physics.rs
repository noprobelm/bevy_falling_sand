use bevy::prelude::*;

use super::super::core::{ConsoleCommand, PrintConsoleLine};

pub struct PhysicsCommandPlugin;

impl Plugin for PhysicsCommandPlugin {
    fn build(&self, _app: &mut App) {
        // Not yet implemented
    }
}

#[derive(Default)]
pub struct PhysicsCommand;

impl ConsoleCommand for PhysicsCommand {
    fn name(&self) -> &'static str {
        "physics"
    }

    fn description(&self) -> &'static str {
        "Physics system operations"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![Box::new(PhysicsDebugCommand)]
    }
}

#[derive(Default)]
pub struct PhysicsDebugCommand;

impl ConsoleCommand for PhysicsDebugCommand {
    fn name(&self) -> &'static str {
        "debug"
    }

    fn description(&self) -> &'static str {
        "Physics debug options"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![
            Box::new(PhysicsDebugEnableCommand),
            Box::new(PhysicsDebugDisableCommand),
        ]
    }
}

#[derive(Default)]
pub struct PhysicsDebugEnableCommand;

impl ConsoleCommand for PhysicsDebugEnableCommand {
    fn name(&self) -> &'static str {
        "enable"
    }

    fn description(&self) -> &'static str {
        "Enable physics debug overlay"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Enabling physics debug overlay...".to_string(),
        ));
    }
}

#[derive(Default)]
pub struct PhysicsDebugDisableCommand;

impl ConsoleCommand for PhysicsDebugDisableCommand {
    fn name(&self) -> &'static str {
        "disable"
    }

    fn description(&self) -> &'static str {
        "Disable physics debug overlay"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Disabling physics debug overlay...".to_string(),
        ));
    }
}
