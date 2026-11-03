// t3ssel8r's pixel-art sampling https://www.youtube.com/watch?v=d6tp43wZqps

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
