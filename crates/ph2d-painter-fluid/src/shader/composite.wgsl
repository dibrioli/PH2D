// Wet-field GPU composite — the band-for-band mirror of
// `ph2d_painter_brush::wet_composite::composite_wet_field_cpu` (the parity ground
// truth). Composites the low-res live diffusion pigment over a backdrop into the
// canvas with a Kubelka–Munk subtractive glaze (ADR-0049 / ADR-0077 D12 / ADR-0080),
// so the per-frame composite runs on the GPU with NO pigment readback.
//
// One thread per canvas pixel (dispatched over the wet bbox via `origin`).
//
// ## ADR-0080 multi-pigment field
// Pigment is **PV (=8) `vec4<f32>` per cell = 32 channels (+ stain)**: 24 mass-weighted K/S
// bands + 3 err + 1 mass (vec4[6] = (err.xyz, mass)) + 1 stain (vec4[7].x) + 3 pad; the
// reduction ignores stain/pad (they're behaviour, not colour). Per pixel the bicubic-sampled
// field is REDUCED to a mixed pigment — `ks_mix = ks_acc/mass`, `err_mix = err_acc/mass`,
// `colour = reflectance_to_rgb(ks_to_refl(ks_mix)) + err_mix` — so overlapping pigments
// mix subtractively (blue+yellow→green). The brush side of the glaze is therefore
// PER-PIXEL (no per-stroke `pcol`/`color_sum`/`ks`/`err` uniform); value-opacity
// (ADR-0079) reads the mixed colour's value. A single pigment reduces to exactly its
// own `prepare_pigment`, so single-colour matches the pre-ADR-0080 path to float precision.
//
// Backdrop + output are packed RGBA8 (`array<u32>`, unpack/pack4x8unorm). The constant
// spectral basis (`base[7][24]` + `m[3][24]`) is uploaded once as a storage buffer.

const NB: u32 = 24u;
const PV: u32 = 8u;
const REFL_FLOOR: f32 = 1.0e-4;

struct U {
    cw: u32,          // canvas width
    ch: u32,          // canvas height
    gw: u32,          // grid (low-res) width
    gh: u32,          // grid (low-res) height
    inv: f32,         // 1 / scale
    coverage_k: f32,  // density→alpha rate (WET_COVERAGE_K)
    ss: u32,          // coverage supersampling factor N (N×N); 1 at full-res
    origin_x: u32,    // dispatch origin in canvas px (bbox top-left)
    origin_y: u32,
    end_x: u32,       // exclusive bbox right/bottom (= CPU loop bound px_hi/py_hi)
    end_y: u32,
    _pad: u32,
};

// The constant spectral basis. Flattened: base[k*NB+i], m[c*NB+i]. std430 array<f32>
// is tightly packed (stride 4).
struct Coeffs {
    base: array<f32, 168>, // 7 * 24
    m: array<f32, 72>,     // 3 * 24
};

@group(0) @binding(0) var<uniform> P: U;
@group(0) @binding(1) var<storage, read> pig_in: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> backdrop: array<u32>;
@group(0) @binding(3) var<storage, read_write> out_buf: array<u32>;
@group(0) @binding(4) var<storage, read> C: Coeffs;
// ADR-0084 backdrop-lift accumulator (low-res `gw·gh`, one f32 per grid cell). Where a wet brush
// lifted dry paint out of the backdrop, the backdrop returns toward the PAPER by `lf` (the
// paper-reveal lerp below). The solver's `cs_lift` writes it; when the lift is dormant it's
// all-zero ⇒ `lf = 0` ⇒ `eff_back = back` ⇒ byte-identical output (the non-destructive
// `lift = 0` path, ADR-0084 §2.4).
@group(0) @binding(5) var<storage, read> lifted_frac: array<f32>;
// ADR-0084 paper-reveal: the session's ORIGINAL canvas content (packed RGBA8, canvas-res, same
// layout as `backdrop`). Lifting reveals the paper, never transparency; `paper == backdrop` ⇒
// `mix(back, paper, lf) = back` ⇒ exact no-op regardless of `lf` (the dormant/one-shot binding).
@group(0) @binding(6) var<storage, read> paper_canvas: array<u32>;

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

// ─── Catmull-Rom bicubic upsample of the low-res pigment field (all PV channels) ───
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
// Returns the PV bicubic-sampled vec4 (extensive K/S + err + mass), floored: K/S bands
// (vec4[0..5]) + mass (vec4[6].w) ≥ 0; err (vec4[6].xyz) stays signed. Mirrors the CPU
// `sample_pigment_bicubic`.
fn sample_field_bicubic(fx: f32, fy: f32) -> array<vec4<f32>, 8> {
    let x0 = floor(fx);
    let y0 = floor(fy);
    let wx = catmull_rom(fx - x0);
    let wy = catmull_rom(fy - y0);
    let gwi = i32(P.gw);
    let ghi = i32(P.gh);
    var out: array<vec4<f32>, 8>;
    for (var v = 0u; v < PV; v = v + 1u) {
        out[v] = vec4<f32>(0.0);
    }
    for (var j = 0; j < 4; j = j + 1) {
        let gy = clamp(i32(y0) - 1 + j, 0, ghi - 1);
        var row: array<vec4<f32>, 8>;
        for (var v = 0u; v < PV; v = v + 1u) {
            row[v] = vec4<f32>(0.0);
        }
        for (var i = 0; i < 4; i = i + 1) {
            let gx = clamp(i32(x0) - 1 + i, 0, gwi - 1);
            let base = (u32(gy) * P.gw + u32(gx)) * PV;
            for (var v = 0u; v < PV; v = v + 1u) {
                row[v] = row[v] + pig_in[base + v] * wx[i];
            }
        }
        for (var v = 0u; v < PV; v = v + 1u) {
            out[v] = out[v] + row[v] * wy[j];
        }
    }
    for (var v = 0u; v < 6u; v = v + 1u) {
        out[v] = max(out[v], vec4<f32>(0.0)); // K/S bands ≥ 0
    }
    out[6] = vec4<f32>(out[6].xyz, max(out[6].w, 0.0)); // err signed, mass ≥ 0
    return out;
}

// ─── Kubelka–Munk spectral basis (mirror of pigment_mix) ───
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
// K–M mix of backdrop `a` toward the (per-pixel) prepared brush at concentration `t` —
// exact (LUT-free) twin of `mix_prepared_exact`. `b_color`/`b_ks`/`b_err` are the mixed
// pigment reduced from the field (ADR-0080), the brush side of the glaze.
fn mix_prepared_exact(a: vec3<f32>, t_in: f32, b_color: vec3<f32>, b_ks: array<f32, 24>, b_err: vec3<f32>) -> vec3<f32> {
    let t = clamp(t_in, 0.0, 1.0);
    let lin = a + (b_color - a) * t;
    let w = 4.0 * t * (1.0 - t);
    if (w < 0.02) {
        return lin;
    }
    let refl_a = to_reflectance(a);
    let rt_a = reflectance_to_rgb(refl_a);
    var rm: array<f32, 24>;
    for (var i = 0u; i < NB; i = i + 1u) {
        let ks_mix = (1.0 - t) * refl_to_ks(refl_a[i]) + t * b_ks[i];
        rm[i] = ks_to_refl(ks_mix);
    }
    let mixed = reflectance_to_rgb(rm);
    let ea = a - rt_a;
    let spec = max(mixed + ea * (1.0 - t) + b_err * t, vec3<f32>(0.0));
    return lin + (spec - lin) * w;
}

// One glaze sub-sample at grid coords (fx,fy) over the linear backdrop. `dry` is set
// when the cell is bare (`mass < 1e-4`); the caller treats that as a backdrop sample.
struct Sub { rgb: vec3<f32>, a: f32, dry: bool }
fn glaze_sample(fx: f32, fy: f32, back: vec3<f32>, back_a: f32) -> Sub {
    let cell = sample_field_bicubic(fx, fy);
    let mass = cell[6].w;
    if (mass < 1.0e-4) {
        return Sub(back, back_a, true);
    }
    // Reduce the field cell to the mixed pigment (ADR-0080): ks_mix = ks_acc/mass etc.
    let inv = 1.0 / mass;
    var ks_mix: array<f32, 24>;
    for (var b = 0u; b < 6u; b = b + 1u) {
        let vv = cell[b] * inv;
        ks_mix[b * 4u + 0u] = vv.x;
        ks_mix[b * 4u + 1u] = vv.y;
        ks_mix[b * 4u + 2u] = vv.z;
        ks_mix[b * 4u + 3u] = vv.w;
    }
    var refl: array<f32, 24>;
    for (var i = 0u; i < NB; i = i + 1u) {
        refl[i] = ks_to_refl(ks_mix[i]);
    }
    let err_mix = cell[6].xyz * inv;
    let pcol = max(reflectance_to_rgb(refl) + err_mix, vec3<f32>(0.0));
    // ADR-0079 value-opacity from the MIXED colour's value. Floor 0.55 (re-tune 2026-06-09):
    // the old 0.3 made dark pigments' lum(mass) near-binary, rendering the rim's per-cell
    // texture as hard pixel teeth — mirrors `VALUE_OPACITY_FLOOR` in `wet_composite.rs`.
    let value = clamp(max(pcol.x, max(pcol.y, pcol.z)), 0.0, 1.0);
    let color_sum = 0.55 + 0.45 * value;
    let amount = mass / color_sum;
    let alpha = 1.0 - exp(-amount * P.coverage_k);
    let out_a = alpha + back_a * (1.0 - alpha);
    let km = mix_prepared_exact(back, alpha, pcol, ks_mix, err_mix);
    var inv_a = 0.0;
    if (out_a > 1.0e-4) {
        inv_a = 1.0 / out_a;
    }
    let straight = (pcol * alpha + back * back_a * (1.0 - alpha)) * inv_a;
    let rgb = clamp(straight + (km - straight) * back_a, vec3<f32>(0.0), vec3<f32>(1.0));
    return Sub(rgb, out_a, false);
}

@compute @workgroup_size(8, 8, 1)
fn cs_composite(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cx = P.origin_x + gid.x;
    let cy = P.origin_y + gid.y;
    if (cx >= P.end_x || cy >= P.end_y) {
        return;
    }
    let pix = cy * P.cw + cx;
    let back_rgba = unpack4x8unorm(backdrop[pix]); // (r,g,b,a) in [0,1]
    let back = vec3<f32>(
        srgb_to_linear(back_rgba.x),
        srgb_to_linear(back_rgba.y),
        srgb_to_linear(back_rgba.z),
    );
    // ADR-0084 — sample `lifted_frac` at the grid cell under the pixel centre (nearest, same
    // low-res mapping the glaze uses).
    let lf_x = u32(clamp(round((f32(cx) + 0.5) * P.inv - 0.5), 0.0, f32(P.gw) - 1.0));
    let lf_y = u32(clamp(round((f32(cy) + 0.5) * P.inv - 0.5), 0.0, f32(P.gh) - 1.0));
    let lf = clamp(lifted_frac[lf_y * P.gw + lf_x], 0.0, 1.0);
    let paper_rgba = unpack4x8unorm(paper_canvas[pix]);
    // ADR-0084 paper-reveal: a lifted pixel returns toward the session's original paper content
    // (Curtis desorption / Rebelle), NEVER toward transparency (alpha holes revealed the dark
    // editor background behind an opaque canvas — the dark-blur smoke). lf = 0 ⇒ untouched.
    let paper_lin = vec3<f32>(
        srgb_to_linear(paper_rgba.x),
        srgb_to_linear(paper_rgba.y),
        srgb_to_linear(paper_rgba.z),
    );
    let eff_back = mix(back, paper_lin, lf); // linear-light rgb
    let eff_back_a = mix(back_rgba.w, paper_rgba.w, lf);
    let ss = max(P.ss, 1u);
    let inv_n = 1.0 / f32(ss);
    var acc_rgb = vec3<f32>(0.0);
    var acc_a = 0.0;
    var any_wet = false;
    for (var sy = 0u; sy < ss; sy = sy + 1u) {
        let fy = clamp((f32(cy) + (f32(sy) + 0.5) * inv_n) * P.inv - 0.5, 0.0, f32(P.gh) - 1.0);
        for (var sx = 0u; sx < ss; sx = sx + 1u) {
            let fx = clamp((f32(cx) + (f32(sx) + 0.5) * inv_n) * P.inv - 0.5, 0.0, f32(P.gw) - 1.0);
            let s = glaze_sample(fx, fy, eff_back, eff_back_a);
            if (!s.dry) {
                any_wet = true;
            }
            acc_rgb = acc_rgb + s.rgb * s.a;
            acc_a = acc_a + s.a;
        }
    }
    if (!any_wet) {
        // Bare paper. The byte-exact backdrop copy must NOT fire when the backdrop was lifted here
        // (else the reveal is ignored) — guard it with `lf <= 1e-6` (ADR-0084 §2.3). When `lf > 0`
        // but no wet field, emit the paper-reveal lerp re-encoded (same per-channel encode as the
        // wet path below).
        if (lf <= 1.0e-6) {
            out_buf[pix] = backdrop[pix]; // byte-exact backdrop copy (bare paper)
        } else {
            let dry_enc = vec3<f32>(
                linear_to_srgb(eff_back.x),
                linear_to_srgb(eff_back.y),
                linear_to_srgb(eff_back.z),
            );
            out_buf[pix] = pack4x8unorm(vec4<f32>(dry_enc, eff_back_a));
        }
        return;
    }
    let final_a = acc_a * inv_n * inv_n;
    var rgb = vec3<f32>(0.0);
    if (acc_a > 1.0e-6) {
        rgb = acc_rgb / acc_a;
    }
    let enc = vec3<f32>(linear_to_srgb(rgb.x), linear_to_srgb(rgb.y), linear_to_srgb(rgb.z));
    out_buf[pix] = pack4x8unorm(vec4<f32>(enc, final_a));
}

// ─── E4: premultiplied preview-texture output (ADR-0078 S2 / fluid-GPU plan §4 E4) ───
//
// Kills the live-preview round-trip (GPU composite → staging readback → CPU premultiply
// → texture re-upload): after `cs_composite` writes the straight-alpha sRGB8 glaze into
// `out_buf`, `cs_premul_tex` premultiplies the SAME region straight into a storage
// texture the sprite renderer samples directly. The texture lives in group(1) (its own
// tiny bind-group layout) so the long-lived group(0) buffer layout — and every existing
// readback path that binds it — is untouched.
@group(1) @binding(0) var preview_tex: texture_storage_2d<rgba8unorm, write>;

// Premultiply one packed RGBA8 word with the EXACT byte semantics of the CPU path the
// shell uses (`shells/desktop/.../painter_bridge.rs` → `ph2d_render::premultiply_rgba8`):
//   rgb' = (rgb · a + 127) / 255   — integer round-to-nearest on the sRGB-ENCODED bytes
//                                    (NO linearisation), alpha unchanged.
// Done in u32 integer math so the GPU texel is byte-identical to the CPU result: the
// quotient `p ∈ [0,255]` is exact; `f32(p)/255` is within 1 ulp of the true ratio, so
// the rgba8unorm `textureStore` conversion (round(clamp(v)·255)) lands back on `p` for
// every input — proven byte-for-byte by `gpu_preview_texture_matches_cpu_premultiply`.
fn premul_word(word: u32) -> vec4<f32> {
    let r = word & 0xffu;
    let g = (word >> 8u) & 0xffu;
    let b = (word >> 16u) & 0xffu;
    let a = (word >> 24u) & 0xffu;
    return vec4<f32>(
        f32((r * a + 127u) / 255u),
        f32((g * a + 127u) / 255u),
        f32((b * a + 127u) / 255u),
        f32(a),
    ) / 255.0;
}

// Region-scoped like `cs_composite` (same origin/end uniforms, dispatched right after it
// in the same pass — WebGPU's implicit inter-dispatch sync makes the fresh `out_buf`
// bytes visible). Pixels outside the per-frame region keep whatever the texture holds
// (the premultiplied backdrop from `cs_premul_init`, or earlier frames of the stroke).
@compute @workgroup_size(8, 8, 1)
fn cs_premul_tex(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cx = P.origin_x + gid.x;
    let cy = P.origin_y + gid.y;
    if (cx >= P.end_x || cy >= P.end_y) {
        return;
    }
    textureStore(preview_tex, vec2<i32>(i32(cx), i32(cy)), premul_word(out_buf[cy * P.cw + cx]));
}

// Stroke-start texture init: fill the region (the full canvas — `begin_stroke` sets
// origin = 0, end = cw/ch) with the PREMULTIPLIED backdrop, entirely on the GPU. The
// per-frame regions only cover the wet band, so pixels never composited must already
// show the backdrop when the renderer samples the texture.
@compute @workgroup_size(8, 8, 1)
fn cs_premul_init(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cx = P.origin_x + gid.x;
    let cy = P.origin_y + gid.y;
    if (cx >= P.end_x || cy >= P.end_y) {
        return;
    }
    textureStore(preview_tex, vec2<i32>(i32(cx), i32(cy)), premul_word(backdrop[cy * P.cw + cx]));
}

// ─── E5: STRAIGHT-alpha texture output (ADR-0078 S2, multi-layer chain) ───
//
// The GPU layer compositor (`ph2d_render::LayerCompositor`) consumes STRAIGHT
// sRGB8 slices (premultiply happens once at the END of the layer chain, via
// `PreviewPremul`). These two entries mirror `cs_premul_tex`/`cs_premul_init`
// MINUS the premultiply — `unpack4x8unorm` → `textureStore` on rgba8unorm is an
// exact byte round-trip (v/255 is representable; the store rounds back to v),
// so the texel is byte-identical to the `out_buf`/`backdrop` word. They bind a
// SECOND canvas-res storage texture through the same group(1) slot (a separate
// bind group at dispatch), so the group(0) buffer layout is untouched.
fn straight_word(word: u32) -> vec4<f32> {
    return unpack4x8unorm(word);
}

// Region-scoped like `cs_premul_tex`, dispatched right after `cs_composite` in
// the same pass: copy the fresh straight-alpha `out_buf` region into the texture.
@compute @workgroup_size(8, 8, 1)
fn cs_straight_tex(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cx = P.origin_x + gid.x;
    let cy = P.origin_y + gid.y;
    if (cx >= P.end_x || cy >= P.end_y) {
        return;
    }
    textureStore(preview_tex, vec2<i32>(i32(cx), i32(cy)), straight_word(out_buf[cy * P.cw + cx]));
}

// Lazy init (first straight-texture frame of a stroke): fill the full canvas with
// the STRAIGHT backdrop — the active layer's pre-stroke pixels — so never-composited
// texels already hold the layer content the compositor slice must carry.
@compute @workgroup_size(8, 8, 1)
fn cs_straight_init(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cx = P.origin_x + gid.x;
    let cy = P.origin_y + gid.y;
    if (cx >= P.end_x || cy >= P.end_y) {
        return;
    }
    textureStore(preview_tex, vec2<i32>(i32(cx), i32(cy)), straight_word(backdrop[cy * P.cw + cx]));
}
