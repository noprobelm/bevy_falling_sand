#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var chunk_texture: texture_2d<f32>;
@group(2) @binding(1) var chunk_sampler: sampler;
@group(2) @binding(2) var effect_data_texture: texture_2d_array<f32>;
@group(2) @binding(3) var effect_data_sampler: sampler;
@group(2) @binding(4) var<uniform> uv_offset: vec2<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let wrapped_uv = fract(mesh.uv + uv_offset);
    let tex_size = vec2<i32>(textureDimensions(chunk_texture, 0));
    let texel = clamp(
        vec2<i32>(floor(wrapped_uv * vec2<f32>(tex_size))),
        vec2<i32>(0),
        tex_size - vec2<i32>(1),
    );

    let effects = textureLoad(effect_data_texture, texel, 0, 0);
    let any_effect = effects.r + effects.g + effects.b + effects.a;
    if any_effect < 0.01 {
        discard;
    }

    let color = textureLoad(chunk_texture, texel, 0);
    if color.a < 0.01 {
        discard;
    }

    return color;
}
