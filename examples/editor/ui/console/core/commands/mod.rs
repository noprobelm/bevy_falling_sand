use bevy::prelude::*;

pub mod brush;
pub mod camera;
pub mod clear;
pub mod exit;
pub mod help;
pub mod particles;
pub mod physics;
pub mod avian;

use brush::BrushCommandPlugin;
use camera::CameraCommandPlugin;
use clear::ClearCommandPlugin;
use exit::ExitCommandPlugin;
use help::HelpCommandPlugin;
use particles::ParticlesCommandPlugin;
use physics::PhysicsCommandPlugin;
use avian::AvianCommandPlugin;

use crate::ui::console::core::commands;

use super::{CommandRegistry, ConsoleCache, ConsoleConfiguration};

pub struct ConsoleCommandsPlugin;

impl Plugin for ConsoleCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_command_registry)
            .add_plugins((
                ClearCommandPlugin,
                ExitCommandPlugin,
                HelpCommandPlugin,
                ParticlesCommandPlugin,
                CameraCommandPlugin,
                PhysicsCommandPlugin,
                BrushCommandPlugin,
            AvianCommandPlugin,
            ));
    }
}

fn init_command_registry(
    mut registry: ResMut<CommandRegistry>,
    mut config: ResMut<ConsoleConfiguration>,
    mut cache: ResMut<ConsoleCache>,
) {
        use commands::{brush::*, camera::*, clear::*, exit::*, help::*, particles::*, physics::*, avian::*};

    registry.register::<ClearCommand>();
    registry.register::<ExitCommand>();
    registry.register::<HelpCommand>();
    registry.register::<ParticlesCommand>();
    registry.register::<CameraCommand>();
    registry.register::<PhysicsCommand>();
    registry.register::<BrushCommand>();
    registry.register::<AvianCommand>();

    config.register_command::<ClearCommand>();
    config.register_command::<ExitCommand>();
    config.register_command::<HelpCommand>();
    config.register_command::<ParticlesCommand>();
    config.register_command::<CameraCommand>();
    config.register_command::<PhysicsCommand>();
    config.register_command::<BrushCommand>();
    config.register_command::<AvianCommand>();
    cache.rebuild_tries(&config);
}
