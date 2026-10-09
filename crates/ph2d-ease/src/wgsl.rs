//! A MESMA conta de [`crate::ease`] e [`crate::order::raw_at`], em WGSL — operação a operação,
//! na mesma ordem. Um literal (via macro) para o nó o concatenar com o seu próprio trecho:
//! `concat!(ph2d_ease::wgsl_lib!(), "…")`. Prefixo `pe_`.
//!
//! Assinaturas: `pe_ease(curve, dir, t, s0, s1)` com `s0 = (overshoot, period, steps, damping)` e
//! `s1 = bezier`; `pe_raw_at(kind, seed, cols, from, cell, axis, origin, reach, i, n, p)`.

/// O texto WGSL da tabela e da ordem.
#[macro_export]
macro_rules! wgsl_lib {
    () => {
        "fn pe_div(a: f32, b: f32) -> f32 {
    let y0 = 1.0 / b;
    let y = fma(fma(-b, y0, 1.0), y0, y0);
    let q = a * y;
    return fma(fma(-q, b, a), y, q);
}
fn pe_round(x: f32) -> f32 {
    return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);
}
fn pe_sinq(x: f32) -> f32 {
    let u = x * x;
    return x * ((((bitcast<f32>(0x391e4867u) * u + bitcast<f32>(0xbb991c14u)) * u
        + bitcast<f32>(0x3da333d8u)) * u + bitcast<f32>(0xbf255ddcu)) * u + bitcast<f32>(0x3fc90fdau));
}
fn pe_sin_turn(turns: f32) -> f32 {
    let u = turns - floor(turns + 0.5);
    let v = 4.0 * u;
    var w = v;
    if (v > 1.0) { w = 2.0 - v; } else if (v < -1.0) { w = -2.0 - v; }
    return pe_sinq(w);
}
fn pe_cos_turn(turns: f32) -> f32 {
    return pe_sin_turn(turns + 0.25);
}
fn pe_exp2(x: f32) -> f32 {
    let k = floor(x);
    let fr = x - k;
    let p = 1.0 + fr * (((((bitcast<f32>(0x395a68efu) * fr + bitcast<f32>(0x3aa65267u)) * fr
        + bitcast<f32>(0x3c1e2545u)) * fr + bitcast<f32>(0x3d634fb5u)) * fr
        + bitcast<f32>(0x3e75fe20u)) * fr + bitcast<f32>(0x3f317218u));
    let ki = i32(clamp(k, -126.0, 127.0));
    return p * bitcast<f32>(u32(ki + 127) << 23u);
}
fn pe_bounce_out(t: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;
    if (t < 1.0 / d1) { return n1 * t * t; }
    if (t < 2.0 / d1) {
        let u = t - 1.5 / d1;
        return n1 * u * u + 0.75;
    }
    if (t < 2.5 / d1) {
        let u = t - 2.25 / d1;
        return n1 * u * u + 0.9375;
    }
    let u = t - 2.625 / d1;
    return n1 * u * u + 0.984375;
}
fn pe_ease_in(curve: i32, t: f32, s0: vec4<f32>) -> f32 {
    if (curve == 1) { return t * t; }
    if (curve == 2) { return t * t * t; }
    if (curve == 3) { return t * t * t * t; }
    if (curve == 4) { return t * t * t * t * t; }
    if (curve == 5) { return 1.0 - sqrt(max(1.0 - t * t, 0.0)); }
    if (curve == 6) {
        let c1 = s0.x;
        let c3 = c1 + 1.0;
        return c3 * t * t * t - c1 * t * t;
    }
    if (curve == 7) { return 1.0 - pe_bounce_out(1.0 - t); }
    if (curve == 9) {
        if (t <= 0.0) { return 0.0; }
        if (t >= 1.0) { return 1.0; }
        return 1.0 - pe_sinq(1.0 - t);
    }
    if (curve == 10) {
        if (t <= 0.0) { return 0.0; }
        if (t >= 1.0) { return 1.0; }
        return pe_exp2(10.0 * t - 10.0);
    }
    if (curve == 11) {
        if (t <= 0.0) { return 0.0; }
        if (t >= 1.0) { return 1.0; }
        let p = max(s0.y, 0.05);
        let q = t - 1.0;
        return -(pe_exp2(10.0 * q) * pe_sin_turn(pe_div(q - p * 0.25, p)));
    }
    if (curve == 12) {
        if (t >= 1.0) { return 1.0; }
        let n = max(pe_round(s0.z), 1.0);
        return pe_div(floor(t * n), n);
    }
    return t;
}
fn pe_spring_x(tau: f32, damping: f32) -> f32 {
    let ln1000 = bitcast<f32>(0x40dd0c55u);
    let inv_ln2 = bitcast<f32>(0x3fb8aa3bu);
    let z = clamp(damping, 0.05, 4.0);
    let decay = pe_exp2(-ln1000 * tau * inv_ln2);
    if (z < 0.999) {
        let w = pe_div(ln1000, z);
        let wd = w * sqrt(1.0 - z * z);
        let turns = wd * tau * bitcast<f32>(0x3e22f983u);
        return 1.0 - decay * (pe_cos_turn(turns) + pe_div(ln1000, wd) * pe_sin_turn(turns));
    }
    if (z <= 1.001) {
        return 1.0 - decay * (1.0 + ln1000 * tau);
    }
    let r = sqrt(z * z - 1.0);
    let w = pe_div(ln1000, z - r);
    let r2 = -w * (z + r);
    let r1 = -ln1000;
    let e2 = pe_exp2(r2 * tau * inv_ln2);
    return 1.0 - pe_div(r2 * decay - r1 * e2, r2 - r1);
}
fn pe_spring(t: f32, damping: f32) -> f32 {
    if (t <= 0.0) { return 0.0; }
    if (t >= 1.0) { return 1.0; }
    return pe_div(pe_spring_x(t, damping), pe_spring_x(1.0, damping));
}
fn pe_bezier(t: f32, b: vec4<f32>) -> f32 {
    if (t <= 0.0) { return 0.0; }
    if (t >= 1.0) { return 1.0; }
    let x1 = clamp(b.x, 0.0, 1.0);
    let x2 = clamp(b.z, 0.0, 1.0);
    let ax = 1.0 + 3.0 * x1 - 3.0 * x2;
    let bx = 3.0 * x2 - 6.0 * x1;
    let cx = 3.0 * x1;
    var lo = 0.0;
    var hi = 1.0;
    for (var k = 0; k < 24; k = k + 1) {
        let mid = 0.5 * (lo + hi);
        let x = ((ax * mid + bx) * mid + cx) * mid;
        if (x < t) { lo = mid; } else { hi = mid; }
    }
    let s = 0.5 * (lo + hi);
    let ay = 1.0 + 3.0 * b.y - 3.0 * b.w;
    let by = 3.0 * b.w - 6.0 * b.y;
    let cy = 3.0 * b.y;
    return ((ay * s + by) * s + cy) * s;
}
fn pe_ease(curve: i32, dir: i32, t: f32, s0: vec4<f32>, s1: vec4<f32>) -> f32 {
    if (curve == 0 || curve == 8) { return t; }
    if (curve == 13) { return pe_spring(t, s0.w); }
    if (curve == 14) { return pe_bezier(t, s1); }
    if (dir == 1) { return 1.0 - pe_ease_in(curve, 1.0 - t, s0); }
    if (dir == 2) {
        if (t < 0.5) { return pe_ease_in(curve, 2.0 * t, s0) * 0.5; }
        return 1.0 - pe_ease_in(curve, 2.0 - 2.0 * t, s0) * 0.5;
    }
    return pe_ease_in(curve, t, s0);
}
fn pe_hash01(i: u32, seed: u32) -> f32 {
    var x = i * 2654435769u + seed * 2246822507u;
    x = x ^ (x >> 16u);
    x = x * 2146121005u;
    x = x ^ (x >> 15u);
    x = x * 2221713035u;
    x = x ^ (x >> 16u);
    return f32(x) / 4294967296.0;
}
fn pe_axis_dist(dx: f32, dy: f32, axis: i32) -> f32 {
    if (axis == 1) { return abs(dx); }
    if (axis == 2) { return abs(dy); }
    return sqrt(dx * dx + dy * dy);
}
fn pe_grid_cell(k: u32, cols: u32) -> vec2<f32> {
    return vec2<f32>(f32(k % cols), f32(k / cols));
}
fn pe_grid_dist(c: vec2<f32>, o: vec2<f32>, axis: i32) -> f32 {
    return pe_axis_dist(c.x - o.x, c.y - o.y, axis);
}
fn pe_grid_raw(cols_in: u32, partida: i32, cell: u32, axis: i32, i: u32, n: u32) -> f32 {
    let cols = max(cols_in, 1u);
    let rows = (n + cols - 1u) / cols;
    var o = vec2<f32>(0.0, 0.0);
    if (partida == 1) { o = vec2<f32>(f32(cols - 1u) * 0.5, f32(rows - 1u) * 0.5); }
    else if (partida == 2) { o = pe_grid_cell(n - 1u, cols); }
    else if (partida == 3) { o = pe_grid_cell(min(cell, n - 1u), cols); }
    let last_row = f32(rows - 1u);
    let last_col = f32((n - 1u) % cols);
    var largura = n;
    if (rows > 1u) { largura = cols; }
    var dmax = max(max(max(pe_grid_dist(vec2<f32>(0.0, 0.0), o, axis),
        pe_grid_dist(vec2<f32>(f32(largura - 1u), 0.0), o, axis)),
        pe_grid_dist(vec2<f32>(0.0, last_row), o, axis)),
        pe_grid_dist(vec2<f32>(last_col, last_row), o, axis));
    if (rows > 1u) {
        dmax = max(dmax, pe_grid_dist(vec2<f32>(f32(cols - 1u), last_row - 1.0), o, axis));
    }
    if (dmax > 0.0) { return pe_div(pe_grid_dist(pe_grid_cell(i, cols), o, axis), dmax); }
    return 0.0;
}
fn pe_raw_at(kind: i32, seed: u32, cols: u32, partida: i32, cell: u32, axis: i32,
             origin: vec2<f32>, reach: f32, i: u32, n: u32, p: vec2<f32>) -> f32 {
    if (kind == 4) {
        let d = pe_axis_dist(p.x - origin.x, p.y - origin.y, axis);
        if (reach > 0.0) { return min(pe_div(d, reach), 1.0); }
        return 0.0;
    }
    if (n <= 1u) { return 0.0; }
    let last = f32(n) - 1.0;
    if (kind == 1) { return pe_div(abs(2.0 * f32(i) - last), last); }
    if (kind == 2) { return pe_hash01(i, seed); }
    if (kind == 3) { return pe_grid_raw(cols, partida, cell, axis, i, n); }
    return pe_div(f32(i), last);
}
"
    };
}
