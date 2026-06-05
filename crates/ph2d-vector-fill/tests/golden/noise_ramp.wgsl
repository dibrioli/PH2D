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

fn ph2d_value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = p - i;
    let u = f * f * (3.0 - 2.0 * f);
    let a = ph2d_cell2(i + vec2<f32>(0.0, 0.0));
    let b = ph2d_cell2(i + vec2<f32>(1.0, 0.0));
    let c = ph2d_cell2(i + vec2<f32>(0.0, 1.0));
    let d = ph2d_cell2(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn ph2d_grad2(c: vec2<f32>) -> vec2<f32> {
    let a = ph2d_cell2(c) * 6.2831855;
    return vec2<f32>(cos(a), sin(a));
}

fn ph2d_simplex_corner(c: vec2<f32>, x: f32, y: f32) -> f32 {
    var tt = 0.5 - x * x - y * y;
    if (tt < 0.0) { return 0.0; }
    tt = tt * tt;
    let g = ph2d_grad2(c);
    return tt * tt * (g.x * x + g.y * y);
}
fn ph2d_simplex(p: vec2<f32>) -> f32 {
    let F2 = 0.3660254;
    let G2 = 0.21132487;
    let s = (p.x + p.y) * F2;
    let i = floor(p.x + s);
    let j = floor(p.y + s);
    let t = (i + j) * G2;
    let x0 = p.x - (i - t);
    let y0 = p.y - (j - t);
    var i1 = 0.0;
    var j1 = 0.0;
    if (x0 > y0) { i1 = 1.0; } else { j1 = 1.0; }
    let x1 = x0 - i1 + G2;
    let y1 = y0 - j1 + G2;
    let x2 = x0 - 1.0 + 2.0 * G2;
    let y2 = y0 - 1.0 + 2.0 * G2;
    var n = 0.0;
    n = n + ph2d_simplex_corner(vec2<f32>(i, j), x0, y0);
    n = n + ph2d_simplex_corner(vec2<f32>(i + i1, j + j1), x1, y1);
    n = n + ph2d_simplex_corner(vec2<f32>(i + 1.0, j + 1.0), x2, y2);
    return clamp(70.0 * n * 0.5 + 0.5, 0.0, 1.0);
}

fn ph2d_perlin(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = p - i;
    let u = f * f * (3.0 - 2.0 * f);
    let g00 = ph2d_grad2(i + vec2<f32>(0.0, 0.0));
    let g10 = ph2d_grad2(i + vec2<f32>(1.0, 0.0));
    let g01 = ph2d_grad2(i + vec2<f32>(0.0, 1.0));
    let g11 = ph2d_grad2(i + vec2<f32>(1.0, 1.0));
    let n00 = dot(g00, f - vec2<f32>(0.0, 0.0));
    let n10 = dot(g10, f - vec2<f32>(1.0, 0.0));
    let n01 = dot(g01, f - vec2<f32>(0.0, 1.0));
    let n11 = dot(g11, f - vec2<f32>(1.0, 1.0));
    let nx0 = mix(n00, n10, u.x);
    let nx1 = mix(n01, n11, u.x);
    return clamp(mix(nx0, nx1, u.y) * 0.5 + 0.5, 0.0, 1.0);
}

fn ph2d_fbm(p: vec2<f32>, lac: f32, pers: f32, oct: u32) -> f32 {
    var amp = 1.0;
    var freq = 1.0;
    var sum = 0.0;
    var norm = 0.0;
    let n = min(oct, 8u);
    for (var k = 0u; k < n; k = k + 1u) {
        sum = sum + amp * ph2d_value_noise(p * freq);
        norm = norm + amp;
        amp = amp * pers;
        freq = freq * lac;
    }
    return sum / max(norm, 1e-5);
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

fn ph2d_noise_dispatch(p: vec2<f32>, kind: u32, freq: f32, lac: f32, pers: f32, oct: u32) -> f32 {
    let q = p * freq;
    switch kind {
        case 0u: { return ph2d_simplex(q); }
        case 1u: { return ph2d_perlin(q); }
        case 2u: { return ph2d_cellular(q, 1.0); }
        case 3u: { return ph2d_fbm(q, lac, pers, oct); }
        default: { return 0.0; }
    }
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

fn ph2d_coord_transform(p: vec2<f32>, mode: u32) -> vec2<f32> {
    switch mode {
        case 0u: { return p; }
        case 1u: { return p; }
        case 2u: { return p; }
        case 3u: { return vec2<f32>(length(p), atan2(p.y, p.x)); }
        default: { return p; }
    }
}

fn fill_main(coord: vec2<f32>) -> vec4<f32> {
    let v0: vec2<f32> = ph2d_coord_transform(coord, params.ucontrol[0].x);
    let v1: f32 = ph2d_noise_dispatch(v0, params.ucontrol[1].x, params.scalars[1].x, params.scalars[1].y, params.scalars[1].z, params.ucontrol[1].y);
    let v2: vec4<f32> = ph2d_eval_stops(v1, 2u);
    return v2;
}
