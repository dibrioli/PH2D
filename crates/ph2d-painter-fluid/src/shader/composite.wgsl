// Wet-field GPU composite — the band-for-band mirror of
// `ph2d_painter_brush::wet_composite::composite_wet_field_cpu` (the parity ground
// truth). Composites the low-res live diffusion pigment over a backdrop into the
// canvas with a Kubelka–Munk subtractive glaze (ADR-0049 / ADR-0077 D12), so the
// per-frame composite runs on the GPU with NO pigment readback (the prior stall).
//
// One thread per canvas pixel (dispatched over the wet bbox via `origin`). Pigment
// is `vec4<f32>` (xyz mass, std430 vec3-stride trap avoided). Backdrop + output are
// packed RGBA8 (`array<u32>`, unpack/pack4x8unorm), matching the CPU byte path.
//
// ## Exact (no-LUT) spectral mix
// The CPU LUT is a per-pixel cache the GPU's parallel ALUs don't need — here
// `to_reflectance`/`reflectance_to_rgb` run inline, mirroring `mix_prepared_exact`.
// The constant basis (`base[7][24]` + `m[3][24]`) and the amortised brush coeffs
// (`ks[24]` + `err`) are uploaded once per composite as a storage buffer.

const NB: u32 = 24u;
const REFL_FLOOR: f32 = 1.0e-4;

struct U {
    cw: u32,          // canvas width
    ch: u32,          // canvas height
    gw: u32,          // grid (low-res) width
    gh: u32,          // grid (low-res) height
    inv: f32,         // 1 / scale
    color_sum: f32,   // coverage normaliser
    coverage_k: f32,  // density→alpha rate (WET_COVERAGE_K)
    _pad0: f32,
    origin_x: u32,    // dispatch origin in canvas px (bbox top-left)
    origin_y: u32,
    end_x: u32,       // exclusive bbox right/bottom (= CPU loop bound px_hi/py_hi)
    end_y: u32,
    pcol: vec4<f32>,  // xyz = pigment chromaticity = brush colour (mix endpoint)
};

// Amortised, constant-per-composite coefficients. `base`/`m` are the spectral basis
// (constant); `ks`/`err` are the prepared brush side. Flattened: base[k*NB+i],
// m[c*NB+i]. std430 array<f32> is tightly packed (stride 4).
struct Coeffs {
    base: array<f32, 168>, // 7 * 24
    m: array<f32, 72>,     // 3 * 24
    ks: array<f32, 24>,
    err: vec4<f32>,        // xyz used
};

@group(0) @binding(0) var<uniform> P: U;
@group(0) @binding(1) var<storage, read> pig_in: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> backdrop: array<u32>;
@group(0) @binding(3) var<storage, read_write> out_buf: array<u32>;
@group(0) @binding(4) var<storage, read> C: Coeffs;

// ─── sRGB transfer (IEC 61966) — float twins of ph2d_color::srgb byte fns ───
fn srgb_to_linear(v: f32) -> f32 {
    if (v <= 0.04045) {
        return v / 12.92;
    }
    return pow((v + 0.055) / 1.055, 2.4);
}
fn linear_to_srgb(linear: f32) -> f32 {
    let v = clamp(linear, 0.0, 1.0);
    if (v <= 0.0031308) {
        return v * 12.92;
    }
    return 1.055 * pow(v, 1.0 / 2.4) - 0.055;
}

// ─── Catmull-Rom bicubic upsample of the low-res pigment ───
fn catmull_rom(t: f32) -> vec4<f32> {
    let t2 = t * t;
    let t3 = t2 * t;
    return vec4<f32>(
        -0.5 * t3 + t2 - 0.5 * t,
        1.5 * t3 - 2.5 * t2 + 1.0,
        -1.5 * t3 + 2.0 * t2 + 0.5 * t,
        0.5 * t3 - 0.5 * t2,
    );
}
fn sample_pigment_bicubic(fx: f32, fy: f32) -> vec3<f32> {
    let x0 = floor(fx);
    let y0 = floor(fy);
    let wx = catmull_rom(fx - x0);
    let wy = catmull_rom(fy - y0);
    let gwi = i32(P.gw);
    let ghi = i32(P.gh);
    var out = vec3<f32>(0.0);
    for (var j = 0; j < 4; j = j + 1) {
        let gy = clamp(i32(y0) - 1 + j, 0, ghi - 1);
        var row = vec3<f32>(0.0);
        for (var i = 0; i < 4; i = i + 1) {
            let gx = clamp(i32(x0) - 1 + i, 0, gwi - 1);
            let p = pig_in[u32(gy) * P.gw + u32(gx)].xyz;
            row = row + p * wx[i];
        }
        out = out + row * wy[j];
    }
    return max(out, vec3<f32>(0.0));
}

// ─── Kubelka–Munk spectral mix (mirror of pigment_mix::mix_prepared_exact) ───
fn rgb_to_weights(rgb: vec3<f32>) -> array<f32, 7> {
    var r = clamp(rgb.x, 0.0, 1.0);
    var g = clamp(rgb.y, 0.0, 1.0);
    var b = clamp(rgb.z, 0.0, 1.0);
    let wv = min(r, min(g, b));
    r = r - wv;
    g = g - wv;
    b = b - wv;
    var c = 0.0;
    var m = 0.0;
    var y = 0.0;
    var rr = 0.0;
    var gg = 0.0;
    var bb = 0.0;
    if (r <= g && r <= b) {
        c = min(g, b);
        gg = g - c;
        bb = b - c;
    } else if (g <= r && g <= b) {
        m = min(r, b);
        rr = r - m;
        bb = b - m;
    } else {
        y = min(r, g);
        rr = r - y;
        gg = g - y;
    }
    return array<f32, 7>(wv, c, m, y, rr, gg, bb);
}
fn to_reflectance(rgb: vec3<f32>) -> array<f32, 24> {
    let wt = rgb_to_weights(rgb);
    var refl: array<f32, 24>;
    for (var i = 0u; i < NB; i = i + 1u) {
        var v = 0.0;
        for (var k = 0u; k < 7u; k = k + 1u) {
            v = v + wt[k] * C.base[k * NB + i];
        }
        refl[i] = clamp(v, REFL_FLOOR, 1.0);
    }
    return refl;
}
fn reflectance_to_rgb(refl: array<f32, 24>) -> vec3<f32> {
    var rgb = vec3<f32>(0.0);
    for (var i = 0u; i < NB; i = i + 1u) {
        let r = refl[i];
        rgb.x = rgb.x + C.m[i] * r;            // m[0*NB+i]
        rgb.y = rgb.y + C.m[NB + i] * r;       // m[1*NB+i]
        rgb.z = rgb.z + C.m[2u * NB + i] * r;  // m[2*NB+i]
    }
    return max(rgb, vec3<f32>(0.0));
}
fn refl_to_ks(r_in: f32) -> f32 {
    let r = clamp(r_in, REFL_FLOOR, 1.0);
    return (1.0 - r) * (1.0 - r) / (2.0 * r);
}
fn ks_to_refl(k_in: f32) -> f32 {
    let k = max(k_in, 0.0);
    return clamp(1.0 + k - sqrt(k * k + 2.0 * k), 0.0, 1.0);
}
// K–M mix of backdrop `a` toward the prepared brush (`P.pcol` + `C.ks` + `C.err`)
// at concentration `t` — exact (LUT-free) twin of `mix_prepared_exact`.
fn mix_prepared_exact(a: vec3<f32>, t_in: f32) -> vec3<f32> {
    let t = clamp(t_in, 0.0, 1.0);
    let b = P.pcol.xyz;
    let lin = a + (b - a) * t;
    let w = 4.0 * t * (1.0 - t);
    if (w < 0.02) {
        return lin;
    }
    let refl_a = to_reflectance(a);
    let rt_a = reflectance_to_rgb(refl_a);
    var rm: array<f32, 24>;
    for (var i = 0u; i < NB; i = i + 1u) {
        let ks_mix = (1.0 - t) * refl_to_ks(refl_a[i]) + t * C.ks[i];
        rm[i] = ks_to_refl(ks_mix);
    }
    let mixed = reflectance_to_rgb(rm);
    let ea = a - rt_a;
    let spec = max(mixed + ea * (1.0 - t) + C.err.xyz * t, vec3<f32>(0.0));
    return lin + (spec - lin) * w;
}

@compute @workgroup_size(8, 8, 1)
fn cs_composite(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cx = P.origin_x + gid.x;
    let cy = P.origin_y + gid.y;
    // Guard the bbox extent (end_x/end_y ≤ cw/ch), so a padded dispatch never
    // composites a pixel the CPU loop (px_lo..px_hi) leaves as the backdrop.
    if (cx >= P.end_x || cy >= P.end_y) {
        return;
    }
    let pix = cy * P.cw + cx;
    let fx = clamp((f32(cx) + 0.5) * P.inv - 0.5, 0.0, f32(P.gw) - 1.0);
    let fy = clamp((f32(cy) + 0.5) * P.inv - 0.5, 0.0, f32(P.gh) - 1.0);
    let p = sample_pigment_bicubic(fx, fy);
    let dens = max(p.x + p.y + p.z, 0.0);
    if (dens < 1.0e-4) {
        out_buf[pix] = backdrop[pix]; // byte-exact backdrop copy (bare paper)
        return;
    }
    let amount = dens / P.color_sum;
    let alpha = 1.0 - exp(-amount * P.coverage_k);
    let back_rgba = unpack4x8unorm(backdrop[pix]); // (r,g,b,a) in [0,1]
    let back_a = back_rgba.w;
    let back = vec3<f32>(
        srgb_to_linear(back_rgba.x),
        srgb_to_linear(back_rgba.y),
        srgb_to_linear(back_rgba.z),
    );
    let out_a = alpha + back_a * (1.0 - alpha);
    let km = mix_prepared_exact(back, alpha);
    var inv_a = 0.0;
    if (out_a > 1.0e-4) {
        inv_a = 1.0 / out_a;
    }
    let pcol = P.pcol.xyz;
    let straight = (pcol * alpha + back * back_a * (1.0 - alpha)) * inv_a;
    let rgb = clamp(straight + (km - straight) * back_a, vec3<f32>(0.0), vec3<f32>(1.0));
    let enc = vec3<f32>(linear_to_srgb(rgb.x), linear_to_srgb(rgb.y), linear_to_srgb(rgb.z));
    out_buf[pix] = pack4x8unorm(vec4<f32>(enc, out_a));
}
