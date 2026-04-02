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
//!    world-sized texture using GPU compute shaders for efficient pixel updates.
//!    An optional effect layer system overlays shader-based visual effects, which can be
//!    accompanied by a custom shader written by the user.
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
//! 3. **Dirty-rect collection** (`PostUpdate`):
//!    [`update_world_color_texture`](pipeline::textures) iterates dirty chunk rects and
//!    packs `[texel_position, srgba_color]` pairs into a
//!    [`ParticleUpdateBuffer`](pipeline::textures::ParticleUpdateBuffer). A single-pass
//!    `update_all_effect_layers` system evaluates all registered effect layers per dirty
//!    texel and packs into an
//!    [`EffectUpdateBuffer`](pipeline::textures::EffectUpdateBuffer).
//!
//! 4. **GPU compute dispatch** (`Render`, `RenderSystems::Queue`):
//!    Extract systems copy the update buffers to the render world. Because only changed pixels are
//!    updated, the synchronization overhead is minimal. `dispatch_chunk_compute` /
//!    `dispatch_effect_compute` run WGSL compute shaders that scatter-write the packed updates
//!    into the storage textures (max 65,535 × 64 = ~4M updates per dispatch).
//!
//! 5. **Toroidal wrapping**: The texture uses modular addressing via a texture origin
//!    resource. When the map origin shifts (infinite world scrolling),
//!    [`handle_origin_shift`](pipeline::textures) clears unloaded chunk texels and
//!    the material UV offset is adjusted to keep the view aligned.
//!
//! 6. **Effect layer system**: Extensible overlay channels registered via the
//!    [`ChunkEffectLayer`] trait. Each layer maps to a texture array layer index and
//!    RGBA channel. One or more [`ChunkEffectMaterial`] shaders read the effect data
//!    texture array to produce effects as desired. Multiple materials can be registered,
//!    each with its own shader, stacked as separate overlay entities.
//!
//! ## Custom Shaders and Effect Layers
//!
//! The effect layer system lets you tag particles with marker components and render them
//! with custom WGSL shaders. Each [`ChunkEffectLayer`] maps a component to a texture array
//! layer index and RGBA channel. One or more [`ChunkEffectMaterial`] shaders read both the
//! color texture and the shared effect data texture array to produce visual effects.
//!
//! You can define your own effects and shaders for particles.
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
//! Implement [`ChunkEffectMaterial`] (which requires `Material2d`)
//! to bind the color texture, effect data texture, and any custom uniforms your shader needs.
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
//!         Self { chunk_texture, effect_data, uv_offset: Vec2::ZERO }
//!     }
//!     fn set_uv_offset(&mut self, offset: Vec2) {
//!         self.uv_offset = offset;
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
//! The shader receives the color texture and the effect data texture array. Each array
//! layer is an RGBA8 texture providing up to 4 channels. Use `textureLoad` with the
//! array layer index to read from different layers. A shader that reads layers 0 and 1:
//!
//! ```wgsl
//! @group(2) @binding(0) var chunk_texture: texture_2d<f32>;
//! @group(2) @binding(1) var chunk_sampler: sampler;
//! @group(2) @binding(2) var effect_data: texture_2d_array<f32>;
//! @group(2) @binding(3) var effect_sampler: sampler;
//! @group(2) @binding(4) var<uniform> uv_offset: vec2<f32>;
//!
//! @fragment
//! fn fragment(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
//!     let wrapped_uv = fract(uv + uv_offset);
//!     let tex_size = vec2<f32>(textureDimensions(chunk_texture, 0));
//!     let texel = vec2<i32>(floor(wrapped_uv * tex_size));
//!     let color = textureLoad(chunk_texture, texel, 0);
//!     let layer0 = textureLoad(effect_data, texel, 0, 0); // array layer 0
//!     let layer1 = textureLoad(effect_data, texel, 1, 0); // array layer 1
//!
//!     var out = color;
//!     // Layer 0, red channel = liquid
//!     out = mix(out, vec4(0.2, 0.5, 1.0, 1.0) * color.a, layer0.r * 0.4);
//!     // Layer 0, blue channel = glow
//!     out = mix(out, vec4(1.0, 0.7, 0.3, 1.0) * color.a, layer0.b * 0.6);
//!     // Layer 1, red channel = heat
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
