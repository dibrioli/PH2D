//! A MESMA mistura de [`crate::perceptual`] em WGSL, operação a operação (prefixo `pc_`). ⚠️ Lê
//! as funções `pe_*` da [`ph2d_ease::wgsl_lib!`] — o kernel compõe
//! `concat!(ph2d_ease::wgsl_lib!(), ph2d_color::wgsl_perceptual!(), …)`.
//! `pc_mix(a, b, t, espaco)`: `espaco` é a escada de [`crate::perceptual::MixSpace`].
//!
//! ⚠️ Gerado das tabelas por bits de [`crate::perceptual`]; o gate
//! `the_wgsl_carries_the_same_matrix_bits` confere cada uma.

/// O texto WGSL da mistura perceptual.
#[macro_export]
macro_rules! wgsl_perceptual {
    () => {
        "fn pc_to_oklab(c: vec3<f32>) -> vec3<f32> {
    let lms = vec3<f32>(pe_cbrt(bitcast<f32>(0x3ed30eb1u) * c.x + bitcast<f32>(0x3f094d16u) * c.y + bitcast<f32>(0x3d52b909u) * c.z), pe_cbrt(bitcast<f32>(0x3e58fd3bu) * c.x + bitcast<f32>(0x3f2e4253u) * c.y + bitcast<f32>(0x3ddbf2f0u) * c.z), pe_cbrt(bitcast<f32>(0x3db4d7ecu) * c.x + bitcast<f32>(0x3e903d74u) * c.y + bitcast<f32>(0x3f214649u) * c.z));
    return vec3<f32>(bitcast<f32>(0x3e578152u) * lms.x + bitcast<f32>(0x3f4b2a89u) * lms.y + bitcast<f32>(0xbb856eceu) * lms.z, bitcast<f32>(0x3ffd2f0eu) * lms.x + bitcast<f32>(0xc01b6e0eu) * lms.y + bitcast<f32>(0x3ee6b438u) * lms.z, bitcast<f32>(0x3cd434b4u) * lms.x + bitcast<f32>(0x3f4863bbu) * lms.y + bitcast<f32>(0xbf4f0560u) * lms.z);
}
fn pc_from_oklab(o: vec3<f32>) -> vec3<f32> {
    let v0 = o.x + bitcast<f32>(0x3ecaecc9u) * o.y + bitcast<f32>(0x3e5cfba9u) * o.z;
    let v1 = o.x + bitcast<f32>(0xbdd8308cu) * o.y + bitcast<f32>(0xbd82c5fau) * o.z;
    let v2 = o.x + bitcast<f32>(0xbdb7437bu) * o.y + bitcast<f32>(0xbfa54f66u) * o.z;
    let lms = vec3<f32>(v0 * v0 * v0, v1 * v1 * v1, v2 * v2 * v2);
    return vec3<f32>(bitcast<f32>(0x408274abu) * lms.x + bitcast<f32>(0xc053b18cu) * lms.y + bitcast<f32>(0x3e6c8362u) * lms.z, bitcast<f32>(0xbfa25c2du) * lms.x + bitcast<f32>(0x40270644u) * lms.y + bitcast<f32>(0xbeaec16au) * lms.z, bitcast<f32>(0xbb897f53u) * lms.x + bitcast<f32>(0xbf34133eu) * lms.y + bitcast<f32>(0x3fda931eu) * lms.z);
}
fn pc_oklab_to_oklch(o: vec3<f32>) -> vec3<f32> {
    let c = sqrt(o.y * o.y + o.z * o.z);
    let h = pe_atan_turn(o.z, o.y);
    return vec3<f32>(o.x, c, h - floor(h));
}
fn pc_oklch_to_oklab(p: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(p.x, p.y * pe_cos_turn(p.z), p.y * pe_sin_turn(p.z));
}
fn pc_na_gama(c: vec3<f32>) -> bool {
    return c.x >= 0.0 && c.x <= 1.0 && c.y >= 0.0 && c.y <= 1.0 && c.z >= 0.0 && c.z <= 1.0;
}
fn pc_corta(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(clamp(c.x, 0.0, 1.0), clamp(c.y, 0.0, 1.0), clamp(c.z, 0.0, 1.0));
}
fn pc_distancia(a: vec3<f32>, b: vec3<f32>) -> f32 {
    let d = vec3<f32>(a.x - b.x, a.y - b.y, a.z - b.z);
    return sqrt(d.x * d.x + d.y * d.y + d.z * d.z);
}
fn pc_gamut_map(lch: vec3<f32>) -> vec3<f32> {
    let jnd = bitcast<f32>(0x3ca3d70au);
    let resolucao = bitcast<f32>(0x358637bdu);
    let rgb = pc_from_oklab(pc_oklch_to_oklab(lch));
    if (pc_na_gama(rgb)) { return rgb; }
    if (lch.x >= 1.0) { return vec3<f32>(1.0, 1.0, 1.0); }
    if (lch.x <= 0.0) { return vec3<f32>(0.0, 0.0, 0.0); }
    var corrente = lch;
    var cortado = pc_corta(rgb);
    if (pc_distancia(pc_to_oklab(cortado), pc_oklch_to_oklab(corrente)) < jnd) { return cortado; }
    var lo = 0.0;
    var hi = lch.y;
    var lo_na_gama = true;
    while (hi - lo > resolucao) {
        let croma = (lo + hi) * 0.5;
        corrente.y = croma;
        let r = pc_from_oklab(pc_oklch_to_oklab(corrente));
        if (lo_na_gama && pc_na_gama(r)) {
            lo = croma;
            continue;
        }
        cortado = pc_corta(r);
        let e = pc_distancia(pc_to_oklab(cortado), pc_oklch_to_oklab(corrente));
        if (e < jnd) {
            lo_na_gama = false;
            lo = croma;
        } else {
            hi = croma;
        }
    }
    return cortado;
}
fn pc_matiz(h1_in: f32, c1: f32, h2_in: f32, c2: f32, t: f32, longa: bool) -> f32 {
    let sem = bitcast<f32>(0x3951b717u);
    var h1 = h1_in;
    var h2 = h2_in;
    if (c1 < sem && !(c2 < sem)) { h1 = h2_in; h2 = h2_in; }
    else if (!(c1 < sem) && c2 < sem) { h1 = h1_in; h2 = h1_in; }
    var d = h2 - h1;
    if (longa) {
        if (d >= 0.0 && d < 0.5) { d = d - 1.0; } else if (d >= -0.5 && d <= 0.0) { d = d + 1.0; }
    } else {
        if (d > 0.5) { d = d - 1.0; } else if (d < -0.5) { d = d + 1.0; }
    }
    let h = h1 + d * t;
    return h - floor(h);
}
fn pc_mix(a: vec4<f32>, b: vec4<f32>, t: f32, espaco: i32) -> vec4<f32> {
    let alfa = a.w + (b.w - a.w) * t;
    if (espaco == 0) {
        return vec4<f32>(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, a.z + (b.z - a.z) * t, alfa);
    }
    let p = pc_to_oklab(a.xyz);
    let q = pc_to_oklab(b.xyz);
    var lch: vec3<f32>;
    if (espaco == 1) {
        lch = pc_oklab_to_oklch(vec3<f32>(p.x + (q.x - p.x) * t, p.y + (q.y - p.y) * t, p.z + (q.z - p.z) * t));
    } else {
        let pl = pc_oklab_to_oklch(p);
        let ql = pc_oklab_to_oklch(q);
        lch = vec3<f32>(pl.x + (ql.x - pl.x) * t, pl.y + (ql.y - pl.y) * t,
            pc_matiz(pl.z, pl.y, ql.z, ql.y, t, espaco == 3));
    }
    let rgb = pc_gamut_map(lch);
    return vec4<f32>(rgb, alfa);
}
"
    };
}
