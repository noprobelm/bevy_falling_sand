#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var color_texture: texture_2d<f32>;
@group(2) @binding(1) var color_sampler: sampler;
@group(2) @binding(2) var<uniform> uv_offset: vec2<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let wrapped_uv = fract(in.uv + uv_offset);
    return textureSample(color_texture, color_sampler, wrapped_uv);
}
