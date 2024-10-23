use bevy::prelude::*;

mod particle_type;
mod systems;

pub use particle_type::*;
use systems::*;

/// Plugin for particle type definitions.
pub struct ParticleTypePlugin;

impl Plugin for ParticleTypePlugin {
    fn build(&self, app: &mut App) {
	app.add_plugins(ParticleTypeSystemsPlugin);
        app.register_type::<ParticleType>()
            .init_resource::<ParticleTypeMap>();
    }
}

