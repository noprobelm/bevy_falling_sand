#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

struct EffectSettings {
    liquid_intensity: f32,
    liquid_speed: f32,
    gas_intensity: f32,
    gas_speed: f32,
    glow_intensity: f32,
    glow_radius: f32,
    burn_intensity: f32,
    padding_size: f32,
}

@group(2) @binding(0) var<uniform> settings: EffectSettings;
@group(2) @binding(1) var chunk_texture: texture_2d<f32>;
@group(2) @binding(2) var chunk_sampler: sampler;
@group(2) @binding(3) var effect_data_texture: texture_2d_array<f32>;
@group(2) @binding(4) var effect_data_sampler: sampler;
@group(2) @binding(5) var<uniform> uv_offset: vec2<f32>;

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn hash2(p: vec2<f32>, t: f32) -> f32 {
    return fract(sin(dot(p + vec2<f32>(t * 1.37, t * 2.13), vec2<f32>(269.5, 183.3))) * 63758.1232);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let wrapped_uv = fract(mesh.uv + uv_offset);
    let tex_size = vec2<i32>(textureDimensions(chunk_texture, 0));
    let texel = clamp(
        vec2<i32>(floor(wrapped_uv * vec2<f32>(tex_size))),
        vec2<i32>(0),
        tex_size - vec2<i32>(1),
    );

    let pad = i32(settings.padding_size);
    let in_center = texel.x >= pad && texel.y >= pad && texel.x < tex_size.x - pad && texel.y < tex_size.y - pad;

    let effects = textureLoad(effect_data_texture, texel, 0, 0);
    let has_liquid = effects.r > 0.5;
    let has_gas = effects.g > 0.5;
    let has_glow = effects.b > 0.5;
    let has_burning = effects.a > 0.5;
    let any_effect = has_liquid || has_gas || has_glow || has_burning;

    let color = textureLoad(chunk_texture, texel, 0);
    let time = globals.time;
    let pos = vec2<f32>(texel);

    // ── Interior: pixel has at least one active effect ──
    if any_effect && in_center {
        var result = color;

        // R channel (liquid): brightness pulse via sine wave
        if has_liquid {
            let t = time * settings.liquid_speed;
            let phase = fract(sin(dot(pos, vec2<f32>(12.9898, 78.233))) * 43758.5453);
            let pulse = sin(t + phase * 6.2832) * 0.5 + 0.5;
            let brightness = 1.0 + pulse * 0.8 * settings.liquid_intensity;
            result = vec4<f32>(result.rgb * brightness, result.a);
        }

        // G channel (gas): edge-aware density + gradient-driven wisps
        if has_gas {
            let t = time * settings.gas_speed;

            // Step 1: Local density via 7x7 kernel (radius 3)
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

            // Step 2: Density gradient from offset samples (±4)
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

            // Step 3: Gradient-distorted wisps
            let flow_offset = gradient * sin(t * 0.3) * 3.0;
            let distorted_pos = pos + flow_offset;
            let wisp1 = sin(distorted_pos.x * 0.4 + distorted_pos.y * 0.3 + t * 0.5) * 0.5 + 0.5;
            let wisp2 = sin(distorted_pos.x * 0.9 - distorted_pos.y * 0.7 + t * 0.8) * 0.5 + 0.5;
            let wisp = wisp1 * 0.6 + wisp2 * 0.4;

            // Step 4: Composite — interior brightness + edge fade
            let brightness = mix(0.85, 1.3, wisp) * settings.gas_intensity;
            let alpha_mod = mix(1.0, 0.2, edge_factor) * (0.8 + wisp * 0.2);

            result = vec4<f32>(result.rgb * brightness, result.a * alpha_mod);
        }

        // B channel (glow): edge detection + halo with weighted blur
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
            let brightness = 1.0 + edge_factor * (1.0 + pulse * 1.5) * settings.glow_intensity;
            let hot_color = mix(result.rgb, vec3<f32>(1.0, 0.6, 0.2), edge_factor * 0.12 * settings.glow_intensity);
            result = vec4<f32>(hot_color * brightness, result.a);
        }

        // A channel (burning): glow intensifies with nearby fire density
        if has_burning {
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
            // density 0..1 based on how many of the 48 neighbors are fire
            let density = clamp(fire_neighbors / 48.0, 0.0, 1.0);
            let fire_tint = mix(result.rgb, vec3<f32>(1.0, 0.5, 0.1), 0.85);
            let brightness = 1.0 + density * 3.0 * settings.burn_intensity;
            result = vec4<f32>(fire_tint * brightness, result.a);
        }

        return result;
    }

    // ── Halo zone: pixel has no active effect but may be near glow/burning pixels ──
    if !in_center {
        discard;
    }

    var halo_color = vec3<f32>(0.0);
    var halo_alpha = 0.0;

    // Glow halo: weighted color blur from nearby glowing pixels
    if color.a > 0.0 {
        let r = i32(settings.glow_radius);
        var glow_accum = vec3<f32>(0.0);
        var total_weight = 0.0;
        var min_dist = settings.glow_radius + 1.0;

        for (var dy = -r; dy <= r; dy++) {
            for (var dx = -r; dx <= r; dx++) {
                let neighbor = texel + vec2<i32>(dx, dy);
                if neighbor.x < 0 || neighbor.y < 0 || neighbor.x >= tex_size.x || neighbor.y >= tex_size.y {
                    continue;
                }
                if textureLoad(effect_data_texture, neighbor, 0, 0).b <= 0.0 { continue; }
                let dist = length(vec2<f32>(f32(dx), f32(dy)));
                if dist > settings.glow_radius { continue; }

                min_dist = min(min_dist, dist);
                let weight = 1.0 - (dist / settings.glow_radius);
                let neighbor_color = textureLoad(chunk_texture, neighbor, 0);
                glow_accum += neighbor_color.rgb * weight;
                total_weight += weight;
            }
        }

        if total_weight > 0.0 {
            let gc = glow_accum / total_weight;
            let falloff = 1.0 - (min_dist / settings.glow_radius);
            let ga = falloff * sqrt(falloff) * 0.8 * settings.glow_intensity;
            halo_color = gc;
            halo_alpha = ga;
        }
    }

    // Burning glow: warm light around fire particles, stronger with more nearby fire
        {
        let burn_radius = 6.0;
        let r = i32(burn_radius);
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

        if fire_weight > 0.0 {
            let fc = fire_color_accum / fire_weight;
            // fire_weight accumulates: more nearby fire = stronger glow
            let intensity = clamp(fire_weight * 0.1, 0.0, 1.0) * settings.burn_intensity;
            let ba = intensity;

            let a_out = ba + halo_alpha * (1.0 - ba);
            if a_out > 0.001 {
                halo_color = (fc * ba + halo_color * halo_alpha * (1.0 - ba)) / a_out;
            }
            halo_alpha = a_out;
        }
    }

    if halo_alpha < 0.01 {
        discard;
    }

    return vec4<f32>(halo_color, halo_alpha);
}
