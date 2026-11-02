// this is like 90% stock bevy pbr prepass

import bevy_pbr::render::{
    mesh_functions,
    view_transformations::position_world_to_clip,
};
import bevy_pbr::prepass::io::{Vertex, VertexOutput, decompress_vertex};
import pixel_sprite::billboard::{billboard_facing, billboard_position};

fn placed(world_from_local: mat4x4<f32>, local: vec3<f32>) -> vec3<f32> {
    return billboard_position(world_from_local, local, billboard_facing(world_from_local));
}

@vertex
fn vertex(packed: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let vertex = decompress_vertex(packed, packed.instance_index);
    let world_from_local = mesh_functions::get_world_from_local(packed.instance_index);

    let world_position = placed(world_from_local, vertex.position);
    out.world_position = vec4<f32>(world_position, 1.0);
    out.position = position_world_to_clip(world_position);
    @if(UNCLIPPED_DEPTH_ORTHO_EMULATION) {
        out.unclipped_depth = out.position.z;
        out.position.z = min(out.position.z, 1.0);
    }

    @if(VERTEX_UVS_A)
    out.uv = vertex.uv;

    @if(NORMAL_PREPASS_OR_DEFERRED_PREPASS)
    out.world_normal = billboard_facing(world_from_local);

    @if(MOTION_VECTOR_PREPASS) {
        let previous_from_local = mesh_functions::get_previous_world_from_local(packed.instance_index);
        out.previous_world_position = vec4<f32>(placed(previous_from_local, vertex.position), 1.0);
    }

    @if(VERTEX_OUTPUT_INSTANCE_INDEX)
    out.instance_index = packed.instance_index;
    @if(VISIBILITY_RANGE_DITHER)
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        packed.instance_index, world_from_local[3]);

    return out;
}
