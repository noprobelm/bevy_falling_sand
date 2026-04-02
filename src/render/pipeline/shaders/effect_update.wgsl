@group(0) @binding(0) var<storage, read> updates: array<vec2<u32>>;
@group(0) @binding(1) var atlas: texture_storage_2d_array<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> params: vec4<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if idx >= params.x {
        return;
    }

    let data = updates[idx];
    let tx = i32(data.x & 0x3FFFu);
    let ty = i32((data.x >> 14u) & 0x3FFFu);
    let layer = i32((data.x >> 28u) & 0xFu);
    let pos = vec2<i32>(tx, ty);
    let r = f32(data.y & 0xFFu) / 255.0;
    let g = f32((data.y >> 8u) & 0xFFu) / 255.0;
    let b = f32((data.y >> 16u) & 0xFFu) / 255.0;
    let a = f32((data.y >> 24u) & 0xFFu) / 255.0;

    textureStore(atlas, pos, layer, vec4<f32>(r, g, b, a));
}
