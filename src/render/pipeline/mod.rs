mod gpu;
pub mod textures;

use bevy::prelude::*;

pub use textures::{
    extract_chunk_image, ChunkEffectApp, ChunkEffectLayer, ChunkEffectMaterial,
    ChunkRenderingConfig, ChunkRenderingPlugin, DefaultChunkEffectMaterial, WorldColorTexture,
    WorldEffectTexture,
};

pub(super) struct RenderPipelinePlugin;

impl Plugin for RenderPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ChunkRenderingPlugin, gpu::GpuChunkRenderingPlugin));
    }
}
