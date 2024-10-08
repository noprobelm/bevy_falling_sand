//! All resources related to particle behavior are found in these modules.
mod debug;
mod map;
mod simulation;

pub use debug::*;
pub use map::*;
pub use simulation::*;

pub(super) struct ParticleResourcesPlugin;

impl bevy::prelude::Plugin for ParticleResourcesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<ChunkMap>()
            .init_resource::<ParticleTypeMap>()
            .init_resource::<DynamicParticleCount>()
            .init_resource::<TotalParticleCount>();
    }
}
