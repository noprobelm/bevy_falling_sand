//! Holds the KDTree resource.
use crate::Particle;
use bevy_spatial::kdtree::KDTree2;

/// A 2-d KDTree for performing spatial queries on particles
pub type ParticleTree = KDTree2<Particle>;
