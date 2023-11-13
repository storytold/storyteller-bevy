#import bevy_pbr::forward_io::VertexOutput

fn inv_lerp(a: f32, b: f32, v: f32) -> f32 {
    if (a == b) { return 1.0; }
    return (v - a) / (b - a);
}

/// Draw grid-lines at a given scale and camera depth.
///
/// @param pos  The reference coordinates for the grid lines.
/// @param depth  Distance of the camera from the fragment being shaded.
/// @param scale
///     `pos` will be multiplied by this value before computing the color value.
///     E.g., a scale of 10 will draw 10 lines for each whole-number increment
///     on the X and Z axes of `pos`.
/// @param depth_min, depth_max
///     Controls the opacity of the grid lines relative to the camera depth.
///     These will be mapped to 0-1 and multiplied by the color value to produce
///     the final output.
fn grid_lines(pos: vec4<f32>, depth: f32, scale: f32, depth_min: f32, depth_max: f32) -> f32 {
    let ref_coords = abs(pos) * scale;
    let fracs = fract(ref_coords);
    let thickness = (0.0333 * scale) / (depth * 100.0);

    let x = smoothstep(0.0, thickness, fracs.x);
    let z = smoothstep(0.0, thickness, fracs.z);
    let value = 1.0 - saturate(x * z);

    let mask = inv_lerp(depth_min, depth_max, depth);

    return saturate(value * mask);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let depth = in.position.z;
    let pos = in.world_position;

    let dense  = grid_lines(pos, depth, 10.0, 0.001,   0.1);
    let base   = grid_lines(pos, depth,  1.0, 0.0001,  0.01);
    let sparse = grid_lines(pos, depth,  0.1, 0.00005, 0.005);

    return vec4<f32>(1.0, 1.0, 1.0, 0.25) * saturate(dense + base + sparse);
}
