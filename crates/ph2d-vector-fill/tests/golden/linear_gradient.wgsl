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

fn ph2d_stop_pos(node: u32, k: u32) -> f32 {
    return params.stop_pos[node][k / 4u][k % 4u];
}
fn ph2d_eval_stops(t: f32, node: u32) -> vec4<f32> {
    let n = params.ucontrol[node].x;
    if (n == 0u) { return vec4<f32>(t, t, t, 1.0); }
    let tc = clamp(t, 0.0, 1.0);
    if (tc <= ph2d_stop_pos(node, 0u)) { return params.stop_colors[node][0]; }
    let last = n - 1u;
    if (tc >= ph2d_stop_pos(node, last)) { return params.stop_colors[node][last]; }
    var result = params.stop_colors[node][0];
    for (var i = 1u; i < n; i = i + 1u) {
        let pa = ph2d_stop_pos(node, i - 1u);
        let pb = ph2d_stop_pos(node, i);
        if (tc >= pa && tc <= pb) {
            let f = (tc - pa) / max(pb - pa, 1e-5);
            result = mix(params.stop_colors[node][i - 1u], params.stop_colors[node][i], f);
        }
    }
    return result;
}

fn ph2d_linear_gradient(uv: vec2<f32>, node: u32) -> vec4<f32> {
    let angle = params.scalars[node].x;
    let dir = vec2<f32>(cos(angle), sin(angle));
    return ph2d_eval_stops(dot(uv, dir), node);
}

fn fill_main(coord: vec2<f32>) -> vec4<f32> {
    let v0: vec4<f32> = ph2d_linear_gradient(coord, 0u);
    return v0;
}
