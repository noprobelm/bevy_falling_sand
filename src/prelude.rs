//! Re-exports commonly used items for convenient importing.

pub use crate::{FallingSandMinimalPlugin, FallingSandPlugin};

pub use crate::core::{
    AttachedToParticleType, ChanceLifetime, ChanceMutation, ChunkCoord, ChunkDirtyState,
    ChunkIndex, ChunkLoader, ChunkLoadingConfig, ChunkLoadingRun, ChunkLoadingState, ChunkRegion,
    ChunkSystems, DespawnAllParticlesSignal, DespawnBatchConfig, DespawnParticleSignal,
    DespawnParticleTypeChildrenSignal, GridPosition, OnSpawnCallback, Particle, ParticleMap,
    ParticleRng, ParticleSimulationRun, ParticleSyncExt, ParticleSystems, ParticleType,
    ParticleTypeRegistry, PendingDespawn, PropagatorFilter, SimulationStepSignal, SpatialEntry,
    SpatialMap, SpawnParticleSignal, SyncParticleSignal, SyncParticleTypeChildrenSignal,
    TimedLifetime, condition_msg_simulation_step_received,
};

#[cfg(feature = "render")]
pub use crate::render::{
    ChunkEffectApp, ChunkEffectLayer, ChunkEffectMaterial, ChunkRenderingConfig,
    ChunkRenderingPlugin, ColorAssignment, ColorGradient, ColorIndex, ColorProfile, ColorRng,
    ColorSource, DefaultChunkEffectMaterial, FallingSandRenderPlugin, ForceColor, Palette,
    ParticleColor, RenderingSystems, TextureSource, WithColor, WorldColorTexture,
    WorldEffectTexture, extract_chunk_image,
};

#[cfg(feature = "movement")]
pub use crate::movement::{
    AirResistance, ChunkIterationState, Density, DespawnDynamicParticlesSignal,
    DespawnStaticParticlesSignal, FallingSandMovementPlugin, Momentum, Movement, MovementRng,
    MovementSystemState, NeighborGroup, ParticleMovementSystems, ParticleResistor, Speed,
};

#[cfg(feature = "debug")]
pub use crate::debug::{
    ActiveChunkColor, ActiveParticleCount, ChunkColor, DebugDirtyRects, DebugParticleCount,
    DebugParticleMap, DirtyRectColor, DynamicParticleCount, FallingSandDebugPlugin,
    ParticleDebugSystems, RigidBodyCount, StaticParticleCount, TotalParticleCount,
};

#[cfg(feature = "reactions")]
pub use crate::reactions::{
    BurnProduct, Burning, Consumes, ContactReaction, ContactRule, Corrodible, Corrosive,
    FallingSandReactionsPlugin, Fire, Flammable, ReactionRng,
};

#[cfg(feature = "physics")]
pub use crate::physics::{
    DirtyChunkUpdateInterval, DouglasPeuckerEpsilon, DynamicRigidBodyProxy,
    FallingSandPhysicsPlugin, PromoteDynamicRigidBodyParticle, StaticRigidBodyParticle,
    SuspendedParticle,
};

#[cfg(feature = "persistence")]
pub use crate::persistence::{
    ChunkPersistenceError, FallingSandPersistencePlugin, LoadParticleTypesSignal,
    ParticlePersistenceConfig, ParticlePersistenceState, ParticleTypesLoadedSignal,
    ParticleTypesPersistedSignal, PendingSaveTasks, PersistChunksSignal,
    PersistParticleTypesSignal,
};

#[cfg(feature = "scenes")]
pub use crate::scenes::{
    DespawnSceneSignal, FallingSandScenesPlugin, ParticleScene, ParticleSceneInstance,
    ParticleSceneRegistry, ParticleSceneRoot, SceneLayer, SpawnSceneSignal,
};
