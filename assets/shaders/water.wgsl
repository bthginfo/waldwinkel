#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    mesh_view_bindings::globals,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::main_pass_post_lighting_processing,
}

@group(2) @binding(100) var<uniform> surface: vec4<f32>;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) front: bool) -> FragmentOutput {
    let pbr = pbr_input_from_standard_material(in, front);
    let p = in.world_position.xz * surface.x;
    let t = globals.time * surface.y;
    let swell = dot(p, vec2(0.25, 0.87)) + t;
    let cross_swell = dot(p, vec2(-0.41, 0.72)) - t * 1.63;
    let wave = sin(swell + sin(cross_swell) * 0.98);
    let crossed = sin(cross_swell + dot(p, vec2(0.21, -0.17)) + t * 0.3);
    let glint = smoothstep(0.78, 1.0, wave * 0.66 + crossed * 0.34);
    // Small ripples disappear smoothly at the zoomed-out map scale.
    let detail = 1.0 - smoothstep(0.2, 1.15, fwidth(swell));
    let base = pbr.material.base_color;
    let trough = (wave * 0.031 + crossed * 0.026) * detail;
    let color = base.rgb * (1.0 + trough)
        + glint * detail * surface.z * vec3(0.58, 0.88, 0.86);

    var out: FragmentOutput;
    out.color = main_pass_post_lighting_processing(pbr, vec4(color, base.a));
    return out;
}
