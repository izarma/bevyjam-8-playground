import bevy_pbr::render::mesh_view_bindings::view;

fn billboard_facing_toward(toward: vec3<f32>, world_from_local: mat4x4<f32>) -> vec3<f32> {
    var facing = vec3<f32>(toward.x, 0.0, toward.z);
    if length(facing) < 1e-4 {
        let own = world_from_local[2].xyz;
        facing = vec3<f32>(own.x, 0.0, own.z);
    }
    return normalize(facing);
}

fn billboard_facing(world_from_local: mat4x4<f32>) -> vec3<f32> {
    if view.clip_from_view[3].w == 1.0 {
        return billboard_facing_toward(view.world_from_view[2].xyz, world_from_local);
    }
    return billboard_facing_toward(view.world_position - world_from_local[3].xyz, world_from_local);
}

fn billboard_position(world_from_local: mat4x4<f32>, local: vec3<f32>, facing: vec3<f32>) -> vec3<f32> {
    let scale = vec2<f32>(length(world_from_local[0].xyz), length(world_from_local[1].xyz));
    return world_from_local[3].xyz
        + cross(vec3<f32>(0.0, 1.0, 0.0), facing) * local.x * scale.x
        + vec3<f32>(0.0, local.y * scale.y, 0.0);
}
