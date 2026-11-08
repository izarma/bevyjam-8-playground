// a sky in big square texels: the view direction is snapped to a grid of angles,
// so the sky's texels stay put as the camera turns, like the ground's do. banded
// colour from zenith to horizon, a hard sun or moon in rings of glow, and stars

import bevy_pbr::render::mesh_view_bindings::{view, globals};
import bevy_pbr::render::utils::coords_to_viewport_uv;
import bevy_render::maths::PI;
@if(TONEMAP_IN_SHADER) import bevy_core_pipeline::tonemapping::tone_mapping;

struct PixelSkySettings {
    zenith: vec4<f32>,
    middle: vec4<f32>,
    horizon: vec4<f32>,
    ground: vec4<f32>,
    sun_color: vec4<f32>,
    moon_color: vec4<f32>,
    glow_color: vec4<f32>,
    sunset_color: vec4<f32>,
    // toward the sun
    sun: vec4<f32>,
    texels_per_degree: f32,
    bands: f32,
    dither: f32,
    horizon_dither: f32,
    horizon_reach: f32,
    sun_size: f32,
    glow_rings: f32,
    glow_reach: f32,
    glow_strength: f32,
    sunset: f32,
    stars: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> sky: PixelSkySettings;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

// clip space as given, at the far plane
@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    return VertexOutput(vec4<f32>(in.position.xy, 0.0, 1.0));
}

fn ray_direction(position: vec2<f32>) -> vec3<f32> {
    let clip = coords_to_viewport_uv(position, view.viewport) * vec2(2.0, -2.0) + vec2(-1.0, 1.0);
    let homogeneous = view.view_from_clip * vec4(clip, 1.0, 1.0);
    let in_view = homogeneous.xyz / homogeneous.w;
    return normalize((view.world_from_view * vec4(in_view, 0.0)).xyz);
}

// 4x4 ordered dither threshold of a texel, 0..1
fn dither_rank(cell: vec2<f32>) -> f32 {
    let m = vec2<u32>(vec2<i32>(cell) & vec2<i32>(3));
    var bayer = array<u32, 16>(0u, 8u, 2u, 10u, 12u, 4u, 14u, 6u, 3u, 11u, 1u, 9u, 15u, 7u, 13u, 5u);
    return (f32(bayer[m.y * 4u + m.x]) + 0.5) / 16.0;
}

fn hash(cell: vec2<f32>) -> f32 {
    var p = fract(vec3<f32>(cell.xyx) * 0.1031);
    p += dot(p, p.yzx + 33.33);
    return fract((p.x + p.y) * p.z);
}

// the angle one texel spans
fn texel_angle() -> f32 {
    return radians(1.0) / sky.texels_per_degree;
}

// texels around the ring of a row; fewer toward the zenith so they stay square
fn columns_at(row: f32) -> f32 {
    let elevation = (row + 0.5) * texel_angle();
    return max(1.0, floor(2.0 * PI * max(cos(elevation), 0.0) / texel_angle()));
}

// the direction through the centre of a texel
fn texel_direction(column: f32, row: f32) -> vec3<f32> {
    let elevation = (row + 0.5) * texel_angle();
    let count = columns_at(row);
    let around = ((column - count * floor(column / count)) + 0.5) * (2.0 * PI / count) - PI;
    return vec3(cos(elevation) * sin(around), sin(elevation), -cos(elevation) * cos(around));
}

fn gradient(g: f32) -> vec3<f32> {
    if g < 0.5 {
        return mix(sky.zenith.rgb, sky.middle.rgb, g * 2.0);
    }
    return mix(sky.middle.rgb, sky.horizon.rgb, (g - 0.5) * 2.0);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let ray = ray_direction(in.position.xy);
    let row = floor(asin(clamp(ray.y, -1.0, 1.0)) / texel_angle());
    let count = columns_at(row);
    let column = floor((atan2(ray.x, -ray.z) + PI) / (2.0 * PI / count));
    let direction = texel_direction(column, row);
    let cell = vec2(column, row);
    let rank = dither_rank(cell);

    // dither everywhere, or only within some degrees of the horizon
    var dither = sky.dither;
    if sky.horizon_dither > 0.0 {
        dither *= 1.0 - smoothstep(0.0, radians(sky.horizon_dither), abs(asin(direction.y)));
    }

    // banded from the horizon colour up to the zenith, and down to the ground
    let g = pow(1.0 - abs(direction.y), 1.0 / max(sky.horizon_reach, 0.05));
    let band = clamp(floor(g * sky.bands + (rank - 0.5) * dither), 0.0, sky.bands - 1.0);
    let t = band / (sky.bands - 1.0);
    var color = gradient(t);
    if direction.y < 0.0 {
        color = mix(sky.ground.rgb, sky.horizon.rgb, t);
    }

    if sky.stars > 0.0 && direction.y > 0.04 {
        let h = hash(cell + 7.0);
        // each star blinks out for a moment every 7 to 20 seconds, on its own
        // timing: drawn from other hashes than the one that picked it, which
        // every star shares the top of
        let phase = hash(cell + 31.0);
        let rate = 0.05 + 0.1 * hash(cell + 53.0);
        let twinkle = step(0.06, fract(phase + globals.time * rate));
        let star = step(1.0 - sky.stars * 0.01, h) * twinkle * smoothstep(0.04, 0.25, direction.y);
        color = mix(color, vec3(0.95, 0.93, 0.85), star * 0.9);
    }

    // the sun, or the moon opposite it once it has set
    {
        let sun = sky.sun.xyz;

        // the low sky on the sun's side warms while the sun is low, in four steps
        if sky.sunset > 0.0 {
            let side = pow(max(dot(normalize(direction.xz + 1e-5), normalize(sun.xz + 1e-5)), 0.0), 2.0);
            let low = (1.0 - smoothstep(0.0, 0.45, sun.y)) * smoothstep(-0.25, 0.0, sun.y);
            let height = 1.0 - smoothstep(0.0, 0.5, abs(direction.y));
            let warmth = side * low * height * sky.sunset;
            let steps = saturate(floor(warmth * 4.0 + (rank - 0.5) * dither) / 4.0);
            color = mix(color, sky.sunset_color.rgb, steps);
        }

        // time_of_day.rs swaps the scene's light to the moon at the same height
        let moon = sun.y < -0.04;
        let toward = select(sun, -sun, moon);
        let angle = degrees(acos(clamp(dot(direction, toward), -1.0, 1.0)));
        let reach = select(sky.glow_reach, sky.glow_reach * 0.6, moon);
        if sky.glow_rings > 0.0 && angle < reach {
            let ring = clamp(floor((1.0 - angle / reach) * sky.glow_rings + (rank - 0.5) * dither), 0.0, sky.glow_rings);
            let strength = select(sky.glow_strength, sky.glow_strength * 0.5, moon);
            color = mix(color, sky.glow_color.rgb, ring / sky.glow_rings * strength);
        }
        let size = select(sky.sun_size, sky.sun_size * 0.7, moon);
        if angle < size * 0.5 {
            color = select(sky.sun_color.rgb, sky.moon_color.rgb, moon);
        }
    }

    var out = vec4<f32>(color, 1.0);
    @if(TONEMAP_IN_SHADER)
    out = tone_mapping(out, view.color_grading);
    return out;
}
