// t3ssel8r's pixel-art sampling https://www.youtube.com/watch?v=d6tp43wZqps

@if(DISTANCE_FOG) import bevy_pbr::render::{mesh_view_bindings::{view, fog}, mesh_view_types::FOG_MODE_LINEAR};

const FOG_STEPS: f32 = 16.0;

fn dither_rank(cell: vec2<i32>) -> f32 {
    var bayer = array<u32, 16>(0u, 8u, 2u, 10u, 12u, 4u, 14u, 6u, 3u, 11u, 1u, 9u, 15u, 7u, 13u, 5u);
    let m = vec2<u32>(cell & vec2<i32>(3));
    return (f32(bayer[m.y * 4u + m.x]) + 0.5) / 16.0;
}

@if(!DISTANCE_FOG)
fn stepped_fog_position(at: vec3<f32>, texel: vec2<i32>) -> vec3<f32> {
    return at;
}

@if(DISTANCE_FOG)
fn stepped_fog_position(at: vec3<f32>, texel: vec2<i32>) -> vec3<f32> {
    if fog.mode != FOG_MODE_LINEAR {
        return at;
    }
    let start = fog.be.x;
    let end = fog.be.y;
    let eye = view.world_position.xyz;
    let distance = length(at - eye);
    let level = saturate((distance - start) / max(end - start, 1e-4));
    let stepped = floor(level * FOG_STEPS + dither_rank(texel)) / FOG_STEPS;
    let rounded = start + stepped * (end - start);
    return eye + (at - eye) * (rounded / max(distance, 1e-4));
}

fn texel_sharp(uv: vec2<f32>, ddx: vec2<f32>, ddy: vec2<f32>, size: vec2<f32>) -> vec2<f32> {
    let texel = uv * size;
    let footprint = clamp((abs(ddx) + abs(ddy)) * size, vec2<f32>(1e-5), vec2<f32>(1.0));
    let edge = floor(texel + 0.5);
    let across = smoothstep(-0.5 * footprint, 0.5 * footprint, texel - edge) - 0.5;
    return (edge + across) / size;
}

fn texel_center(
    uv: vec2<f32>,
    size: vec2<f32>,
    position: vec3<f32>,
    ddx: vec2<f32>,
    ddy: vec2<f32>,
    position_ddx: vec3<f32>,
    position_ddy: vec3<f32>,
) -> vec3<f32> {
    let offset = (floor(uv * size) + 0.5) / size - uv;
    // offset = ddx * a + ddy * b, solved for the screen step (a, b)
    let det = ddx.x * ddy.y - ddx.y * ddy.x;
    // a face seen edge on maps uv to almost a line and can't be inverted;
    // its texels cover no pixels worth stepping anyway
    if abs(det) <= 1e-3 * length(ddx) * length(ddy) {
        return position;
    }
    let a = (offset.x * ddy.y - offset.y * ddy.x) / det;
    let b = (ddx.x * offset.y - ddx.y * offset.x) / det;
    return position + position_ddx * a + position_ddy * b;
}
