#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals
#import bevy_falling_sand::effects::has_effect_in_radius
#import bevy_falling_sand::effects::quad_uv_to_world_texel

struct GlowSettings {
    intensity: f32,
    radius: f32,
}

@group(2) @binding(0) var<uniform> settings: GlowSettings;
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

    let color = textureLoad(chunk_texture, texel, 0);
    if color.a < 0.01 {
        discard;
    }

    let effects = textureLoad(effect_data_texture, texel, 0, 0);
    let has_glow = effects.b > 0.5;
    let radius_i = i32(settings.radius);

    if !has_glow && !has_effect_in_radius(effect_data_texture, 0, 2, texel, tex_size, radius_i, 4) {
        discard;
    }

    let time = globals.time;

    if has_glow {
        let edge_r = 12;
        var empty_count = 0.0;
        var total_count = 0.0;
        for (var dy = -edge_r; dy <= edge_r; dy++) {
            for (var dx = -edge_r; dx <= edge_r; dx++) {
                if dx == 0 && dy == 0 { continue; }
                let n = texel + vec2<i32>(dx, dy);
                total_count += 1.0;
                if n.x < 0 || n.y < 0 || n.x >= tex_size.x || n.y >= tex_size.y { continue; }
                if textureLoad(effect_data_texture, n, 0, 0).b <= 0.0 {
                    empty_count += 1.0;
                }
            }
        }
        let raw_edge = empty_count / total_count;
        let edge_factor = clamp((raw_edge - 0.05) / 0.95, 0.0, 1.0);
        let pulse = sin(time * 2.0) * 0.5 + 0.5;
        let brightness = 1.0 + edge_factor * (1.0 + pulse * 1.5) * settings.intensity;
        let hot_color = mix(color.rgb, vec3<f32>(1.0, 0.6, 0.2), edge_factor * 0.12 * settings.intensity);
        return vec4<f32>(hot_color * brightness, color.a);
    }

    var glow_accum = vec3<f32>(0.0);
    var total_weight = 0.0;
    var min_dist = settings.radius + 1.0;

    for (var dy = -radius_i; dy <= radius_i; dy++) {
        for (var dx = -radius_i; dx <= radius_i; dx++) {
            let neighbor = texel + vec2<i32>(dx, dy);
            if neighbor.x < 0 || neighbor.y < 0 || neighbor.x >= tex_size.x || neighbor.y >= tex_size.y {
                continue;
            }
            if textureLoad(effect_data_texture, neighbor, 0, 0).b <= 0.0 { continue; }
            let dist = length(vec2<f32>(f32(dx), f32(dy)));
            if dist > settings.radius { continue; }

            min_dist = min(min_dist, dist);
            let weight = 1.0 - (dist / settings.radius);
            let neighbor_color = textureLoad(chunk_texture, neighbor, 0);
            glow_accum += neighbor_color.rgb * weight;
            total_weight += weight;
        }
    }

    if total_weight <= 0.0 {
        discard;
    }

    let gc = glow_accum / total_weight;
    let falloff = 1.0 - (min_dist / settings.radius);
    let ga = falloff * sqrt(falloff) * 0.8 * settings.intensity;

    if ga < 0.01 {
        discard;
    }

    return vec4<f32>(gc, ga);
}
