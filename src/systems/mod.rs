//! All systems driving particle behavior are found in these modules.

use crate::SimulationRun;
use bevy::prelude::*;

mod burning;
mod color;
mod debug;
mod hibernation;
mod map;
mod material;
mod movement;
mod particle_deserializer;
mod scenes;

pub use burning::*;
pub use color::*;
pub use debug::*;
pub use hibernation::*;
pub use map::*;
pub use material::*;
pub use movement::*;
pub use particle_deserializer::*;
pub use scenes::*;

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
                (
                    handle_particles,
                    reset_chunks.after(handle_particles),
                    handle_fire.before(handle_burning),
                    handle_burning.before(handle_particles),
                )
                    .in_set(ParticleSimulationSet)
                    .run_if(resource_exists::<SimulationRun>),
            )
            .add_systems(Update, (color_particles.after(handle_new_particles),))
            .add_systems(
                Update,
                (
		    color_flowing_particles,
                    color_randomizing_particles,
                )
                    .run_if(resource_exists::<SimulationRun>),
            )
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
            .observe(on_clear_chunk_map)
            .observe(on_solid_added)
            .observe(on_movable_solid_added)
            .observe(on_liquid_added)
            .observe(on_gas_added)
            .observe(on_reset_density)
            .observe(on_reset_movement_priority)
            .observe(on_reset_velocity)
            .observe(on_reset_particle_color)
            .observe(on_reset_momentum)
            .observe(on_reset_fire)
            .observe(on_reset_burns)
            .observe(on_reset_burning)
            .observe(on_reset_randomizes_color)
            .observe(on_reset_flows_color);
    }
}
