//! All systems driving particle behavior are found in these modules.

use crate::SimulationRun;
use bevy::prelude::*;

mod color;
mod debug;
mod hibernation;
mod map;
mod movement;
mod burning;
mod scenes;
mod particle_deserializer;

pub use color::*;
pub use debug::*;
pub use hibernation::*;
pub use map::*;
pub use movement::*;
pub use burning::*;
pub use scenes::*;
pub use particle_deserializer::*;

/// System set for systems that influence particle management.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleSimulationSet;

/// System set for systems that provide debugging functionality.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleDebugSet;

/// Plugin for all systems related to falling sand particles.
pub(super) struct ParticleSystemsPlugin;

impl bevy::prelude::Plugin for ParticleSystemsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, handle_new_particles)
            .add_systems(Update, handle_new_particle_types)
            .add_systems(
                Update,
                (handle_particles, reset_chunks.after(handle_particles))
                    .in_set(ParticleSimulationSet)
                    .run_if(resource_exists::<SimulationRun>),
            )
            .add_systems(Update, (handle_fire, handle_burning))
            .add_systems(Update, color_particles.after(handle_new_particles))
            .add_systems(
                Update,
                (color_chunks, count_dynamic_particles, count_total_particles)
                    .in_set(ParticleDebugSet)
                    .run_if(resource_exists::<crate::resources::DebugParticles>),
            )
            .add_systems(
                Update,
                save_scene_system.run_if(on_event::<crate::events::SaveSceneEvent>()),
            )
            .add_systems(
                Update,
                load_scene_system
                    .before(handle_new_particles)
                    .run_if(on_event::<crate::events::LoadSceneEvent>()),
            )
            .add_systems(
                Update,
                deserialize_particle_types
                    .before(handle_new_particles)
                    .run_if(on_event::<crate::events::DeserializeParticleTypesEvent>()),
            )
            .observe(on_remove_particle)
            .observe(on_clear_chunk_map);
    }
}
