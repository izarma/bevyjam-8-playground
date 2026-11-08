// 3 things going on here
// stealing the pixel-art rendering techniques from this video https://www.youtube.com/watch?v=d6tp43wZqps&t=624s
// sampling jackdaw's environment map so we can use the resulting ambient light in the shader
// and lighting for sprites

import bevy_pbr::render::{
    clustered_forward as clustering,
    forward_io::{Vertex, VertexOutput, FragmentOutput, decompress_vertex},
    mesh_functions,
    mesh_view_bindings::{clustered_lights, lights, view},
    mesh_types::MESH_FLAGS_SHADOW_RECEIVER_BIT,
    mesh_view_types::{
        DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
        POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
    },
    pbr_bindings,
    pbr_fragment::pbr_input_from_vertex_output,
    pbr_functions,
    pbr_types::{PbrInput, STANDARD_MATERIAL_FLAGS_UNLIT_BIT},
    shadows,
    view_transformations::position_world_to_clip,
};
import bevy_render::maths::PI;
@if(ENVIRONMENT_MAP) import bevy_pbr::render::mesh_view_bindings::light_probes;
@if(ENVIRONMENT_MAP) import pixel_art::lighting::sky_irradiance;
import pixel_art::billboard::{billboard_facing, billboard_point, facing_toward};
import pixel_art::lighting::{light_arriving, spot_cone};
import pixel_art::texel::{dither_rank, stepped_fog_position, texel_center, texel_sharp};

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var sprite_texture: texture_2d<f32>;

// for the transparancy trick
fn premultiplied(texel: vec2<i32>, first: vec2<i32>, last: vec2<i32>) -> vec4<f32> {
    let color = textureLoad(sprite_texture, clamp(texel, first, last), 0);
    return vec4<f32>(color.rgb * color.a, color.a);
}

fn sprite_color(uv: vec2<f32>, ddx: vec2<f32>, ddy: vec2<f32>, frame: vec4<f32>) -> vec4<f32> {
    let size = vec2<f32>(textureDimensions(sprite_texture));
    let first = vec2<i32>(round(frame.xy * size));
    let last = vec2<i32>(round(frame.zw * size)) - 1;
    let at = texel_sharp(uv, ddx, ddy, size) * size - 0.5;
    let corner = vec2<i32>(floor(at));
    let f = at - floor(at);
    let blended = mix(
        mix(premultiplied(corner, first, last), premultiplied(corner + vec2<i32>(1, 0), first, last), f.x),
        mix(
            premultiplied(corner + vec2<i32>(0, 1), first, last),
            premultiplied(corner + vec2<i32>(1, 1), first, last),
            f.x,
        ),
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
    @location(3) uv_b: vec2<f32>,
    @if(VERTEX_OUTPUT_INSTANCE_INDEX)
    @location(6) @interpolate(flat) instance_index: u32,
    @if(VISIBILITY_RANGE_DITHER)
    @location(7) @interpolate(flat) visibility_range_dither: i32,
    @location(8) @interpolate(flat) frame: vec4<f32>,
    @location(9) local: vec2<f32>,
    @location(10) @interpolate(flat) origin: vec3<f32>,
    @location(11) @interpolate(flat) x: vec3<f32>,
    @location(12) @interpolate(flat) own: vec3<f32>,
    @location(13) @interpolate(flat) y: vec3<f32>,
}

struct Quad {
    // the frame's uv rect as (min, max)
    @location(8) @interpolate(flat) frame: vec4<f32>,
    @location(9) local: vec2<f32>,
    @location(10) @interpolate(flat) origin: vec3<f32>,
    @location(11) @interpolate(flat) x: vec3<f32>,
    @location(12) @interpolate(flat) own: vec3<f32>,
    @location(13) @interpolate(flat) y: vec3<f32>,
}

// 90% normal bevy vertex shader
@vertex
fn vertex(packed: Vertex) -> BillboardOutput {
    var out: BillboardOutput;
    let vertex = decompress_vertex(packed, packed.instance_index);
    let world_from_local = mesh_functions::get_world_from_local(packed.instance_index);

    out.origin = world_from_local[3].xyz;
    out.x = world_from_local[0].xyz;
    out.y = world_from_local[1].xyz;
    out.own = world_from_local[2].xyz;
    out.local = vertex.position.xy;
    let facing = billboard_facing(out.origin, out.own);
    out.world_position = vec4<f32>(billboard_point(out.origin, out.x, out.y, out.local, facing), 1.0);
    out.world_normal = facing;
    out.position = position_world_to_clip(out.world_position.xyz);

    out.uv = vertex.uv;
    out.uv_b = vertex.uv_b;
    out.frame = vec4<f32>(min(vertex.uv, vertex.uv_b), max(vertex.uv, vertex.uv_b));
    @if(VERTEX_OUTPUT_INSTANCE_INDEX)
    out.instance_index = packed.instance_index;
    @if(VISIBILITY_RANGE_DITHER)
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        packed.instance_index, world_from_local[3]);
    return out;
}

// from bevy_pbr/src/light_probe/environment_map.wesl
// reads bevys point/spot/direction lights and jackdaws env map
fn sprite_lighting(in: PbrInput, quad: Quad, local: vec2<f32>, spread: f32) -> vec4<f32> {
    let diffuse = in.material.base_color.rgb;
    let receives = (in.flags & MESH_FLAGS_SHADOW_RECEIVER_BIT) != 0u;
    var arriving = vec3<f32>(0.0);

    let view_z = dot(
        vec4<f32>(
            view.view_from_world[0].z,
            view.view_from_world[1].z,
            view.view_from_world[2].z,
            view.view_from_world[3].z,
        ),
        in.world_position,
    );

    for (var i = 0u; i < lights.n_directional_lights; i += 1u) {
        let light = &lights.directional_lights[i];
        var shadow = 1.0;
        if receives && ((*light).flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
            let facing = facing_toward((*light).direction_to_light, quad.own);
            let at = billboard_point(quad.origin, quad.x, quad.y, local, facing);
            shadow = shadows::fetch_directional_shadow(i, vec4<f32>(at, 1.0), facing, view_z, in.frag_coord.xy);
        }
        arriving += (*light).color.rgb * shadow;
    }

    let ranges = clustering::unpack_clusterable_object_index_ranges(
        clustering::view_fragment_cluster_index(in.frag_coord.xy, view_z, in.is_orthographic));
    for (var i = ranges.first_point_light_index_offset; i < ranges.first_spot_light_index_offset; i += 1u) {
        let id = clustering::get_clusterable_object_id(i);
        let light = &clustered_lights.data[id];
        let facing = facing_toward((*light).position_radius.xyz - quad.origin, quad.own);
        let at = billboard_point(quad.origin, quad.x, quad.y, local, facing);
        var shadow = 1.0;
        if receives && ((*light).flags & POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
            shadow = shadows::fetch_point_shadow(id, vec4<f32>(at, 1.0), facing, in.frag_coord.xy);
        }
        arriving += light_arriving(id, at, spread) * shadow;
    }
    for (var i = ranges.first_spot_light_index_offset; i < ranges.first_rect_light_index_offset; i += 1u) {
        let id = clustering::get_clusterable_object_id(i);
        let light = &clustered_lights.data[id];
        let facing = facing_toward((*light).position_radius.xyz - quad.origin, quad.own);
        let at = billboard_point(quad.origin, quad.x, quad.y, local, facing);
        var shadow = 1.0;
        if receives && ((*light).flags & POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
            shadow = shadows::fetch_spot_shadow(
                id, vec4<f32>(at, 1.0), facing, (*light).shadow_map_near_z, in.frag_coord.xy);
        }
        arriving += light_arriving(id, at, spread) * spot_cone(id, at) * shadow;
    }

    var ambient = diffuse * lights.ambient_color.rgb * in.diffuse_occlusion;

    // sampling jackdaw env map to get the ambient light
    @if(ENVIRONMENT_MAP) {
        if light_probes.view_cubemap_index >= 0 {
            let all_round = (sky_irradiance(vec3<f32>(1.0, 0.0, 0.0))
                + sky_irradiance(vec3<f32>(-1.0, 0.0, 0.0))
                + sky_irradiance(vec3<f32>(0.0, 1.0, 0.0))
                + sky_irradiance(vec3<f32>(0.0, -1.0, 0.0))
                + sky_irradiance(vec3<f32>(0.0, 0.0, 1.0))
                + sky_irradiance(vec3<f32>(0.0, 0.0, -1.0))) / 6.0;
            ambient += diffuse * all_round * light_probes.intensity_for_view * in.diffuse_occlusion;
        }
    }
    return vec4<f32>(
        (diffuse * arriving / PI + ambient) * view.exposure,
        in.material.base_color.a,
    );
}

fn metres_per_uv(local_dx: f32, local_dy: f32, uv_dx: f32, uv_dy: f32) -> f32 {
    return abs(local_dx * uv_dx + local_dy * uv_dy) / max(uv_dx * uv_dx + uv_dy * uv_dy, 1e-12);
}

fn sprite_spread(quad: Quad, ddx: vec2<f32>, ddy: vec2<f32>, local_ddx: vec2<f32>, local_ddy: vec2<f32>) -> f32 {
    let frame = quad.frame.zw - quad.frame.xy;
    let width = metres_per_uv(local_ddx.x, local_ddy.x, ddx.x, ddy.x) * frame.x * length(quad.x);
    let height = metres_per_uv(local_ddx.y, local_ddy.y, ddx.y, ddy.y) * frame.y * length(quad.y);
    return sqrt(width * height / PI);
}

@fragment
fn fragment(in: VertexOutput, quad: Quad, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    let ddx = dpdx(in.uv);
    let ddy = dpdy(in.uv);
    let local_ddx = vec3<f32>(dpdx(quad.local), 0.0);
    let local_ddy = vec3<f32>(dpdy(quad.local), 0.0);
    let color = sprite_color(in.uv, ddx, ddy, quad.frame);
    // the material's alpha is how far it has faded in
    let tint = pbr_bindings::material.base_color;
    let texel = vec2<i32>(in.uv * vec2<f32>(textureDimensions(sprite_texture)));
    if tint.a < dither_rank(texel) {
        discard;
    }
    var pbr_input = pbr_input_from_vertex_output(in, is_front, false);
    pbr_input.material.flags = pbr_bindings::material.flags;
    pbr_input.material.base_color = pbr_functions::alpha_discard(
        pbr_bindings::material.flags,
        pbr_bindings::material.alpha_cutoff,
        color * vec4<f32>(tint.rgb, 1.0),
    );

    let size = vec2<f32>(textureDimensions(sprite_texture));
    let local = texel_center(in.uv, size, vec3<f32>(quad.local, 0.0), ddx, ddy, local_ddx, local_ddy).xy;
    let seen = billboard_point(quad.origin, quad.x, quad.y, local, billboard_facing(quad.origin, quad.own));
    pbr_input.world_position = vec4<f32>(seen, 1.0);

    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) != 0u {
        out.color = pbr_input.material.base_color;
    } else {
        out.color = sprite_lighting(pbr_input, quad, local, sprite_spread(quad, ddx, ddy, local_ddx.xy, local_ddy.xy));
    }
    var fogged = pbr_input;
    fogged.world_position = vec4<f32>(stepped_fog_position(seen, texel), 1.0);
    out.color = pbr_functions::main_pass_post_lighting_processing(fogged, out.color);
    return out;
}
