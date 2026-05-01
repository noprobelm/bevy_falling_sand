#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_falling_sand::effects::quad_uv_to_world_texel

@group(2) @binding(0) var intermediate: texture_2d<f32>;
@group(2) @binding(1) var intermediate_sampler: sampler;
@group(2) @binding(2) var<uniform> uv_offset: vec2<f32>;
@group(2) @binding(3) var<uniform> quad_world_rect: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let tex_size = vec2<i32>(textureDimensions(intermediate, 0));
    let texel = quad_uv_to_world_texel(mesh.uv, quad_world_rect, tex_size, uv_offset);
    return textureLoad(intermediate, texel, 0);
}
