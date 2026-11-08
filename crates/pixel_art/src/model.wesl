import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_bindings,
    pbr_fragment::pbr_input_from_vertex_output,
    pbr_functions,
    pbr_types::{STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT, STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT},
};
import pixel_art::texel::{texel_center, texel_sharp};
import pixel_art::lighting::matte_lighting;

fn premultiplied(texel: vec2<i32>, size: vec2<i32>) -> vec4<f32> {
    let color = textureLoad(pbr_bindings::base_color_texture, ((texel % size) + size) % size, 0);
    return vec4<f32>(color.rgb * color.a, color.a);
}

fn sharp_color(uv: vec2<f32>, ddx: vec2<f32>, ddy: vec2<f32>) -> vec4<f32> {
    let size = vec2<i32>(textureDimensions(pbr_bindings::base_color_texture));
    let at = texel_sharp(uv, ddx, ddy, vec2<f32>(size)) * vec2<f32>(size) - 0.5;
    let corner = vec2<i32>(floor(at));
    let f = at - floor(at);
    let blended = mix(
        mix(premultiplied(corner, size), premultiplied(corner + vec2<i32>(1, 0), size), f.x),
        mix(premultiplied(corner + vec2<i32>(0, 1), size), premultiplied(corner + vec2<i32>(1, 1), size), f.x),
        f.y,
    );
    return vec4<f32>(blended.rgb / max(blended.a, 1e-4), blended.a);
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    let flags = pbr_bindings::material.flags;
    var pbr_input = pbr_input_from_vertex_output(
        in, is_front, (flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u);
    pbr_input.material.flags = flags;
    pbr_input.material.base_color = pbr_bindings::material.base_color;
    @if(VERTEX_COLORS)
    pbr_input.material.base_color *= in.color;

    @if(VERTEX_UVS_A) {
        let ddx = dpdx(in.uv);
        let ddy = dpdy(in.uv);
        let world_ddx = dpdx(in.world_position.xyz);
        let world_ddy = dpdy(in.world_position.xyz);
        if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) != 0u {
            var color = pbr_bindings::material.base_color * sharp_color(in.uv, ddx, ddy);
            @if(VERTEX_COLORS)
            color *= in.color;
            pbr_input.material.base_color = color;
            // every fragment of a texel is lit, shadowed and fogged as one
            let size = vec2<f32>(textureDimensions(pbr_bindings::base_color_texture));
            pbr_input.world_position = vec4<f32>(
                texel_center(in.uv, size, in.world_position.xyz, ddx, ddy, world_ddx, world_ddy),
                1.0,
            );
        }
    }
    pbr_input.material.base_color = pbr_functions::alpha_discard(
        pbr_input.material.flags,
        pbr_bindings::material.alpha_cutoff,
        pbr_input.material.base_color,
    );

    var out: FragmentOutput;
    out.color = vec4<f32>(matte_lighting(pbr_input), pbr_input.material.base_color.a);
    out.color = pbr_functions::main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
