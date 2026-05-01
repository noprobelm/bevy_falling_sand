//! World-sized texture rendering for optimized particle visualization.
//!
//! Instead of per-chunk images, this module creates a world-sized color texture and
//! an effect data texture array with a single sprite and one or more effect overlay entities.
//! Dirty rect writes still only touch changed pixels.

use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::atomic::Ordering;

use bevy::ecs::component::ComponentId;
use bevy::ecs::system::SystemState;
use bevy::image::ImageSampler;
use bevy::mesh::Mesh2d;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, TextureDimension, TextureFormat, TextureUsages,
};
use bevy_shader::ShaderRef;
use bevy_sprite_render::{AlphaMode2d, Material2d, Material2dPlugin, MeshMaterial2d};

use super::gpu::ComputePipelineReadyFlag;
use crate::core::{
    ChunkCoord, ChunkDirtyState, ChunkIndex, ChunkLoadingState, ChunkRegion, ChunkSystems,
    Particle, ParticleMap, ParticleSystems,
};
use crate::render::particle::components::ParticleColor;
use crate::render::schedule::RenderingSystems;
use bevy::platform::collections::HashMap;

/// Plugin that enables world-texture-based particle rendering.
///
/// Creates a single world-sized color texture rendered as a sprite.
/// Effect overlays use `Mesh2d` + `Material2d` for custom shader effects.
#[derive(Default)]
pub struct ChunkRenderingPlugin;

impl Plugin for ChunkRenderingPlugin {
    fn build(&self, app: &mut App) {
        bevy_shader::load_shader_library!(app, "shaders/effects.wgsl");
        bevy::asset::embedded_asset!(app, "shaders/world_color.wgsl");

        app.add_plugins(Material2dPlugin::<WorldColorMaterial>::default())
            .init_resource::<ChunkRenderingConfig>()
            .init_resource::<ParticleUpdateBuffer>()
            .init_resource::<EffectUpdateBuffer>()
            .init_resource::<ChunkEffectActivity>()
            .add_systems(
                Update,
                (
                    setup_world_textures.run_if(
                        resource_exists::<ParticleMap>
                            .and(not(resource_exists::<WorldColorTexture>)),
                    ),
                    ApplyDeferred,
                )
                    .chain()
                    .in_set(RenderingSystems::ChunkImage),
            )
            .add_systems(
                Update,
                (
                    handle_origin_shift.after(RenderingSystems::ChunkImage),
                    update_world_color_uv_offset.after(RenderingSystems::ChunkImage),
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    redirty_all_chunks_on_pipeline_ready.before(ParticleSystems::Simulation),
                    update_world_color_texture
                        .after(ParticleSystems::Simulation)
                        .after(ChunkSystems::Cleanup),
                ),
            );

        bevy::asset::embedded_asset!(app, "shaders/default_chunk_effect.wgsl");
    }
}

/// Configuration for chunk-based rendering.
#[derive(Resource)]
pub struct ChunkRenderingConfig {
    /// Background color for empty pixels
    pub background_color: Color,
}

impl Default for ChunkRenderingConfig {
    fn default() -> Self {
        Self {
            background_color: Color::NONE,
        }
    }
}

/// Material for rendering the world color texture with toroidal UV wrapping.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct WorldColorMaterial {
    /// The world color texture.
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    /// UV offset for toroidal wrapping, computed from the origin shift.
    #[uniform(2)]
    pub uv_offset: Vec2,
}

impl Material2d for WorldColorMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_falling_sand/render/pipeline/shaders/world_color.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Resource storing the initial origin of the world texture.
///
/// The texture uses toroidal addressing — texel coordinates wrap via modulo.
/// This origin is captured at texture creation time and never changes.
#[derive(Resource, Clone, Copy)]
pub struct WorldTextureOrigin(pub IVec2);

/// Resource storing the handle to the world-sized color texture.
#[derive(Resource)]
pub struct WorldColorTexture(pub Handle<Image>);

/// Resource storing the handle to the world-sized effect data texture.
#[derive(Resource)]
pub struct WorldEffectTexture(pub Handle<Image>);

/// Resource storing the entity for the world color sprite.
///
/// Kept as a resource to prevent the entity from being despawned.
#[derive(Resource)]
struct WorldColorEntity(Entity);

/// Resource storing the entity for a world effect overlay.
///
/// Generic over the material type so that each registered `ChunkEffectMaterial`
/// gets its own overlay entity.
#[derive(Resource)]
struct WorldEffectEntity<M: ChunkEffectMaterial>(Entity, PhantomData<fn() -> M>);

/// Per-material z-offset for effect overlay stacking.
#[derive(Resource)]
struct EffectOverlayZOffset<M: ChunkEffectMaterial>(f32, PhantomData<fn() -> M>);

/// Buffer of particle color updates to send to the GPU compute shader.
///
/// Each entry is a packed `[position, color]` pair where:
/// - `position = tx | (ty << 16)` (texel coordinates)
/// - `color = r | (g << 8) | (b << 16) | (a << 24)` (sRGB bytes)
#[derive(Resource, Default)]
pub struct ParticleUpdateBuffer {
    /// Packed `[position, color]` pairs for GPU dispatch.
    pub updates: Vec<[u32; 2]>,
}

/// Buffer of effect data updates to send to the GPU compute shader.
///
/// Each entry is a packed `[position, rgba]` pair where:
/// - `position = tx (bits 0–13) | ty (bits 14–27) | array_layer (bits 28–31)`
/// - `rgba = r | (g << 8) | (b << 16) | (a << 24)` (raw effect channel bytes)
#[derive(Resource, Default)]
pub struct EffectUpdateBuffer {
    /// Packed `[position, rgba]` pairs for GPU dispatch.
    pub updates: Vec<[u32; 2]>,
}

/// CPU-side mirror of the world effect data texture array.
///
/// The single-pass `update_all_effect_layers` system writes to all channels and layers
/// of this buffer, then packs dirty texels into `EffectUpdateBuffer`.
#[derive(Resource)]
pub struct WorldEffectShadowBuffer {
    /// RGBA8 pixel data mirroring the GPU effect texture array layout.
    /// Length = width * height * 4 * `layer_count`.
    pub data: Vec<u8>,
    /// Texture width in pixels.
    pub width: u32,
    /// Texture height in pixels.
    pub height: u32,
    /// Number of texture array layers.
    pub layer_count: u32,
}

#[derive(Clone, Copy)]
struct RegisteredEffectLayer {
    layer: usize,
    channel: usize,
    component_id: ComponentId,
}

#[derive(Resource, Default)]
struct EffectLayerRegistry {
    layers: Vec<RegisteredEffectLayer>,
    texture_layer_count: usize,
    active_texture_layers: Vec<usize>,
}

#[derive(Resource, Default)]
struct EffectSystemCache {
    state: Option<
        SystemState<(
            Res<'static, ParticleMap>,
            Res<'static, ChunkIndex>,
            Res<'static, WorldEffectShadowBuffer>,
            Query<'static, 'static, (&'static ChunkRegion, &'static ChunkDirtyState)>,
        )>,
    >,
    dirty_entries: Vec<(usize, usize, Option<Entity>, ChunkCoord)>,
}

/// Per-chunk per-channel counters tracking how many texels in each chunk currently
/// hold a non-zero value for each `(layer, channel)` slot of the effect data texture.
///
/// Updated incrementally by `update_all_effect_layers` as texels transition between
/// zero and non-zero values, and cleared per chunk when chunks unload. Region culling
/// reads this to compute each material's draw bounds.
#[derive(Resource, Default)]
pub struct ChunkEffectActivity {
    per_chunk: HashMap<ChunkCoord, Vec<u32>>,
    slots_per_chunk: usize,
}

impl ChunkEffectActivity {
    fn ensure_slots(&mut self, slots: usize) {
        if self.slots_per_chunk == slots {
            return;
        }
        self.slots_per_chunk = slots;
        for counts in self.per_chunk.values_mut() {
            counts.resize(slots, 0);
        }
    }

    fn inc(&mut self, coord: ChunkCoord, slot: usize) {
        let slots = self.slots_per_chunk;
        let entry = self
            .per_chunk
            .entry(coord)
            .or_insert_with(|| vec![0; slots]);
        if slot < entry.len() {
            entry[slot] = entry[slot].saturating_add(1);
        }
    }

    fn dec(&mut self, coord: ChunkCoord, slot: usize) {
        let drop_entry;
        {
            let Some(entry) = self.per_chunk.get_mut(&coord) else {
                return;
            };
            if slot < entry.len() && entry[slot] > 0 {
                entry[slot] -= 1;
            }
            drop_entry = entry.iter().all(|&c| c == 0);
        }
        if drop_entry {
            self.per_chunk.remove(&coord);
        }
    }

    fn clear_chunk(&mut self, coord: ChunkCoord) {
        self.per_chunk.remove(&coord);
    }

    fn any_active(&self, coord: ChunkCoord, slots: &[usize]) -> bool {
        let Some(counts) = self.per_chunk.get(&coord) else {
            return false;
        };
        slots.iter().any(|&s| s < counts.len() && counts[s] > 0)
    }

    fn iter(&self) -> impl Iterator<Item = (ChunkCoord, &Vec<u32>)> {
        self.per_chunk.iter().map(|(c, v)| (*c, v))
    }
}

/// World-space bounding rectangle of `M`'s active region, or `None` when nothing is active.
///
/// Bounds the chunks where any of [`ChunkEffectMaterial::affected_channels`] currently
/// has data, padded outward by [`ChunkEffectMaterial::padding`] texels.
#[derive(Resource)]
pub struct ChunkEffectActiveRegion<M: ChunkEffectMaterial>(
    pub Option<IRect>,
    PhantomData<fn() -> M>,
);

impl<M: ChunkEffectMaterial> Default for ChunkEffectActiveRegion<M> {
    fn default() -> Self {
        Self(None, PhantomData)
    }
}

/// Trait that maps a type-level marker to an RGBA channel in the effect data texture array.
///
/// The effect data is stored as a 2D texture array where each array layer is an RGBA8 texture
/// providing 4 channels. Layers and channels are specified via [`layer()`](Self::layer) and
/// [`channel()`](Self::channel). The single-pass `update_all_effect_layers` system checks for
/// the presence of `Self::Source` on each particle entity and writes 255 or 0 to the
/// corresponding slot.
///
/// The layer type itself does not need to be a `Component` — it is only used as a type-level
/// marker. The `Source` associated type specifies which component to check on particle entities.
pub trait ChunkEffectLayer: Send + Sync + 'static {
    /// The component to check for on particle entities.
    type Source: Component;

    /// The texture array layer index this effect maps to.
    ///
    /// Each array layer provides 4 RGBA channels. Defaults to 0.
    #[must_use]
    fn layer() -> usize {
        0
    }

    /// The RGBA channel index within the texture array layer.
    /// Must be 0 (R), 1 (G), 2 (B), or 3 (A).
    fn channel() -> usize;
}

/// Trait for effect materials that overlay chunk images with shader-based effects.
///
/// The framework spawns one overlay quad per registered material and resizes it each
/// frame to the bounding box of chunks where any of the material's
/// [`affected_channels`](Self::affected_channels) currently has non-zero data, expanded
/// by [`padding`](Self::padding) texels. The overlay is hidden entirely when no chunk
/// has data for any of those channels.
///
/// Implementors must:
/// - Bind the world color texture and the effect data texture array (via `AsBindGroup`).
/// - Carry a `uv_offset: Vec2` uniform and implement [`set_uv_offset`](Self::set_uv_offset).
/// - Carry a `quad_world_rect: Vec4` uniform and implement
///   [`set_quad_world_rect`](Self::set_quad_world_rect). The shader uses this to map the
///   local quad UV to a world texel via `bevy_falling_sand::effects::quad_uv_to_world_texel`.
/// - Declare every `(layer, channel)` pair their shader reads via
///   [`affected_channels`](Self::affected_channels). An empty slice means "render whenever
///   any registered channel has data anywhere" — appropriate only for fallback materials
///   that read all channels.
/// - Override [`padding`](Self::padding) to the maximum texel radius any neighborhood
///   loop in the shader reaches outside the local channel.
pub trait ChunkEffectMaterial: Material2d + Sized {
    /// Creates the material from the world color texture and the effect data texture array.
    fn new(chunk_texture: Handle<Image>, effect_data: Handle<Image>) -> Self;

    /// Sets the UV offset for toroidal wrapping when the map origin shifts.
    fn set_uv_offset(&mut self, offset: Vec2);

    /// Stores the framework-computed quad rectangle uniform consumed by
    /// `bevy_falling_sand::effects::quad_uv_to_world_texel` to map the local quad UV to
    /// a texel in the world effect texture array. The value is
    /// `Vec4(origin_x, origin_y, size_x, size_y)` in world units, with `origin` relative
    /// to the current map origin.
    fn set_quad_world_rect(&mut self, rect: Vec4);

    /// `(layer, channel)` pairs that the fragment shader reads from the effect data
    /// texture array. Drives the active-region bounding box. Empty = "any active channel
    /// activates this material," used by fallback materials only.
    #[must_use]
    fn affected_channels() -> &'static [(usize, usize)] {
        &[]
    }

    /// Texel radius the shader's neighborhood loops reach outside any channel-active
    /// pixel. The active region's bounding box is expanded by this many texels so
    /// halo / blur effects extend past the underlying particles.
    #[must_use]
    fn padding() -> u32 {
        0
    }
}

#[derive(Resource, Default)]
struct EffectMaterialRegistry {
    registered_type_ids: HashSet<TypeId>,
    next_z: f32,
}

/// Default effect material that renders the base color where any effect channel is active.
///
/// Provides a basic pass-through shader that discards pixels with no active effect channels.
/// Users who just want data textures without custom visual effects can use this material.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct DefaultChunkEffectMaterial {
    /// The world color texture.
    #[texture(0)]
    #[sampler(1)]
    pub chunk_texture: Handle<Image>,
    /// The effect data texture array (RGBA8 per layer).
    #[texture(2, dimension = "2d_array")]
    #[sampler(3)]
    pub effect_data: Handle<Image>,
    /// UV offset for toroidal wrapping, computed from the origin shift.
    #[uniform(4)]
    pub uv_offset: Vec2,
    /// Quad rectangle uniform; consumed by `bevy_falling_sand::effects::quad_uv_to_world_texel`.
    #[uniform(5)]
    pub quad_world_rect: Vec4,
}

impl Material2d for DefaultChunkEffectMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_falling_sand/render/pipeline/shaders/default_chunk_effect.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

impl ChunkEffectMaterial for DefaultChunkEffectMaterial {
    fn new(chunk_texture: Handle<Image>, effect_data: Handle<Image>) -> Self {
        Self {
            chunk_texture,
            effect_data,
            uv_offset: Vec2::ZERO,
            quad_world_rect: Vec4::ZERO,
        }
    }

    fn set_uv_offset(&mut self, offset: Vec2) {
        self.uv_offset = offset;
    }

    fn set_quad_world_rect(&mut self, rect: Vec4) {
        self.quad_world_rect = rect;
    }
}

/// Extension trait for registering chunk effect layers and materials.
///
/// Multiple materials can be registered, each with its own shader. All materials
/// share the same effect data texture array. If `add_chunk_effect_layer` is called
/// without a prior `add_chunk_effect_material`, the [`DefaultChunkEffectMaterial`]
/// is used automatically.
pub trait ChunkEffectApp {
    /// Registers a chunk effect layer that maps component `T` to a texture array layer and RGBA channel.
    ///
    /// The layer definition is stored in the `EffectLayerRegistry`. The single-pass
    /// `update_all_effect_layers` system evaluates all registered layers per dirty texel.
    fn add_chunk_effect_layer<T: ChunkEffectLayer>(&mut self) -> &mut Self;

    /// Registers an effect material and its supporting infrastructure.
    ///
    /// Each call with a distinct material type spawns a separate overlay entity with
    /// its own shader, stacked by z-order. All materials share the same effect data
    /// texture array. Calling with the same type twice is a no-op.
    fn add_chunk_effect_material<M: ChunkEffectMaterial>(&mut self) -> &mut Self
    where
        M::Data: PartialEq + Eq + std::hash::Hash + Clone;
}

impl ChunkEffectApp for App {
    fn add_chunk_effect_material<M: ChunkEffectMaterial>(&mut self) -> &mut Self
    where
        M::Data: PartialEq + Eq + std::hash::Hash + Clone,
    {
        if !self.world().contains_resource::<EffectMaterialRegistry>() {
            self.init_resource::<EffectMaterialRegistry>();
            self.init_resource::<EffectLayerRegistry>();
            self.init_resource::<EffectSystemCache>();
            self.add_systems(
                PostUpdate,
                update_all_effect_layers.in_set(RenderingSystems::ChunkEffectLayerUpdate),
            );
        }

        let type_id = TypeId::of::<M>();
        let mut registry = self.world_mut().resource_mut::<EffectMaterialRegistry>();
        if registry.registered_type_ids.contains(&type_id) {
            return self;
        }
        let z = 0.1 + registry.next_z;
        registry.next_z += 0.01;
        registry.registered_type_ids.insert(type_id);

        self.insert_resource(EffectOverlayZOffset::<M>(z, PhantomData));
        self.add_plugins(Material2dPlugin::<M>::default());
        self.init_resource::<ChunkEffectActiveRegion<M>>();
        self.add_systems(
            Update,
            setup_world_effect_overlay::<M>
                .run_if(
                    resource_exists::<WorldColorTexture>
                        .and(resource_exists::<WorldEffectTexture>)
                        .and(not(resource_exists::<WorldEffectEntity<M>>)),
                )
                .after(RenderingSystems::ChunkImage),
        );
        self.add_systems(
            PostUpdate,
            (compute_active_region::<M>, update_effect_overlay::<M>)
                .chain()
                .in_set(RenderingSystems::ChunkEffectRegion),
        );

        self
    }

    fn add_chunk_effect_layer<T: ChunkEffectLayer>(&mut self) -> &mut Self {
        if !self.world().contains_resource::<EffectMaterialRegistry>() {
            self.add_chunk_effect_material::<DefaultChunkEffectMaterial>();
        }

        let component_id = self.world_mut().register_component::<T::Source>();
        let layer = T::layer();
        let channel = T::channel();

        let mut registry = self.world_mut().resource_mut::<EffectLayerRegistry>();
        registry.layers.push(RegisteredEffectLayer {
            layer,
            channel,
            component_id,
        });
        registry.texture_layer_count = registry.texture_layer_count.max(layer + 1);
        if !registry.active_texture_layers.contains(&layer) {
            registry.active_texture_layers.push(layer);
            registry.active_texture_layers.sort_unstable();
        }

        self
    }
}

/// Convert a `Color` to sRGB `[u8; 4]` bytes (RGBA order).
#[inline]
fn color_to_srgb_bytes(color: Color) -> [u8; 4] {
    let c = color.to_srgba();
    [
        (c.red * 255.0) as u8,
        (c.green * 255.0) as u8,
        (c.blue * 255.0) as u8,
        (c.alpha * 255.0) as u8,
    ]
}

/// Look up a particle's color at `pos`, falling back to `bg` when absent.
#[inline]
fn particle_color_bytes(
    map: &ParticleMap,
    particle_query: &Query<&ParticleColor, With<Particle>>,
    pos: IVec2,
    bg: [u8; 4],
) -> [u8; 4] {
    map.get_copied(pos)
        .ok()
        .flatten()
        .and_then(|e| particle_query.get(e).ok())
        .map_or(bg, |pc| color_to_srgb_bytes(pc.0))
}

/// Convert a world position to texel coordinates using toroidal wrapping.
///
/// Uses the initial texture origin so that texel addresses are stable across
/// origin shifts. Coordinates wrap via `rem_euclid` and are always valid.
#[inline(always)]
const fn world_to_texel(
    pos: IVec2,
    tex_origin: IVec2,
    world_w: i32,
    world_h: i32,
) -> (usize, usize) {
    let tx = (pos.x - tex_origin.x).rem_euclid(world_w);
    let ty = (tex_origin.y + world_h - 1 - pos.y).rem_euclid(world_h);
    (tx as usize, ty as usize)
}

/// Creates the world-sized color and effect textures and spawns the color sprite.
///
/// Runs once when `ParticleMap` exists but `WorldColorTexture` doesn't.
#[allow(clippy::needless_pass_by_value)]
fn setup_world_textures(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WorldColorMaterial>>,
    config: Res<ChunkRenderingConfig>,
    map: Res<ParticleMap>,
    registry: Option<Res<EffectLayerRegistry>>,
) {
    let width = map.width();
    let height = map.height();
    let origin = map.origin();

    let effect_layer_count = registry
        .as_ref()
        .map_or(1, |r| r.texture_layer_count.max(1)) as u32;

    // Color texture
    let bg_bytes = color_to_srgb_bytes(config.background_color);
    let mut color_data = vec![0u8; (width * height * 4) as usize];
    for pixel in color_data.chunks_exact_mut(4) {
        pixel.copy_from_slice(&bg_bytes);
    }
    let mut color_image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        color_data,
        TextureFormat::Rgba8Unorm,
        default(),
    );
    color_image.sampler = ImageSampler::nearest();
    color_image.texture_descriptor.usage =
        TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
    let color_handle = images.add(color_image);

    // Effect data texture array
    let effect_data = vec![0u8; (width * height * 4 * effect_layer_count) as usize];
    let mut effect_image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: effect_layer_count,
        },
        TextureDimension::D2,
        effect_data,
        TextureFormat::Rgba8Unorm,
        default(),
    );
    effect_image.sampler = ImageSampler::nearest();
    effect_image.texture_descriptor.usage =
        TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
    effect_image.texture_view_descriptor =
        Some(bevy::render::render_resource::TextureViewDescriptor {
            dimension: Some(bevy::render::render_resource::TextureViewDimension::D2Array),
            ..default()
        });
    let effect_handle = images.add(effect_image);

    let center_x = origin.x as f32 + width as f32 / 2.0;
    let center_y = origin.y as f32 + height as f32 / 2.0;

    let quad = meshes.add(Rectangle::new(width as f32, height as f32));
    let material = materials.add(WorldColorMaterial {
        texture: color_handle.clone(),
        uv_offset: Vec2::ZERO,
    });

    let entity = commands
        .spawn((
            Mesh2d(quad),
            MeshMaterial2d(material),
            Transform::from_xyz(center_x, center_y, 0.0),
        ))
        .id();

    commands.insert_resource(WorldTextureOrigin(origin));
    commands.insert_resource(WorldColorTexture(color_handle));
    commands.insert_resource(WorldEffectTexture(effect_handle));
    commands.insert_resource(WorldColorEntity(entity));
    commands.insert_resource(WorldEffectShadowBuffer {
        data: vec![0u8; (width * height * 4 * effect_layer_count) as usize],
        width,
        height,
        layer_count: effect_layer_count,
    });
}

/// Spawns the unit-sized effect overlay entity with `MeshMaterial2d<M>`. Each frame the
/// overlay's transform is resized to the active region by [`update_effect_overlay`], or
/// hidden when no chunk holds data for any of `M::affected_channels()`.
#[allow(clippy::needless_pass_by_value)]
fn setup_world_effect_overlay<M: ChunkEffectMaterial>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<M>>,
    color_tex: Res<WorldColorTexture>,
    effect_tex: Res<WorldEffectTexture>,
    z_offset: Res<EffectOverlayZOffset<M>>,
) {
    let quad = meshes.add(Rectangle::new(1.0, 1.0));
    let handle = materials.add(M::new(color_tex.0.clone(), effect_tex.0.clone()));

    let entity = commands
        .spawn((
            Mesh2d(quad),
            MeshMaterial2d(handle),
            Transform::from_xyz(0.0, 0.0, z_offset.0),
            Visibility::Hidden,
        ))
        .id();

    commands.insert_resource(WorldEffectEntity::<M>(entity, PhantomData));
}

/// Computes the world-space bounding rectangle of all chunks where any of
/// `M::affected_channels()` currently has non-zero data, padded by `M::padding()` texels.
/// An empty `affected_channels()` is treated as "any active chunk in
/// [`ChunkEffectActivity`] contributes" — appropriate for fallback materials that read
/// every channel.
#[allow(clippy::needless_pass_by_value)]
fn compute_active_region<M: ChunkEffectMaterial>(
    activity: Res<ChunkEffectActivity>,
    chunk_index: Res<ChunkIndex>,
    mut region: ResMut<ChunkEffectActiveRegion<M>>,
) {
    let channels = M::affected_channels();
    let slots: Vec<usize> = channels.iter().map(|&(l, c)| l * 4 + c).collect();

    let mut bbox: Option<IRect> = None;
    for (coord, _) in activity.iter() {
        if !slots.is_empty() && !activity.any_active(coord, &slots) {
            continue;
        }
        let chunk_rect = chunk_index.chunk_coord_to_chunk_region(coord);
        bbox = Some(bbox.map_or(chunk_rect, |b| b.union(chunk_rect)));
    }

    region.0 = bbox.map(|mut b| {
        let pad = M::padding() as i32;
        if pad > 0 {
            b.min -= IVec2::splat(pad);
            b.max += IVec2::splat(pad);
        }
        b
    });
}

/// Resizes the overlay quad to the material's active region and pushes the rectangle
/// (relative to the current map origin) plus the UV offset into the material uniforms.
/// Hides the entity when no channel is active anywhere.
#[allow(clippy::needless_pass_by_value)]
fn update_effect_overlay<M: ChunkEffectMaterial>(
    map: Res<ParticleMap>,
    tex_origin: Option<Res<WorldTextureOrigin>>,
    effect_entity: Option<Res<WorldEffectEntity<M>>>,
    region: Res<ChunkEffectActiveRegion<M>>,
    mut materials: ResMut<Assets<M>>,
    material_query: Query<&MeshMaterial2d<M>>,
    mut transform_query: Query<(&mut Transform, &mut Visibility)>,
) {
    let Some(tex_origin) = tex_origin else { return };
    let Some(effect_entity) = effect_entity else {
        return;
    };
    let Ok((mut t, mut visibility)) = transform_query.get_mut(effect_entity.0) else {
        return;
    };

    let Some(rect) = region.0 else {
        if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    let map_origin = map.origin();
    let size_x = (rect.max.x - rect.min.x + 1) as f32;
    let size_y = (rect.max.y - rect.min.y + 1) as f32;
    let cx = size_x.mul_add(0.5, rect.min.x as f32);
    let cy = size_y.mul_add(0.5, rect.min.y as f32);
    t.translation.x = cx;
    t.translation.y = cy;
    t.scale.x = size_x;
    t.scale.y = size_y;

    if *visibility == Visibility::Hidden {
        *visibility = Visibility::Inherited;
    }

    if let Ok(mat_handle) = material_query.get(effect_entity.0)
        && let Some(material) = materials.get_mut(&mat_handle.0)
    {
        let width = map.width() as f32;
        let height = map.height() as f32;
        let shift = map_origin - tex_origin.0;
        material.set_uv_offset(Vec2::new(shift.x as f32 / width, -shift.y as f32 / height));
        material.set_quad_world_rect(Vec4::new(
            (rect.min.x - map_origin.x) as f32,
            (rect.min.y - map_origin.y) as f32,
            size_x,
            size_y,
        ));
    }
}

fn redirty_all_chunks_on_pipeline_ready(
    flag: Option<Res<ComputePipelineReadyFlag>>,
    mut has_fired: Local<bool>,
    mut chunks: Query<(&ChunkRegion, &mut ChunkDirtyState)>,
) {
    if *has_fired {
        return;
    }
    let Some(flag) = flag else { return };
    if !flag.0.load(Ordering::Relaxed) {
        return;
    }
    *has_fired = true;
    for (region, mut dirty_state) in &mut chunks {
        dirty_state.mark_dirty_rect(region.region());
    }
}

#[allow(clippy::needless_pass_by_value)]
fn update_world_color_uv_offset(
    map: Res<ParticleMap>,
    tex_origin: Option<Res<WorldTextureOrigin>>,
    color_entity: Option<Res<WorldColorEntity>>,
    mut materials: ResMut<Assets<WorldColorMaterial>>,
    material_query: Query<&MeshMaterial2d<WorldColorMaterial>>,
    mut transform_query: Query<&mut Transform>,
) {
    let Some(tex_origin) = tex_origin else { return };
    let Some(color_entity) = color_entity else {
        return;
    };

    let width = map.width() as f32;
    let height = map.height() as f32;
    let origin = map.origin();
    let center_x = origin.x as f32 + width / 2.0;
    let center_y = origin.y as f32 + height / 2.0;

    if let Ok(mut t) = transform_query.get_mut(color_entity.0) {
        t.translation.x = center_x;
        t.translation.y = center_y;
    }

    if let Ok(mat_handle) = material_query.get(color_entity.0)
        && let Some(material) = materials.get_mut(&mat_handle.0)
    {
        let shift = origin - tex_origin.0;
        material.uv_offset = Vec2::new(shift.x as f32 / width, -shift.y as f32 / height);
    }
}

#[allow(
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::similar_names
)]
fn handle_origin_shift(
    loading_state: Res<ChunkLoadingState>,
    map: Res<ParticleMap>,
    chunk_index: Res<ChunkIndex>,
    config: Res<ChunkRenderingConfig>,
    tex_origin: Option<Res<WorldTextureOrigin>>,
    mut update_buffer: ResMut<ParticleUpdateBuffer>,
    mut effect_staging: Option<ResMut<WorldEffectShadowBuffer>>,
    mut effect_update_buffer: ResMut<EffectUpdateBuffer>,
    mut activity: ResMut<ChunkEffectActivity>,
) {
    if !loading_state.origin_shifted {
        return;
    }
    let Some(tex_origin) = tex_origin else { return };

    let origin = tex_origin.0;
    let world_w = map.width() as i32;
    let world_h = map.height() as i32;

    let [r, g, b, a] = color_to_srgb_bytes(config.background_color);
    let packed_bg =
        u32::from(r) | (u32::from(g) << 8) | (u32::from(b) << 16) | (u32::from(a) << 24);

    let effect_layer_count = effect_staging
        .as_ref()
        .map_or(0, |s| s.layer_count as usize);

    let cs = chunk_index.chunk_size() as i32;
    for &(coord, _) in &loading_state.unloaded_this_frame {
        activity.clear_chunk(coord);
        let min = IVec2::new(coord.x() * cs, coord.y() * cs);
        let max = min + IVec2::splat(cs - 1);
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                let pos = IVec2::new(x, y);
                let (tx, ty) = world_to_texel(pos, origin, world_w, world_h);
                let packed_pos = (tx as u32) | ((ty as u32) << 16);
                update_buffer.updates.push([packed_pos, packed_bg]);
                if let Some(ref mut staging) = effect_staging {
                    let world_w_u = staging.width as usize;
                    let layer_stride = staging.height as usize * world_w_u;
                    for layer_idx in 0..effect_layer_count {
                        let pi = (layer_idx * layer_stride + ty * world_w_u + tx) * 4;
                        staging.data[pi] = 0;
                        staging.data[pi + 1] = 0;
                        staging.data[pi + 2] = 0;
                        staging.data[pi + 3] = 0;
                        let effect_packed_pos =
                            (tx as u32) | ((ty as u32) << 14) | ((layer_idx as u32) << 28);
                        effect_update_buffer.updates.push([effect_packed_pos, 0]);
                    }
                }
            }
        }
    }
}

/// Collects dirty particle color data into the update buffer for GPU compute dispatch.
#[allow(clippy::needless_pass_by_value)]
fn update_world_color_texture(
    map: Res<ParticleMap>,
    config: Res<ChunkRenderingConfig>,
    color_tex: Option<Res<WorldColorTexture>>,
    tex_origin: Option<Res<WorldTextureOrigin>>,
    mut update_buffer: ResMut<ParticleUpdateBuffer>,
    dirty_chunks: Query<(&ChunkRegion, &ChunkDirtyState)>,
    particle_query: Query<&ParticleColor, With<Particle>>,
) {
    update_buffer.updates.clear();

    if color_tex.is_none() {
        return;
    }
    let Some(tex_origin) = tex_origin else { return };

    let bg_bytes = color_to_srgb_bytes(config.background_color);

    let origin = tex_origin.0;
    let world_w = map.width() as i32;
    let world_h = map.height() as i32;

    for (region, dirty_state) in dirty_chunks.iter() {
        let Some(dirty_rect) = dirty_state.current else {
            continue;
        };

        let rect = region.region();

        if let Some(ref positions) = dirty_state.current_positions {
            for &pos in positions {
                if !rect.contains(pos) {
                    continue;
                }
                let (tx, ty) = world_to_texel(pos, origin, world_w, world_h);

                let [r, g, b, a] = particle_color_bytes(&map, &particle_query, pos, bg_bytes);

                let packed_pos = (tx as u32) | ((ty as u32) << 16);
                let packed_color = u32::from(r)
                    | (u32::from(g) << 8)
                    | (u32::from(b) << 16)
                    | (u32::from(a) << 24);
                update_buffer.updates.push([packed_pos, packed_color]);
            }
        } else {
            let min_x = dirty_rect.min.x.max(rect.min.x);
            let max_x = dirty_rect.max.x.min(rect.max.x);
            let min_y = dirty_rect.min.y.max(rect.min.y);
            let max_y = dirty_rect.max.y.min(rect.max.y);

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let pos = IVec2::new(x, y);
                    let (tx, ty) = world_to_texel(pos, origin, world_w, world_h);

                    let [r, g, b, a] = particle_color_bytes(&map, &particle_query, pos, bg_bytes);

                    let packed_pos = (tx as u32) | ((ty as u32) << 16);
                    let packed_color = u32::from(r)
                        | (u32::from(g) << 8)
                        | (u32::from(b) << 16)
                        | (u32::from(a) << 24);
                    update_buffer.updates.push([packed_pos, packed_color]);
                }
            }
        }
    }
}

/// Single-pass system that evaluates all registered effect layers and packs the update buffer.
///
/// Iterates dirty positions once, looks up each entity once, evaluates all registered
/// effect layers, writes to the staging buffer, and packs the update buffer inline.
/// Updates [`ChunkEffectActivity`] counters as texels transition between zero and
/// non-zero values so region culling has fresh data.
fn update_all_effect_layers(world: &mut World) {
    let Some(registry) = world.get_resource::<EffectLayerRegistry>() else {
        return;
    };
    if registry.layers.is_empty() {
        return;
    }
    let layers = registry.layers.clone();
    let active_texture_layers = registry.active_texture_layers.clone();
    let texture_layer_count = registry.texture_layer_count;

    let Some(&WorldTextureOrigin(origin)) = world.get_resource::<WorldTextureOrigin>() else {
        return;
    };

    world
        .resource_mut::<ChunkEffectActivity>()
        .ensure_slots(texture_layer_count * 4);

    let mut cache = world.resource_mut::<EffectSystemCache>();
    let mut dirty_entries = std::mem::take(&mut cache.dirty_entries);
    dirty_entries.clear();
    let mut cached_state = cache.state.take();

    let state = cached_state.get_or_insert_with(|| SystemState::new(world));

    {
        let (map, chunk_index, staging, dirty_chunks) = state.get(world);
        collect_dirty_entries(
            chunk_index.chunk_size() as i32,
            staging.width as i32,
            staging.height as i32,
            origin,
            &map,
            dirty_chunks.iter(),
            &mut dirty_entries,
        );
    }

    world.resource_scope(|world, mut staging: Mut<WorldEffectShadowBuffer>| {
        world.resource_scope(|world, mut update_buffer: Mut<EffectUpdateBuffer>| {
            world.resource_scope(|world, mut activity: Mut<ChunkEffectActivity>| {
                update_buffer.updates.clear();
                let world_w = staging.width as usize;
                let layer_stride = staging.height as usize * world_w;

                for &(tx, ty, entity_opt, chunk_coord) in &dirty_entries {
                    apply_effect_entry(
                        tx,
                        ty,
                        entity_opt,
                        chunk_coord,
                        &layers,
                        &active_texture_layers,
                        layer_stride,
                        world_w,
                        world,
                        &mut staging,
                        &mut activity,
                        &mut update_buffer,
                    );
                }
            });
        });
    });

    let mut cache = world.resource_mut::<EffectSystemCache>();
    cache.dirty_entries = dirty_entries;
    cache.state = cached_state;
}

fn collect_dirty_entries<'a>(
    chunk_size: i32,
    world_w: i32,
    world_h: i32,
    origin: IVec2,
    map: &ParticleMap,
    dirty_chunks: impl Iterator<Item = (&'a ChunkRegion, &'a ChunkDirtyState)>,
    out: &mut Vec<(usize, usize, Option<Entity>, ChunkCoord)>,
) {
    for (region, dirty_state) in dirty_chunks {
        let Some(dirty_rect) = dirty_state.current else {
            continue;
        };
        let rect = region.region();
        let chunk_coord = ChunkCoord::new(
            rect.min.x.div_euclid(chunk_size),
            rect.min.y.div_euclid(chunk_size),
        );

        if let Some(ref positions) = dirty_state.current_positions {
            for &pos in positions {
                if !rect.contains(pos) {
                    continue;
                }
                let (tx, ty) = world_to_texel(pos, origin, world_w, world_h);
                let entity = map.get_copied(pos).ok().flatten();
                out.push((tx, ty, entity, chunk_coord));
            }
        } else {
            let min_x = dirty_rect.min.x.max(rect.min.x);
            let max_x = dirty_rect.max.x.min(rect.max.x);
            let min_y = dirty_rect.min.y.max(rect.min.y);
            let max_y = dirty_rect.max.y.min(rect.max.y);

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let pos = IVec2::new(x, y);
                    let (tx, ty) = world_to_texel(pos, origin, world_w, world_h);
                    let entity = map.get_copied(pos).ok().flatten();
                    out.push((tx, ty, entity, chunk_coord));
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_effect_entry(
    tx: usize,
    ty: usize,
    entity_opt: Option<Entity>,
    chunk_coord: ChunkCoord,
    layers: &[RegisteredEffectLayer],
    active_texture_layers: &[usize],
    layer_stride: usize,
    world_w: usize,
    world: &World,
    staging: &mut WorldEffectShadowBuffer,
    activity: &mut ChunkEffectActivity,
    update_buffer: &mut EffectUpdateBuffer,
) {
    for layer_def in layers {
        let val = entity_opt.map_or(0, |e| {
            world.get_entity(e).ok().map_or(0u8, |er| {
                if er.contains_id(layer_def.component_id) {
                    255
                } else {
                    0
                }
            })
        });
        let idx = (layer_def.layer * layer_stride + ty * world_w + tx) * 4 + layer_def.channel;
        let old = staging.data[idx];
        if old != val {
            let slot = layer_def.layer * 4 + layer_def.channel;
            if old == 0 && val > 0 {
                activity.inc(chunk_coord, slot);
            } else if old > 0 && val == 0 {
                activity.dec(chunk_coord, slot);
            }
            staging.data[idx] = val;
        }
    }

    for &layer_idx in active_texture_layers {
        let base = (layer_idx * layer_stride + ty * world_w + tx) * 4;
        let packed_pos = (tx as u32) | ((ty as u32) << 14) | ((layer_idx as u32) << 28);
        let packed_rgba = u32::from(staging.data[base])
            | (u32::from(staging.data[base + 1]) << 8)
            | (u32::from(staging.data[base + 2]) << 16)
            | (u32::from(staging.data[base + 3]) << 24);
        update_buffer.updates.push([packed_pos, packed_rgba]);
    }
}

/// Builds a chunk-sized RGBA8 image by querying particle colors from the ECS.
///
/// Returns `(data, width, height)` where data is RGBA8 sRGB bytes.
/// Row 0 = top of image = max Y in the chunk region.
pub fn extract_chunk_image(
    map: &ParticleMap,
    chunk_index: &ChunkIndex,
    config: &ChunkRenderingConfig,
    chunk_coord: ChunkCoord,
    get_color: impl Fn(Entity) -> Option<Color>,
) -> (Vec<u8>, u32, u32) {
    let cs = chunk_index.chunk_size() as i32;
    let chunk_min = IVec2::new(chunk_coord.x() * cs, chunk_coord.y() * cs);
    let chunk_max = chunk_min + IVec2::splat(cs - 1);

    let bg_bytes = color_to_srgb_bytes(config.background_color);

    let cs_u = cs as usize;
    let mut data = vec![0u8; cs_u * cs_u * 4];

    for y in chunk_min.y..=chunk_max.y {
        for x in chunk_min.x..=chunk_max.x {
            let px = (x - chunk_min.x) as usize;
            let py = (chunk_max.y - y) as usize;
            let pi = (py * cs_u + px) * 4;

            let bytes = if let Ok(Some(entity)) = map.get_copied(IVec2::new(x, y)) {
                get_color(entity).map_or(bg_bytes, color_to_srgb_bytes)
            } else {
                bg_bytes
            };

            data[pi..pi + 4].copy_from_slice(&bytes);
        }
    }

    (data, cs as u32, cs as u32)
}
