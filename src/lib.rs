//! `bevy_falling_sand` is a generic plugin for adding falling sand physics to your Bevy project.

use bevy::prelude::*;
use bevy_turborand::prelude::*;

pub use components::*;
pub use resources::*;
pub use systems::*;
pub use gizmos::*;

mod components;
mod resources;
mod systems;
mod gizmos;

pub struct FallingSandPlugin;

impl Plugin for FallingSandPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RngPlugin::default())
            .init_resource::<ChunkMap>()
            .init_resource::<ParticleParentMap>()
            .init_gizmo_group::<DebugGizmos>()
            .add_systems(Startup, setup_particles)
            .add_systems(Last, handle_new_particles)
            .add_systems(Update, handle_new_particles)
            .add_systems(
                Update,
                (handle_particles, reset_chunks.after(handle_particles))
                    .in_set(ParticleMovementSet),
            )
            .add_systems(Update, color_particles)
            .add_systems(
                Update,
                color_chunks
                    .in_set(ParticleDebugSet)
                    .run_if(resource_exists::<DebugParticles>),
            );
    }
}
