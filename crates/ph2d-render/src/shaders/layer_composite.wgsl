// layer_composite.wgsl — GPU layer compositor (Painter W3, Block 2).
//
// docs/Painter_projeto/02_layers.md §2.11 + §2.12. The real-time sibling
// of the CPU reference `ph2d_tool_painter::compositor::composite`.
//
// ## What it mirrors
//
// The **source of truth** for the blend math is
// `ph2d_painter_brush::blend::apply(mode, dst, src)` (22 W3C Compositing
// Level 1 modes, straight linear-sRGB). The **structure** mirrors the CPU
// `composite_into`: top-down (panel order top-first, blended bottom-to-top),
// groups composite their children into a sub-accumulator and blend that as a
// single layer, opacity folds into the source alpha, decode sRGB→linear on
// read / encode linear→sRGB on write. The sRGB transfer literals are kept
// bit-identical to `ph2d_color::srgb` (same discipline as stamp.wgsl's gamma
// fix + the OKLab coefficient gate `shader_oklab_coefficients_bit_identical`).
//
// ## Per-pixel stack machine (instead of recursion)
//
// WGSL has no recursion. The CPU tree-walk is flattened on the Rust side into
// a linear op-list (`LayerOp`): `Layer{slot,mode,opacity}`,
// `PushGroup`, `PopGroupBlend{mode,opacity}` — emitted in the same
// bottom-to-top order the CPU recurses. Each pixel maintains a small stack of
// linear-sRGB accumulators (`array<vec4<f32>, MAX_STACK>`); a group's
// "sub-buffer" at one pixel is just one `vec4`. The op-list is identical for
// every pixel, so the `switch` on `op.kind` / `blend_mode` is **uniform
// control flow** (no warp divergence) — only the data-dependent dodge/burn
// branches diverge, and only along the blend gradient.
//
// ## Color space / determinism
//
// Layers bind as a `texture_2d_array<rgba8unorm>` (NOT `…UnormSrgb`): we want
// the raw byte/255 so the in-shader transfer function matches the Rust LUT
// (`srgb_to_linear_byte`) — hardware sRGB-texture decode is a different
// piecewise approximation and would NOT agree byte-for-byte. As with
// stamp.wgsl, writing the same formula in WGSL and Rust does NOT guarantee
// bit-identical *runtime* results — `pow`/`sqrt` are ULP-bounded (≤1-4 ULP)
// across Metal/Vulkan/DX12. The textual gate
// `shader_blend_modes_bit_identical_with_rust` pins the *literals*; the GPU
// readback gate asserts the output matches the CPU reference within ±1 byte.

// Group nesting cap (mirror of ph2d_tool_painter::layers::MAX_GROUP_DEPTH).
// The ROOT accumulator lives in a register (`acc0`) — the hot path, where
// every layer is at depth 0 — and only group interiors touch the array, so
// dynamic indexing (which Metal spills to thread-local memory) never lands on
// the per-pixel critical path. `stack[d]` holds group depth `d+1`.
const MAX_GROUP_STACK: u32 = 8u;
const F32_EPSILON: f32 = 1.1920929e-7; // f32::EPSILON (2^-23), mirror of Rust

// Op kinds — keep in sync with the Rust `LayerOpKind` discriminants in
// `layer_compositor.rs`.
const OP_LAYER: u32 = 0u;
const OP_PUSH_GROUP: u32 = 1u;
const OP_POP_GROUP: u32 = 2u;
// W4 (ADR-0045): a non-destructive adjustment transforms the current-depth
// accumulator (everything BELOW it) in place, then blends the result back over
// itself by the adjustment's own opacity in its blend mode — the GPU mirror of
// the CPU `composite_into` Adjustment arm. `layer_slot` is reused as the index
// into `adj_params`; `blend_mode` + `opacity` are the adjustment's.
const OP_ADJUSTMENT: u32 = 3u;

// Adjustment kind codes — mirror the Rust mapping in the painter tool's flatten
// (`AdjustmentKind` → code). Only the implemented kinds; an unknown code is an
// identity no-op (forward-compatible with kinds the GPU shader doesn't know yet).
const ADJ_HSB: u32 = 0u;
const ADJ_BRIGHTNESS_CONTRAST: u32 = 1u;
const ADJ_INVERT: u32 = 2u;
const ADJ_POSTERIZE: u32 = 3u;
const ADJ_THRESHOLD: u32 = 4u;
const ADJ_EXPOSURE: u32 = 5u;
const ADJ_VIBRANCE: u32 = 6u;

// One flattened compositor op. 16 bytes; `layer_slot` is the texture-array
// slice for OP_LAYER, ignored otherwise; `blend_mode` + `opacity` apply to
// OP_LAYER and OP_POP_GROUP.
struct Op {
    kind: u32,
    layer_slot: u32,
    blend_mode: u32,
    opacity: f32,
}

// Per-adjustment params (16 bytes; mirrors Rust `AdjParamsGpu`). `kind` is an
// `ADJ_*` code; `p0/p1/p2` are the kind's scalar params (see `apply_adjustment`).
struct AdjParams {
    kind: u32,
    p0: f32,
    p1: f32,
    p2: f32,
}

struct Globals {
    canvas_width: u32,
    canvas_height: u32,
    // Dirty-rect window (the dispatch covers region_w × region_h; sample
    // coords are region origin + local, output coords are local). A full
    // composite passes the whole canvas as the region.
    region_x: u32,
    region_y: u32,
    region_w: u32,
    region_h: u32,
    op_count: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read> ops: array<Op>;
@group(0) @binding(1) var<uniform> g: Globals;
// Canvas-sized straight sRGB8 layer pixels, one array slice per cached layer.
@group(0) @binding(2) var layers: texture_2d_array<f32>;
// Region-sized straight sRGB8 output (rgba8unorm storage, write-only).
@group(0) @binding(3) var out_tex: texture_storage_2d<rgba8unorm, write>;
// 256-entry sRGB→linear decode LUT — `srgb_lut[b] = srgb_to_linear_byte(b)`,
// computed on the CPU with the canonical `ph2d_color::srgb` transfer and
// uploaded once. Replacing the per-texel `pow()` with an L1-cached lookup is
// BOTH bit-exact (identical f32 to the CPU reference — no hardware-sRGB
// approximation, no f16 loss) AND ~20× faster than 150 `pow`/pixel. The
// `srgb_lut_matches_cpu_transfer` gate pins the table to the Rust function.
@group(0) @binding(4) var<storage, read> srgb_lut: array<f32, 256>;
// Per-adjustment params, indexed by an OP_ADJUSTMENT op's `layer_slot`.
@group(0) @binding(5) var<storage, read> adj_params: array<AdjParams>;

// ── sRGB gamma transfer (bit-identical to ph2d_color::srgb) ───────────────
// Encode mirror of `linear_to_srgb_byte` on a normalized [0,1] value (the
// store quantization handles the ×255). Same literals as stamp.wgsl's gamma
// fix. Decode is the LUT above (the hot direction — 50× more calls).
fn linear_to_srgb(v: f32) -> f32 {
    let c = clamp(v, 0.0, 1.0);
    if c <= 0.0031308 {
        return c * 12.92;
    }
    return 1.055 * pow(c, 1.0 / 2.4) - 0.055;
}

// Decode one straight sRGB8 texel of layer slice `slot` at canvas `coord` to
// straight linear RGBA via the LUT. Alpha is linear coverage (no transfer),
// per the CPU `compositor::decode`. `raw.{r,g,b}` are `byte/255` (unorm), so
// `round(raw * 255)` recovers the exact source byte to index the table.
fn decode_layer(slot: u32, coord: vec2<i32>) -> vec4<f32> {
    let raw = textureLoad(layers, coord, i32(slot), 0);
    return vec4<f32>(
        srgb_lut[u32(raw.r * 255.0 + 0.5)],
        srgb_lut[u32(raw.g * 255.0 + 0.5)],
        srgb_lut[u32(raw.b * 255.0 + 0.5)],
        raw.a,
    );
}

// Encode a straight linear accumulator → straight sRGB8 normalized, matching
// `compositor::encode` byte-for-byte: RGB through `linear_to_srgb` then
// `floor(x*255 + 0.5)` round-half-up (NOT the unorm-store's round-half-even),
// alpha clamped + same rounding. Dividing the integer back by 255 makes the
// `rgba8unorm` store re-quantize to the exact same byte.
fn encode_final(acc: vec4<f32>) -> vec4<f32> {
    let r = floor(linear_to_srgb(acc.r) * 255.0 + 0.5) / 255.0;
    let gg = floor(linear_to_srgb(acc.g) * 255.0 + 0.5) / 255.0;
    let b = floor(linear_to_srgb(acc.b) * 255.0 + 0.5) / 255.0;
    let a = floor(clamp(acc.a, 0.0, 1.0) * 255.0 + 0.5) / 255.0;
    return vec4<f32>(r, gg, b, a);
}

// ── Separable blend functions — W3C Compositing L1 §9 ────────────────────
fn screen(cb: f32, cs: f32) -> f32 {
    return cb + cs - cb * cs;
}

fn hard_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        return 2.0 * cb * cs;
    }
    return 1.0 - 2.0 * (1.0 - cb) * (1.0 - cs);
}

fn color_dodge(cb: f32, cs: f32) -> f32 {
    if cb <= 0.0 {
        return 0.0;
    }
    if cs >= 1.0 {
        return 1.0;
    }
    return min(cb / (1.0 - cs), 1.0);
}

fn color_burn(cb: f32, cs: f32) -> f32 {
    if cb >= 1.0 {
        return 1.0;
    }
    if cs <= 0.0 {
        return 0.0;
    }
    return 1.0 - min((1.0 - cb) / cs, 1.0);
}

fn soft_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        return cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb);
    }
    var d: f32;
    if cb <= 0.25 {
        d = ((16.0 * cb - 12.0) * cb + 4.0) * cb;
    } else {
        d = sqrt(cb);
    }
    return cb + (2.0 * cs - 1.0) * (d - cb);
}

fn blend_sep(mode: u32, cb: f32, cs: f32) -> f32 {
    switch mode {
        case 0u: { return cs; }                                  // Normal
        case 1u: { return cb * cs; }                             // Multiply
        case 2u: { return min(cb, cs); }                         // Darken
        case 3u: { return color_burn(cb, cs); }                  // ColorBurn
        case 4u: { return clamp(cb + cs - 1.0, 0.0, 1.0); }      // LinearBurn
        case 5u: { return max(cb, cs); }                         // Lighten
        case 6u: { return screen(cb, cs); }                      // Screen
        case 7u: { return color_dodge(cb, cs); }                 // ColorDodge
        case 8u: { return min(cb + cs, 1.0); }                   // Add (LinearDodge)
        case 9u: { return hard_light(cs, cb); }                  // Overlay
        case 10u: { return soft_light(cb, cs); }                 // SoftLight
        case 11u: { return hard_light(cb, cs); }                 // HardLight
        case 12u: {                                              // VividLight
            if cs <= 0.5 {
                return color_burn(cb, clamp(2.0 * cs, 0.0, 1.0));
            }
            return color_dodge(cb, clamp(2.0 * cs - 1.0, 0.0, 1.0));
        }
        case 13u: { return clamp(cb + 2.0 * cs - 1.0, 0.0, 1.0); } // LinearLight
        case 14u: { return abs(cb - cs); }                       // Difference
        case 15u: { return cb + cs - 2.0 * cb * cs; }            // Exclusion
        default: { return cs; }
    }
}

// ── Non-separable (HSL) blend functions — W3C Compositing L1 §10 ──────────
fn lum(c: vec3<f32>) -> f32 {
    return 0.3 * c.r + 0.59 * c.g + 0.11 * c.b;
}

fn clip_color(c: vec3<f32>) -> vec3<f32> {
    let l = lum(c);
    let n = min(c.r, min(c.g, c.b));
    let x = max(c.r, max(c.g, c.b));
    var out = c;
    if n < 0.0 {
        let denom = l - n;
        if abs(denom) > F32_EPSILON {
            out = l + (out - l) * l / denom;
        }
    }
    if x > 1.0 {
        let denom = x - l;
        if abs(denom) > F32_EPSILON {
            out = l + (out - l) * (1.0 - l) / denom;
        }
    }
    return out;
}

fn set_lum(c: vec3<f32>, l: f32) -> vec3<f32> {
    let d = l - lum(c);
    return clip_color(c + vec3<f32>(d));
}

fn sat(c: vec3<f32>) -> f32 {
    return max(c.r, max(c.g, c.b)) - min(c.r, min(c.g, c.b));
}

// W3C `SetSat`: stretch the mid/max channels to saturation `s`, preserving
// the original min→max channel ORDER. Mirrors the Rust `set_sat`, which sorts
// the three channels by value with a *stable* comparator (ties keep original
// R<G<B order). Implemented with SCALARS (no local arrays) so it adds no
// register/stack pressure to the kernel — keeping occupancy (and thus memory-
// latency hiding) high on the hot path. `rank_*` is each channel's position in
// the stable ascending sort (0 = min, 1 = mid, 2 = max); ties break R<G<B,
// matching Rust's stable sort, via the `<` / `<=` asymmetry below.
fn set_sat(c: vec3<f32>, s: f32) -> vec3<f32> {
    // Stable ascending rank of each channel. For a tie, the earlier channel
    // (R<G<B) ranks lower — encoded by using `<` against earlier channels and
    // `<=`... we instead count, for each channel, how many channels are
    // strictly below it, plus earlier channels that are equal.
    let r = c.r;
    let g = c.g;
    let b = c.b;
    let rank_r = u32(g < r) + u32(b < r);
    let rank_g = u32(r <= g) + u32(b < g);
    let rank_b = u32(r <= b) + u32(g <= b);
    let cmin = min(r, min(g, b));
    let cmax = max(r, max(g, b));
    // mid = the value whose rank is 1.
    var cmid = r;
    if rank_g == 1u { cmid = g; }
    if rank_b == 1u { cmid = b; }

    var out_min = 0.0;
    var out_mid = 0.0;
    var out_max = 0.0;
    if cmax > cmin {
        out_mid = (cmid - cmin) * s / (cmax - cmin);
        out_max = s;
    }
    // Scatter the three rank-slots back to the original channels.
    let or = select(select(out_min, out_mid, rank_r == 1u), out_max, rank_r == 2u);
    let og = select(select(out_min, out_mid, rank_g == 1u), out_max, rank_g == 2u);
    let ob = select(select(out_min, out_mid, rank_b == 1u), out_max, rank_b == 2u);
    return vec3<f32>(or, og, ob);
}

fn blend_hsl(mode: u32, cb: vec3<f32>, cs: vec3<f32>) -> vec3<f32> {
    switch mode {
        case 16u: { return set_lum(set_sat(cs, sat(cb)), lum(cb)); } // Hue
        case 17u: { return set_lum(set_sat(cb, sat(cs)), lum(cb)); } // Saturation
        case 18u: { return set_lum(cs, lum(cb)); }                   // Color
        case 19u: { return set_lum(cb, lum(cs)); }                   // Luminosity
        default: { return cs; }
    }
}

fn is_hsl(mode: u32) -> bool {
    return mode == 16u || mode == 17u || mode == 18u || mode == 19u;
}

// Plain source-over (`Normal`) — used by `Behind` with swapped roles.
fn over(top: vec4<f32>, bottom: vec4<f32>) -> vec4<f32> {
    let at = clamp(top.a, 0.0, 1.0);
    let abm = clamp(bottom.a, 0.0, 1.0);
    let ao = at + abm * (1.0 - at);
    if ao <= F32_EPSILON {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    let premul = at * top.rgb + abm * (1.0 - at) * bottom.rgb;
    return vec4<f32>(clamp(premul / ao, vec3<f32>(0.0), vec3<f32>(1.0)), ao);
}

// Composite `src` over `dst` under `mode`. Straight linear-sRGB RGBA in
// [0,1]. Port of `ph2d_painter_brush::blend::apply` over FINITE inputs: the
// Rust side additionally routes outputs through `sanitize01` (NaN/±inf → 0,
// defense-in-depth); this port uses plain `clamp` instead, because layer
// inputs are LUT-bounded-finite (decode_layer reads rgba8unorm → finite LUT)
// and every division is guarded by `ao > F32_EPSILON`, so a non-finite value
// can't arise here (audit 2026-06-01 LOW — confirmed unreachable). The literal
// constants are still pinned to Rust by shader_blend_modes_bit_identical.
fn apply_blend(mode: u32, dst: vec4<f32>, src: vec4<f32>) -> vec4<f32> {
    let ab = clamp(dst.a, 0.0, 1.0);
    let as_ = clamp(src.a, 0.0, 1.0);
    let cb = dst.rgb;
    let cs = src.rgb;

    // Compositing specials bypass the blend function B.
    if mode == 21u {
        // Clear: keep backdrop color, attenuate its alpha by the source's.
        return vec4<f32>(cb, ab * (1.0 - as_));
    }
    if mode == 20u {
        // Behind: paint under the backdrop (dst stays on top of src).
        return over(dst, src);
    }

    var blended: vec3<f32>;
    if is_hsl(mode) {
        blended = blend_hsl(mode, cb, cs);
    } else {
        blended = vec3<f32>(
            blend_sep(mode, cb.r, cs.r),
            blend_sep(mode, cb.g, cs.g),
            blend_sep(mode, cb.b, cs.b),
        );
    }

    let ao = as_ + ab * (1.0 - as_);
    if ao <= F32_EPSILON {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    // Cs' = (1-αb)·Cs + αb·B ; Co = (αs·Cs' + αb·(1-αs)·Cb) / αo
    let cs_eff = (1.0 - ab) * cs + ab * blended;
    let co_premul = as_ * cs_eff + ab * (1.0 - as_) * cb;
    let out = clamp(co_premul / ao, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(out, ao);
}

// ── Adjustment kernels (W4) — GPU mirror of ph2d_painter_brush::adjustments ──
// Inputs/outputs are STRAIGHT LINEAR rgb (the accumulator space). Display-space
// kinds (Invert/Posterize/Threshold) convert linear↔sRGB internally — same as
// the CPU. OKLab uses `pow(x, 1/3)` for the cube root (the CPU uses libm cbrt;
// ULP-bounded, asserted within tolerance by the GPU↔CPU parity gate).

// Continuous sRGB→linear (the LUT decode is byte-input only; display-space
// adjustments need the f32 transfer). Bit-literal twin of the encode below.
fn srgb_to_linear_f32(c: f32) -> f32 {
    let v = clamp(c, 0.0, 1.0);
    if v <= 0.04045 {
        return v / 12.92;
    }
    return pow((v + 0.055) / 1.055, 2.4);
}

// Linear sRGB → OKLab. Coefficients bit-identical to
// `ph2d_color::oklab::OklabColor::from_linear` (the rounded f32 literals the
// Rust source uses — NOT the full-precision spec values, or the GPU↔CPU parity
// drifts). Pinned by `shader_adjustment_coefficients_bit_identical_with_rust`.
fn oklab_from_linear(c: vec3<f32>) -> vec3<f32> {
    let l = 0.41222147 * c.r + 0.5363325 * c.g + 0.051445993 * c.b;
    let m = 0.2119035 * c.r + 0.6806995 * c.g + 0.10739696 * c.b;
    let s = 0.08830246 * c.r + 0.28171884 * c.g + 0.6299787 * c.b;
    let l_ = pow(max(l, 0.0), 0.3333333333);
    let m_ = pow(max(m, 0.0), 0.3333333333);
    let s_ = pow(max(s, 0.0), 0.3333333333);
    return vec3<f32>(
        0.21045426 * l_ + 0.7936178 * m_ - 0.004072047 * s_,
        1.9779985 * l_ - 2.4285922 * m_ + 0.4505937 * s_,
        0.025904037 * l_ + 0.78277177 * m_ - 0.80867577 * s_,
    );
}

// OKLab → linear sRGB. Coefficients bit-identical to `OklabColor::to_linear`.
fn oklab_to_linear(lab: vec3<f32>) -> vec3<f32> {
    let l_ = lab.x + 0.39633778 * lab.y + 0.21580376 * lab.z;
    let m_ = lab.x - 0.105561346 * lab.y - 0.06385417 * lab.z;
    let s_ = lab.x - 0.08948418 * lab.y - 1.2914855 * lab.z;
    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;
    return vec3<f32>(
        4.0767417 * l3 - 3.3077116 * m3 + 0.23096994 * s3,
        -1.268438 * l3 + 2.6097574 * m3 - 0.34131938 * s3,
        -0.0041960863 * l3 - 0.7034186 * m3 + 1.7076147 * s3,
    );
}

// Apply an adjustment kind to a straight-linear rgb triple (RGB transform only;
// the caller preserves alpha = coverage). Unknown kind → identity.
fn apply_adjustment(ap: AdjParams, rgb: vec3<f32>) -> vec3<f32> {
    switch ap.kind {
        case 0u: { // ADJ_HSB — p0=hue(turns), p1=sat(-1..1), p2=bright(-1..1)
            let hue_rad = ap.p0 * 6.2831853072;
            let hc = cos(hue_rad);
            let hs = sin(hue_rad);
            let chroma_scale = max(1.0 + ap.p1, 0.0);
            let lab = oklab_from_linear(rgb);
            let a = (lab.y * hc - lab.z * hs) * chroma_scale;
            let b = (lab.y * hs + lab.z * hc) * chroma_scale;
            var out = oklab_to_linear(vec3<f32>(lab.x, a, b));
            if ap.p2 > 0.0 {
                out = out + (vec3<f32>(1.0) - out) * ap.p2;
            } else if ap.p2 < 0.0 {
                out = out * (1.0 + ap.p2);
            }
            return out;
        }
        case 1u: { // ADJ_BRIGHTNESS_CONTRAST — p0=brightness, p1=contrast
            let PIVOT = 0.21404114;
            let scale = 1.0 + ap.p1;
            var v = clamp((rgb - vec3<f32>(PIVOT)) * scale + vec3<f32>(PIVOT), vec3<f32>(0.0), vec3<f32>(1.0));
            if ap.p0 > 0.0 {
                v = v + (vec3<f32>(1.0) - v) * ap.p0;
            } else if ap.p0 < 0.0 {
                v = v * (1.0 + ap.p0);
            }
            return v;
        }
        case 2u: { // ADJ_INVERT — display-space negative
            return vec3<f32>(
                srgb_to_linear_f32(1.0 - linear_to_srgb(rgb.r)),
                srgb_to_linear_f32(1.0 - linear_to_srgb(rgb.g)),
                srgb_to_linear_f32(1.0 - linear_to_srgb(rgb.b)),
            );
        }
        case 3u: { // ADJ_POSTERIZE — p0=levels (2..32), display-space quantize
            let levels = clamp(ap.p0, 2.0, 32.0);
            let steps = levels - 1.0;
            let s = vec3<f32>(linear_to_srgb(rgb.r), linear_to_srgb(rgb.g), linear_to_srgb(rgb.b));
            let q = round(s * steps) / steps;
            return vec3<f32>(srgb_to_linear_f32(q.r), srgb_to_linear_f32(q.g), srgb_to_linear_f32(q.b));
        }
        case 4u: { // ADJ_THRESHOLD — p0=cut (0..1), display-space luma → B/W
            let luma = 0.299 * linear_to_srgb(rgb.r) + 0.587 * linear_to_srgb(rgb.g) + 0.114 * linear_to_srgb(rgb.b);
            let v = select(0.0, 1.0, luma >= ap.p0);
            return vec3<f32>(v, v, v);
        }
        case 5u: { // ADJ_EXPOSURE — p0=EV, p1=offset, p2=gamma_correction
            let gain = exp2(ap.p0);
            let inv_gamma = 1.0 / max(1.0 + ap.p2, 0.001);
            let v = max(rgb * gain + vec3<f32>(ap.p1), vec3<f32>(0.0));
            return pow(v, vec3<f32>(inv_gamma));
        }
        case 6u: { // ADJ_VIBRANCE — p0=vibrance, p1=saturation (OKLab chroma)
            let sat_mul = max(1.0 + ap.p1, 0.0);
            let lab = oklab_from_linear(rgb);
            let chroma = sqrt(lab.y * lab.y + lab.z * lab.z);
            if chroma > 1e-6 {
                let nc = min(chroma / 0.4, 1.0);
                let vib_mul = max(1.0 + ap.p0 * (1.0 - nc), 0.0);
                let scale = sat_mul * vib_mul;
                return oklab_to_linear(vec3<f32>(lab.x, lab.y * scale, lab.z * scale));
            }
            return rgb;
        }
        default: { return rgb; }
    }
}

// Apply an OP_ADJUSTMENT over the current accumulator (everything below). Mirror
// of the CPU arm: transform acc.rgb by the kind, blend the result back over acc
// in the adjustment's mode, then lerp by the adjustment's opacity; coverage
// (acc.a) is preserved.
fn apply_adjustment_op(op: Op, acc: vec4<f32>) -> vec4<f32> {
    let ap = adj_params[op.layer_slot];
    let adj_rgb = apply_adjustment(ap, acc.rgb);
    let src_px = vec4<f32>(adj_rgb, acc.a);
    let blended = apply_blend(op.blend_mode, acc, src_px);
    let t = clamp(op.opacity, 0.0, 1.0);
    return vec4<f32>(mix(acc.rgb, blended.rgb, t), acc.a);
}

// Map a thread to its (output local, canvas global) coordinate. Returns false
// for threads outside the region / canvas (which must early-out). `out_local`
// is where to store; `coord` is where to sample the layers.
fn resolve_pixel(gid: vec3<u32>, out_local: ptr<function, vec2<i32>>, coord: ptr<function, vec2<i32>>) -> bool {
    let lx = gid.x;
    let ly = gid.y;
    if lx >= g.region_w || ly >= g.region_h {
        return false;
    }
    let gx = g.region_x + lx;
    let gy = g.region_y + ly;
    // Defense-in-depth: the Rust side clamps the region to canvas bounds, but
    // guard so a custom dispatch can't fetch out of range.
    if gx >= g.canvas_width || gy >= g.canvas_height {
        return false;
    }
    *out_local = vec2<i32>(i32(lx), i32(ly));
    *coord = vec2<i32>(i32(gx), i32(gy));
    return true;
}

// FLAT entry — the common case (op-list has NO groups). One accumulator in a
// register, zero stack array: minimal registers → high occupancy → the
// dispatch saturates memory bandwidth (which is the floor for a full-canvas
// many-layer recomposite). The Rust side dispatches this when the op-list
// contains only OP_LAYER.
@compute @workgroup_size(8, 8, 1)
fn cs_flat(@builtin(global_invocation_id) gid: vec3<u32>) {
    var out_local: vec2<i32>;
    var coord: vec2<i32>;
    if !resolve_pixel(gid, &out_local, &coord) {
        return;
    }
    var acc = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    for (var i: u32 = 0u; i < g.op_count; i = i + 1u) {
        let op = ops[i];
        if op.kind == OP_ADJUSTMENT {
            acc = apply_adjustment_op(op, acc);
        } else {
            var s = decode_layer(op.layer_slot, coord);
            s.a = s.a * op.opacity;
            acc = apply_blend(op.blend_mode, acc, s);
        }
    }
    textureStore(out_tex, out_local, encode_final(acc));
}

// GROUPED entry — op-list contains groups. Adds a per-pixel sub-accumulator
// stack (depths 1..=MAX_GROUP_STACK); the root stays in `acc0`. Heavier
// register footprint (lower occupancy) than `cs_flat`, but only paid when the
// document actually nests groups.
@compute @workgroup_size(8, 8, 1)
fn cs_grouped(@builtin(global_invocation_id) gid: vec3<u32>) {
    var out_local: vec2<i32>;
    var coord: vec2<i32>;
    if !resolve_pixel(gid, &out_local, &coord) {
        return;
    }
    var acc0 = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var stack: array<vec4<f32>, MAX_GROUP_STACK>;
    var sp: u32 = 0u; // current depth: 0 = root (acc0), d ≥ 1 = stack[d-1]

    for (var i: u32 = 0u; i < g.op_count; i = i + 1u) {
        let op = ops[i];
        switch op.kind {
            case 0u: { // OP_LAYER — blend over the current-depth accumulator
                var s = decode_layer(op.layer_slot, coord);
                s.a = s.a * op.opacity;
                if sp == 0u {
                    acc0 = apply_blend(op.blend_mode, acc0, s);
                } else {
                    let d = sp - 1u;
                    stack[d] = apply_blend(op.blend_mode, stack[d], s);
                }
            }
            case 1u: { // OP_PUSH_GROUP — open a transparent sub-accumulator
                if sp < MAX_GROUP_STACK {
                    sp = sp + 1u;
                    stack[sp - 1u] = vec4<f32>(0.0, 0.0, 0.0, 0.0);
                }
            }
            case 2u: { // OP_POP_GROUP — blend the sub-accumulator as one layer
                if sp > 0u {
                    var sub = stack[sp - 1u];
                    sp = sp - 1u;
                    sub.a = sub.a * op.opacity;
                    if sp == 0u {
                        acc0 = apply_blend(op.blend_mode, acc0, sub);
                    } else {
                        let d = sp - 1u;
                        stack[d] = apply_blend(op.blend_mode, stack[d], sub);
                    }
                }
            }
            case 3u: { // OP_ADJUSTMENT — transform the current-depth accumulator
                if sp == 0u {
                    acc0 = apply_adjustment_op(op, acc0);
                } else {
                    let d = sp - 1u;
                    stack[d] = apply_adjustment_op(op, stack[d]);
                }
            }
            default: {}
        }
    }

    textureStore(out_tex, out_local, encode_final(acc0));
}
