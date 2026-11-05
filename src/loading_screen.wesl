import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct LoadingScreen {
    swept: f32,
    pixels_per_texel: f32,
}

@group(1) @binding(0) var<uniform> loading_screen: LoadingScreen;

// width of the dithered part
const FRONT: f32 = 0.6;
// clockwise from the right. 0 is left to right
const ANGLE_DEGREES: f32 = 45.0;

fn bayer(cell: vec2<u32>) -> f32 {
    let lo = cell & vec2<u32>(1u);
    let hi = (cell >> vec2<u32>(1u)) & vec2<u32>(1u);
    let rank = 4u * (((lo.x ^ lo.y) << 1u) | lo.y) + (((hi.x ^ hi.y) << 1u) | hi.y);
    return (f32(rank) + 0.5) / 16.0;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let cell = vec2<u32>(floor(in.position.xy / loading_screen.pixels_per_texel));
    let front = loading_screen.swept * (1.0 + FRONT) - FRONT;
    let angle = radians(ANGLE_DEGREES);
    let direction = vec2(cos(angle), sin(angle));
    let corner = in.size * direction;
    let first = min(corner.x, 0.0) + min(corner.y, 0.0);
    let last = max(corner.x, 0.0) + max(corner.y, 0.0);
    let along = (dot(in.uv * in.size, direction) - first) / (last - first);
    let cover = saturate((along - front) / FRONT);
    if cover <= bayer(cell) {
        discard;
    }
    return vec4(0.0, 0.0, 0.0, 1.0);
}
