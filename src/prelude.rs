//! Re-exports commonly used items for convenient importing.

pub use crate::{FallingSandMinimalPlugin, FallingSandPlugin};

pub use crate::core::{
    condition_msg_simulation_step_received, AttachedToParticleType, ChanceLifetime, ChunkCoord,
    ChunkDirtyState, ChunkIndex, ChunkLoader, ChunkLoadingConfig, ChunkLoadingState, ChunkRegion,
    ChunkLoadingRun, ChunkSystems, DespawnAllParticlesSignal, DespawnBatchConfig, DespawnParticleSignal,
    DespawnParticleTypeChildrenSignal, GridPosition, OnSpawnCallback, Particle, ParticleMap,
    ParticleRng, ParticleSimulationRun, ParticleSyncExt, ParticleSystems, ParticleType,
    ParticleTypeRegistry, PendingDespawn, SimulationStepSignal, SpatialEntry, SpatialMap,
    SpawnParticleSignal, SyncParticleSignal, SyncParticleTypeChildrenSignal, TimedLifetime,
};

#[cfg(feature = "render")]
pub use crate::render::{
    extract_chunk_image, ChunkEffectApp, ChunkEffectLayer, ChunkEffectMaterial,
    ChunkRenderingConfig, ChunkRenderingPlugin, ColorAssignment, ColorGradient, ColorIndex,
    ColorProfile, ColorRng, ColorSource, DefaultChunkEffectMaterial, FallingSandRenderPlugin,
    ForceColor, Palette, ParticleColor, RenderingSystems, TextureSource, WithColor,
    WorldColorTexture, WorldEffectTexture,
};

#[cfg(feature = "movement")]
pub use crate::movement::{
    AirResistance, ChunkIterationState, Density, DespawnDynamicParticlesSignal,
    DespawnStaticParticlesSignal, FallingSandMovementPlugin, Momentum, Movement, MovementRng,
    MovementSystemState, NeighborGroup, ParticleMovementSet, ParticleResistor, Speed,
};

#[cfg(feature = "debug")]
pub use crate::debug::{
    ActiveChunkColor, ActiveParticleCount, ChunkColor, DebugDirtyRects, DebugParticleCount,
    DebugParticleMap, DirtyRectColor, DynamicParticleCount, FallingSandDebugPlugin,
    ParticleDebugSet, RigidBodyCount, StaticParticleCount, TotalParticleCount,
};

#[cfg(feature = "reactions")]
pub use crate::reactions::{
    BurnProduct, Burning, Consumes, ContactReaction, ContactRule, FallingSandReactionsPlugin, Fire,
    Flammable, ReactionRng,
};

#[cfg(feature = "physics")]
pub use crate::physics::{
    DirtyChunkUpdateInterval, DouglasPeuckerEpsilon, DynamicRigidBodyProxy,
    PromoteDynamicRigidBodyParticle, FallingSandPhysicsPlugin, StaticRigidBodyParticle,
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
    FallingSandScenesPlugin, ParticleScene, ParticleSceneRegistry, SpawnSceneSignal,
};
