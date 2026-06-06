// stamp.wgsl — W1 unified compute shader for the painter brush engine.
//
// ADR-0044 §2.9 (W1 unified shader rampa) +
// docs/Painter_projeto/01_brush_engine.md §1.8.2 (workgroup 8×8) + §1.8.3.
//
// Workgroup: 8×8 threads. Dispatch is 3D — one workgroup-z per stamp,
// workgroups_x × workgroups_y covering the largest stamp's bounding box
// (`ceil(max_footprint / 8)²`). Threads outside a smaller stamp's box
// early-out via bounds check.
//
// ## Output / ping-pong (T1.5 scope)
//
// **Two-texture ping-pong** (T1.5 ship). The pipeline binds DIFFERENT
// textures on `canvas_in` (read, `texture_2d<f32>` @binding(3)) and
// `canvas_out` (write, `texture_storage_2d<rgba8unorm>` @binding(2)) —
// the caller alternates A/B on each dispatch so the previous stamp's
// result is visible to the next stamp's blend, preserving Porter-Duff
// alpha-over ordering across overlapping stamps.
//
// Why not single-texture read+write? `read_write` storage on
// `rgba8unorm` requires adapter-specific features (wgpu
// `Features::TEXTURE_FORMAT_RGBA8UNORM_STORAGE` + the read+write
// access combo); the two-texture path is the portable baseline that
// works on every wgpu 28 backend (Metal/Vulkan/DX12/GL/WebGPU). W5+
// may upgrade to single-texture when the tier policy (ADR-0053)
// declares the feature available.
//
// **Per-dispatch ordering inside a workgroup:** when a single dispatch
// carries multiple stamps via `stamps[wg.z]`, overlap between two
// stamps in the SAME dispatch still has undefined order at the pixel
// (two workgroups writing the same texel race). Callers wanting strict
// temporal ordering MUST dispatch one stamp at a time and swap A↔B
// between calls; that's the StampScheduler's contract.
//
// ## ABI mirror — Stamp 96 bytes (ADR-0044 §2.3)
//
// Field order and offsets MUST match the Rust `Stamp` struct byte-for-byte.
// Rust-side guards: 17 compile-time `offset_of!` asserts in
// `crates/ph2d-painter-brush/src/stamp.rs`. Shader-side guard:
// `StampPipeline::stamp_struct_size_wgsl()==96` test in
// `stamp_pipeline.rs`, executed via naga reflection in `cargo test`.
//
// Critical hazard: `grain_offset_uv` lives at offset 68 (NOT align 8).
// WGSL `vec2<f32>` requires align 8 in every address space, so naga
// would insert 4 bytes of padding before it, shifting every subsequent
// field and breaking the ABI. We declare two scalars at offsets 68/72
// instead, and rebuild the vec2 in `stamp_grain_offset_uv()`.

struct Stamp {
    // offset 0..8 : position_world: [f32; 2]
    position_world_x: f32,
    position_world_y: f32,
    // offset 8..12
    size_px: f32,
    // offset 12..16
    rotation_rad: f32,
    // offset 16..32 : pressure / tilt / azimuth / barrel_roll
    pressure: f32,
    tilt: f32,
    azimuth: f32,
    barrel_roll: f32,
    // offset 32..48 : color_oklab: [f32; 4]
    color_oklab_l: f32,
    color_oklab_a: f32,
    color_oklab_b: f32,
    color_oklab_alpha: f32,
    // offset 48..60 : opacity / flow / wet_amount
    opacity: f32,
    flow: f32,
    wet_amount: f32,
    // offset 60..68 : shape_layer / grain_layer
    shape_layer: u32,
    grain_layer: u32,
    // offset 68..76 : grain_offset_uv (2× scalar — vide header hazard)
    grain_offset_u: f32,
    grain_offset_v: f32,
    // offset 76..80
    grain_scale: f32,
    // offset 80..96 : flags / rendering_mode / pigment_mode / _pad
    flags: u32,
    rendering_mode: u32,
    pigment_mode: u32,
    _pad: u32,
}

struct Globals {
    canvas_width: u32,
    canvas_height: u32,
    stamp_count: u32,
    _pad: u32,
}

// Flag bitmask — sync with crates/ph2d-painter-brush/src/stamp.rs::FLAG_*.
// **T1.6 consumes** HOVER_PREVIEW + PREDICTED_SAMPLE (early-out per
// ADR-0050) AND FLIP_X + FLIP_Y (sample-coord transform in `cs_stamp`).
// Remaining bits — GRAIN_BEHAVIOR_MOVING, BURNT_EDGES, WET_EDGES,
// LUMINANCE_BLENDING, GRAIN_PROCEDURAL, FLUID_SAMPLE — will be wired by
// the corresponding subsystems: T-grain W5+, T-wet-mix W7+, T-fluid
// W15+. All bits declared here so naga matches the Rust ABI at parse
// time and so the bitmask layout stays single-source.
const FLAG_SHAPE_FLIP_X: u32           = 1u;
const FLAG_SHAPE_FLIP_Y: u32           = 2u;
const FLAG_GRAIN_BEHAVIOR_MOVING: u32  = 4u;
const FLAG_BURNT_EDGES: u32            = 8u;
const FLAG_WET_EDGES: u32              = 16u;
const FLAG_LUMINANCE_BLENDING: u32     = 32u;
const FLAG_GRAIN_PROCEDURAL: u32       = 64u;
const FLAG_FLUID_SAMPLE: u32           = 128u;
const FLAG_HOVER_PREVIEW: u32          = 256u;
const FLAG_PREDICTED_SAMPLE: u32       = 512u;

// Maximum stamp footprint in pixels (mirrors MAX_STAMP_SIZE_PX in
// stamp.rs, which mirrors `PropertiesParams::max_size_px` upper bound
// from spec §1.3.11). Shader-side clamp is defense-in-depth — the CPU
// `encode()` path already filters and clamps, but this guards any
// future caller that bypasses the canonical path via a custom dispatch
// built on `StampPipeline::bind_group_layout` (audit 2026-05-26 D-3.H4).
const MAX_STAMP_SIZE_PX: u32 = 2048u;

@group(0) @binding(0) var<storage, read> stamps: array<Stamp>;
@group(0) @binding(1) var<uniform> globals: Globals;
@group(0) @binding(2) var canvas_out: texture_storage_2d<rgba8unorm, write>;
// T1.5: ping-pong read texture — the previous frame's canvas state.
// `texture_2d<f32>` (sampled, no sampler — accessed via `textureLoad`),
// portable across every wgpu 28 backend. Caller MUST bind a DIFFERENT
// texture here than `canvas_out` (same texture in both bindings is a
// validation error in wgpu when one of the views is storage-write).
@group(0) @binding(3) var canvas_in: texture_2d<f32>;

fn stamp_grain_offset_uv(s: Stamp) -> vec2<f32> {
    return vec2<f32>(s.grain_offset_u, s.grain_offset_v);
}

// OKLab → linear sRGB (D65) — Björn Ottosson 2020.
// Mirrors `OklabColor::to_linear` in ph2d-color (verified by reference
// red sample test). Inputs unclamped; out-of-gamut returns negative RGB.
fn oklab_to_linear_srgb(L: f32, a: f32, b: f32) -> vec3<f32> {
    let l_ = L + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = L - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = L - 0.0894841775 * a - 1.2914855480 * b;
    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;
    return vec3<f32>(
        4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3,
        -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3,
        -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3,
    );
}

// ── sRGB gamma transfer (gamma fix 2026-05-31) ───────────────────────────
//
// The canvas is sRGB-encoded (the sprite it backs uploads as
// `Rgba8UnormSrgb`), so alpha-over compositing must run in LINEAR light.
// These mirror `ph2d_color::srgb::{srgb_to_linear_byte, linear_to_srgb_byte}`
// (IEC 61966) on normalized [0,1] values — the literals are kept
// bit-identical to the Rust side so the CPU MVP (`cpu_render.rs`) and this
// GPU path stay in agreement (same discipline as the OKLab coefficient gate).
fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        return v / 12.92;
    }
    return pow((v + 0.055) / 1.055, 2.4);
}

fn linear_to_srgb(v: f32) -> f32 {
    let c = clamp(v, 0.0, 1.0);
    if c <= 0.0031308 {
        return c * 12.92;
    }
    return 1.055 * pow(c, 1.0 / 2.4) - 0.055;
}

// Gamma applies to STRAIGHT values; the canvas is stored premultiplied, so
// un-premultiply → decode/encode the RGB → re-premultiply. Alpha is light-
// linear (never gamma-encoded) and passes through.
fn premul_srgb_to_premul_linear(p: vec4<f32>) -> vec4<f32> {
    if p.a <= 0.0 {
        return vec4<f32>(0.0, 0.0, 0.0, p.a);
    }
    let s = p.rgb / p.a;
    let lin = vec3<f32>(srgb_to_linear(s.r), srgb_to_linear(s.g), srgb_to_linear(s.b));
    return vec4<f32>(lin * p.a, p.a);
}

fn premul_linear_to_premul_srgb(p: vec4<f32>) -> vec4<f32> {
    if p.a <= 0.0 {
        return vec4<f32>(0.0, 0.0, 0.0, p.a);
    }
    let s = p.rgb / p.a;
    let enc = vec3<f32>(linear_to_srgb(s.r), linear_to_srgb(s.g), linear_to_srgb(s.b));
    return vec4<f32>(enc * p.a, p.a);
}

// ── Procedural shape kernels (slot dispatch — T1.6) ──────────────────────
//
// Each kernel is a pure function `(uv) -> alpha` analytically identical to
// its CPU counterpart in `library.rs::shape_*`. The CPU side dispatches via
// `shape_alpha_for_slot(slot, u, v)`; the shader dispatches via
// `shape_alpha_for_slot(stamp.shape_layer, uv)`. **No atlas binding** —
// shapes are 100 % procedural through T1.6 to preserve HR-5 cross-OS
// bit-identical reproducibility (atlas R8 quantization would introduce
// ~1/255 per-pixel drift). Atlas binding lands when the first custom-art
// shape PNG ships (W6+: flat_chisel / bristle_spread / splatter_spread).
//
// Slot canon (mirrors `library.rs::*_SLOT` constants — ABI):
//   0 = round_hard   (Hermite smoothstep on radial 0.85..=1.0)
//   1 = round_soft   ((1 - d²)² radial — Gaussian-equivalent)
//   2 = square_hard  (Chebyshev distance + smoothstep 0.90..=1.0)
//   3 = oval_hard    (2:1 stretched radial — calligraphic pen-nib for
//                     `shape_rotation_follow`; added R4 V-2)
//
// `smooth_t` (not `smooth`) — `smooth` is a reserved WGSL keyword (sampler
// interpolation qualifier in fragment inputs).
fn round_hard_shape(uv: vec2<f32>) -> f32 {
    // Audit T1.6 O-3 + U-1: WGSL `length()` is an intrinsic whose lowering
    // on Metal / Vulkan / DX12 is implementation-defined (may use fused
    // reciprocal-sqrt + multiply, or `sqrt(dot(v,v))`, or a backend-
    // specific path). Cross-backend ULP equality is NOT mandated. The
    // CPU side uses scalar `(dx*dx + dy*dy).sqrt()` — to keep the gate
    // `cpu_shader_shape_kernels_textual_parity` honest (and to give CPU
    // and GPU the same numerical SOURCE pipeline), the shader uses the
    // same scalar form, not `length()`. **Note:** writing the same
    // formula in WGSL and Rust does NOT guarantee bit-identical
    // runtime results — `sqrt`, `cos`, `sin` are ULP-bounded across
    // backends (typically ≤1-4 ULP). HR-5 strict bit-identical requires
    // the `det-painter` feature path (declared in Cargo.toml; wiring
    // progressive — see ph2d-painter-brush::lib.rs Status section).
    let dx = uv.x - 0.5;
    let dy = uv.y - 0.5;
    let d = sqrt(dx * dx + dy * dy) / 0.5;
    let edge_t = clamp((d - 0.85) / 0.15, 0.0, 1.0);
    let smooth_t = edge_t * edge_t * (3.0 - 2.0 * edge_t);
    return 1.0 - smooth_t;
}

fn round_soft_shape(uv: vec2<f32>) -> f32 {
    let dx = uv.x - 0.5;
    let dy = uv.y - 0.5;
    let d_sq = (dx * dx + dy * dy) / 0.25; // normalize over radius 0.5
    if d_sq >= 1.0 {
        return 0.0;
    }
    let one_minus_d_sq = 1.0 - d_sq;
    return one_minus_d_sq * one_minus_d_sq;
}

fn square_hard_shape(uv: vec2<f32>) -> f32 {
    let dx = abs(uv.x - 0.5);
    let dy = abs(uv.y - 0.5);
    let d = max(dx, dy) / 0.5;
    let edge_t = clamp((d - 0.90) / 0.10, 0.0, 1.0);
    let smooth_t = edge_t * edge_t * (3.0 - 2.0 * edge_t);
    return 1.0 - smooth_t;
}

// `oval_hard` (slot 3, audit T1.6 V-2) — 2:1 oblong via stretched
// radial distance. Pairs with `shape_rotation_follow=true` to render a
// calligraphic pen-nib that aligns its long axis to the stroke direction.
fn oval_hard_shape(uv: vec2<f32>) -> f32 {
    let dx = (uv.x - 0.5) / 0.5;
    let dy = (uv.y - 0.5) / 0.25;
    let d = sqrt(dx * dx + dy * dy);
    let edge_t = clamp((d - 0.85) / 0.15, 0.0, 1.0);
    let smooth_t = edge_t * edge_t * (3.0 - 2.0 * edge_t);
    return 1.0 - smooth_t;
}

// Dispatch a procedural shape by `shape_layer` slot. Out-of-range slots
// fall back to `round_hard` (safer-degrade — same behavior as
// `library::shape_alpha_for_slot`).
fn shape_alpha_for_slot(slot: u32, uv: vec2<f32>) -> f32 {
    switch slot {
        case 0u: { return round_hard_shape(uv); }
        case 1u: { return round_soft_shape(uv); }
        case 2u: { return square_hard_shape(uv); }
        case 3u: { return oval_hard_shape(uv); }
        default: { return round_hard_shape(uv); }
    }
}

// ── Six rendering modes (ADR-0044 §2.4 + spec §1.5.2) ──
//
// **Premul invariant:** every mode below takes `src` and `dst` in
// premultiplied-alpha linear sRGB, and returns a premultiplied
// vec4<f32>. Callers can re-feed the result as `dst` in subsequent
// dispatches (T1.5+ ping-pong) without ever needing to un/re-multiply
// at the boundary — the canvas storage stays in premul throughout.
//
// **Glaze modes** (Light/Uniform/Intense/Heavy) accumulate alpha via
// Porter-Duff-style "over" composition with mode-specific α attenuation.
// **Blending modes** (Uniform/Intense Blending) do continuous color mix
// (lerp in straight color space, re-premultiplied) — used for wet
// media. Spec §1.5.2 mentions `+ accumulator` for Blending modes
// (wet ink reservoir); the slot lands in T-wet-mix W7+ subsystem
// (ADR-0044 §2.4 note — no Stamp ABI field for accumulator yet, so
// T1.4 implements with accumulator ≡ 0).
//
// **T1.5 ping-pong stage:** `dst` is loaded from `canvas_in` (read-only
// texture bound separately from `canvas_out`). Caller ping-pongs A↔B
// across stamps so each stamp's output is visible to the next stamp's
// blend. Premul invariant preserved end-to-end — canvas storage stays
// in premultiplied linear sRGB throughout the stroke.
//
// **Out-of-gamut clamp policy (T1.4):** out-of-gamut OKLab inputs
// produce negative LinearRGB components, which we clamp per-channel
// to `[0,1]` BEFORE feeding the rendering modes (cs_stamp body). This
// is a *component-clip* gamut map — not perceptually-correct. ADR-0051
// (T-color full) defines the proper OKLCH chroma-reduction gamut map;
// the boundary lands then. Documented behavior, not bug.

fn light_glaze(src: vec4<f32>, dst: vec4<f32>) -> vec4<f32> {
    // Partial-preserve under: dst attenuated by only 0.6 × α_s.
    return src + dst * (1.0 - src.a * 0.6);
}

fn uniform_glaze(src: vec4<f32>, dst: vec4<f32>) -> vec4<f32> {
    // Porter-Duff "over" — Photoshop Normal.
    return src + dst * (1.0 - src.a);
}

fn intense_glaze(src: vec4<f32>, dst: vec4<f32>) -> vec4<f32> {
    // Aggressive alpha curve — `α^0.5` covers faster while preserving
    // layering. Spec §1.5.2: `new = s * α_s^0.5 + L * (1 - α_s^0.5)`,
    // where `s` is the straight stamp color.
    //
    // Algebraic rearrangement preserves precision at small α: since
    // `s_premul = s_straight * α_s`, we have
    //   s_straight * sqrt(α_s) = s_premul / sqrt(α_s)
    // so we can avoid the `1/α_s` blow-up that the naive
    // `s_premul / α_s * sqrt(α_s)` would suffer at α_s → 0 (audit
    // 2026-05-26 D-3.M15).
    let alpha_s = clamp(src.a, 0.0, 1.0);
    if alpha_s < 1e-6 {
        return dst;
    }
    let aa = sqrt(alpha_s);
    let src_curved_rgb = src.rgb / aa; // = src_straight * sqrt(α_s)
    return vec4<f32>(src_curved_rgb, aa) + dst * (1.0 - aa);
}

fn heavy_glaze(src: vec4<f32>, dst: vec4<f32>) -> vec4<f32> {
    let r = src + dst * (1.0 - src.a * 0.85);
    return clamp(r, vec4<f32>(0.0), vec4<f32>(1.0));
}

fn uniform_blending(src: vec4<f32>, dst: vec4<f32>) -> vec4<f32> {
    // Continuous color mix preserving premultiplied invariant.
    // Audit 2026-05-26 D-3.C2 fix: previous impl mixed premul `dst`
    // with unmul `src`, breaking the invariant when `dst.a < 1`.
    //
    // Method: unmul both sides, lerp in straight color space by α_s,
    // compute output alpha via Porter-Duff "over" (Blending modes
    // still accumulate alpha to reach opaque after enough strokes),
    // re-premultiply at the end.
    //
    // Spec §1.5.2 mentions `+ accumulator` for wet ink reservoir
    // (ADR-0044 §2.4) — no Stamp ABI slot yet, treated as 0 here.
    let alpha_s = src.a;
    let alpha_d = dst.a;
    let result_a = alpha_s + alpha_d * (1.0 - alpha_s);
    if result_a < 1e-6 {
        return vec4<f32>(0.0);
    }
    let src_rgb = src.rgb / max(alpha_s, 1e-6);
    let dst_rgb = dst.rgb / max(alpha_d, 1e-6);
    let mixed_rgb = mix(dst_rgb, src_rgb, alpha_s);
    return vec4<f32>(mixed_rgb * result_a, result_a);
}

fn intense_blending(src: vec4<f32>, dst: vec4<f32>, wet: f32) -> vec4<f32> {
    // Wet pull — partial smudge of dst color into src before mix.
    // `wet=0` collapses to `uniform_blending`; `wet=1` fully averages
    // the brushed source with the canvas surface (smudge).
    //
    // **`wet` parameter semantics:** spec §1.5.2 names this `pull
    // (= wet_mix.pull)`. The Stamp ABI compresses the wet-mix subsystem
    // into a single `wet_amount` float (`stamp.rs:60`); the scheduler
    // populates it with the `pull` factor for Intense Blending and
    // with `dilution * load` for other wet contexts. Audit 2026-05-26
    // D-3.H5 — divergence documented, no spec amendment yet
    // (T-wet-mix W7+ will introduce slot separation if needed).
    //
    // Full Stokes-style smudge fluid sim is T-wet-mix W7+ (ADR-0049
    // Fluid Brushes Extension).
    let alpha_s = src.a;
    let alpha_d = dst.a;
    let result_a = alpha_s + alpha_d * (1.0 - alpha_s);
    if result_a < 1e-6 {
        return vec4<f32>(0.0);
    }
    let src_rgb = src.rgb / max(alpha_s, 1e-6);
    let dst_rgb = dst.rgb / max(alpha_d, 1e-6);
    let pull = clamp(wet, 0.0, 1.0);
    let smudged_src_rgb = mix(src_rgb, dst_rgb, 0.5 * pull);
    let mixed_rgb = mix(dst_rgb, smudged_src_rgb, alpha_s);
    return vec4<f32>(mixed_rgb * result_a, result_a);
}

fn apply_rendering_mode(mode: u32, src: vec4<f32>, dst: vec4<f32>, wet: f32) -> vec4<f32> {
    // `pigment_mode` (Stamp ABI slot, ADR-0044 §2.5) is handled in `cs_stamp`
    // BEFORE this dispatch: `PigmentMode::Mixbox` replaces the alpha-over colour
    // with a subtractive spectral mix; `Linear` (default) feeds the rendering
    // modes below unchanged. Mixbox is forced to Linear under `--features
    // det-painter` (ADR-0044 §2.5.1 — `exp`/`pow` aren't bit-identical cross-OS).
    switch mode {
        case 0u: { return light_glaze(src, dst); }
        case 1u: { return uniform_glaze(src, dst); }
        case 2u: { return intense_glaze(src, dst); }
        case 3u: { return heavy_glaze(src, dst); }
        case 4u: { return uniform_blending(src, dst); }
        case 5u: { return intense_blending(src, dst, wet); }
        default: { return uniform_glaze(src, dst); }
    }
}

// ── Mixbox — clean-room SPECTRAL subtractive pigment mixing (W5) ──────────────
//
// EXACT mirror of `crate::mixbox` (CPU): reconstruct a reflectance spectrum over a
// broad/overlapping Gaussian channel basis (asymmetric widths so blue+yellow → a
// clean green, not teal), mix by the WEIGHTED GEOMETRIC MEAN of the reflectances
// (the subtractive operator), integrate back, re-anchor the round-trip per colour
// (self-mix = identity), and keep the endpoints exact via a `4t(1-t)` blend with
// the linear lerp. The basis is recomputed per-invocation (`exp` is cheap on the
// GPU; Mixbox is an opt-in per-brush mode) so there are NO magic constants or
// extra bindings — only the same `MB_CENTERS`/`MB_SIGMAS` as the CPU. Working
// space = straight LINEAR sRGB (the canonical Mixbox space).
const MB_NB: u32 = 24u;
const MB_FLOOR: f32 = 1.0e-4;
// R, G, B (band 0 = short λ) — mirror of `mixbox::{CENTERS, SIGMAS}`.
fn mb_center(c: u32) -> f32 { return array<f32, 3>(19.0, 12.0, 4.0)[c]; }
fn mb_sigma(c: u32) -> f32 { return array<f32, 3>(5.0, 3.4, 7.0)[c]; }

/// Subtractive spectral mix of two straight-linear-sRGB colours at ratio `t`,
/// round-trip re-anchored (self-mix is the identity).
fn mb_spectral_mix(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> {
    // Channel responses (peak-1) + their sums (for the normalised integration).
    var wr: array<f32, 24>;
    var wg: array<f32, 24>;
    var wb: array<f32, 24>;
    var sr = 0.0;
    var sg = 0.0;
    var sb = 0.0;
    for (var i = 0u; i < MB_NB; i = i + 1u) {
        let fi = f32(i);
        let dr = (fi - mb_center(0u)) / mb_sigma(0u);
        let dg = (fi - mb_center(1u)) / mb_sigma(1u);
        let db = (fi - mb_center(2u)) / mb_sigma(2u);
        wr[i] = exp(-dr * dr); sr = sr + wr[i];
        wg[i] = exp(-dg * dg); sg = sg + wg[i];
        wb[i] = exp(-db * db); sb = sb + wb[i];
    }
    // WGM of the reconstructed reflectances, integrated back; plus each colour's
    // own round-trip for the re-anchor correction.
    var mixed = vec3<f32>(0.0);
    var rta = vec3<f32>(0.0);
    var rtb = vec3<f32>(0.0);
    for (var i = 0u; i < MB_NB; i = i + 1u) {
        let ra = clamp(wr[i] * a.r + wg[i] * a.g + wb[i] * a.b, MB_FLOOR, 1.0);
        let rb = clamp(wr[i] * b.r + wg[i] * b.g + wb[i] * b.b, MB_FLOOR, 1.0);
        let rm = pow(ra, 1.0 - t) * pow(rb, t);
        let mvec = vec3<f32>(wr[i] / sr, wg[i] / sg, wb[i] / sb);
        mixed = mixed + mvec * rm;
        rta = rta + mvec * ra;
        rtb = rtb + mvec * rb;
    }
    let ea = a - rta;
    let eb = b - rtb;
    return max(mixed + ea * (1.0 - t) + eb * t, vec3<f32>(0.0));
}

/// Endpoint-exact pigment lerp: linear at the ends, full subtractive at 50/50.
fn mb_lerp(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> {
    let tt = clamp(t, 0.0, 1.0);
    let spec = mb_spectral_mix(a, b, tt);
    return mix(mix(a, b, tt), spec, 4.0 * tt * (1.0 - tt));
}

@compute @workgroup_size(8, 8, 1)
fn cs_stamp(
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let stamp_idx = wg.z;
    if stamp_idx >= globals.stamp_count {
        return;
    }
    let stamp = stamps[stamp_idx];

    // Skip degenerate stamps. Defense-in-depth — the CPU encode side
    // already filters NaN/Inf/non-positive size_px and clamps to
    // `MAX_STAMP_SIZE_PX`, but a future caller bypassing
    // `StampPipeline::encode` (e.g. a custom dispatch built on the
    // bind group layout) could land here with garbage. WGSL `>` with
    // NaN is always false, so `!(size_px > 0.0)` catches NaN cleanly;
    // +Inf passes the `> 0` check but `min(ceil(+Inf), 2048)` saturates
    // safely to 2048 instead of `u32::MAX` (audit 2026-05-26 D-3.H4+L9).
    if !(stamp.size_px > 0.0) {
        return;
    }
    let footprint = min(u32(ceil(stamp.size_px)), MAX_STAMP_SIZE_PX);
    if footprint == 0u {
        return;
    }
    // HOVER_PREVIEW / PREDICTED_SAMPLE bits are reserved (ADR-0050) —
    // current scheduler should not emit them to the GPU buffer yet;
    // defensively skip if set so T-input migration can land without race.
    if (stamp.flags & (FLAG_HOVER_PREVIEW | FLAG_PREDICTED_SAMPLE)) != 0u {
        return;
    }

    let pixel_local_x = wg.x * 8u + lid.x;
    let pixel_local_y = wg.y * 8u + lid.y;
    if pixel_local_x >= footprint || pixel_local_y >= footprint {
        return;
    }

    // uv_axis ∈ [0,1]² across the footprint using **pixel-center convention**
    // (pixel `i` lives at `uv = (i+0.5)/footprint`). Mirrors the
    // texture-sample convention (audit 2026-05-26 D-1.M1).
    //
    // **T1.6 stamp-space transform pipeline** (apply in order):
    //   1. axis-aligned uv → centered `[-0.5, +0.5]²`
    //   2. flip — invert axis components per FLAG_SHAPE_FLIP_{X,Y}
    //   3. rotate — apply `rotation_rad` rotation matrix
    //   4. un-center → uv ∈ `[-Δ, 1+Δ]²` (Δ depends on rotation; outside
    //      `[0, 1]²` → shape kernel returns 0)
    //   5. dispatch shape by `stamp.shape_layer` slot
    //
    // The world-space write coordinate uses the original axis-aligned
    // `(pixel_local_x, pixel_local_y) - center_offset` offset — only the
    // **shape sample uv** rotates. The footprint stays axis-aligned (the
    // bounding box). For non-radial shapes (`square_hard`) the
    // `StampScheduler` enlarges `size_px` by √2 when `rotation_rad != 0`
    // so the rotated shape stays inside the bounding box (see
    // `library::rotated_footprint_scale`).
    let footprint_f = f32(footprint);
    let center_offset = (footprint_f - 1.0) * 0.5;
    let uv_axis = (vec2<f32>(f32(pixel_local_x), f32(pixel_local_y)) + vec2<f32>(0.5)) / footprint_f;

    // 1. Center to `[-0.5, +0.5]`.
    var uv_centered = uv_axis - vec2<f32>(0.5);

    // 2. Flip axes per flag bits. Done BEFORE rotation so flip+rotation
    // composes as the user expects (flip is in shape-local space).
    if (stamp.flags & FLAG_SHAPE_FLIP_X) != 0u {
        uv_centered.x = -uv_centered.x;
    }
    if (stamp.flags & FLAG_SHAPE_FLIP_Y) != 0u {
        uv_centered.y = -uv_centered.y;
    }

    // 3. Rotate the sample direction. We rotate by `-rotation_rad` here
    // because we're transforming the **sample coordinate** into the
    // shape's reference frame (inverse of the rotation we'd apply to the
    // shape itself). Use trig identity `cos(-x) = cos(x)` and
    // `sin(-x) = -sin(x)` to (a) skip one unary-negation per stamp, and
    // (b) avoid feeding a backend intrinsic a negated argument — which
    // on Metal/Vulkan COULD take a different precision path than the
    // base argument (theoretical, not measured). Audit T1.6 U-9 +
    // R5-B1-1: original draft claimed "eliminates ±0 sign-bit drift"
    // but that was WRONG (`u9_trig_identity_old_vs_new_form_equivalence`
    // test in scheduler proves both forms produce bit-identical outputs
    // for ±0 inputs). `cos/sin` lower to backend intrinsics (Metal
    // `metal::cos`, SPIR-V `OpExtInst Cos/Sin`, DX12 `cos/sin`); WGSL
    // spec mandates ULP-bounded precision only (typically ≤1-4 ULP),
    // **NOT bit-identical cross-backend** (audit T1.6 U-1). For HR-5
    // strict cross-OS reproducibility, set `--features det-painter`
    // (currently no-op; lookup-table-pinned trig lands T-numerical-
    // parity W2+).
    let cos_r = cos(stamp.rotation_rad);
    let sin_r = -sin(stamp.rotation_rad);
    let uv_rotated = vec2<f32>(
        uv_centered.x * cos_r - uv_centered.y * sin_r,
        uv_centered.x * sin_r + uv_centered.y * cos_r,
    );

    // 4. Un-center back to `[0, 1]` shape sample space. Outside `[0, 1]²`
    // the shape kernel returns 0 — handled inside `shape_alpha_for_slot`
    // (round_soft has explicit d_sq guard; round_hard/square_hard clamp
    // edge_t to 1.0 → alpha 0).
    let uv = uv_rotated + vec2<f32>(0.5);

    // 5. Dispatch to per-slot shape kernel.
    let shape_alpha = shape_alpha_for_slot(stamp.shape_layer, uv);
    if shape_alpha < (1.0 / 255.0) {
        return;
    }

    // World-space pixel coords — axis-aligned write target (footprint is
    // the bounding box; only the shape sample rotates). Centering uses
    // `f32` so odd footprints don't drift half a pixel (audit 2026-05-26
    // D-1.M1); sub-pixel `position_world` snaps to nearest integer via
    // `floor(x + 0.5)` — IEEE 754 well-defined round-half-up, identical
    // across Metal / Vulkan / D3D12 backends (audit 2026-05-26 D-2.F3 —
    // WGSL `round()` halfway is impl-defined, `floor(x + 0.5)` is not).
    let world_x_f = stamp.position_world_x + f32(pixel_local_x) - center_offset;
    let world_y_f = stamp.position_world_y + f32(pixel_local_y) - center_offset;
    let world_x = i32(floor(world_x_f + 0.5));
    let world_y = i32(floor(world_y_f + 0.5));
    if world_x < 0
       || world_y < 0
       || world_x >= i32(globals.canvas_width)
       || world_y >= i32(globals.canvas_height) {
        return;
    }
    let coord = vec2<i32>(world_x, world_y);

    let rgb_linear = oklab_to_linear_srgb(
        stamp.color_oklab_l,
        stamp.color_oklab_a,
        stamp.color_oklab_b,
    );
    // Out-of-gamut OKLab → component-clip clamp to `[0,1]` BEFORE
    // entering the rendering modes. T1.4 contract: write-only storage
    // texture stays in `[0,1]` premultiplied space; downstream
    // (T1.5+ ping-pong) reads `dst` already in that space. ADR-0051
    // (T-color full) replaces this with perceptually-correct OKLCH
    // chroma-reduction gamut map at the boundary. Audit 2026-05-26
    // D-3.H8+M8 / D-2.F2.
    let rgb_clamped = clamp(rgb_linear, vec3<f32>(0.0), vec3<f32>(1.0));
    // Combined alpha = stamp color alpha · brush opacity · flow · shape α.
    let combined_alpha = clamp(
        stamp.color_oklab_alpha * stamp.opacity * stamp.flow * shape_alpha,
        0.0,
        1.0,
    );
    let src_premul = vec4<f32>(rgb_clamped * combined_alpha, combined_alpha);

    // T1.5 ping-pong: `dst` is the previous canvas state at this pixel
    // (caller binds different textures to `canvas_in` and `canvas_out`
    // and swaps A↔B between stamps). For the very first stamp of a
    // stroke, the caller initializes `canvas_in` with the source image
    // (set_source pixels), so `dst` carries the existing artwork.
    //
    // **Premul invariant:** `canvas_in` storage is premultiplied linear
    // sRGB (caller guarantees by uploading set_source RGBA through the
    // straight→premul conversion before the first dispatch). The shader
    // never un/re-multiplies at the boundary — `dst` enters the
    // rendering-mode functions in the same premul space they expect.
    //
    // `textureLoad` with a constant mip level 0 is the portable read on
    // every wgpu 28 backend; no sampler needed. `vec4<i32>(coord)` is
    // safe because the bounds check above guarantees `world_x/world_y`
    // are in `[0, canvas_w/h)`.
    // Gamma fix (2026-05-31): the stored canvas is sRGB-encoded premultiplied;
    // decode to premultiplied LINEAR so the alpha-over rendering modes blend in
    // linear light (mirror of cpu_render.rs's decode-on-read).
    let dst = premul_srgb_to_premul_linear(textureLoad(canvas_in, coord, 0));

    var result: vec4<f32>;
    if stamp.pigment_mode == 1u {
        // Mixbox: the deposited pigment MIXES subtractively with the wet backdrop.
        // The new canvas colour = `mb_lerp(backdrop, brush, deposit_fraction)`; the
        // coverage accumulates as a normal alpha-over. `deposit_fraction` is the
        // share of the final pigment that is NEW (1 over a blank canvas → pure
        // brush; 0.5 over opaque paint → an even mix → blue+yellow = green). This
        // replaces the linear colour blend the rendering modes would do; the wet
        // glaze/blend accumulation × pigment is a documented follow-up.
        let out_a = combined_alpha + dst.a * (1.0 - combined_alpha);
        let dst_straight = dst.rgb / max(dst.a, 1e-4);
        let deposit = combined_alpha / max(out_a, 1e-4);
        let mixed = mb_lerp(dst_straight, rgb_clamped, deposit);
        result = vec4<f32>(mixed * out_a, out_a);
    } else {
        result = apply_rendering_mode(stamp.rendering_mode, src_premul, dst, stamp.wet_amount);
    }
    // NaN guard: WGSL `clamp(NaN, lo, hi)` is implementation-defined
    // (Metal returns low, SPIR-V can propagate NaN). `result == result`
    // is `false` only for NaN — replace NaN components with 0 before
    // clamp to keep `textureStore(rgba8unorm, …)` deterministic across
    // backends (audit 2026-05-26 D-2.F7).
    let result_finite = select(vec4<f32>(0.0), result, result == result);
    let result_clamped = clamp(result_finite, vec4<f32>(0.0), vec4<f32>(1.0));

    // Re-encode premultiplied linear → premultiplied sRGB before store, so the
    // canvas stays sRGB for the `Rgba8UnormSrgb` sprite (mirror of the CPU
    // encode-on-write). Gamma fix (2026-05-31).
    textureStore(canvas_out, coord, premul_linear_to_premul_srgb(result_clamped));
}
