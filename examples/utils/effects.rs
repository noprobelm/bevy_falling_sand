use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
};
use bevy_falling_sand::prelude::*;
use bevy_shader::ShaderRef;
use bevy_sprite_render::{AlphaMode2d, Material2d};
use serde::{Deserialize, Serialize};

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<LiquidEffect>()
            .register_type::<GasEffect>()
            .register_type::<GlowEffect>()
            .register_type::<BurnEffect>()
            .register_particle_sync_component::<LiquidEffect>()
            .register_particle_sync_component::<GasEffect>()
            .register_particle_sync_component::<GlowEffect>()
            .register_particle_sync_component::<BurnEffect>()
            .add_chunk_effect_material::<LiquidEffectMaterial>()
            .add_chunk_effect_material::<GasEffectMaterial>()
            .add_chunk_effect_material::<GlowEffectMaterial>()
            .add_chunk_effect_material::<BurningEffectMaterial>()
            .add_chunk_effect_layer::<LiquidEffectLayer>()
            .add_chunk_effect_layer::<GasEffectLayer>()
            .add_chunk_effect_layer::<GlowEffectLayer>()
            .add_chunk_effect_layer::<BurningEffectLayer>()
            .add_systems(
                PostUpdate,
                (add_burn_overlay, remove_burn_overlay)
                    .after(ParticleSystems::Simulation)
                    .before(RenderingSystems::ChunkEffectLayerUpdate),
            );
    }
}

#[derive(
    Component,
    Clone,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct LiquidEffect;

#[derive(
    Component,
    Clone,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct GasEffect;

#[derive(
    Component,
    Clone,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct GlowEffect;

#[derive(
    Component,
    Clone,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct BurnEffect;

#[derive(Component, Clone, Default, Debug)]
pub struct BurningOverlay;

pub struct LiquidEffectLayer;
impl ChunkEffectLayer for LiquidEffectLayer {
    type Source = LiquidEffect;
    fn channel() -> usize {
        0
    }
}

pub struct GasEffectLayer;
impl ChunkEffectLayer for GasEffectLayer {
    type Source = GasEffect;
    fn channel() -> usize {
        1
    }
}

pub struct GlowEffectLayer;
impl ChunkEffectLayer for GlowEffectLayer {
    type Source = GlowEffect;
    fn channel() -> usize {
        2
    }
}

pub struct BurningEffectLayer;
impl ChunkEffectLayer for BurningEffectLayer {
    type Source = BurningOverlay;
    fn channel() -> usize {
        3
    }
}

#[derive(ShaderType, Debug, Clone)]
pub struct LiquidSettings {
    pub intensity: f32,
    pub speed: f32,
}

#[derive(ShaderType, Debug, Clone)]
pub struct GasSettings {
    pub intensity: f32,
    pub speed: f32,
}

#[derive(ShaderType, Debug, Clone)]
pub struct GlowSettings {
    pub intensity: f32,
    pub radius: f32,
}

#[derive(ShaderType, Debug, Clone)]
pub struct BurningSettings {
    pub intensity: f32,
    pub _padding: f32,
}

// ---------------------------------------------------------------------------
// Materials
// ---------------------------------------------------------------------------

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct LiquidEffectMaterial {
    #[uniform(0)]
    pub settings: LiquidSettings,
    #[texture(1)]
    #[sampler(2)]
    pub chunk_texture: Handle<Image>,
    #[texture(3, dimension = "2d_array")]
    #[sampler(4)]
    pub effect_data: Handle<Image>,
    #[uniform(5)]
    pub uv_offset: Vec2,
}

impl Material2d for LiquidEffectMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/liquid_effect.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

impl ChunkEffectMaterial for LiquidEffectMaterial {
    fn new(chunk_texture: Handle<Image>, effect_data: Handle<Image>) -> Self {
        Self {
            settings: LiquidSettings {
                intensity: 1.0,
                speed: 3.0,
            },
            chunk_texture,
            effect_data,
            uv_offset: Vec2::ZERO,
        }
    }
    fn set_uv_offset(&mut self, offset: Vec2) {
        self.uv_offset = offset;
    }
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct GasEffectMaterial {
    #[uniform(0)]
    pub settings: GasSettings,
    #[texture(1)]
    #[sampler(2)]
    pub chunk_texture: Handle<Image>,
    #[texture(3, dimension = "2d_array")]
    #[sampler(4)]
    pub effect_data: Handle<Image>,
    #[uniform(5)]
    pub uv_offset: Vec2,
}

impl Material2d for GasEffectMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/gas_effect.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

impl ChunkEffectMaterial for GasEffectMaterial {
    fn new(chunk_texture: Handle<Image>, effect_data: Handle<Image>) -> Self {
        Self {
            settings: GasSettings {
                intensity: 1.0,
                speed: 3.0,
            },
            chunk_texture,
            effect_data,
            uv_offset: Vec2::ZERO,
        }
    }
    fn set_uv_offset(&mut self, offset: Vec2) {
        self.uv_offset = offset;
    }
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct GlowEffectMaterial {
    #[uniform(0)]
    pub settings: GlowSettings,
    #[texture(1)]
    #[sampler(2)]
    pub chunk_texture: Handle<Image>,
    #[texture(3, dimension = "2d_array")]
    #[sampler(4)]
    pub effect_data: Handle<Image>,
    #[uniform(5)]
    pub uv_offset: Vec2,
}

impl Material2d for GlowEffectMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/glow_effect.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

impl ChunkEffectMaterial for GlowEffectMaterial {
    fn new(chunk_texture: Handle<Image>, effect_data: Handle<Image>) -> Self {
        Self {
            settings: GlowSettings {
                intensity: 1.0,
                radius: 12.0,
            },
            chunk_texture,
            effect_data,
            uv_offset: Vec2::ZERO,
        }
    }
    fn set_uv_offset(&mut self, offset: Vec2) {
        self.uv_offset = offset;
    }
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct BurningEffectMaterial {
    #[uniform(0)]
    pub settings: BurningSettings,
    #[texture(1)]
    #[sampler(2)]
    pub chunk_texture: Handle<Image>,
    #[texture(3, dimension = "2d_array")]
    #[sampler(4)]
    pub effect_data: Handle<Image>,
    #[uniform(5)]
    pub uv_offset: Vec2,
}

impl Material2d for BurningEffectMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/burning_effect.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

impl ChunkEffectMaterial for BurningEffectMaterial {
    fn new(chunk_texture: Handle<Image>, effect_data: Handle<Image>) -> Self {
        Self {
            settings: BurningSettings {
                intensity: 1.0,
                _padding: 0.0,
            },
            chunk_texture,
            effect_data,
            uv_offset: Vec2::ZERO,
        }
    }
    fn set_uv_offset(&mut self, offset: Vec2) {
        self.uv_offset = offset;
    }
}

fn mark_chunk_dirty(
    pos: IVec2,
    chunk_index: &ChunkIndex,
    chunk_query: &mut Query<&mut ChunkDirtyState>,
) {
    let coord = chunk_index.world_to_chunk_coord(pos);
    let Some(chunk_entity) = chunk_index.get(coord) else {
        return;
    };
    let Ok(mut dirty_state) = chunk_query.get_mut(chunk_entity) else {
        return;
    };
    dirty_state.mark_dirty(pos);
    match &mut dirty_state.current {
        Some(rect) => *rect = rect.union_point(pos),
        None => dirty_state.current = Some(IRect::from_center_size(pos, IVec2::ONE)),
    }
}

fn add_burn_overlay(
    mut commands: Commands,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    added_burning: Query<(Entity, &GridPosition), (Added<Burning>, Without<BurningOverlay>)>,
    added_burn_effect: Query<(Entity, &GridPosition), (Added<BurnEffect>, Without<BurningOverlay>)>,
) {
    for (entity, pos) in added_burning.iter().chain(added_burn_effect.iter()) {
        commands.entity(entity).insert(BurningOverlay);
        mark_chunk_dirty(pos.0, &chunk_index, &mut chunk_query);
    }
}

fn remove_burn_overlay(
    mut commands: Commands,
    chunk_index: Res<ChunkIndex>,
    mut chunk_query: Query<&mut ChunkDirtyState>,
    has_overlay: Query<
        (Entity, &GridPosition),
        (With<BurningOverlay>, Without<Burning>, Without<BurnEffect>),
    >,
) {
    for (entity, pos) in has_overlay.iter() {
        commands.entity(entity).remove::<BurningOverlay>();
        mark_chunk_dirty(pos.0, &chunk_index, &mut chunk_query);
    }
}
