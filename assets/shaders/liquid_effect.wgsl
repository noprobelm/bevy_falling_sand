#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals
#import bevy_falling_sand::effects::quad_uv_to_world_texel

struct LiquidSettings {
    intensity: f32,
    speed: f32,
}

@group(2) @binding(0) var<uniform> settings: LiquidSettings;
@group(2) @binding(1) var chunk_texture: texture_2d<f32>;
@group(2) @binding(2) var chunk_sampler: sampler;
@group(2) @binding(3) var effect_data_texture: texture_2d_array<f32>;
@group(2) @binding(4) var effect_data_sampler: sampler;
@group(2) @binding(5) var<uniform> uv_offset: vec2<f32>;
@group(2) @binding(6) var<uniform> quad_world_rect: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let tex_size = vec2<i32>(textureDimensions(chunk_texture, 0));
    let texel = quad_uv_to_world_texel(mesh.uv, quad_world_rect, tex_size, uv_offset);

    let effects = textureLoad(effect_data_texture, texel, 0, 0);
    if effects.r < 0.5 {
        discard;
    }

    let color = textureLoad(chunk_texture, texel, 0);
    if color.a < 0.01 {
        discard;
    }

    let pos = vec2<f32>(texel);
    let t = globals.time * settings.speed;
    let phase = fract(sin(dot(pos, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let pulse = sin(t + phase * 6.2832) * 0.5 + 0.5;
    let brightness = 1.0 + pulse * 0.8 * settings.intensity;

    return vec4<f32>(color.rgb * brightness, color.a);
}
