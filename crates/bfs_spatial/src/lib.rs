use bevy::prelude::{App, Plugin};
use bevy_spatial::{kdtree::KDTree2, AutomaticUpdate, SpatialStructure};
use bfs_core::Particle;
use std::time::Duration;

pub struct FallingSandSpatialPlugin {
    pub frequency: Duration,
}

impl Plugin for FallingSandSpatialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            AutomaticUpdate::<Particle>::new()
                .with_spatial_ds(SpatialStructure::KDTree2)
                .with_frequency(self.frequency),
        );
    }
}

pub type ParticleTree = KDTree2<Particle>;
