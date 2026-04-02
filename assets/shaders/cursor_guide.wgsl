#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct CursorGuideMaterial {
    cursor_world_pos: vec2<f32>,
    grid_size: vec2<f32>,
    line_color: vec4<f32>,
    line_width: f32,
    fade_power: f32,
    fade_end: f32,
}

@group(2) @binding(0) var<uniform> material: CursorGuideMaterial;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let world_pos = mesh.world_position.xy;

    let frac_pos = fract(world_pos);

    let half_line = material.line_width * 0.5;
    let line_x = smoothstep(0.0, half_line, frac_pos.x) * (1.0 - smoothstep(1.0 - half_line, 1.0, frac_pos.x));
    let line_y = smoothstep(0.0, half_line, frac_pos.y) * (1.0 - smoothstep(1.0 - half_line, 1.0, frac_pos.y));

    let grid_line = 1.0 - (line_x * line_y);

    let delta = abs(world_pos - material.cursor_world_pos);
    let dist = max(delta.x, delta.y);

    let fade = pow(1.0 - smoothstep(0.0, material.fade_end, dist), material.fade_power);

    let alpha = grid_line * fade * material.line_color.a;

    return vec4<f32>(material.line_color.rgb, alpha);
}
