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
// ## Output / ping-pong (T1.4 scope)
//
// `canvas_out` is `texture_storage_2d<rgba8unorm, write>` (write-only —
// portable baseline; read_write storage on rgba8unorm requires
// adapter-specific features and is gated to W5+ via ADR-0044 §2.9).
//
// **Limitation registered as follow-up:** without read-side access,
// stamps that overlap in the SAME dispatch produce undefined ordering
// (last-thread-wins race). For Day 5 single-stamp validation this is
// inconsequential. Multi-stamp alpha-over correctness lands in T1.5
// when the StampScheduler emits stamps with temporal ordering (ping-pong
// canvas A↔B per stamp); see ADR-0044 §2.9 + spec §1.8.3 caminho
// W5+-especializado.
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
// T1.4 consumes only HOVER_PREVIEW and PREDICTED_SAMPLE (defensive early-out
// per ADR-0050); the remaining bits (FLIP_X/Y, GRAIN_BEHAVIOR_MOVING,
// BURNT_EDGES, WET_EDGES, LUMINANCE_BLENDING, GRAIN_PROCEDURAL, FLUID_SAMPLE)
// will be wired by the corresponding subsystems in T1.5+ (shape atlas),
// W7+ (wet mix), W15+ (fluid). Declared here so naga matches the Rust ABI
// at parse time and so the bitmask layout stays single-source.
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

// Round-hard procedural shape. **Semantically-equivalent radial
// smoothstep** of `library::round_hard_shape` (CPU side) — the same
// analytic formula, but sampled on the GPU at the stamp's actual
// footprint grid (the CPU atlas in `library.rs` is fixed 256² R8 for
// T1.5+ atlas binding; the two grids differ for stamps where size_px ≠
// 256 — they agree on the analytic curve, not on quantized pixels).
//
//   d        = ‖uv - (0.5, 0.5)‖ / 0.5           // normalized radial distance
//   edge_t   = clamp((d - 0.85) / 0.15, 0, 1)    // smoothstep transition band
//   smooth   = edge_t² · (3 - 2·edge_t)          // Hermite smoothstep
//   alpha    = 1 - smooth                        // opaque core, fading edge
//
// Inline procedural is the T1.4 baseline (no shape atlas wired yet).
// T1.5+ swaps this for an atlas sample via the shape texture array bound
// at @group(0) @binding(3) — public API of StampPipeline absorbs the
// change without breaking callers.
fn round_hard_shape(uv: vec2<f32>) -> f32 {
    let d = length(uv - vec2<f32>(0.5, 0.5)) / 0.5;
    let edge_t = clamp((d - 0.85) / 0.15, 0.0, 1.0);
    // `smooth_t` (not `smooth`) — `smooth` is a reserved WGSL keyword
    // (sampler interpolation qualifier in fragment inputs).
    let smooth_t = edge_t * edge_t * (3.0 - 2.0 * edge_t);
    return 1.0 - smooth_t;
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
// **T1.4 write-only stage:** since `canvas_out` is write-only, `dst` is
// the *initial* (cleared/loaded) state for this dispatch — overlapping
// stamps in the same dispatch race. Multi-stamp accumulation arrives
// in T1.5 via per-stamp ping-pong. For Day 5 (single stamp on cleared
// canvas) the result is identical to the W5+ correct path.
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
    // `pigment_mode` is a Stamp ABI slot (ADR-0044 §2.5) reserved for
    // the Mixbox path; T1.4 W1-unified shader ignores the variant and
    // always uses Linear pigment (lerp in OKLab-derived linear sRGB).
    // Mixbox compute lands in T1.X+ — gated by `--features det-painter`
    // exclusion per ADR-0044 §2.5.1.
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

    // uv ∈ [0,1]² across the footprint using **pixel-center convention**
    // (pixel `i` lives at `uv = (i+0.5)/footprint`). Mirrors the
    // texture-sample convention so atlas binding in T1.5 lands at the
    // same sample positions (audit 2026-05-26 D-1.M1).
    let footprint_f = f32(footprint);
    let uv = (vec2<f32>(f32(pixel_local_x), f32(pixel_local_y)) + vec2<f32>(0.5)) / footprint_f;

    // T1.4 inline procedural shape (round_hard). T1.5 atlas sampling via
    // `shape_layer` index. Shape × pressure/flow attenuation = stamp alpha.
    let shape_alpha = round_hard_shape(uv);
    if shape_alpha < (1.0 / 255.0) {
        return;
    }

    // World-space pixel coords. T1.4 ignores `rotation_rad` (axis-aligned
    // footprint); T1.5 applies the rotation matrix when the
    // `StampScheduler` emits oriented stamps.
    //
    // Centering uses `f32` so odd footprints don't drift half a pixel
    // (audit 2026-05-26 D-1.M1); sub-pixel `position_world` is snapped
    // to nearest integer via `floor(x + 0.5)` — IEEE 754 well-defined
    // round-half-up, identical across Metal / Vulkan / D3D12 backends
    // (audit 2026-05-26 D-2.F3 — WGSL `round()` halfway is impl-defined,
    // `floor(x + 0.5)` is not). T1.5 may upgrade to bilinear sub-pixel
    // write once the canvas is rgba16f (W5+).
    let center_offset = (footprint_f - 1.0) * 0.5;
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

    // T1.4 write-only stage: `dst` is the texture's initial state at this
    // pixel for THIS dispatch. With overlapping stamps in the same
    // dispatch the result depends on dispatch order (race). Day 5 smoke
    // = 1 stamp on cleared canvas, no overlap, no race.
    //
    // T1.5 ping-pong: caller binds `canvas_in: texture_2d<f32>` at
    // @binding(3) and calls `textureLoad(canvas_in, coord, 0)` here
    // before applying the rendering mode; this shader's public dispatch
    // surface absorbs that change without breaking T1.4 callers (extra
    // binding is `Some` only in the W5+ pipeline variant).
    let dst = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    let result = apply_rendering_mode(stamp.rendering_mode, src_premul, dst, stamp.wet_amount);
    // NaN guard: WGSL `clamp(NaN, lo, hi)` is implementation-defined
    // (Metal returns low, SPIR-V can propagate NaN). `result == result`
    // is `false` only for NaN — replace NaN components with 0 before
    // clamp to keep `textureStore(rgba8unorm, …)` deterministic across
    // backends (audit 2026-05-26 D-2.F7).
    let result_finite = select(vec4<f32>(0.0), result, result == result);
    let result_clamped = clamp(result_finite, vec4<f32>(0.0), vec4<f32>(1.0));

    textureStore(canvas_out, coord, result_clamped);
}
