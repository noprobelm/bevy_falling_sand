//! Particle color assignment and rendering pipeline
//!
//! ## Overview
//!
//! This module provides two major subsystems:
//!
//! 1. **Particle coloring** ([`particle`]) — assigns colors to particles based on their
//!    parent [`ParticleType`](crate::ParticleType)'s [`ColorProfile`]. Supports
//!    palette-based and gradient-based color sources with sequential or random assignment.
//!
//! 2. **Chunk rendering pipeline** ([`pipeline`]) — renders all particles into a single
//!    world-sized texture using GPU compute shaders for efficient pixel updates. An effect
//!    layer system overlays shader-based visual effects via user-written WGSL materials,
//!    with overlay quads sized per frame to the active region.
//!
//! ## Render Pipeline Flow
//!
//! 1. **Setup** (`Update`, [`RenderingSystems::ChunkImage`]):
//!    [`setup_world_textures`](pipeline::textures) creates a world-sized color texture
//!    and effect texture when the [`ParticleMap`](crate::ParticleMap) exists. A
//!    [`WorldColorMaterial`](pipeline::textures::WorldColorMaterial) sprite is spawned
//!    covering the full loaded region.
//!
//! 2. **Color registration** (`PreUpdate`, [`ParticleSystems::Registration`](crate::ParticleSystems)):
//!    When a particle is spawned or synced, a propagator reads its parent's [`ColorProfile`]
//!    and inserts [`ParticleColor`] + [`ColorIndex`] components. [`ForceColor`] overrides
//!    the profile; [`WithColor`] restores a saved index for scene persistence.
//!
//! 3. **Dirty-rect collection** (`PostUpdate`, [`RenderingSystems::ChunkEffectLayerUpdate`]):
//!    [`update_world_color_texture`](pipeline::textures) iterates dirty chunk rects and
//!    packs `[texel_position, srgba_color]` pairs into a
//!    [`ParticleUpdateBuffer`](pipeline::textures::ParticleUpdateBuffer). A single-pass
//!    `update_all_effect_layers` system evaluates all registered effect layers per dirty
//!    texel, packs into an [`EffectUpdateBuffer`](pipeline::textures::EffectUpdateBuffer),
//!    and incrementally updates [`ChunkEffectActivity`] counters as texels transition
//!    between zero and non-zero.
//!
//! 4. **Region culling** (`PostUpdate`, [`RenderingSystems::ChunkEffectRegion`]):
//!    For each registered material, `compute_active_region` builds a bounding box over
//!    the chunks where any of [`ChunkEffectMaterial::affected_channels`] is non-zero,
//!    padded by [`ChunkEffectMaterial::padding`] texels. `update_effect_overlay` resizes
//!    the material's overlay quad to that box and pushes it into the material's
//!    `quad_world_rect` uniform — or hides the entity entirely when nothing is active.
//!
//! 5. **GPU compute dispatch** (`Render`, `RenderSystems::Queue`):
//!    Extract systems copy the update buffers to the render world. Because only changed pixels are
//!    updated, the synchronization overhead is minimal. `dispatch_chunk_compute` /
//!    `dispatch_effect_compute` run WGSL compute shaders that scatter-write the packed updates
//!    into the storage textures (max 65,535 × 64 = ~4M updates per dispatch).
//!
//! 6. **Toroidal wrapping**: The texture uses modular addressing via a texture origin
//!    resource. When the map origin shifts (infinite world scrolling),
//!    [`handle_origin_shift`](pipeline::textures) clears unloaded chunk texels, zeroes
//!    their activity counters, and adjusts the material UV offset to keep the view aligned.
//!
//! 7. **Effect layer system**: Extensible overlay channels registered via the
//!    [`ChunkEffectLayer`] trait. Each layer maps to a texture array layer index and
//!    RGBA channel. Each [`ChunkEffectMaterial`] reads the effect data texture array to
//!    produce its effect. Multiple materials can be registered, each with its own shader,
//!    stacked as separate overlay entities.
//!
//! ## Custom Shaders and Effect Layers
//!
//! The effect layer system lets you tag particles with marker components and render them
//! with custom WGSL shaders. Each [`ChunkEffectLayer`] maps a component to a texture array
//! layer index and RGBA channel. Each [`ChunkEffectMaterial`] reads the color texture and
//! the shared effect data texture array to produce its effect; multiple materials can be
//! stacked as independent overlay entities.
//!
//! ### Step 1 — Define marker components and effect layers
//!
//! Each layer maps a marker [`Component`] to a texture array layer index and RGBA channel
//! (0=R, 1=G, 2=B, 3=A). The `layer()` method defaults to 0; override it to write to
//! additional texture array layers. Add the marker to any [`ParticleType`](crate::ParticleType)
//! whose particles should participate in that effect.
//!
//! ```ignore
//! use bevy::prelude::*;
//! use bevy_falling_sand::render::{ChunkEffectLayer};
//!
//! #[derive(Component, Default, Clone, Reflect)]
//! pub struct LiquidEffect;
//!
//! pub struct LiquidEffectLayer;
//!
//! impl ChunkEffectLayer for LiquidEffectLayer {
//!     type Source = LiquidEffect;
//!     // layer() defaults to 0
//!     fn channel() -> usize { 0 } // Red channel of layer 0
//! }
//!
//! #[derive(Component, Default, Clone, Reflect)]
//! pub struct GlowEffect;
//!
//! pub struct GlowEffectLayer;
//!
//! impl ChunkEffectLayer for GlowEffectLayer {
//!     type Source = GlowEffect;
//!     // layer() defaults to 0
//!     fn channel() -> usize { 2 } // Blue channel of layer 0
//! }
//!
//! // A fifth effect that overflows to texture array layer 1.
//! // Each layer provides 4 RGBA channels, so once layer 0 is full,
//! // override layer() to write to the next array layer.
//! #[derive(Component, Default, Clone, Reflect)]
//! pub struct HeatEffect;
//!
//! pub struct HeatEffectLayer;
//!
//! impl ChunkEffectLayer for HeatEffectLayer {
//!     type Source = HeatEffect;
//!     fn layer() -> usize { 1 } // Second texture array layer
//!     fn channel() -> usize { 0 } // Red channel of layer 1
//! }
//! ```
//!
//! ### Step 2 — Implement a custom material
//!
//! Implement [`ChunkEffectMaterial`] (which requires `Material2d`) to bind the color
//! texture, the effect data texture array, the UV offset uniform, the per-frame
//! `quad_world_rect` uniform, and any custom uniforms your shader needs.
//!
//! Declare the `(layer, channel)` pairs the shader reads via
//! [`ChunkEffectMaterial::affected_channels`]. Each frame the framework sizes the
//! overlay quad to the bounding box of chunks where any of those channels has data,
//! padded by [`ChunkEffectMaterial::padding`] texels, and hides the overlay entirely
//! when nothing is active.
//!
//! ```ignore
//! use bevy::prelude::*;
//! use bevy::sprite::{AlphaMode2d, Material2d};
//! use bevy::render::render_resource::{AsBindGroup, ShaderRef};
//! use bevy::reflect::TypePath;
//! use bevy_falling_sand::render::ChunkEffectMaterial;
//!
//! #[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
//! pub struct MyEffectMaterial {
//!     #[texture(0)]
//!     #[sampler(1)]
//!     pub chunk_texture: Handle<Image>,
//!     #[texture(2, dimension = "2d_array")]
//!     #[sampler(3)]
//!     pub effect_data: Handle<Image>,
//!     #[uniform(4)]
//!     pub uv_offset: Vec2,
//!     #[uniform(5)]
//!     pub quad_world_rect: Vec4,
//! }
//!
//! impl Material2d for MyEffectMaterial {
//!     fn fragment_shader() -> ShaderRef {
//!         "shaders/my_effects.wgsl".into()
//!     }
//!     fn alpha_mode(&self) -> AlphaMode2d {
//!         AlphaMode2d::Blend
//!     }
//! }
//!
//! impl ChunkEffectMaterial for MyEffectMaterial {
//!     fn new(chunk_texture: Handle<Image>, effect_data: Handle<Image>) -> Self {
//!         Self {
//!             chunk_texture,
//!             effect_data,
//!             uv_offset: Vec2::ZERO,
//!             quad_world_rect: Vec4::ZERO,
//!         }
//!     }
//!     fn set_uv_offset(&mut self, offset: Vec2) {
//!         self.uv_offset = offset;
//!     }
//!     fn set_quad_world_rect(&mut self, rect: Vec4) {
//!         self.quad_world_rect = rect;
//!     }
//!     fn affected_channels() -> &'static [(usize, usize)] {
//!         &[(0, 0), (0, 2), (1, 0)]
//!     }
//!     fn padding() -> u32 {
//!         12 // largest neighborhood radius the shader reads
//!     }
//! }
//! ```
//!
//! ### Step 3 — Register materials and layers
//!
//! Use the [`ChunkEffectApp`] extension trait to register materials and layers.
//! Multiple materials can be registered — each gets its own overlay entity and shader,
//! stacked by z-order. All materials share the same effect data texture array.
//! If no custom material is registered, [`DefaultChunkEffectMaterial`] is used
//! automatically.
//!
//! ```ignore
//! use bevy::prelude::*;
//! use bevy_falling_sand::render::ChunkEffectApp;
//!
//! // Single material for all effects:
//! fn build(app: &mut App) {
//!     app.add_chunk_effect_material::<MyEffectMaterial>()
//!        .add_chunk_effect_layer::<LiquidEffectLayer>()
//!        .add_chunk_effect_layer::<GlowEffectLayer>()
//!        .add_chunk_effect_layer::<HeatEffectLayer>();
//! }
//!
//! // Or multiple materials with separate shaders:
//! fn build_multi(app: &mut App) {
//!     app.add_chunk_effect_material::<LiquidMaterial>()
//!        .add_chunk_effect_material::<GlowMaterial>()
//!        .add_chunk_effect_layer::<LiquidEffectLayer>()
//!        .add_chunk_effect_layer::<GlowEffectLayer>()
//!        .add_chunk_effect_layer::<HeatEffectLayer>();
//! }
//! ```
//!
//! ### Step 4 — Write the WGSL shader
//!
//! The shader receives the color texture, the effect data texture array, the UV offset,
//! and the `quad_world_rect` uniform that maps the local quad UV to a world texel.
//! Import the `bevy_falling_sand::effects` helper module to map UVs to texels.
//!
//! ```wgsl
//! #import bevy_falling_sand::effects::quad_uv_to_world_texel
//!
//! @group(2) @binding(0) var chunk_texture: texture_2d<f32>;
//! @group(2) @binding(1) var chunk_sampler: sampler;
//! @group(2) @binding(2) var effect_data: texture_2d_array<f32>;
//! @group(2) @binding(3) var effect_sampler: sampler;
//! @group(2) @binding(4) var<uniform> uv_offset: vec2<f32>;
//! @group(2) @binding(5) var<uniform> quad_world_rect: vec4<f32>;
//!
//! @fragment
//! fn fragment(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
//!     let tex_size = vec2<i32>(textureDimensions(chunk_texture, 0));
//!     let texel = quad_uv_to_world_texel(uv, quad_world_rect, tex_size, uv_offset);
//!
//!     let color = textureLoad(chunk_texture, texel, 0);
//!     let layer0 = textureLoad(effect_data, texel, 0, 0);
//!     let layer1 = textureLoad(effect_data, texel, 1, 0);
//!
//!     var out = color;
//!     out = mix(out, vec4(0.2, 0.5, 1.0, 1.0) * color.a, layer0.r * 0.4);
//!     out = mix(out, vec4(1.0, 0.7, 0.3, 1.0) * color.a, layer0.b * 0.6);
//!     out = mix(out, vec4(1.0, 0.2, 0.0, 1.0) * color.a, layer1.r * 0.5);
//!     return out;
//! }
//! ```

/// Particle color assignment — profiles, components, and propagation.
pub mod particle;
/// Chunk rendering pipeline — textures, materials, and GPU dispatch.
pub mod pipeline;
/// System set definitions for the rendering pipeline.
pub mod schedule;

use bevy::prelude::*;

pub use particle::*;
pub use pipeline::*;
pub use schedule::RenderingSystems;

use particle::ParticleColorPlugin;
use pipeline::RenderPipelinePlugin;
use schedule::SchedulePlugin;

/// Plugin providing particle rendering for the falling sand simulation.
///
/// Registers color assignment, world texture rendering, and GPU compute dispatch.
#[derive(Default)]
pub struct FallingSandRenderPlugin;

impl Plugin for FallingSandRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SchedulePlugin, ParticleColorPlugin, RenderPipelinePlugin));
    }
}
