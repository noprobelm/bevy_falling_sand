#define_import_path bevy_falling_sand::effects

// Maps a fragment's local quad UV to a texel in the world effect / color texture.
//
// `quad_world_rect` is `(min_x, min_y, size_x, size_y)` in world coordinates, relative
// to the current map origin (the framework subtracts the map origin before passing it).
// `uv_offset` is the same toroidal-wrap offset used by the world-sized color sprite.
fn quad_uv_to_world_texel(
    uv: vec2<f32>,
    quad_world_rect: vec4<f32>,
    tex_size: vec2<i32>,
    uv_offset: vec2<f32>,
) -> vec2<i32> {
    let tex_size_f = vec2<f32>(tex_size);
    let world_u = (quad_world_rect.x + uv.x * quad_world_rect.z) / tex_size_f.x;
    let world_v = 1.0 - (quad_world_rect.y + (1.0 - uv.y) * quad_world_rect.w) / tex_size_f.y;
    let wrapped = fract(vec2<f32>(world_u, world_v) + uv_offset);
    let texel_f = floor(wrapped * tex_size_f);
    return clamp(vec2<i32>(texel_f), vec2<i32>(0), tex_size - vec2<i32>(1));
}
