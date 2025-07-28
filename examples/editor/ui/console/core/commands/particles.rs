use bevy::prelude::*;
use bevy_falling_sand::prelude::ClearDynamicParticlesEvent;

use super::super::core::{ConsoleCommand, PrintConsoleLine};

pub struct ParticlesCommandPlugin;

impl Plugin for ParticlesCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_despawn_dynamic_particles);
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
            Box::new(ParticlesDebugCommand),
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
        println!("Executing: particles reset wall all");
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
        println!("Executing: particles reset dynamic all");
        console_writer.write(PrintConsoleLine::new(
            "Resetting all dynamic particles...".to_string(),
        ));
    }
}

#[derive(Default)]
pub struct ParticlesDebugCommand;

impl ConsoleCommand for ParticlesDebugCommand {
    fn name(&self) -> &'static str {
        "debug"
    }

    fn description(&self) -> &'static str {
        "Particle debug options"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![Box::new(ParticlesDebugCountCommand)]
    }
}

#[derive(Default)]
pub struct ParticlesDebugCountCommand;

impl ConsoleCommand for ParticlesDebugCountCommand {
    fn name(&self) -> &'static str {
        "count"
    }

    fn description(&self) -> &'static str {
        "Show particle count"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        _commands: &mut Commands,
    ) {
        println!("Executing: particles debug count");
        console_writer.write(PrintConsoleLine::new(
            "Current particle count: 1234 particles".to_string(),
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
        vec![Box::new(ParticlesDespawnDynamicCommand)]
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

fn on_despawn_dynamic_particles(
    _trigger: Trigger<ClearDynamicParticlesEvent>,
    mut evw_clear_dynamic_particles: EventWriter<ClearDynamicParticlesEvent>,
) {
    println!("Here");
    evw_clear_dynamic_particles.write(ClearDynamicParticlesEvent);
}