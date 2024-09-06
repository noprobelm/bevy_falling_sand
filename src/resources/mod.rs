//! All resources related to particle behavior are found in these modules.
use bevy::utils::Duration;
use bevy_spatial::{AutomaticUpdate, SpatialStructure};

use super::Particle;

mod debug;
mod kdtree;
mod map;
mod simulation;

pub use debug::*;
pub use kdtree::*;
pub use map::*;
pub use simulation::*;

pub(super) struct ParticleResourcesPlugin;

impl bevy::prelude::Plugin for ParticleResourcesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<ChunkMap>()
            .init_resource::<ParticleTypeMap>()
            .init_resource::<DynamicParticleCount>()
            .init_resource::<TotalParticleCount>()
            .init_resource::<SimulationRun>()
            .add_plugins(
                AutomaticUpdate::<Particle>::new()
                    .with_spatial_ds(SpatialStructure::KDTree2)
                    .with_frequency(Duration::from_millis(200)),
            );
    }
}
