use bevy::prelude::*;

use crate::brush::BrushTypeState;

use super::super::core::{ConsoleCommand, PrintConsoleLine};

pub struct BrushCommandPlugin;

impl Plugin for BrushCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_update_brush_type);
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
        vec![Box::new(BrushSetCommand)]
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
        vec![Box::new(BrushSetTypeCommand)]
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
            "Setting brush type to 'line'".to_string(),
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

#[derive(Clone, Event, Hash, Debug, Eq, PartialEq, PartialOrd)]
struct SetBrushTypeEvent(pub BrushTypeState);

fn on_update_brush_type(
    trigger: Trigger<SetBrushTypeEvent>,
    mut brush_type_state_next: ResMut<NextState<BrushTypeState>>,
) {
    brush_type_state_next.set(trigger.event().0);
}
