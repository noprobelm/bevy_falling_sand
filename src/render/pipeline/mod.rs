mod gpu;
pub mod textures;

use bevy::prelude::*;

pub use textures::{
    ChunkEffectApp, ChunkEffectLayer, ChunkEffectMaterial, ChunkRenderingConfig,
    ChunkRenderingPlugin, DefaultChunkEffectMaterial, WorldColorTexture, WorldEffectTexture,
    extract_chunk_image,
};

pub(super) struct RenderPipelinePlugin;

impl Plugin for RenderPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ChunkRenderingPlugin, gpu::GpuChunkRenderingPlugin));
    }
}
