import bevy_pbr::render::mesh_view_bindings::view;

// the way a quad faces when seen from `toward`: turned upright to it when billboarded, else its own forward
fn facing_toward(toward: vec3<f32>, own: vec3<f32>) -> vec3<f32> {
    @if(!BILLBOARD) {
        return normalize(own);
    }
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

// where `local` lands on a quad at `origin` whose local x and y run along `x` and `y`
fn billboard_point(origin: vec3<f32>, x: vec3<f32>, y: vec3<f32>, local: vec2<f32>, facing: vec3<f32>) -> vec3<f32> {
    @if(BILLBOARD) {
        let up = vec3<f32>(0.0, 1.0, 0.0);
        return origin + cross(up, facing) * local.x * length(x) + up * local.y * length(y);
    }
    return origin + x * local.x + y * local.y;
}
