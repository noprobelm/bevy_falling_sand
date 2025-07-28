use bevy::prelude::*;
use bevy_falling_sand::prelude::{
    ClearDynamicParticlesEvent, ClearParticleMapEvent, ClearParticleTypeChildrenEvent,
    ClearStaticParticlesEvent,
};

use super::super::core::{ConsoleCommand, PrintConsoleLine};

pub struct ParticlesCommandPlugin;

impl Plugin for ParticlesCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_despawn_dynamic_particles)
            .add_observer(on_despawn_static_particles)
            .add_observer(on_despawn_named_particles)
            .add_observer(on_despawn_all_particles);
    }
}

#[derive(Default)]
pub struct ParticlesCommand;

impl ConsoleCommand for ParticlesCommand {
    fn name(&self) -> &'static str {
        "particles"
    }

    fn description(&self) -> &'static str {
        "Particle system operations"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![
            Box::new(ParticlesResetCommand),
            Box::new(ParticlesDespawnCommand),
        ]
    }
}

#[derive(Default)]
pub struct ParticlesResetCommand;

impl ConsoleCommand for ParticlesResetCommand {
    fn name(&self) -> &'static str {
        "reset"
    }

    fn description(&self) -> &'static str {
        "Reset particle-related components"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![
            Box::new(ParticlesResetWallCommand),
            Box::new(ParticlesResetDynamicCommand),
        ]
    }
}

#[derive(Default)]
pub struct ParticlesResetWallCommand;

impl ConsoleCommand for ParticlesResetWallCommand {
    fn name(&self) -> &'static str {
        "wall"
    }

    fn description(&self) -> &'static str {
        "Reset wall particles"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![Box::new(ParticlesResetWallAllCommand)]
    }
}

#[derive(Default)]
pub struct ParticlesResetDynamicCommand;

impl ConsoleCommand for ParticlesResetDynamicCommand {
    fn name(&self) -> &'static str {
        "dynamic"
    }

    fn description(&self) -> &'static str {
        "Reset dynamic particles"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![Box::new(ParticlesResetDynamicAllCommand)]
    }
}

#[derive(Default)]
pub struct ParticlesResetWallAllCommand;

impl ConsoleCommand for ParticlesResetWallAllCommand {
    fn name(&self) -> &'static str {
        "all"
    }

    fn description(&self) -> &'static str {
        "Reset all wall particles"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Resetting all wall particles...".to_string(),
        ));
    }
}

#[derive(Default)]
pub struct ParticlesResetDynamicAllCommand;

impl ConsoleCommand for ParticlesResetDynamicAllCommand {
    fn name(&self) -> &'static str {
        "all"
    }

    fn description(&self) -> &'static str {
        "Reset all dynamic particles"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Resetting all dynamic particles...".to_string(),
        ));
    }
}

#[derive(Default)]
pub struct ParticlesDespawnCommand;

impl ConsoleCommand for ParticlesDespawnCommand {
    fn name(&self) -> &'static str {
        "despawn"
    }

    fn description(&self) -> &'static str {
        "Despawn particles from the world"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![
            Box::new(ParticlesDespawnDynamicCommand),
            Box::new(ParticlesDespawnStaticCommand),
            Box::new(ParticlesDespawnAllCommand),
            Box::new(ParticlesDespawnNamedCommand),
        ]
    }
}

#[derive(Default)]
pub struct ParticlesDespawnDynamicCommand;

impl ConsoleCommand for ParticlesDespawnDynamicCommand {
    fn name(&self) -> &'static str {
        "dynamic"
    }

    fn description(&self) -> &'static str {
        "Despawn dynamic particles from the world"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Despawning all dynamic particles from the world".to_string(),
        ));
        commands.trigger(ClearDynamicParticlesEvent);
    }
}

#[derive(Default)]
pub struct ParticlesDespawnStaticCommand;

impl ConsoleCommand for ParticlesDespawnStaticCommand {
    fn name(&self) -> &'static str {
        "static"
    }

    fn description(&self) -> &'static str {
        "Despawn static particles from the world"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Despawning all static particles from the world".to_string(),
        ));
        commands.trigger(ClearStaticParticlesEvent);
    }
}

#[derive(Default)]
pub struct ParticlesDespawnAllCommand;

impl ConsoleCommand for ParticlesDespawnAllCommand {
    fn name(&self) -> &'static str {
        "all"
    }

    fn description(&self) -> &'static str {
        "Despawn all particles from the world"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Despawning all particles from the world".to_string(),
        ));
        commands.trigger(ClearParticleMapEvent);
    }
}

#[derive(Default)]
pub struct ParticlesDespawnNamedCommand;

impl ConsoleCommand for ParticlesDespawnNamedCommand {
    fn name(&self) -> &'static str {
        "named"
    }

    fn description(&self) -> &'static str {
        "Despawn all particles of specified name from the world"
    }

    fn execute_action(
        &self,
        args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        let name = args.join(" ");
        console_writer.write(PrintConsoleLine::new(format!(
            "Despawning all '{}' particles from the world",
            { &name }
        )));
        commands.trigger(ClearParticleTypeChildrenEvent(name));
    }
}

fn on_despawn_dynamic_particles(
    _trigger: Trigger<ClearDynamicParticlesEvent>,
    mut evw_clear_dynamic_particles: EventWriter<ClearDynamicParticlesEvent>,
) {
    evw_clear_dynamic_particles.write(ClearDynamicParticlesEvent);
}

fn on_despawn_static_particles(
    _trigger: Trigger<ClearStaticParticlesEvent>,
    mut evw_clear_static_particles: EventWriter<ClearStaticParticlesEvent>,
) {
    evw_clear_static_particles.write(ClearStaticParticlesEvent);
}

fn on_despawn_all_particles(
    _trigger: Trigger<ClearParticleMapEvent>,
    mut evw_clear_particle_map: EventWriter<ClearParticleMapEvent>,
) {
    evw_clear_particle_map.write(ClearParticleMapEvent);
}

fn on_despawn_named_particles(
    trigger: Trigger<ClearParticleTypeChildrenEvent>,
    mut evw_clear_particle_type_children: EventWriter<ClearParticleTypeChildrenEvent>,
) {
    evw_clear_particle_type_children.write(trigger.event().clone());
}
