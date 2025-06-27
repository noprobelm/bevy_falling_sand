#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(
    clippy::nursery,
    clippy::pedantic,
    nonstandard_style,
    rustdoc::broken_intra_doc_links,
    missing_docs
)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]
//! Provides spatial querying functionality for particles in the Falling Sand simulation.
use bevy::prelude::{App, Plugin};
use bevy_spatial::{kdtree::KDTree2, AutomaticUpdate, SpatialStructure};
use bfs_core::Particle;
use std::time::Duration;

/// The spatial plugin, which provides constructs for performing kdtree spatial queries on
/// particles.
pub struct FallingSandSpatialPlugin {
    /// The frequency at which the kdtree is updated.
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

/// Type alias for a [`KDTree2`](https://docs.rs/bevy_spatial/latest/bevy_spatial/kdtree/struct.KDTree2.html) structure containing particles.
pub type ParticleTree = KDTree2<Particle>;
