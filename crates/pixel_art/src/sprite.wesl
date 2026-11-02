// 3 things going on here
// stealing the pixel-art rendering techniques from this video https://www.youtube.com/watch?v=d6tp43wZqps&t=624s
// sampling jackdaw's environment map so we can use the resulting ambient light in the shader
// and lighting for sprites

import bevy_pbr::render::{
    clustered_forward as clustering,
    forward_io::{Vertex, VertexOutput, FragmentOutput, decompress_vertex},
    mesh_functions,
    mesh_view_bindings::{clustered_lights, lights, view},
    mesh_view_types::POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVE,
    pbr_ambient,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions,
    pbr_lighting::getDistanceAttenuation,
    pbr_types::PbrInput,
    view_transformations::position_world_to_clip,
};
import bevy_render::maths::{PI, quat_rotate};
@if(ENVIRONMENT_MAP) import bevy_pbr::render::mesh_view_bindings::{environment_map_sampler, light_probes};
@if(ENVIRONMENT_MAP && MULTIPLE_LIGHT_PROBES_IN_ARRAY)
import bevy_pbr::render::mesh_view_bindings::diffuse_environment_maps;
@if(ENVIRONMENT_MAP && !MULTIPLE_LIGHT_PROBES_IN_ARRAY)
import bevy_pbr::render::mesh_view_bindings::diffuse_environment_map;
import pixel_art::billboard::{billboard_facing, billboard_position};
import pixel_art::texel::texel_sharp;

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var sprite_texture: texture_2d<f32>;

// for the transparancy trick
fn premultiplied(texel: vec2<i32>) -> vec4<f32> {
    let last = vec2<i32>(textureDimensions(sprite_texture)) - 1;
    let color = textureLoad(sprite_texture, clamp(texel, vec2<i32>(0), last), 0);
    return vec4<f32>(color.rgb * color.a, color.a);
}

fn sprite_color(uv: vec2<f32>, ddx: vec2<f32>, ddy: vec2<f32>) -> vec4<f32> {
    let size = vec2<f32>(textureDimensions(sprite_texture));
    let at = texel_sharp(uv, ddx, ddy, size) * size - 0.5;
    let corner = vec2<i32>(floor(at));
    let f = at - floor(at);
    let blended = mix(
        mix(premultiplied(corner), premultiplied(corner + vec2<i32>(1, 0)), f.x),
        mix(premultiplied(corner + vec2<i32>(0, 1)), premultiplied(corner + vec2<i32>(1, 1)), f.x),
        f.y,
    );
    return vec4<f32>(blended.rgb / max(blended.a, 1e-4), blended.a);
}

// this is just a copy of bevy's vertex output
// but for web, position needs to be invariant for it to survive apparently
struct BillboardOutput {
    @invariant @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @if(VERTEX_OUTPUT_INSTANCE_INDEX)
    @location(6) @interpolate(flat) instance_index: u32,
    @if(VISIBILITY_RANGE_DITHER)
    @location(7) @interpolate(flat) visibility_range_dither: i32,
}

// 90% normal bevy vertex shader
@vertex
fn vertex(packed: Vertex) -> BillboardOutput {
    var out: BillboardOutput;
    let vertex = decompress_vertex(packed, packed.instance_index);
    let world_from_local = mesh_functions::get_world_from_local(packed.instance_index);

    let facing = billboard_facing(world_from_local);
    out.world_position = vec4<f32>(billboard_position(world_from_local, vertex.position, facing), 1.0);
    out.world_normal = facing;
    out.position = position_world_to_clip(out.world_position.xyz);

    out.uv = vertex.uv;
    @if(VERTEX_OUTPUT_INSTANCE_INDEX)
    out.instance_index = packed.instance_index;
    @if(VISIBILITY_RANGE_DITHER)
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        packed.instance_index, world_from_local[3]);
    return out;
}

// from bevy_pbr/src/light_probe/environment_map.wesl
@if(ENVIRONMENT_MAP)
fn environment_irradiance(direction: vec3<f32>) -> vec3<f32> {
    var toward = quat_rotate(light_probes.view_rotation, direction);
    toward.z = -toward.z;
    @if(MULTIPLE_LIGHT_PROBES_IN_ARRAY) {
        return textureSampleLevel(
            diffuse_environment_maps[light_probes.view_cubemap_index],
            environment_map_sampler,
            toward,
            0.0,
        ).rgb;
    } @else {
        return textureSampleLevel(diffuse_environment_map, environment_map_sampler, toward, 0.0).rgb;
    }
}

// the color / amount of light at `at` from light `id`
fn light_arriving(id: u32, at: vec3<f32>) -> vec3<f32> {
    let light = &clustered_lights.data[id];
    let to_light = (*light).position_radius.xyz - at;
    let inverse_range_squared = (*light).color_inverse_square_range.w;
    return (*light).color_inverse_square_range.rgb
        * getDistanceAttenuation(dot(to_light, to_light), inverse_range_squared);
}

// i dont actually understand how bevy reads spotlights and we arent using them
// this is sorta derived from bevy_pbr/src/render/shadows.wesl
fn spot_cone(id: u32, at: vec3<f32>) -> f32 {
    let light = &clustered_lights.data[id];
    let packed = (*light).light_custom_data;
    var axis = vec3<f32>(packed.x, 0.0, packed.y);
    axis.y = sqrt(max(0.0, 1.0 - axis.x * axis.x - axis.z * axis.z));
    if ((*light).flags & POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVE) != 0u {
        axis.y = -axis.y;
    }
    let toward = normalize((*light).position_radius.xyz - at);
    let cone = saturate(dot(-axis, toward) * packed.z + packed.w);
    return cone * cone;
}

// reads bevys point/spot/direction lights and jackdaws env map
fn sprite_lighting(in: PbrInput) -> vec4<f32> {
    let diffuse = in.material.base_color.rgb;
    let at = in.world_position.xyz;
    var arriving = vec3<f32>(0.0);

    for (var i = 0u; i < lights.n_directional_lights; i += 1u) {
        arriving += lights.directional_lights[i].color.rgb;
    }

    let view_z = dot(
        vec4<f32>(
            view.view_from_world[0].z,
            view.view_from_world[1].z,
            view.view_from_world[2].z,
            view.view_from_world[3].z,
        ),
        in.world_position,
    );
    let ranges = clustering::unpack_clusterable_object_index_ranges(
        clustering::view_fragment_cluster_index(in.frag_coord.xy, view_z, in.is_orthographic));
    for (var i = ranges.first_point_light_index_offset; i < ranges.first_spot_light_index_offset; i += 1u) {
        arriving += light_arriving(clustering::get_clusterable_object_id(i), at);
    }
    for (var i = ranges.first_spot_light_index_offset; i < ranges.first_rect_light_index_offset; i += 1u) {
        let id = clustering::get_clusterable_object_id(i);
        arriving += light_arriving(id, at) * spot_cone(id, at);
    }

    var ambient = pbr_ambient::ambient_light(
        in.world_position, in.N, in.V, 1.0, diffuse, vec3<f32>(0.0), 1.0, in.diffuse_occlusion);

    // sampling jackdaw env map to get the ambient light
    @if(ENVIRONMENT_MAP) {
        if light_probes.view_cubemap_index >= 0 {
            let all_round = (environment_irradiance(vec3<f32>(1.0, 0.0, 0.0))
                + environment_irradiance(vec3<f32>(-1.0, 0.0, 0.0))
                + environment_irradiance(vec3<f32>(0.0, 1.0, 0.0))
                + environment_irradiance(vec3<f32>(0.0, -1.0, 0.0))
                + environment_irradiance(vec3<f32>(0.0, 0.0, 1.0))
                + environment_irradiance(vec3<f32>(0.0, 0.0, -1.0))) / 6.0;
            ambient += diffuse * all_round * light_probes.intensity_for_view * in.diffuse_occlusion;
        }
    }
    return vec4<f32>(
        (diffuse * arriving / PI + ambient) * view.exposure,
        in.material.base_color.a,
    );
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    let color = sprite_color(in.uv, dpdx(in.uv), dpdy(in.uv));
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = pbr_functions::alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        color,
    );

    var out: FragmentOutput;
    out.color = sprite_lighting(pbr_input);
    out.color = pbr_functions::main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
