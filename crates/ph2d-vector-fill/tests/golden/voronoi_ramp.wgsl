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
fn ph2d_noise1(x: f32) -> f32 {
    let xc = select(x, 0.0, x == 0.0);
    var h: u32 = bitcast<u32>(xc) ^ 0x9e3779b9u;
    h = h ^ (h >> 16u);
    h = h * 0x7feb352du;
    h = h ^ (h >> 15u);
    h = h * 0x846ca68bu;
    h = h ^ (h >> 16u);
    return f32(h >> 8u) / 16777216.0;
}

fn ph2d_cell2(c: vec2<f32>) -> f32 {
    return ph2d_noise1(c.x * 113.0 + c.y * 271.7);
}

fn ph2d_cellular(p: vec2<f32>, jitter: f32) -> f32 {
    let i = floor(p);
    let f = p - i;
    var md = 8.0;
    for (var dy = -1.0; dy <= 1.0; dy = dy + 1.0) {
        for (var dx = -1.0; dx <= 1.0; dx = dx + 1.0) {
            let o = vec2<f32>(dx, dy);
            let cc = i + o;
            let fp = vec2<f32>(ph2d_cell2(cc), ph2d_cell2(cc + vec2<f32>(19.3, 71.7)));
            let diff = o + jitter * fp - f;
            md = min(md, dot(diff, diff));
        }
    }
    return clamp(sqrt(md), 0.0, 1.0);
}

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

fn fill_main(coord: vec2<f32>) -> vec4<f32> {
    let v0: f32 = ph2d_cellular(coord * f32(params.ucontrol[0].x), params.scalars[0].x);
    let v1: vec4<f32> = ph2d_eval_stops(v0, 1u);
    return v1;
}
