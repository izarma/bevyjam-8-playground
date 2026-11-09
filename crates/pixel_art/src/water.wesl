import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    mesh_view_bindings::{globals, lights, view},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions,
    pbr_types::PbrInput,
    mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
    shadows,
    view_transformations::{ndc_to_frag_coord, position_ndc_to_world, position_world_to_ndc},
};
@if(DEPTH_PREPASS) import bevy_pbr::prepass::utils::prepass_depth;
import pixel_art::texel::{stepped_fog_position, texel_sharp};
import pixel_art::lighting::matte_lighting;
import bevy_render::maths::PI;
@if(STANDARD_MATERIAL_SPECULAR_TRANSMISSION)
import bevy_pbr::render::mesh_view_bindings::view_transmission_texture;

const WOBBLE_SCALE: f32 = 0.2;
const WOBBLE_SPEED: f32 = 0.5;

const LAP_SPREAD: f32 = 4.0;

const GLINT_PATCH: f32 = 2.0;
const TAU: f32 = 6.2831855;

struct PixelWaterSettings {
    shallow: vec4<f32>,
    deep: vec4<f32>,
    foam_color: vec4<f32>,
    depth_range: f32,
    bands: f32,

    texels_per_meter: f32,
    foam_depth: f32,
    lap_depth: f32,
    lap_speed: f32,
    refraction: f32,

    foam: mat2x2<f32>,
    second_foam: mat2x2<f32>,
    foam_drift: vec2<f32>,
    second_foam_drift: vec2<f32>,
    glint: f32,
    glint_sharpness: f32,
    glint_wobble: f32,
    glint_speed: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> water: PixelWaterSettings;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var foam_mask: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var foam_sampler: sampler;

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn foam_layer(at: vec2<f32>, ddx: vec2<f32>, ddy: vec2<f32>, layer: mat2x2<f32>, slide: vec2<f32>) -> f32 {
    return textureSampleGrad(foam_mask, foam_sampler, layer * at + slide, layer * ddx, layer * ddy).r;
}

fn value_noise(p: vec2<f32>) -> f32 {
    let cell = floor(p);
    let f = smoothstep(vec2<f32>(0.0), vec2<f32>(1.0), fract(p));
    return mix(
        mix(hash(cell), hash(cell + vec2<f32>(1.0, 0.0)), f.x),
        mix(hash(cell + vec2<f32>(0.0, 1.0)), hash(cell + vec2<f32>(1.0, 1.0)), f.x),
        f.y,
    );
}

fn on_screen(pixel: vec2<f32>) -> vec2<f32> {
    let first = view.viewport.xy;
    return clamp(pixel, first, first + view.viewport.zw - 1.0);
}

fn pixel_depth(pixel: vec2<f32>) -> f32 {
    @if(DEPTH_PREPASS) {
        return prepass_depth(vec4<f32>(on_screen(pixel), 0.0, 0.0), 0u);
    } @else {
        return 0.0;
    }
}

// if we dont blend depth around the pixel we are sampling for foam
// it will be noisy when you or the camera move because of small changes in depth
fn buffer_depth(ndc: vec2<f32>) -> f32 {
    let at = ndc_to_frag_coord(ndc) - 0.5;
    let low = floor(at);
    let f = at - low;
    return mix(
        mix(pixel_depth(low), pixel_depth(low + vec2<f32>(1.0, 0.0)), f.x),
        mix(pixel_depth(low + vec2<f32>(0.0, 1.0)), pixel_depth(low + vec2<f32>(1.0, 1.0)), f.x),
        f.y,
    );
}

fn sun_glint(center: vec2<f32>, lighting: PbrInput, frag: vec2<f32>) -> vec3<f32> {
    if water.glint <= 0.0 || lights.n_directional_lights == 0u {
        return vec3<f32>(0.0);
    }
    let square = floor(center * water.texels_per_meter);
    let tipping = square / GLINT_PATCH;
    let tip = vec2<f32>(
        value_noise(tipping + globals.time * water.glint_speed),
        value_noise(tipping + 53.0 - globals.time * water.glint_speed),
    ) * 2.0 - 1.0;
    let tipped = normalize(vec3<f32>(tip.x * water.glint_wobble, 1.0, tip.y * water.glint_wobble));
    let sun = &lights.directional_lights[0];
    let bounce = reflect(-lighting.V, tipped);
    let toward = max(dot(bounce, (*sun).direction_to_light), 0.0);
    let flash = step(0.5, pow(toward, water.glint_sharpness));
    if flash == 0.0 {
        return vec3<f32>(0.0);
    }
    let view_z = dot(vec4<f32>(
        view.view_from_world[0].z,
        view.view_from_world[1].z,
        view.view_from_world[2].z,
        view.view_from_world[3].z,
    ), lighting.world_position);
    var shade = 1.0;
    if ((*sun).flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
        shade = shadows::fetch_directional_shadow(
            0u, lighting.world_position, vec3<f32>(0.0, 1.0, 0.0), view_z, frag);
    }
    return (*sun).color.rgb / PI * view.exposure * water.glint * shade;
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    if in.world_normal.y < 0.5 {
        discard;
    }

    let pbr_input = pbr_input_from_standard_material(in, is_front);
    let ddx = dpdx(in.world_position.xz);
    let ddy = dpdy(in.world_position.xz);
    let center = texel_sharp(in.world_position.xz, ddx, ddy, vec2<f32>(water.texels_per_meter));
    let squares_per_pixel = (abs(ddx) + abs(ddy)) * water.texels_per_meter;
    let resolved = saturate(2.0 - 2.0 * max(squares_per_pixel.x, squares_per_pixel.y));

    // measure depth at the square's center, unless something in front covers it
    var ndc = position_world_to_ndc(vec3<f32>(center.x, in.world_position.y, center.y));
    var below = buffer_depth(ndc.xy);
    if below > ndc.z {
        ndc = position_world_to_ndc(in.world_position.xyz);
        below = buffer_depth(ndc.xy);
    }
    let depth = in.world_position.y - position_ndc_to_world(vec3<f32>(ndc.xy, below)).y;

    let deepness = saturate(depth / water.depth_range);
    let band = floor(deepness * water.bands) / max(water.bands - 1.0, 1.0);
    var color = mix(water.shallow, water.deep, saturate(band));

    let plain = foam_layer(center, ddx, ddy, water.foam, water.foam_drift * globals.time);
    let turned = foam_layer(center, ddx, ddy, water.second_foam, water.second_foam_drift * globals.time);
    let coverage = 1.0 - (1.0 - plain) * (1.0 - turned);

    let beat = value_noise(center / LAP_SPREAD) * TAU;
    let lap = water.lap_depth * (0.5 + 0.5 * sin(globals.time * water.lap_speed + beat));
    let foam = select(mix(coverage, step(0.5, coverage), resolved), 1.0, depth < water.foam_depth + lap);
    color = mix(color, water.foam_color, foam);

    // each square is lit as one; its only shine is the sun's glint
    var lighting = pbr_input;
    lighting.world_position = vec4<f32>(center.x, in.world_position.y, center.y, 1.0);
    lighting.V = pbr_functions::calculate_view(lighting.world_position, pbr_input.is_orthographic);
    lighting.material.base_color = vec4<f32>(color.rgb, 1.0);
    let lit = matte_lighting(lighting);

    var out: FragmentOutput;
    @if(STANDARD_MATERIAL_SPECULAR_TRANSMISSION) {
        let square = center * water.texels_per_meter;
        let wobble = vec2<f32>(
            value_noise(square * WOBBLE_SCALE + globals.time * WOBBLE_SPEED),
            value_noise(square * WOBBLE_SCALE + 37.0 - globals.time * WOBBLE_SPEED),
        ) * 2.0 - 1.0;
        let shift = floor(wobble * water.refraction * deepness * resolved + 0.5) / water.texels_per_meter;

        let seen = position_world_to_ndc(in.world_position.xyz + vec3<f32>(shift.x, 0.0, shift.y));
        var pixel = on_screen(floor(ndc_to_frag_coord(seen.xy)));
        if pixel_depth(pixel) > seen.z {
            pixel = floor(in.position.xy);
        }
        let bed = textureLoad(view_transmission_texture, vec2<i32>(pixel), 0).rgb;
        out.color = vec4<f32>(mix(bed, lit, color.a), 1.0);
    } @else {
        out.color = vec4<f32>(lit, 1.0);
    }
    // foam is too churned up to catch the sun
    out.color += vec4<f32>(sun_glint(center, lighting, in.position.xy) * (1.0 - foam), 0.0);
    var fogged = lighting;
    fogged.world_position = vec4<f32>(
        stepped_fog_position(
            lighting.world_position.xyz,
            vec2<i32>(floor(center * water.texels_per_meter)),
            max(squares_per_pixel.x, squares_per_pixel.y),
        ),
        1.0,
    );
    out.color = pbr_functions::main_pass_post_lighting_processing(fogged, out.color);
    return out;
}
