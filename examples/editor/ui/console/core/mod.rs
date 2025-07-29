pub mod commands;
pub mod core;

use bevy::prelude::*;
pub use core::*;

use commands::{
    camera::CameraCommandPlugin, clear::ClearCommandPlugin, exit::ExitCommandPlugin,
    help::HelpCommandPlugin, particles::ParticlesCommandPlugin,
};

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConsoleState>()
            .init_resource::<ConsoleConfiguration>()
            .init_resource::<ConsoleCache>()
            .init_resource::<CommandRegistry>()
            .add_event::<ConsoleCommandEntered>()
            .add_event::<PrintConsoleLine>()
            .add_systems(Startup, init_command_registry)
            .add_systems(Update, command_handler)
            .add_plugins((
                ClearCommandPlugin,
                ExitCommandPlugin,
                HelpCommandPlugin,
                ParticlesCommandPlugin,
                CameraCommandPlugin,
            ));
    }
}

fn init_command_registry(
    mut registry: ResMut<CommandRegistry>,
    mut config: ResMut<ConsoleConfiguration>,
    mut cache: ResMut<ConsoleCache>,
) {
    use commands::{camera::*, clear::*, exit::*, help::*, particles::*, physics::*};

    registry.register::<ClearCommand>();
    registry.register::<ExitCommand>();
    registry.register::<HelpCommand>();
    registry.register::<ParticlesCommand>();
    registry.register::<CameraCommand>();
    registry.register::<PhysicsCommand>();

    config.register_command::<ClearCommand>();
    config.register_command::<ExitCommand>();
    config.register_command::<HelpCommand>();
    config.register_command::<ParticlesCommand>();
    config.register_command::<CameraCommand>();
    config.register_command::<PhysicsCommand>();
    cache.rebuild_tries(&config);
}
