// === ph2d-vector-fill generated shader (ADR-0060) ===
struct FillParams {
    colors: array<vec4<f32>, 16>,
    scalars: array<vec4<f32>, 16>,
    ucontrol: array<vec4<u32>, 16>,
    stop_colors: array<array<vec4<f32>, 8>, 16>,
    stop_pos: array<array<vec4<f32>, 2>, 16>,
    time: f32,
};
@group(0) @binding(0) var<uniform> params: FillParams;

fn fill_main(coord: vec2<f32>) -> vec4<f32> {
    let v0: vec4<f32> = params.colors[0];
    return v0;
}
