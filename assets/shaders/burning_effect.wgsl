#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals
#import bevy_falling_sand::effects::has_effect_in_radius
#import bevy_falling_sand::effects::quad_uv_to_world_texel

struct BurningSettings {
    intensity: f32,
}

@group(2) @binding(0) var<uniform> settings: BurningSettings;
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
    let has_burning = effects.a > 0.5;
    let halo_radius = 6;

    if !has_burning && !has_effect_in_radius(effect_data_texture, 0, 3, texel, tex_size, halo_radius, 2) {
        discard;
    }

    let color = textureLoad(chunk_texture, texel, 0);

    if has_burning && color.a > 0.01 {
        let burn_r = 3;
        var fire_neighbors = 0.0;
        for (var dy = -burn_r; dy <= burn_r; dy++) {
            for (var dx = -burn_r; dx <= burn_r; dx++) {
                if dx == 0 && dy == 0 { continue; }
                let n = texel + vec2<i32>(dx, dy);
                if n.x < 0 || n.y < 0 || n.x >= tex_size.x || n.y >= tex_size.y { continue; }
                if textureLoad(effect_data_texture, n, 0, 0).a > 0.5 {
                    fire_neighbors += 1.0;
                }
            }
        }
        let density = clamp(fire_neighbors / 48.0, 0.0, 1.0);
        let fire_tint = mix(color.rgb, vec3<f32>(1.0, 0.5, 0.1), 0.85);
        let brightness = 1.0 + density * 3.0 * settings.intensity;
        return vec4<f32>(fire_tint * brightness, color.a);
    }

    let burn_radius = f32(halo_radius);
    let r = halo_radius;
    var fire_weight = 0.0;
    var fire_color_accum = vec3<f32>(0.0);

    for (var dy = -r; dy <= r; dy++) {
        for (var dx = -r; dx <= r; dx++) {
            let neighbor = texel + vec2<i32>(dx, dy);
            if neighbor.x < 0 || neighbor.y < 0 || neighbor.x >= tex_size.x || neighbor.y >= tex_size.y {
                continue;
            }
            if textureLoad(effect_data_texture, neighbor, 0, 0).a <= 0.0 { continue; }
            let dist = length(vec2<f32>(f32(dx), f32(dy)));
            if dist > burn_radius { continue; }
            let w = 1.0 - (dist / burn_radius);
            let neighbor_color = textureLoad(chunk_texture, neighbor, 0);
            let warm_color = mix(neighbor_color.rgb, vec3<f32>(1.0, 0.5, 0.1), 0.85);
            fire_color_accum += warm_color * w;
            fire_weight += w;
        }
    }

    if fire_weight <= 0.0 {
        discard;
    }

    let fc = fire_color_accum / fire_weight;
    let intensity = clamp(fire_weight * 0.1, 0.0, 1.0) * settings.intensity;

    if intensity < 0.01 {
        discard;
    }

    return vec4<f32>(fc, intensity);
}
