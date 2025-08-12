use bevy::prelude::*;

use crate::physics::DespawnRigidBodies;

use super::super::core::{ConsoleCommand, PrintConsoleLine};

pub struct AvianCommandPlugin;

impl Plugin for AvianCommandPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Default)]
pub struct AvianCommand;

impl ConsoleCommand for AvianCommand {
    fn name(&self) -> &'static str {
        "avian"
    }

    fn description(&self) -> &'static str {
        "TODO: Add description for avian command"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![Box::new(AvianDespawn)]
    }
}

#[derive(Default)]
pub struct AvianDespawn;

impl ConsoleCommand for AvianDespawn {
    fn name(&self) -> &'static str {
        "despawn"
    }

    fn description(&self) -> &'static str {
        "Despawn rigid bodies"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![
            Box::new(AvianDespawnDynamic),
            Box::new(AvianDespawnStatic),
            Box::new(AvianDespawnAll),
        ]
    }
}

#[derive(Default)]
pub struct AvianDespawnDynamic;

impl ConsoleCommand for AvianDespawnDynamic {
    fn name(&self) -> &'static str {
        "dynamic"
    }

    fn description(&self) -> &'static str {
        "Despawn all dynamic rigid bodies"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Executing avian despawn dynamic...".to_string(),
        ));
        commands.trigger(DespawnRigidBodies::Dynamic)
    }
}

#[derive(Default)]
pub struct AvianDespawnStatic;

impl ConsoleCommand for AvianDespawnStatic {
    fn name(&self) -> &'static str {
        "static"
    }

    fn description(&self) -> &'static str {
        "Despawn all static rigid bodies"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Executing avian despawn static...".to_string(),
        ));
        commands.trigger(DespawnRigidBodies::Static)
    }
}

#[derive(Default)]
pub struct AvianDespawnAll;

impl ConsoleCommand for AvianDespawnAll {
    fn name(&self) -> &'static str {
        "all"
    }

    fn description(&self) -> &'static str {
        "Despawn all rigid bodies"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Executing avian despawn all...".to_string(),
        ));
        commands.trigger(DespawnRigidBodies::All)
    }
}
