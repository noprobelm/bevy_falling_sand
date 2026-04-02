use avian2d::math::Vector;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::tasks::Task;

use super::geometry::MeshGenerationResult;
use crate::core::ChunkCoord;

pub(super) struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StaticRigidBodyParticleMeshData>()
            .init_resource::<StaticRigidBodyParticleColliders>()
            .init_resource::<PreviousFrameDirtyChunks>()
            .init_resource::<DouglasPeuckerEpsilon>()
            .init_resource::<DirtyChunkUpdateInterval>()
            .init_resource::<ChunkLastProcessedTime>()
            .init_resource::<PendingMeshTasks>()
            .init_resource::<ChunkOccupancy>();
    }
}

/// Configures the epsilon tolerance for the Douglas-Peucker polygon simplification algorithm.
///
/// Lower values preserve more detail in collision meshes but produce more vertices.
/// Higher values simplify aggressively, improving performance at the cost of precision.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::physics::DouglasPeuckerEpsilon;
///
/// fn setup(mut commands: Commands) {
///     commands.insert_resource(DouglasPeuckerEpsilon(1.0));
/// }
/// ```
#[derive(Resource, Debug)]
pub struct DouglasPeuckerEpsilon(pub f32);

impl Default for DouglasPeuckerEpsilon {
    fn default() -> Self {
        Self(0.5)
    }
}

/// Configures how often dirty chunks recalculate their collision meshes (in seconds).
///
/// Chunks that just stopped being dirty are always processed immediately.
/// Currently dirty chunks are throttled to this interval to improve performance.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_falling_sand::physics::DirtyChunkUpdateInterval;
///
/// fn setup(mut commands: Commands) {
///     commands.insert_resource(DirtyChunkUpdateInterval(0.2));
/// }
/// ```
#[derive(Resource, Debug)]
pub struct DirtyChunkUpdateInterval(pub f32);

impl Default for DirtyChunkUpdateInterval {
    fn default() -> Self {
        Self(0.1)
    }
}

#[derive(Resource, Default, Debug)]
pub(super) struct PreviousFrameDirtyChunks(
    pub(super) bevy::platform::collections::HashSet<ChunkCoord>,
);

#[derive(Resource, Default, Debug)]
pub(super) struct ChunkLastProcessedTime(pub(super) HashMap<ChunkCoord, f32>);

#[derive(Resource, Default)]
pub(super) struct PendingMeshTasks {
    pub(super) tasks: HashMap<ChunkCoord, Task<MeshGenerationResult>>,
}

pub(super) type ChunkMeshData = (Vec<Vec<Vector>>, Vec<Vec<[u32; 3]>>);

#[derive(Resource, Default, Debug)]
pub(super) struct StaticRigidBodyParticleMeshData {
    pub(super) chunks: HashMap<ChunkCoord, ChunkMeshData>,
}

#[derive(Resource, Default, Debug)]
pub(super) struct StaticRigidBodyParticleColliders(pub(super) HashMap<ChunkCoord, Entity>);

#[derive(Resource, Default)]
pub(super) struct ChunkOccupancy {
    pub(super) bitmaps: HashMap<ChunkCoord, Vec<bool>>,
}
