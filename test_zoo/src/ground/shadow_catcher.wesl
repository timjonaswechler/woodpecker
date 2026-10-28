import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    mesh_view_bindings,
    mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
    shadows::fetch_directional_shadow,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> shadow_color: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    let view_z = dot(vec4<f32>(
        mesh_view_bindings::view.view_from_world[0].z,
        mesh_view_bindings::view.view_from_world[1].z,
        mesh_view_bindings::view.view_from_world[2].z,
        mesh_view_bindings::view.view_from_world[3].z,
    ), in.world_position);

    var visibility = 1.0;
    for (var i = 0u; i < mesh_view_bindings::lights.n_directional_lights; i += 1u) {
        if (mesh_view_bindings::lights.directional_lights[i].flags
                & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
            visibility = min(visibility, fetch_directional_shadow(
                i,
                in.world_position,
                normalize(in.world_normal),
                view_z,
                in.position.xy,
            ));
        }
    }

    // Lit ground has zero opacity; only occluded light contributes to the image.
    var out: FragmentOutput;
    out.color = vec4<f32>(shadow_color.rgb, shadow_color.a * (1.0 - visibility));
    return out;
}
