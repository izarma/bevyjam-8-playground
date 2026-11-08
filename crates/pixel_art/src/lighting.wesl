// our alternative to pbr lighting
import bevy_pbr::render::{
    clustered_forward as clustering,
    mesh_types::MESH_FLAGS_SHADOW_RECEIVER_BIT,
    mesh_view_bindings::{clustered_lights, lights, view},
    mesh_view_types::{
        DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
        POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
        POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVE,
    },
    pbr_types::PbrInput,
    shadows,
};
import bevy_render::maths::{PI, quat_rotate};
@if(ENVIRONMENT_MAP) import bevy_pbr::render::mesh_view_bindings::{environment_map_sampler, light_probes};
@if(ENVIRONMENT_MAP && MULTIPLE_LIGHT_PROBES_IN_ARRAY)
import bevy_pbr::render::mesh_view_bindings::diffuse_environment_maps;
@if(ENVIRONMENT_MAP && !MULTIPLE_LIGHT_PROBES_IN_ARRAY)
import bevy_pbr::render::mesh_view_bindings::diffuse_environment_map;

@if(ENVIRONMENT_MAP)
fn sky_irradiance(direction: vec3<f32>) -> vec3<f32> {
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

fn source_attenuation(distance_squared: f32, inverse_range_squared: f32, radius: f32) -> f32 {
    let factor = distance_squared * inverse_range_squared;
    let window = saturate(1.0 - factor * factor);
    return window * window / max(distance_squared + radius * radius, 0.0001);
}

// the color / amount of light at `at` from point or spot light `id`
fn light_arriving(id: u32, at: vec3<f32>, spread: f32) -> vec3<f32> {
    let light = &clustered_lights.data[id];
    let to_light = (*light).position_radius.xyz - at;
    let radius = (*light).position_radius.w;
    return (*light).color_inverse_square_range.rgb * source_attenuation(
        dot(to_light, to_light),
        (*light).color_inverse_square_range.w,
        sqrt(radius * radius + spread * spread),
    );
}

// how much of a spot light's cone `at` is in; derived from bevy_pbr's spot light
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

// every light's colour times how squarely the surface faces it, shadowed, plus
// the ambient and the sky's light from the way it faces, plus its own glow
fn matte_lighting(in: PbrInput) -> vec3<f32> {
    let diffuse = in.material.base_color.rgb;
    let n = in.N;
    let at = in.world_position;
    let receives = (in.flags & MESH_FLAGS_SHADOW_RECEIVER_BIT) != 0u;
    let view_z = dot(vec4<f32>(
        view.view_from_world[0].z,
        view.view_from_world[1].z,
        view.view_from_world[2].z,
        view.view_from_world[3].z,
    ), at);
    var arriving = vec3<f32>(0.0);

    for (var i = 0u; i < lights.n_directional_lights; i += 1u) {
        let light = &lights.directional_lights[i];
        let facing = saturate(dot(n, (*light).direction_to_light));
        if facing <= 0.0 {
            continue;
        }
        var shadow = 1.0;
        if receives && ((*light).flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
            shadow = shadows::fetch_directional_shadow(i, at, in.world_normal, view_z, in.frag_coord.xy);
        }
        arriving += (*light).color.rgb * facing * shadow;
    }

    let ranges = clustering::unpack_clusterable_object_index_ranges(
        clustering::view_fragment_cluster_index(in.frag_coord.xy, view_z, in.is_orthographic));
    for (var i = ranges.first_point_light_index_offset; i < ranges.first_spot_light_index_offset; i += 1u) {
        let id = clustering::get_clusterable_object_id(i);
        let to_light = normalize(clustered_lights.data[id].position_radius.xyz - at.xyz);
        var shadow = 1.0;
        if receives && (clustered_lights.data[id].flags & POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
            shadow = shadows::fetch_point_shadow(id, at, in.world_normal, in.frag_coord.xy);
        }
        arriving += light_arriving(id, at.xyz, 0.0) * saturate(dot(n, to_light)) * shadow;
    }
    for (var i = ranges.first_spot_light_index_offset; i < ranges.first_rect_light_index_offset; i += 1u) {
        let id = clustering::get_clusterable_object_id(i);
        let light = &clustered_lights.data[id];
        let to_light = normalize((*light).position_radius.xyz - at.xyz);
        var shadow = 1.0;
        if receives && ((*light).flags & POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
            shadow = shadows::fetch_spot_shadow(id, at, in.world_normal, (*light).shadow_map_near_z, in.frag_coord.xy);
        }
        arriving += light_arriving(id, at.xyz, 0.0) * saturate(dot(n, to_light)) * spot_cone(id, at.xyz) * shadow;
    }

    var ambient = diffuse * lights.ambient_color.rgb * in.diffuse_occlusion;
    @if(ENVIRONMENT_MAP) {
        if light_probes.view_cubemap_index >= 0 {
            ambient += diffuse * sky_irradiance(n) * light_probes.intensity_for_view * in.diffuse_occlusion;
        }
    }
    let glow = in.material.emissive.rgb * mix(1.0, view.exposure, in.material.emissive.a);
    return (diffuse * arriving / PI + ambient) * view.exposure + glow;
}
