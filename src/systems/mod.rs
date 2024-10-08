//! All systems driving particle behavior are found in these modules.

use bevy::prelude::*;

mod particle;
mod map;
mod movement;
mod hibernation;
mod color;
mod debug;
mod scenes;

pub use particle::*;
pub use map::*;
pub use movement::*;
pub use hibernation::*;
pub use color::*;
pub use debug::*;
pub use scenes::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleSimulationSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParticleDebugSet;

pub(super) struct ParticleSystemsPlugin;

impl bevy::prelude::Plugin for ParticleSystemsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                handle_new_particles.before(handle_particles),
                handle_particles,
                reset_chunks.after(handle_particles),
            )
                .in_set(ParticleSimulationSet),
        )
        .add_systems(Update, color_particles)
        .add_systems(
            Update,
            (color_chunks, count_dynamic_particles, count_total_particles)
                .in_set(ParticleDebugSet)
                .run_if(resource_exists::<crate::resources::DebugParticles>),
        )
        .add_systems(Startup, setup_particle_types)
        .add_systems(
            Update,
            save_scene_system.run_if(on_event::<crate::events::SaveSceneEvent>()),
        )
        .add_systems(
            Update,
            load_scene_system.run_if(on_event::<crate::events::LoadSceneEvent>()),
        )

        .observe(on_remove_particle)
        .observe(on_clear_chunk_map);
    }
}
