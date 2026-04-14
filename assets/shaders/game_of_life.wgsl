// Conway's Game of Life compute shader.

@group(0) @binding(0) var gol_in: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(1) var gol_out: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var world_color: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(3) var<uniform> params: GolParams;

struct GolParams {
    width: u32,
    height: u32,
}

fn is_gol_alive(loc: vec2<i32>) -> bool {
    let v = textureLoad(gol_in, loc);
    return v.r > 0.5;
}

fn is_particle_present(loc: vec2<i32>) -> bool {
    let v = textureLoad(world_color, loc);
    return v.a > 0.01;
}

fn is_occupied(loc: vec2<i32>, offset: vec2<i32>) -> i32 {
    let w = i32(params.width);
    let h = i32(params.height);
    let p = vec2<i32>(
        (loc.x + offset.x + w) % w,
        (loc.y + offset.y + h) % h,
    );
    return i32(is_gol_alive(p) || is_particle_present(p));
}

fn count_neighbors(loc: vec2<i32>) -> i32 {
    return is_occupied(loc, vec2(-1, -1)) + is_occupied(loc, vec2(-1, 0)) + is_occupied(loc, vec2(-1, 1)) + is_occupied(loc, vec2(0, -1)) + is_occupied(loc, vec2(0, 1)) + is_occupied(loc, vec2(1, -1)) + is_occupied(loc, vec2(1, 0)) + is_occupied(loc, vec2(1, 1));
}

@compute @workgroup_size(16, 16, 1)
fn update(@builtin(global_invocation_id) gid: vec3<u32>) {
    let loc = vec2<i32>(i32(gid.x), i32(gid.y));
    if gid.x >= params.width || gid.y >= params.height {
        return;
    }

    let n = count_neighbors(loc);
    let currently_alive = is_gol_alive(loc);

    // B3/S23: born with exactly 3 neighbors, survives with 2 or 3
    var alive: bool;
    if n == 3 {
        alive = true;
    } else if n == 2 {
        alive = currently_alive;
    } else {
        alive = false;
    }

    // Golden/amber alive cells on transparent background for overlay compositing
    let color = select(vec4(0.0, 0.0, 0.0, 0.0), vec4(0.965, 0.682, 0.176, 1.0), alive);
    textureStore(gol_out, loc, color);
}

// Spawn entry point: writes cells from a packed-position buffer into the
// current GoL input texture so they participate in the next update step.

@group(0) @binding(0) var<storage, read> spawns: array<u32>;
@group(0) @binding(1) var gol_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> spawn_params: vec4<u32>;

@compute @workgroup_size(64)
fn spawn(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if idx >= spawn_params.x {
        return;
    }
    let packed = spawns[idx];
    let pos = vec2<i32>(i32(packed & 0xFFFFu), i32(packed >> 16u));
    textureStore(gol_tex, pos, vec4(0.965, 0.682, 0.176, 1.0));
}
