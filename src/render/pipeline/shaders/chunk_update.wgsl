@group(0) @binding(0) var<storage, read> updates: array<vec2<u32>>;
@group(0) @binding(1) var atlas: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> params: vec4<u32>;

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if idx >= params.x {
        return;
    }

    let data = updates[idx];
    let pos = vec2<i32>(i32(data.x & 0xFFFFu), i32(data.x >> 16u));
    let r = f32(data.y & 0xFFu) / 255.0;
    let g = f32((data.y >> 8u) & 0xFFu) / 255.0;
    let b = f32((data.y >> 16u) & 0xFFu) / 255.0;
    let a = f32((data.y >> 24u) & 0xFFu) / 255.0;

    textureStore(atlas, pos, vec4<f32>(srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b), a));
}
