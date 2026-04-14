use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    BufferBindingType, BufferInitDescriptor, BufferUsages, CachedComputePipelineId,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache,
    ShaderStages, StorageTextureAccess, TextureFormat, TextureViewDescriptor,
    TextureViewDimension,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;
use bevy::render::{Extract, ExtractSchedule, Render, RenderApp, RenderSystems};

use super::textures::{
    EffectUpdateBuffer, ParticleUpdateBuffer, WorldColorTexture, WorldEffectTexture,
};

#[derive(Resource, Default)]
struct GpuParticleUpdateBuffer {
    data: Vec<[u32; 2]>,
    texture_handle: Option<Handle<Image>>,
}

#[derive(Resource, Default)]
struct GpuEffectUpdateBuffer {
    data: Vec<[u32; 2]>,
    texture_handle: Option<Handle<Image>>,
}

#[derive(Resource)]
struct ChunkUpdatePipeline {
    bind_group_layout_descriptor: BindGroupLayoutDescriptor,
    pipeline_id: CachedComputePipelineId,
}

#[derive(Resource)]
struct EffectUpdatePipeline {
    bind_group_layout_descriptor: BindGroupLayoutDescriptor,
    pipeline_id: CachedComputePipelineId,
}

#[derive(Resource, Clone)]
pub struct ComputePipelineReadyFlag(pub(crate) Arc<AtomicBool>);

fn compute_bind_group_layout_entries(
    view_dimension: TextureViewDimension,
) -> Vec<BindGroupLayoutEntry> {
    vec![
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::StorageTexture {
                access: StorageTextureAccess::WriteOnly,
                format: TextureFormat::Rgba8Unorm,
                view_dimension,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

pub(super) struct GpuChunkRenderingPlugin;

impl Plugin for GpuChunkRenderingPlugin {
    fn build(&self, app: &mut App) {
        bevy::asset::embedded_asset!(app, "shaders/chunk_update.wgsl");
        bevy::asset::embedded_asset!(app, "shaders/effect_update.wgsl");

        let flag = ComputePipelineReadyFlag(Arc::new(AtomicBool::new(false)));
        app.insert_resource(flag.clone());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .insert_resource(flag)
            .init_resource::<GpuParticleUpdateBuffer>()
            .init_resource::<GpuEffectUpdateBuffer>()
            .add_systems(
                ExtractSchedule,
                (extract_particle_updates, extract_effect_updates),
            )
            .add_systems(
                Render,
                (dispatch_chunk_compute, dispatch_effect_compute)
                    .in_set(RenderSystems::Queue)
                    .after(RenderSystems::PrepareAssets),
            );
    }

    fn finish(&self, app: &mut App) {
        let asset_server = app.world().resource::<AssetServer>();
        let color_shader: Handle<Shader> = asset_server
            .load("embedded://bevy_falling_sand/render/pipeline/shaders/chunk_update.wgsl");
        let effect_shader: Handle<Shader> = asset_server
            .load("embedded://bevy_falling_sand/render/pipeline/shaders/effect_update.wgsl");

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        let color_entries = compute_bind_group_layout_entries(TextureViewDimension::D2);
        let effect_entries = compute_bind_group_layout_entries(TextureViewDimension::D2Array);

        let color_bgl = BindGroupLayoutDescriptor::new("chunk_update_bgl", &color_entries);
        let effect_bgl = BindGroupLayoutDescriptor::new("effect_update_bgl", &effect_entries);

        let (color_pipeline_id, effect_pipeline_id) = {
            let pipeline_cache = render_app.world().resource::<PipelineCache>();

            let color_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("chunk_update_pipeline".into()),
                layout: vec![color_bgl.clone()],
                shader: color_shader,
                shader_defs: vec![],
                entry_point: Some("main".into()),
                push_constant_ranges: vec![],
                zero_initialize_workgroup_memory: true,
            });

            let effect_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("effect_update_pipeline".into()),
                layout: vec![effect_bgl.clone()],
                shader: effect_shader,
                shader_defs: vec![],
                entry_point: Some("main".into()),
                push_constant_ranges: vec![],
                zero_initialize_workgroup_memory: true,
            });

            (color_id, effect_id)
        };

        render_app.insert_resource(ChunkUpdatePipeline {
            bind_group_layout_descriptor: color_bgl,
            pipeline_id: color_pipeline_id,
        });
        render_app.insert_resource(EffectUpdatePipeline {
            bind_group_layout_descriptor: effect_bgl,
            pipeline_id: effect_pipeline_id,
        });
    }
}

#[allow(clippy::needless_pass_by_value)]
fn extract_particle_updates(
    mut gpu_buffer: ResMut<GpuParticleUpdateBuffer>,
    update_buffer: Extract<Res<ParticleUpdateBuffer>>,
    color_tex: Extract<Option<Res<WorldColorTexture>>>,
) {
    gpu_buffer.data.clone_from(&update_buffer.updates);
    gpu_buffer.texture_handle = color_tex.as_ref().map(|t| t.0.clone());
}

#[allow(clippy::needless_pass_by_value)]
fn extract_effect_updates(
    mut gpu_buffer: ResMut<GpuEffectUpdateBuffer>,
    update_buffer: Extract<Res<EffectUpdateBuffer>>,
    effect_tex: Extract<Option<Res<WorldEffectTexture>>>,
) {
    gpu_buffer.data.clone_from(&update_buffer.updates);
    gpu_buffer.texture_handle = effect_tex.as_ref().map(|t| t.0.clone());
}

#[allow(clippy::too_many_arguments)]
fn dispatch_compute(
    label: &str,
    data: &[[u32; 2]],
    texture_handle: &Handle<Image>,
    pipeline: &bevy::render::render_resource::ComputePipeline,
    bgl_descriptor: &BindGroupLayoutDescriptor,
    view_dimension: TextureViewDimension,
    pipeline_cache: &PipelineCache,
    gpu_images: &RenderAssets<GpuImage>,
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
) {
    const MAX_PER_DISPATCH: usize = 65535 * 64;

    let Some(gpu_image) = gpu_images.get(texture_handle) else {
        return;
    };

    let storage_view = gpu_image.texture.create_view(&TextureViewDescriptor {
        format: Some(TextureFormat::Rgba8Unorm),
        dimension: Some(view_dimension),
        ..default()
    });

    let bind_group_layout = pipeline_cache.get_bind_group_layout(bgl_descriptor);

    for chunk in data.chunks(MAX_PER_DISPATCH) {
        let mut data_bytes = Vec::with_capacity(chunk.len() * 8);
        for &[pos, color] in chunk {
            data_bytes.extend_from_slice(&pos.to_le_bytes());
            data_bytes.extend_from_slice(&color.to_le_bytes());
        }

        let storage_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some(label),
            contents: &data_bytes,
            usage: BufferUsages::STORAGE,
        });

        let update_count = chunk.len() as u32;
        let mut param_bytes = [0u8; 16];
        param_bytes[0..4].copy_from_slice(&update_count.to_le_bytes());

        let uniform_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some(label),
            contents: &param_bytes,
            usage: BufferUsages::UNIFORM,
        });

        let bind_group = render_device.create_bind_group(
            label,
            &bind_group_layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: storage_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&storage_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        );

        let mut encoder =
            render_device.create_command_encoder(&CommandEncoderDescriptor { label: Some(label) });

        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some(label),
                ..default()
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            let workgroups = (chunk.len() as u32).div_ceil(64);
            pass.dispatch_workgroups(workgroups, 1, 1);
        }

        render_queue.submit(std::iter::once(encoder.finish()));
    }
}

#[allow(clippy::needless_pass_by_value)]
fn dispatch_chunk_compute(
    pipeline_res: Res<ChunkUpdatePipeline>,
    pipeline_cache: Res<PipelineCache>,
    gpu_buffer: Res<GpuParticleUpdateBuffer>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    ready_flag: Res<ComputePipelineReadyFlag>,
) {
    let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_res.pipeline_id) else {
        return;
    };

    ready_flag.0.store(true, Ordering::Relaxed);

    if gpu_buffer.data.is_empty() {
        return;
    }
    let Some(handle) = &gpu_buffer.texture_handle else {
        return;
    };

    dispatch_compute(
        "chunk_update",
        &gpu_buffer.data,
        handle,
        pipeline,
        &pipeline_res.bind_group_layout_descriptor,
        TextureViewDimension::D2,
        &pipeline_cache,
        &gpu_images,
        &render_device,
        &render_queue,
    );
}

#[allow(clippy::needless_pass_by_value)]
fn dispatch_effect_compute(
    pipeline_res: Res<EffectUpdatePipeline>,
    pipeline_cache: Res<PipelineCache>,
    gpu_buffer: Res<GpuEffectUpdateBuffer>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    if gpu_buffer.data.is_empty() {
        return;
    }
    let Some(handle) = &gpu_buffer.texture_handle else {
        return;
    };
    let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_res.pipeline_id) else {
        return;
    };

    dispatch_compute(
        "effect_update",
        &gpu_buffer.data,
        handle,
        pipeline,
        &pipeline_res.bind_group_layout_descriptor,
        TextureViewDimension::D2Array,
        &pipeline_cache,
        &gpu_images,
        &render_device,
        &render_queue,
    );
}
