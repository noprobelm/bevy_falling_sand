//! Holds the KDTree resource.
use bevy_spatial::kdtree::KDTree2;

use crate::components::Particle;

/// A 2-d KDTree for performing spatial queries on particles
pub type ParticleTree = KDTree2<Particle>;
