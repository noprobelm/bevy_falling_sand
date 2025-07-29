use bevy::prelude::*;

use crate::brush::{Brush, BrushModeState, BrushSize, BrushTypeState, MaxBrushSize};
use crate::particles::SelectedParticle;
use crate::ui::particle_search::ParticleSearchCache;
use bevy_falling_sand::prelude::Particle;

use super::super::core::{ConsoleCommand, PrintConsoleLine};

pub struct BrushCommandPlugin;

impl Plugin for BrushCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_update_brush_type)
            .add_observer(on_update_brush_mode)
            .add_observer(on_update_brush_size)
            .add_observer(on_show_brush_info)
            .add_observer(on_set_brush_particle);
    }
}

#[derive(Default)]
pub struct BrushCommand;

impl ConsoleCommand for BrushCommand {
    fn name(&self) -> &'static str {
        "brush"
    }

    fn description(&self) -> &'static str {
        "Change brush characteristics"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![Box::new(BrushSetCommand), Box::new(BrushInfoCommand)]
    }
}

#[derive(Default)]
pub struct BrushSetCommand;

impl ConsoleCommand for BrushSetCommand {
    fn name(&self) -> &'static str {
        "set"
    }

    fn description(&self) -> &'static str {
        "Set new brush characteristics"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![
            Box::new(BrushSetTypeCommand),
            Box::new(BrushSetModeCommand),
            Box::new(BrushSetSizeCommand),
            Box::new(BrushSetParticleCommand),
        ]
    }
}

#[derive(Default)]
pub struct BrushSetTypeCommand;

impl ConsoleCommand for BrushSetTypeCommand {
    fn name(&self) -> &'static str {
        "type"
    }

    fn description(&self) -> &'static str {
        "Set the brush type"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![
            Box::new(BrushSetTypeLineCommand),
            Box::new(BrushSetTypeCircleCommand),
            Box::new(BrushSetTypeCursorCommand),
        ]
    }
}

#[derive(Default)]
pub struct BrushSetTypeLineCommand;

impl ConsoleCommand for BrushSetTypeLineCommand {
    fn name(&self) -> &'static str {
        "line"
    }

    fn description(&self) -> &'static str {
        "Set the brush type to 'line'"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Setting brush type to 'line'".to_string(),
        ));
        commands.trigger(SetBrushTypeEvent(BrushTypeState::Line));
    }
}

#[derive(Default)]
pub struct BrushSetTypeCircleCommand;

impl ConsoleCommand for BrushSetTypeCircleCommand {
    fn name(&self) -> &'static str {
        "circle"
    }

    fn description(&self) -> &'static str {
        "Set the brush type to 'circle'"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Setting brush type to 'circle'".to_string(),
        ));
        commands.trigger(SetBrushTypeEvent(BrushTypeState::Circle));
    }
}

#[derive(Default)]
pub struct BrushSetTypeCursorCommand;

impl ConsoleCommand for BrushSetTypeCursorCommand {
    fn name(&self) -> &'static str {
        "cursor"
    }

    fn description(&self) -> &'static str {
        "Set the brush type to 'cursor'"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Setting brush type to 'cursor'".to_string(),
        ));
        commands.trigger(SetBrushTypeEvent(BrushTypeState::Cursor));
    }
}

#[derive(Default)]
pub struct BrushSetModeCommand;

impl ConsoleCommand for BrushSetModeCommand {
    fn name(&self) -> &'static str {
        "mode"
    }

    fn description(&self) -> &'static str {
        "Set the brush mode"
    }

    fn subcommand_types(&self) -> Vec<Box<dyn ConsoleCommand>> {
        vec![
            Box::new(BrushSetModeSpawnCommand),
            Box::new(BrushSetModeDespawnCommand),
        ]
    }
}

#[derive(Default)]
pub struct BrushSetModeSpawnCommand;

impl ConsoleCommand for BrushSetModeSpawnCommand {
    fn name(&self) -> &'static str {
        "spawn"
    }

    fn description(&self) -> &'static str {
        "Set the brush mode to 'spawn'"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Setting brush mode to 'spawn'".to_string(),
        ));
        commands.trigger(BrushSetModeEvent(BrushModeState::Spawn));
    }
}

#[derive(Default)]
pub struct BrushSetModeDespawnCommand;

impl ConsoleCommand for BrushSetModeDespawnCommand {
    fn name(&self) -> &'static str {
        "despawn"
    }

    fn description(&self) -> &'static str {
        "Set the brush mode to 'despawn'"
    }

    fn execute_action(
        &self,
        _args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        console_writer.write(PrintConsoleLine::new(
            "Setting brush mode to 'despawn'".to_string(),
        ));
        commands.trigger(BrushSetModeEvent(BrushModeState::Despawn));
    }
}

#[derive(Default)]
pub struct BrushSetSizeCommand;

impl ConsoleCommand for BrushSetSizeCommand {
    fn name(&self) -> &'static str {
        "size"
    }

    fn description(&self) -> &'static str {
        "Set the brush size (usage: brush set size <value>)"
    }

    fn execute_action(
        &self,
        args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        if args.is_empty() {
            console_writer.write(PrintConsoleLine::new(
                "Error: size value required (usage: brush set size <value>)".to_string(),
            ));
            return;
        }

        match args[0].parse::<usize>() {
            Ok(size) => {
                if size == 0 {
                    console_writer.write(PrintConsoleLine::new(
                        "Error: brush size must be greater than 0".to_string(),
                    ));
                } else {
                    console_writer.write(PrintConsoleLine::new(format!(
                        "Setting brush size to {}",
                        size
                    )));
                    commands.trigger(BrushSetSizeEvent(size));
                }
            }
            Err(_) => {
                console_writer.write(PrintConsoleLine::new(format!(
                    "Error: '{}' is not a valid size value",
                    args[0]
                )));
            }
        }
    }
}

#[derive(Default)]
pub struct BrushSetParticleCommand;

impl ConsoleCommand for BrushSetParticleCommand {
    fn name(&self) -> &'static str {
        "particle"
    }

    fn description(&self) -> &'static str {
        "Set the brush particle type (usage: brush set particle <particle_name>)"
    }

    fn execute_action(
        &self,
        args: &[String],
        console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        if args.is_empty() {
            console_writer.write(PrintConsoleLine::new(
                "Error: particle name required (usage: brush set particle <particle_name>)"
                    .to_string(),
            ));
            return;
        }

        let particle_name = args.join(" "); // Handle multi-word particle names
        commands.trigger(BrushSetParticleEvent(particle_name));
    }
}

#[derive(Default)]
pub struct BrushInfoCommand;

impl ConsoleCommand for BrushInfoCommand {
    fn name(&self) -> &'static str {
        "info"
    }

    fn description(&self) -> &'static str {
        "Display current brush information"
    }

    fn execute_action(
        &self,
        _args: &[String],
        _console_writer: &mut EventWriter<PrintConsoleLine>,
        commands: &mut Commands,
    ) {
        commands.trigger(ShowBrushInfoEvent);
    }
}

#[derive(Clone, Event, Hash, Debug, Eq, PartialEq, PartialOrd)]
struct SetBrushTypeEvent(pub BrushTypeState);

#[derive(Clone, Event, Hash, Debug, Eq, PartialEq, PartialOrd)]
struct BrushSetModeEvent(pub BrushModeState);

#[derive(Clone, Event, Hash, Debug, Eq, PartialEq, PartialOrd)]
struct BrushSetSizeEvent(pub usize);

#[derive(Clone, Event, Hash, Debug, Eq, PartialEq, PartialOrd)]
struct ShowBrushInfoEvent;

#[derive(Clone, Event, Hash, Debug, Eq, PartialEq, PartialOrd)]
struct BrushSetParticleEvent(pub String);

fn on_update_brush_type(
    trigger: Trigger<SetBrushTypeEvent>,
    mut brush_type_state_next: ResMut<NextState<BrushTypeState>>,
) {
    brush_type_state_next.set(trigger.event().0);
}

fn on_update_brush_mode(
    trigger: Trigger<BrushSetModeEvent>,
    mut brush_mode_state_next: ResMut<NextState<BrushModeState>>,
) {
    brush_mode_state_next.set(trigger.event().0);
}

fn on_update_brush_size(
    trigger: Trigger<BrushSetSizeEvent>,
    mut brush_size_query: Query<&mut BrushSize>,
) -> Result {
    let mut brush_size = brush_size_query.single_mut()?;
    let size = trigger.event().0;
    brush_size.0 = size;

    Ok(())
}

fn on_show_brush_info(
    _trigger: Trigger<ShowBrushInfoEvent>,
    brush_size_query: Query<&BrushSize, With<Brush>>,
    max_brush_size: Res<MaxBrushSize>,
    brush_type_state: Res<State<BrushTypeState>>,
    brush_mode_state: Res<State<BrushModeState>>,
    selected_particle: Res<SelectedParticle>,
    mut console_writer: EventWriter<PrintConsoleLine>,
) {
    console_writer.write(PrintConsoleLine::new("Current brush settings:".to_string()));

    if let Ok(brush_size) = brush_size_query.single() {
        console_writer.write(PrintConsoleLine::new(format!(
            "  Size: {} (soft cap: {})",
            brush_size.0, max_brush_size.0
        )));
    }

    console_writer.write(PrintConsoleLine::new(format!(
        "  Type: {:?}",
        brush_type_state.get()
    )));

    console_writer.write(PrintConsoleLine::new(format!(
        "  Mode: {:?}",
        brush_mode_state.get()
    )));

    console_writer.write(PrintConsoleLine::new(format!(
        "  Particle: {}",
        selected_particle.0.name
    )));
}

fn on_set_brush_particle(
    trigger: Trigger<BrushSetParticleEvent>,
    particle_cache: Option<Res<ParticleSearchCache>>,
    mut selected_particle: ResMut<SelectedParticle>,
    mut console_writer: EventWriter<PrintConsoleLine>,
) {
    let requested_name = &trigger.event().0;

    // Check if particle cache is available
    let Some(cache) = particle_cache else {
        console_writer.write(PrintConsoleLine::new(
            "Error: particle cache not available".to_string(),
        ));
        return;
    };

    // Find exact match first
    if let Some(exact_match) = cache.all_particles.iter().find(|p| *p == requested_name) {
        let static_name: &'static str = Box::leak(exact_match.clone().into_boxed_str());
        selected_particle.0 = Particle::new(static_name);
        console_writer.write(PrintConsoleLine::new(format!(
            "Setting brush particle to '{}'",
            static_name
        )));
        return;
    }

    // Find case-insensitive match
    if let Some(case_match) = cache
        .all_particles
        .iter()
        .find(|p| p.to_lowercase() == requested_name.to_lowercase())
    {
        let static_name: &'static str = Box::leak(case_match.clone().into_boxed_str());
        selected_particle.0 = Particle::new(static_name);
        console_writer.write(PrintConsoleLine::new(format!(
            "Setting brush particle to '{}' (matched case-insensitively)",
            static_name
        )));
        return;
    }

    // Find partial matches for suggestions
    let partial_matches: Vec<&String> = cache
        .all_particles
        .iter()
        .filter(|p| p.to_lowercase().contains(&requested_name.to_lowercase()))
        .take(5)
        .collect();

    if !partial_matches.is_empty() {
        let suggestions = partial_matches
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        console_writer.write(PrintConsoleLine::new(format!(
            "Error: particle '{}' not found. Did you mean {}",
            requested_name, suggestions
        )));
    } else {
        console_writer.write(PrintConsoleLine::new(format!(
            "Error: particle '{}' not found",
            requested_name
        )));
    }
}
