// t3ssel8r's pixel-art sampling https://www.youtube.com/watch?v=d6tp43wZqps

fn texel_sharp(uv: vec2<f32>, ddx: vec2<f32>, ddy: vec2<f32>, size: vec2<f32>) -> vec2<f32> {
    let texel = uv * size;
    let footprint = clamp((abs(ddx) + abs(ddy)) * size, vec2<f32>(1e-5), vec2<f32>(1.0));
    let edge = floor(texel + 0.5);
    let across = smoothstep(-0.5 * footprint, 0.5 * footprint, texel - edge) - 0.5;
    return (edge + across) / size;
}
