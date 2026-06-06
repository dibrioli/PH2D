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
// W4 bespoke — display-space 1-D transfer LUTs (binding 6 `adj_luts`). `p0` is
// reused as the base float index of this op's table block in `adj_luts`.
const ADJ_CURVES: u32 = 7u;
const ADJ_LEVELS: u32 = 8u;

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
// W4 — display-space transfer LUTs for Curves (3×256 R/G/B) and Levels (1×256).
// Built CPU-side from `curves_display_luts`/`levels_display_lut` (the SAME table
// the CPU compositor reads), concatenated per Curves/Levels op; `AdjParams.p0`
// holds the op's base float offset. Stride per channel = 256 (`DISPLAY_LUT_N`).
@group(0) @binding(6) var<storage, read> adj_luts: array<f32>;

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

// Rec.709 display luma of a straight-linear pixel — the perceptual brightness in
// DISPLAY (sRGB-encoded) space, mirror of `ph2d_painter_brush::adjustments::
// spatial::display_luma` (used by Bloom's bright-pass + Shadows/Highlights' tone
// maps). The channels go through the sRGB encode first (so the weighting is
// perceptual), then the standard luma coefficients.
fn display_luma(rgb: vec3<f32>) -> f32 {
    let r = linear_to_srgb(rgb.r);
    let g = linear_to_srgb(rgb.g);
    let b = linear_to_srgb(rgb.b);
    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
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
        case 7u: { // ADJ_CURVES — p0=base; 3 tables (R/G/B), 256 each, display-space
            let base = u32(ap.p0);
            let sr = clamp(linear_to_srgb(rgb.r), 0.0, 1.0);
            let sg = clamp(linear_to_srgb(rgb.g), 0.0, 1.0);
            let sb = clamp(linear_to_srgb(rgb.b), 0.0, 1.0);
            let or_ = adj_luts[base + 0u * 256u + u32(sr * 255.0 + 0.5)];
            let og = adj_luts[base + 1u * 256u + u32(sg * 255.0 + 0.5)];
            let ob = adj_luts[base + 2u * 256u + u32(sb * 255.0 + 0.5)];
            return vec3<f32>(srgb_to_linear_f32(or_), srgb_to_linear_f32(og), srgb_to_linear_f32(ob));
        }
        case 8u: { // ADJ_LEVELS — p0=base; 1 table (channel-uniform), display-space
            let base = u32(ap.p0);
            let sr = clamp(linear_to_srgb(rgb.r), 0.0, 1.0);
            let sg = clamp(linear_to_srgb(rgb.g), 0.0, 1.0);
            let sb = clamp(linear_to_srgb(rgb.b), 0.0, 1.0);
            let or_ = adj_luts[base + u32(sr * 255.0 + 0.5)];
            let og = adj_luts[base + u32(sg * 255.0 + 0.5)];
            let ob = adj_luts[base + u32(sb * 255.0 + 0.5)];
            return vec3<f32>(srgb_to_linear_f32(or_), srgb_to_linear_f32(og), srgb_to_linear_f32(ob));
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
        } else if op.kind == OP_LAYER {
            var s = decode_layer(op.layer_slot, coord);
            s.a = s.a * op.opacity;
            acc = apply_blend(op.blend_mode, acc, s);
        }
        // Other kinds (groups, OP_SPATIAL) never reach cs_flat — the Rust side
        // dispatches cs_grouped for groups and the segmented pass-graph for any
        // spatial op. The explicit OP_LAYER guard is defence-in-depth.
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

// ═══════════════════════════════════════════════════════════════════════════
// SPATIAL PASS-GRAPH (Painter W4) — multi-pass infra for neighbourhood filters
// ═══════════════════════════════════════════════════════════════════════════
//
// A spatial adjustment (Gaussian/Bloom/Sharpen/…) reads a RADIUS of neighbours,
// which the single-pass per-pixel compositor (acc in a register) cannot do. The
// Rust side splits the op-list at each root-level spatial adjustment into
// segments and drives this graph: cs_segment materialises the below-composite
// into a linear Rgba32Float texture → cs_blur_h/cs_blur_v run the separable
// kernel through a ping-pong pair → cs_combine blends the result back over the
// base → cs_segment continues the layers above → cs_encode writes straight
// sRGB8. All passes work in STRAIGHT LINEAR space (the accumulator space); the
// final encode is the only linear→sRGB step (byte-identical to the single-pass
// `encode_final`). These entry points share every helper above (apply_blend,
// decode_layer, apply_adjustment, encode_final) — no duplicated math.
//
// Bindings 7–20 are this graph's; the single-pass path (0–6) is untouched. Each
// pipeline binds only the subset its entry point uses.

// ── cs_segment — composite ops[op_start..op_end] into a linear intermediate ──
struct SegGlobals {
    canvas_width: u32,
    canvas_height: u32,
    region_x: u32,
    region_y: u32,
    region_w: u32,
    region_h: u32,
    op_start: u32,
    op_end: u32,
    seg_from_base: u32, // 0 → start accumulator from zero; else from base_in
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}
@group(0) @binding(7) var<uniform> sg: SegGlobals;
// Linear straight-sRGB Rgba32Float intermediate this segment writes.
@group(0) @binding(8) var seg_out: texture_storage_2d<rgba32float, write>;
// The running base (previous segment / combine result), work_region-sized so
// its local coord is 1:1 with seg_out. Only read when sg.seg_from_base != 0.
@group(0) @binding(9) var base_in: texture_2d<f32>;

// A segment is a complete root-level run with any fully-contained groups (the
// Rust side only breaks at depth-0 spatial ops). Mirror of cs_grouped, but the
// root accumulator starts from base_in and the result is written LINEAR (no
// encode) for the next pass to sample. OP_SPATIAL placeholders are no-ops here.
@compute @workgroup_size(8, 8, 1)
fn cs_segment(@builtin(global_invocation_id) gid: vec3<u32>) {
    let lx = gid.x;
    let ly = gid.y;
    if lx >= sg.region_w || ly >= sg.region_h {
        return;
    }
    let out_local = vec2<i32>(i32(lx), i32(ly));
    let gx = sg.region_x + lx;
    let gy = sg.region_y + ly;
    if gx >= sg.canvas_width || gy >= sg.canvas_height {
        // Halo pixel outside the canvas — clamp-to-edge is the kernel's job;
        // here we simply leave it transparent (the layers have no data there).
        textureStore(seg_out, out_local, vec4<f32>(0.0, 0.0, 0.0, 0.0));
        return;
    }
    let coord = vec2<i32>(i32(gx), i32(gy));

    var acc0 = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    if sg.seg_from_base != 0u {
        acc0 = textureLoad(base_in, out_local, 0);
    }
    var stack: array<vec4<f32>, MAX_GROUP_STACK>;
    var sp: u32 = 0u;

    for (var i: u32 = sg.op_start; i < sg.op_end; i = i + 1u) {
        let op = ops[i];
        switch op.kind {
            case 0u: { // OP_LAYER
                var s = decode_layer(op.layer_slot, coord);
                s.a = s.a * op.opacity;
                if sp == 0u {
                    acc0 = apply_blend(op.blend_mode, acc0, s);
                } else {
                    let d = sp - 1u;
                    stack[d] = apply_blend(op.blend_mode, stack[d], s);
                }
            }
            case 1u: { // OP_PUSH_GROUP
                if sp < MAX_GROUP_STACK {
                    sp = sp + 1u;
                    stack[sp - 1u] = vec4<f32>(0.0, 0.0, 0.0, 0.0);
                }
            }
            case 2u: { // OP_POP_GROUP
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
            case 3u: { // OP_ADJUSTMENT (per-pixel)
                if sp == 0u {
                    acc0 = apply_adjustment_op(op, acc0);
                } else {
                    let d = sp - 1u;
                    stack[d] = apply_adjustment_op(op, stack[d]);
                }
            }
            default: {} // OP_SPATIAL placeholder / unknown — no-op
        }
    }

    textureStore(seg_out, out_local, acc0);
}

// ── blur passes — separable (cs_blur_h/v) + directional (cs_blur_dir) ────────
//
// PREMULTIPLIED convolution: the FIRST blur pass premultiplies each tap (rgb*a)
// on read (`premul_read != 0`), so transparent texels — which carry garbage RGB
// under a=0 — contribute zero. Colour AND coverage (alpha) blur together →
// soft feather into transparency, no colour leak (the combine un-premultiplies).
// Later passes read already-premultiplied data (premul_read = 0). For an opaque
// base (a=1 everywhere) premultiply is the identity, so this is a no-op there.
struct BlurGlobals {
    width: u32,
    height: u32,
    half: u32, // kernel reaches ±half; weights[0..=half], weights[0] = centre
    _pad0: u32,
    dir_x: f32, // motion direction (cs_blur_dir); ignored by cs_blur_h/v
    dir_y: f32,
    premul_read: f32, // 1 = premultiply each tap on read (first pass); 0 = already premul
    _pad2: f32,
}
@group(0) @binding(10) var<uniform> blur_g: BlurGlobals;
@group(0) @binding(11) var blur_src: texture_2d<f32>;
@group(0) @binding(12) var blur_dst: texture_storage_2d<rgba32float, write>;
// Symmetric Gaussian weights weights[0..=half] (centre + flanks), normalised so
// the full 2·half+1 kernel sums to 1. Built CPU-side by `gaussian_weights`; the
// SAME values feed the CPU parity reference (the math is the painter impl's
// canonical `apply_gaussian` once reconciled — the mechanism is what this proves).
@group(0) @binding(13) var<storage, read> blur_weights: array<f32>;

// Read a tap, premultiplying on the first pass (see the premul note above).
fn load_blur_tap(p: vec2<i32>) -> vec4<f32> {
    let t = textureLoad(blur_src, p, 0);
    if blur_g.premul_read != 0.0 {
        return vec4<f32>(t.rgb * t.a, t.a);
    }
    return t;
}

fn blur_core(gid: vec3<u32>, dir: vec2<i32>) {
    let x = gid.x;
    let y = gid.y;
    if x >= blur_g.width || y >= blur_g.height {
        return;
    }
    let p = vec2<i32>(i32(x), i32(y));
    let lo = vec2<i32>(0, 0);
    let hi = vec2<i32>(i32(blur_g.width) - 1, i32(blur_g.height) - 1);
    var acc = load_blur_tap(p) * blur_weights[0];
    for (var i: u32 = 1u; i <= blur_g.half; i = i + 1u) {
        let off = dir * i32(i);
        let pa = clamp(p + off, lo, hi);
        let pb = clamp(p - off, lo, hi);
        let w = blur_weights[i];
        acc = acc + (load_blur_tap(pa) + load_blur_tap(pb)) * w;
    }
    textureStore(blur_dst, p, acc);
}
@compute @workgroup_size(8, 8, 1)
fn cs_blur_h(@builtin(global_invocation_id) gid: vec3<u32>) {
    blur_core(gid, vec2<i32>(1, 0));
}
@compute @workgroup_size(8, 8, 1)
fn cs_blur_v(@builtin(global_invocation_id) gid: vec3<u32>) {
    blur_core(gid, vec2<i32>(0, 1));
}

// Directional (motion) blur — a single 1-D pass: average `2·half+1` taps along
// `(dir_x, dir_y)` (a unit vector, computed CPU-side from the angle so no GPU
// sin/cos parity drift). Nearest sampling at rounded offsets, clamp-to-edge.
// The impl's canonical apply_motion_blur may bilinear-sample for smoothness — a
// quality refinement; the mechanism (swappable blur stage) is what this proves.
@compute @workgroup_size(8, 8, 1)
fn cs_blur_dir(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if x >= blur_g.width || y >= blur_g.height {
        return;
    }
    let p = vec2<f32>(f32(x), f32(y));
    let dir = vec2<f32>(blur_g.dir_x, blur_g.dir_y);
    let lo = vec2<i32>(0, 0);
    let hi = vec2<i32>(i32(blur_g.width) - 1, i32(blur_g.height) - 1);
    let half_off = vec2<f32>(0.5, 0.5);
    var acc = load_blur_tap(vec2<i32>(p)) * blur_weights[0];
    for (var i: u32 = 1u; i <= blur_g.half; i = i + 1u) {
        let off = dir * f32(i);
        // floor(x + 0.5) = round-half-up — bit-identical to the Rust reference
        // (WGSL `round` is ties-to-even, which would pick a different tap on a
        // half-integer offset).
        let pa = clamp(vec2<i32>(floor(p + off + half_off)), lo, hi);
        let pb = clamp(vec2<i32>(floor(p - off + half_off)), lo, hi);
        let w = blur_weights[i];
        acc = acc + (load_blur_tap(pa) + load_blur_tap(pb)) * w;
    }
    textureStore(blur_dst, vec2<i32>(p), acc);
}

// ── cs_combine — derive the kernel result from base+blurred, blend over base ──
struct CombineGlobals {
    width: u32,
    height: u32,
    blend_mode: u32,
    combine_mode: u32, // 0 = Gaussian (passthrough); 1 = Sharpen (unsharp); 2 = Bloom (additive glow)
    opacity: f32,
    amount: f32, // unsharp amount (mode 1) / bloom intensity (mode 2)
    _pad2: f32,
    _pad3: f32,
}
@group(0) @binding(14) var<uniform> comb_g: CombineGlobals;
@group(0) @binding(15) var comb_base: texture_2d<f32>;
@group(0) @binding(16) var comb_blurred: texture_2d<f32>;
@group(0) @binding(17) var comb_dst: texture_storage_2d<rgba32float, write>;

// Un-premultiply a premultiplied texel → straight RGBA (with its feathered alpha).
fn unpremultiply(c: vec4<f32>) -> vec4<f32> {
    if c.a > F32_EPSILON {
        return vec4<f32>(c.rgb / c.a, c.a);
    }
    return vec4<f32>(0.0, 0.0, 0.0, c.a);
}

// Spatial mirror of apply_adjustment_op, PREMULTIPLY-aware (the kernel ran in
// premultiplied space). Then, by `combine_mode`:
//   - GAUSSIAN/MOTION/CHROMA (0): adjusted = unpremultiply(blurred) — the kernel
//     result with its FEATHERED alpha (soft coverage into transparency).
//   - SHARPEN (1): adjusted = base + amount·(base − blurred) (unsharp, 4 channels).
//   - BLOOM (2): the second input is the blurred bright-pass GLOW (premultiplied,
//     NOT un-premultiplied); add `intensity·glow` onto the premultiplied base and
//     un-premultiply → the glow haloes outward into transparency. Mirror of
//     `apply_bloom`'s additive step. `intensity` rides `amount`.
//   - result = (Normal) ? adjusted : blend(base, adjusted) — Normal REPLACES the
//     base so the feathered edge fades to transparent (not composited over it).
//   - out = lerp(base, result, opacity) on ALL 4 channels (coverage is feathered).
// A neutral kernel (radius 0) ⇒ blurred == base ⇒ exact identity. For an opaque
// base, premultiply is the identity and this reduces to the old preserve-base-alpha
// behaviour, so the opaque-base parity gates are unchanged.
@compute @workgroup_size(8, 8, 1)
fn cs_combine(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if x >= comb_g.width || y >= comb_g.height {
        return;
    }
    let p = vec2<i32>(i32(x), i32(y));
    let acc = textureLoad(comb_base, p, 0); // straight base (materialise output)
    var adj: vec4<f32>;
    if comb_g.combine_mode == 2u { // COMBINE_BLOOM — additive premultiplied glow
        let glow_pm = textureLoad(comb_blurred, p, 0); // blurred bright-pass (premul)
        let base_pm = vec4<f32>(acc.rgb * acc.a, acc.a);
        let bloomed_pm = vec4<f32>(
            base_pm.rgb + comb_g.amount * glow_pm.rgb,
            clamp(base_pm.a + comb_g.amount * glow_pm.a, 0.0, 1.0),
        );
        adj = unpremultiply(bloomed_pm);
    } else {
        let blurred = unpremultiply(textureLoad(comb_blurred, p, 0));
        adj = blurred;
        if comb_g.combine_mode == 1u { // COMBINE_SHARPEN — unsharp mask (4 channels)
            adj = clamp(
                acc + comb_g.amount * (acc - blurred),
                vec4<f32>(0.0),
                vec4<f32>(1.0),
            );
        }
    }
    var result = adj;
    if comb_g.blend_mode != 0u { // not Normal → blend the adjusted over the base
        result = apply_blend(comb_g.blend_mode, acc, adj);
    }
    let t = clamp(comb_g.opacity, 0.0, 1.0);
    textureStore(comb_dst, p, mix(acc, result, t));
}

// ── cs_encode — final linear intermediate → straight sRGB8 (cropped) ─────────
struct EncodeGlobals {
    out_w: u32,
    out_h: u32,
    src_off_x: u32, // offset of the requested region inside the work_region texture
    src_off_y: u32,
}
@group(0) @binding(18) var<uniform> enc_g: EncodeGlobals;
@group(0) @binding(19) var enc_src: texture_2d<f32>;
@group(0) @binding(20) var enc_out: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8, 1)
fn cs_encode(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if x >= enc_g.out_w || y >= enc_g.out_h {
        return;
    }
    let out_local = vec2<i32>(i32(x), i32(y));
    let src = vec2<i32>(i32(enc_g.src_off_x + x), i32(enc_g.src_off_y + y));
    let acc = textureLoad(enc_src, src, 0);
    textureStore(enc_out, out_local, encode_final(acc));
}

// ── cs_chroma — chromatic-aberration GATHER (per-channel radial shift) ───────
// A different spatial primitive: not a neighbourhood average but a per-channel
// resample of the below-composite at radially-shifted coords (R/G/B diverge
// toward the edges → colour fringing). `dir = local − centre`; channel c samples
// at `local − dir·scale_c`. `centre` (work-local) + `scale_c = shift_c/half_diag`
// are precomputed CPU-side, so the per-pixel path is exact IEEE (no sqrt) and
// nearest sampling (`floor(x+0.5)`) is parity-robust. Combine = passthrough.
struct ChromaGlobals {
    width: u32,
    height: u32,
    center_x: f32, // canvas centre in work_region-local coords
    center_y: f32,
    scale_r: f32, // shift_c / half_diag (px-at-corner → per-unit-dir displacement)
    scale_g: f32,
    scale_b: f32,
    _pad: f32,
}
@group(0) @binding(21) var<uniform> chroma_g: ChromaGlobals;
@group(0) @binding(22) var chroma_src: texture_2d<f32>;
@group(0) @binding(23) var chroma_dst: texture_storage_2d<rgba32float, write>;

@compute @workgroup_size(8, 8, 1)
fn cs_chroma(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if x >= chroma_g.width || y >= chroma_g.height {
        return;
    }
    let p = vec2<i32>(i32(x), i32(y));
    let local_f = vec2<f32>(f32(x), f32(y));
    let dir = local_f - vec2<f32>(chroma_g.center_x, chroma_g.center_y);
    let lo = vec2<i32>(0, 0);
    let hi = vec2<i32>(i32(chroma_g.width) - 1, i32(chroma_g.height) - 1);
    let half_off = vec2<f32>(0.5, 0.5);
    let sr = clamp(vec2<i32>(floor(local_f - dir * chroma_g.scale_r + half_off)), lo, hi);
    let sg = clamp(vec2<i32>(floor(local_f - dir * chroma_g.scale_g + half_off)), lo, hi);
    let sb = clamp(vec2<i32>(floor(local_f - dir * chroma_g.scale_b + half_off)), lo, hi);
    // PREMULTIPLIED gather: each shifted channel is weighted by ITS texel's alpha,
    // so a transparent shifted texel contributes zero colour (no blue speckle).
    // Coverage (alpha) is sampled UNSHIFTED — the lens shifts colour, not coverage.
    // Output is premultiplied (combine un-premultiplies); opaque base ⇒ identity.
    let tr = textureLoad(chroma_src, sr, 0);
    let tg = textureLoad(chroma_src, sg, 0);
    let tb = textureLoad(chroma_src, sb, 0);
    let a = textureLoad(chroma_src, p, 0).a; // coverage from the unshifted centre
    textureStore(chroma_dst, p, vec4<f32>(tr.r * tr.a, tg.g * tg.a, tb.b * tb.a, a));
}

// ── cs_bloom_bright — Bloom's bright-pass (the only Bloom-specific pass) ──────
// Extract the bright EXCESS as a PREMULTIPLIED glow: a pixel whose display luma
// is above `threshold` (softened by the `falloff` knee) contributes `color·α·w`
// with coverage `α·w`. Transparent texels → 0 (premultiplied), so the glow that
// the separable blur then spreads carries coverage and haloes outward past the
// source into transparency. Mirror of `apply_bloom`'s bright-pass. The blurred
// glow is consumed by `cs_combine`'s COMBINE_BLOOM (additive) step — the blur
// runs with `premul_read = 0` (this output is already premultiplied).
struct BloomGlobals {
    width: u32,
    height: u32,
    threshold: f32,
    falloff: f32,
}
@group(0) @binding(24) var<uniform> bloom_g: BloomGlobals;
@group(0) @binding(25) var bloom_base: texture_2d<f32>;
@group(0) @binding(26) var bloom_glow: texture_storage_2d<rgba32float, write>;

@compute @workgroup_size(8, 8, 1)
fn cs_bloom_bright(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if x >= bloom_g.width || y >= bloom_g.height {
        return;
    }
    let p = vec2<i32>(i32(x), i32(y));
    let base = textureLoad(bloom_base, p, 0); // straight linear RGBA
    let knee = max(bloom_g.falloff, 1e-3);
    let w_bright = smoothstep(bloom_g.threshold, bloom_g.threshold + knee, display_luma(base.rgb));
    let k = clamp(base.a, 0.0, 1.0) * w_bright;
    textureStore(bloom_glow, p, vec4<f32>(base.rgb * k, k));
}
