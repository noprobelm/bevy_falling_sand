#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals
#import bevy_falling_sand::effects::quad_uv_to_world_texel

struct GasSettings {
    intensity: f32,
    speed: f32,
}

@group(2) @binding(0) var<uniform> settings: GasSettings;
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
    if effects.g < 0.5 {
        discard;
    }

    let color = textureLoad(chunk_texture, texel, 0);
    if color.a < 0.01 {
        discard;
    }

    let pos = vec2<f32>(texel);
    let t = globals.time * settings.speed;

    let gas_r = 3;
    var gas_count = 0.0;
    var gas_total = 0.0;
    for (var dy = -gas_r; dy <= gas_r; dy++) {
        for (var dx = -gas_r; dx <= gas_r; dx++) {
            let n = texel + vec2<i32>(dx, dy);
            gas_total += 1.0;
            if n.x >= 0 && n.y >= 0 && n.x < tex_size.x && n.y < tex_size.y {
                if textureLoad(effect_data_texture, n, 0, 0).g > 0.5 {
                    gas_count += 1.0;
                }
            }
        }
    }
    let density = gas_count / gas_total;
    let edge_factor = 1.0 - density;

    let g_off = 4;
    let s_right = texel + vec2<i32>(g_off, 0);
    let s_left = texel + vec2<i32>(-g_off, 0);
    let s_up = texel + vec2<i32>(0, -g_off);
    let s_down = texel + vec2<i32>(0, g_off);
    let gas_right = select(0.0, 1.0, s_right.x < tex_size.x && textureLoad(effect_data_texture, clamp(s_right, vec2<i32>(0), tex_size - vec2<i32>(1)), 0, 0).g > 0.5);
    let gas_left = select(0.0, 1.0, s_left.x >= 0 && textureLoad(effect_data_texture, clamp(s_left, vec2<i32>(0), tex_size - vec2<i32>(1)), 0, 0).g > 0.5);
    let gas_up = select(0.0, 1.0, s_up.y >= 0 && textureLoad(effect_data_texture, clamp(s_up, vec2<i32>(0), tex_size - vec2<i32>(1)), 0, 0).g > 0.5);
    let gas_down = select(0.0, 1.0, s_down.y < tex_size.y && textureLoad(effect_data_texture, clamp(s_down, vec2<i32>(0), tex_size - vec2<i32>(1)), 0, 0).g > 0.5);
    let gradient = vec2<f32>(gas_right - gas_left, gas_up - gas_down);

    let flow_offset = gradient * sin(t * 0.3) * 3.0;
    let distorted_pos = pos + flow_offset;
    let wisp1 = sin(distorted_pos.x * 0.4 + distorted_pos.y * 0.3 + t * 0.5) * 0.5 + 0.5;
    let wisp2 = sin(distorted_pos.x * 0.9 - distorted_pos.y * 0.7 + t * 0.8) * 0.5 + 0.5;
    let wisp = wisp1 * 0.6 + wisp2 * 0.4;

    let brightness = mix(0.85, 1.3, wisp) * settings.intensity;
    let alpha_mod = mix(1.0, 0.2, edge_factor) * (0.8 + wisp * 0.2);

    return vec4<f32>(color.rgb * brightness, color.a * alpha_mod);
}
