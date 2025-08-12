use bevy::prelude::*;
use bfs_internal::prelude::RigidBody;

use super::super::core::{ConsoleCommand, PrintConsoleLine};

pub struct AvianCommandPlugin;

impl Plugin for AvianCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_despawn_rigid_bodies);
    }
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

#[derive(Event)]
pub enum DespawnRigidBodies {
    Dynamic,
    Static,
    All,
}

fn on_despawn_rigid_bodies(
    trigger: Trigger<DespawnRigidBodies>,
    mut commands: Commands,
    rigid_body_query: Query<(Entity, &RigidBody)>,
) {
    match trigger.event() {
        DespawnRigidBodies::Dynamic => rigid_body_query.iter().for_each(|(entity, rigid_body)| {
            if rigid_body == &RigidBody::Dynamic {
                commands.entity(entity).despawn();
            }
        }),
        DespawnRigidBodies::Static => rigid_body_query.iter().for_each(|(entity, rigid_body)| {
            if rigid_body == &RigidBody::Static {
                commands.entity(entity).despawn();
            }
        }),
        DespawnRigidBodies::All => rigid_body_query.iter().for_each(|(entity, _)| {
            commands.entity(entity).despawn();
        }),
    }
}
