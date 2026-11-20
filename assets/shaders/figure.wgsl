#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::main_pass_post_lighting_processing,
}

@group(2) @binding(100) var<uniform> environment: vec4<f32>;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) front: bool) -> FragmentOutput {
    let pbr = pbr_input_from_standard_material(in, front);
    let n = normalize(pbr.world_normal);
    let sun = normalize(vec3(-0.784, 0.557, -0.276));
    let key = max(dot(n, sun), 0.0) * environment.x;
    let sky = n.y * 0.5 + 0.5;
    let lambert = (0.58 + 0.21 * sky) * environment.y;
    let fill_tint = mix(vec3(0.89, 0.98, 1.09), vec3(1.08, 0.99, 0.88), environment.w);
    let warm = vec3(1.16, 1.02, 0.84) * key * 0.52;
    let rim = (0.045 + 0.065 * environment.z) * max(dot(n, vec3(0.42, 0.63, 0.65)), 0.0);
    let base = pbr.material.base_color;
    let color = base.rgb * (fill_tint * lambert + warm + rim);
    var out: FragmentOutput;
    out.color = main_pass_post_lighting_processing(pbr, vec4(color, base.a));
    return out;
}
