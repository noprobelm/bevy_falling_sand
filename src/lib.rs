//! `bevy_falling_sand` is a generic plugin for adding falling sand physics to your Bevy project.

use bevy::prelude::*;
use bevy_turborand::prelude::*;

pub use components::*;
pub use resources::*;
pub use systems::*;

mod components;
mod resources;
mod systems;

pub struct FallingSandPlugin;

impl Plugin for FallingSandPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RngPlugin::default());

        app.init_resource::<ParticleMap>();
        app.init_resource::<ParticleParentMap>();

	app.add_systems(Startup, setup_particle_types);

	app.add_systems(Update, handle_new_particles);

	app.add_systems(FixedUpdate, handle_particles);
        app.add_systems(FixedUpdate, reset_chunks.after(handle_particles));

	app.add_systems(Update, color_particles);
    }
}
