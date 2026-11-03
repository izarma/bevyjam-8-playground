import bevy_pbr::render::mesh_view_bindings::view;


fn facing_toward(toward: vec3<f32>, own: vec3<f32>) -> vec3<f32> {
    var facing = vec3<f32>(toward.x, 0.0, toward.z);
    if length(facing) < 1e-4 {
        facing = vec3<f32>(own.x, 0.0, own.z);
    }
    return normalize(facing);
}

fn billboard_facing(origin: vec3<f32>, own: vec3<f32>) -> vec3<f32> {
    if view.clip_from_view[3].w == 1.0 {
        return facing_toward(view.world_from_view[2].xyz, own);
    }
    return facing_toward(view.world_position - origin, own);
}

fn billboard_scale(world_from_local: mat4x4<f32>) -> vec2<f32> {
    return vec2<f32>(length(world_from_local[0].xyz), length(world_from_local[1].xyz));
}

fn billboard_point(origin: vec3<f32>, scale: vec2<f32>, local: vec2<f32>, facing: vec3<f32>) -> vec3<f32> {
    return origin
        + cross(vec3<f32>(0.0, 1.0, 0.0), facing) * local.x * scale.x
        + vec3<f32>(0.0, local.y * scale.y, 0.0);
}
