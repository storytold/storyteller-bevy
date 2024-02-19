#import bevy_pbr::forward_io::VertexOutput

const COLOR_MULTIPLIER: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 0.8);

@group(1) @binding(0)
var<uniform> material_color: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    return material_color * COLOR_MULTIPLIER;
}
