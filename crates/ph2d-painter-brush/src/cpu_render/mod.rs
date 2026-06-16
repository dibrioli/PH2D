//! CPU stamp render — paridade ULP-bounded com `shader/stamp.wgsl`.
//!
//! ## Por que existir um caminho CPU
//!
//! `StampPipeline` (GPU compute) é o caminho canônico do spec §1.2. Ele
//! está completo, validado por naga, com ABI 96B FROZEN e gate de
//! coefficients bit-identicos (`shader_oklab_coefficients_bit_identical_
//! with_rust`). O que falta para usá-lo numa stroke real é integração
//! de lifecycle GPU (texture creation + upload + ping-pong dispatch +
//! download + sync) no bridge do shell.
//!
//! Para o **Day-7 marker** (primeira pintura visível) este módulo entrega
//! a **mesma matemática** em CPU — kernels analíticos via
//! `library::shape_alpha_for_slot()` (**4 procedural slots wired T1.6**:
//! `round_hard=0` / `round_soft=1` / `square_hard=2` / `oval_hard=3`
//! — last one added R4 V-2) + as mesmas constantes OKLab + as mesmas
//! funções de rendering mode (em Rust, textualmente pinadas via gate
//! `cpu_shader_shape_kernels_textual_parity` e algebricamente cobertas
//! pelos rendering-mode parity tests). É o
//! caminho que valida que scheduler + Stamp ABI + rendering modes +
//! shape kernels formam uma cadeia coerente, sem depender ainda da
//! integração GPU end-to-end.
//!
//! ## Paridade com shader — escopo do claim
//!
//! - **OKLab → linear sRGB:** mesmas 15 constantes do shader (
//!   `stamp.wgsl::oklab_to_linear_srgb`); CPU usa os literais
//!   `0.396_337_78` etc. que o gate `shader_oklab_coefficients_bit_
//!   identical_with_rust` cobre.
//! - **Shape dispatch (T1.6):** both CPU and shader call
//!   `library::shape_alpha_for_slot(slot, u, v)` / `shape_alpha_for_slot`
//!   on `stamp.shape_layer` — **four procedural slots wired in T1.6**
//!   (`round_hard=0`, `round_soft=1`, `square_hard=2`, `oval_hard=3`
//!   — last added R4 V-2 for `shape_rotation_follow` demo). Each
//!   slot's analytic formula is **independently pinned** in both CPU
//!   (`library::shape_*`) and shader (`stamp.wgsl::*_shape`) by the
//!   textual gate `cpu_shader_shape_kernels_textual_parity` — runtime
//!   CPU↔GPU pixel parity requires a wgpu device in CI and is tracked
//!   as **T-numerical-parity follow-up W2+** (audit T1.6 O-2). **No
//!   atlas binding** — atlas R8 quantization would drift ~1/255 per
//!   pixel and break HR-5. Atlas binding lands when custom-art shape
//!   PNGs ship (W6+ — `flat_chisel` etc.); see `library.rs` header.
//! - **Rotation + flip (T1.6):** the shape sample coordinate is
//!   transformed per `stamp.rotation_rad` + `FLAG_SHAPE_FLIP_{X,Y}` flag
//!   bits BEFORE the shape kernel is called. The footprint (world-space
//!   write target) stays axis-aligned; only the **shape sample uv**
//!   rotates. For non-radial shapes (`square_hard`), the `StampScheduler`
//!   enlarges `size_px` by `√2` when `rotation_rad != 0` so the rotated
//!   shape stays inside the bounding box.
//! - **Rendering modes:** cópias 1:1 das funções do shader (Light /
//!   Uniform / Intense / Heavy Glaze + Uniform / Intense Blending).
//!   Tests `rendering_mode_zero_src_is_identity_on_dst`,
//!   `uniform_glaze_full_opacity_replaces_dst`,
//!   `rendering_mode_premul_invariant_preserved`,
//!   `uniform_blending_at_full_alpha_equals_src_color`,
//!   `intense_glaze_precision_at_small_alpha` (round 1 A-H2) +
//!   `cpu_shader_textual_parity_all_six_modes` (round 2 F3 + round
//!   6 R6-LN-2) cobrem propriedades algébricas + paridade textual
//!   com shader em todos os 6 modos.
//!   **ULP-bounded near alpha_s → 0:** o termo `1/max(alpha_s, 1e-6)`
//!   no Uniform/Intense Blending pode acumular ULP drift entre
//!   backends GPU (Metal/Vulkan/D3D12) e Rust f32 quando `alpha_s` é
//!   muito pequeno; em região `alpha_s ∈ [1e-6, 1.0]` a paridade
//!   textual é estável, mas o output em pixels permanece **ULP-bounded
//!   (~1-4 ULP) cross-backend, NOT bit-identical** — `sqrt/cos/sin`
//!   nas GPU backends têm precisão impl-defined per WGSL spec (audit
//!   T1.6 U-1).
//! - **Premul invariant:** funções `apply_rendering_mode` operam sobre
//!   `vec4` premul (entrada e saída). Storage do canvas neste módulo é
//!   STRAIGHT (mesma convenção do `bgremoval` template + RasterEditTool's
//!   `set_source`); a conversão premul-↔-straight acontece por pixel dentro
//!   de `apply_one_stamp`. O shader em contraste armazena PREMUL em
//!   `texture_storage_2d<rgba8unorm>`. Na migração futura para o caminho
//!   GPU (T-perf W5+), o bridge deverá fazer straight-→-premul no upload de
//!   `set_source → canvas_a` e premul-→-straight no download
//!   `canvas_front → canvas_rgba`. Audit T1.5 round 1 A-C1: diferença de
//!   convenção entre layers, não regressão.
//!
//! ## Fronteira de troca para GPU
//!
//! Quando o bridge ganhar `PainterGpuState` (T-perf W5+ ou posterior),
//! `PainterTool::queue_pointer` passará a publicar Stamps em vez de
//! aplicar CPU. A API pública (`current_preview`, `take_pending_commit`,
//! etc.) não muda; só a implementação interna troca CPU→GPU. Este
//! módulo permanece como path de **reference** para golden tests dentro
//! de **um mesmo Rust toolchain target** — cross-OS bit-identical
//! requer `--features det-painter` + libm-pinned trig (T-numerical-
//! parity W2+; audit T1.6 U-1).
//!
//! Follow-up registrado: GPU integration cycle não bloqueia Day-7
//! ship; sessão T-perf substitui `apply_stamps` por `dispatch_stamps`
//! com `StampPipeline::encode` + retained texture state + straight↔premul
//! conversion at the upload/download boundary.

use crate::rendering_mode::RenderingMode;
use crate::stamp::Stamp;
use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_byte};

/// Aplica `stamps` sobre `canvas` (RGBA8 straight, sem gamma — treated as
/// linear matching wgpu `rgba8unorm` semantics).
///
/// `canvas.len()` deve ser `width * height * 4`. Stamps com `size_px` não-
/// finito ou `<= 0` são pulados (paridade defensiva com shader).
///
/// **Single-source semantics:** o canvas é mutado in-place; cada stamp
/// lê o estado pós-stamps-anteriores (sequencial), garantindo ordem
/// Porter-Duff alpha-over correta — equivalente ao ping-pong A↔B do
/// pipeline GPU (single dispatch por stamp + swap).
pub fn apply_stamps(canvas: &mut [u8], width: u32, height: u32, stamps: &[Stamp]) {
    apply_stamps_with_options(canvas, width, height, stamps, false);
}

/// Like [`apply_stamps`], but `alpha_lock` restricts painting to existing alpha
/// (§2.10): a pixel that starts fully transparent is left untouched, and the
/// destination alpha is held constant (only the color blends within the locked
/// alpha). Used when the active layer has its alpha locked.
pub fn apply_stamps_with_options(
    canvas: &mut [u8],
    width: u32,
    height: u32,
    stamps: &[Stamp],
    alpha_lock: bool,
) {
    apply_stamps_buildup_inner(canvas, None, width, height, stamps, alpha_lock, 0.0);
}

/// Like [`apply_stamps_with_options`] (build-up / `accumulate`), but ALSO
/// accumulates each dab's per-pixel coverage into `coverage` (`cov ← cov +
/// α·(1−cov)`). The build-up render itself is byte-identical to
/// [`apply_stamps_with_options`]; the coverage buffer is a side output used by
/// the global paper-tooth path so build-up strokes interact with the canvas
/// grain too, not only the wash path.
#[allow(clippy::too_many_arguments)]
pub fn apply_stamps_buildup(
    canvas: &mut [u8],
    coverage: &mut [f32],
    width: u32,
    height: u32,
    stamps: &[Stamp],
    alpha_lock: bool,
    paper_grain: f32,
) {
    let n = (width as usize) * (height as usize);
    assert_eq!(coverage.len(), n, "coverage size must match width*height");
    apply_stamps_buildup_inner(
        canvas,
        Some(coverage),
        width,
        height,
        stamps,
        alpha_lock,
        paper_grain,
    );
}

#[allow(clippy::too_many_arguments)]
fn apply_stamps_buildup_inner(
    canvas: &mut [u8],
    mut coverage: Option<&mut [f32]>,
    width: u32,
    height: u32,
    stamps: &[Stamp],
    alpha_lock: bool,
    paper_grain: f32,
) {
    // Audit T1.5 round 1 A-L4 — production-grade length guard so a
    // mismatch panics on the spot instead of reading/writing past the
    // canvas in release builds.
    assert_eq!(
        canvas.len(),
        (width as usize) * (height as usize) * 4,
        "canvas buffer size must match width*height*4 RGBA8"
    );
    for stamp in stamps {
        apply_one_stamp(
            canvas,
            coverage.as_deref_mut(),
            width,
            height,
            stamp,
            alpha_lock,
            paper_grain,
        );
    }
}

/// Base spatial frequency for procedural grain — world pixels → noise space.
const GRAIN_BASE_FREQ: f32 = 0.35;
/// Fixed grain seed (per-brush variation rides in `grain_offset_uv`; v1 = 0).
const GRAIN_SEED: u32 = 0x9e37_79b9;

/// **Global canvas paper tooth** (the #1 "real media vs digital" differentiator —
/// Procreate Grain / Photoshop Texture Depth, and watercolor granulation). A static
/// paper-height field in WORLD space (so it is consistent across strokes and does
/// NOT crawl with the brush), with a **pressure-aware tooth threshold**: a light
/// touch only deposits on the paper's crests (the valleys show through), firmer
/// pressure fills the valleys (Photoshop Depth/Min-Depth; USPTO 10217253). Returns a
/// coverage multiplier ∈[0,1]. `strength` 0 = off (flat); ~0.45 = a tasteful tooth.
const PAPER_TOOTH_FREQ: f32 = 0.22; // ~4.5px wavelength paper grain
const PAPER_TOOTH_SEED: u32 = 0x70a9_e2c5;
/// Even at full pressure the valleys are only this fraction filled, so the tooth
/// never fully vanishes (a flat fill would read digital again) — Photoshop Min-Depth.
const PAPER_TOOTH_MAX_FILL: f32 = 0.7;
/// Smoothstep softness of the tooth threshold.
const PAPER_TOOTH_SOFT: f32 = 0.24;

/// Raw paper-tooth height at a world pixel ∈[0,1] (1 = crest, 0 = valley) — the
/// SAME world-space field [`paper_tooth_factor`] thresholds. Exposed so any pen-up
/// pass reads the identical paper surface for granulation (pigment sediments into
/// the valleys), keeping the live tooth and the dry-down grain spatially coherent.
#[inline]
pub(crate) fn paper_tooth_height(world_x: f32, world_y: f32) -> f32 {
    crate::grain_noise::grain_value(
        crate::grain_noise::GRAIN_SIMPLEX,
        world_x * PAPER_TOOTH_FREQ,
        world_y * PAPER_TOOTH_FREQ,
        PAPER_TOOTH_SEED,
    )
}

#[inline]
fn paper_tooth_factor(world_x: f32, world_y: f32, pressure: f32, strength: f32) -> f32 {
    if strength <= 0.0 {
        return 1.0;
    }
    // Multi-octave paper height in world space ∈[0,1] (1 = crest, 0 = valley).
    let tooth = paper_tooth_height(world_x, world_y);
    // Pressure raises the deposit threshold's "fill" — high pressure fills valleys.
    let fill = pressure.clamp(0.0, 1.0) * PAPER_TOOTH_MAX_FILL;
    let thr = 1.0 - fill; // crests (tooth > thr) deposit; valleys (tooth < thr) show paper
    let lo = thr - PAPER_TOOTH_SOFT;
    let hi = thr + PAPER_TOOTH_SOFT;
    let t = ((tooth - lo) / (hi - lo).max(1e-4)).clamp(0.0, 1.0);
    let cov = t * t * (3.0 - 2.0 * t); // smoothstep
    // Blend toward full coverage by (1−strength) so `strength` dials the tooth depth.
    1.0 - strength * (1.0 - cov)
}

/// Procedural-grain coverage multiplier ∈ [0,1] for one sample. `1.0` (no-op) when
/// the stamp carries no procedural grain. Texturized = world-space static paper;
/// Moving = stamp-relative (grain smears with the stroke). Multiply blend with
/// `depth`. Mirror of `stamp.wgsl::grain_factor`.
#[inline]
fn grain_factor(stamp: &Stamp, world_x: f32, world_y: f32, u: f32, v: f32) -> f32 {
    if (stamp.flags & crate::stamp::FLAG_GRAIN_PROCEDURAL) == 0 {
        return 1.0;
    }
    let gtype = stamp.grain_layer & 0xFF;
    let depth = (((stamp.grain_layer >> 8) & 0xFF) as f32) / 255.0;
    let (sx, sy) = if (stamp.flags & crate::stamp::FLAG_GRAIN_BEHAVIOR_MOVING) != 0 {
        let mf = 32.0 / stamp.grain_scale.max(0.01);
        (u * mf, v * mf) // stamp-relative (u,v ∈ [0,1]) — grain follows the stroke
    } else {
        let f = GRAIN_BASE_FREQ / stamp.grain_scale.max(0.01);
        (world_x * f, world_y * f) // static paper texture
    };
    let n = crate::grain_noise::grain_value(
        gtype,
        sx + stamp.grain_offset_uv[0],
        sy + stamp.grain_offset_uv[1],
        GRAIN_SEED,
    );
    1.0 - depth * (1.0 - n)
}

#[allow(clippy::too_many_arguments)]
fn apply_one_stamp(
    canvas: &mut [u8],
    mut coverage: Option<&mut [f32]>,
    width: u32,
    height: u32,
    stamp: &Stamp,
    alpha_lock: bool,
    paper_grain: f32,
) {
    // Paridade com shader: filtra degenerate sizes antes de qualquer
    // arithmetic. Mantém canvas inalterado para inputs lixo. NaN é
    // caught pelo `is_finite()` (não-finito); ≤ 0 captura zero/negativo.
    if !stamp.size_px.is_finite()
        || stamp.size_px <= 0.0
        || !stamp.position_world[0].is_finite()
        || !stamp.position_world[1].is_finite()
    {
        return;
    }
    let footprint = (stamp.size_px.ceil() as u32).min(crate::stamp::MAX_STAMP_SIZE_PX);
    if footprint == 0 {
        return;
    }
    // HOVER_PREVIEW / PREDICTED_SAMPLE — pular como o shader faz.
    if (stamp.flags & (crate::stamp::FLAG_HOVER_PREVIEW | crate::stamp::FLAG_PREDICTED_SAMPLE)) != 0
    {
        return;
    }

    let footprint_f = footprint as f32;
    let center_offset = (footprint_f - 1.0) * 0.5;
    // Use the TRUE (fractional) diameter for the shape normalization + AA so the
    // sub-pixel sampling is accurate (footprint is its integer ceil).
    let inv_size = 1.0 / stamp.size_px;
    let aa = crate::library::SHAPE_AA_PX * inv_size;

    // OKLab → linear sRGB (mesmos coefficients do shader, gate-protected).
    let [l, a, b, color_alpha] = stamp.color_oklab;
    let rgb_linear = oklab_to_linear_srgb(l, a, b);
    // Component clip (paridade D-3.H8+M8 / D-2.F2). Out-of-gamut → 0..1.
    let rgb_clamped = [
        rgb_linear[0].clamp(0.0, 1.0),
        rgb_linear[1].clamp(0.0, 1.0),
        rgb_linear[2].clamp(0.0, 1.0),
    ];

    let opacity = stamp.opacity.clamp(0.0, 1.0);
    let flow = stamp.flow.clamp(0.0, 1.0);
    let mode = RenderingMode::from_u32(stamp.rendering_mode);
    let wet = stamp.wet_amount;
    let shape_slot = stamp.shape_layer;
    // Mixbox: precompute the brush pigment ONCE per stamp (constant across the
    // footprint) — the per-pixel `mix_prepared` then skips the brush-side solve.
    let pigment =
        (stamp.pigment_mode == 1).then(|| crate::pigment_mix::prepare_pigment(rgb_clamped));

    // T1.6 stamp-space transform: precompute rotation cos/sin + flip
    // signs once per stamp (constant across all per-pixel iterations of
    // this stamp). Rotation by `-rotation_rad` because we transform the
    // sample coordinate into the shape's reference frame (inverse of the
    // rotation we'd apply to the shape itself). Audit T1.6 U-9 (revised
    // post A1-U9): use trig identity `cos(-x) = cos(x)` and `sin(-x) =
    // -sin(x)` to (a) skip one unary-negation per stamp, and (b) avoid
    // feeding a negated argument to backend intrinsics — which on
    // Metal/Vulkan COULD take a different precision path than the base
    // argument (theoretical concern, not measured). Both forms produce
    // bit-identical outputs at ±0 (verified by `u9_trig_identity_old_
    // vs_new_form_equivalence` test).
    let cos_r = stamp.rotation_rad.cos();
    let sin_r = -stamp.rotation_rad.sin();
    let flip_x_sign: f32 = if (stamp.flags & crate::stamp::FLAG_SHAPE_FLIP_X) != 0 {
        -1.0
    } else {
        1.0
    };
    let flip_y_sign: f32 = if (stamp.flags & crate::stamp::FLAG_SHAPE_FLIP_Y) != 0 {
        -1.0
    } else {
        1.0
    };

    let canvas_w = width as i32;
    let canvas_h = height as i32;

    for py in 0..footprint {
        for px in 0..footprint {
            // Round half-up positioning (paridade D-2.F3). The footprint grid maps
            // `px → world_x` CONSECUTIVELY (`floor(a + px)` steps by exactly 1), so
            // there are no gaps or double-writes — same pixels as before.
            let world_x =
                (stamp.position_world[0] + (px as f32) - center_offset + 0.5).floor() as i32;
            let world_y =
                (stamp.position_world[1] + (py as f32) - center_offset + 0.5).floor() as i32;
            if world_x < 0 || world_y < 0 || world_x >= canvas_w || world_y >= canvas_h {
                continue;
            }
            // **Sub-pixel shape sample (anti-wobble).** Sample the shape at this
            // pixel CENTRE's offset from the dab's TRUE fractional centre — NOT a
            // regular footprint grid — so a slow diagonal stroke shifts coverage
            // smoothly instead of snapping a whole pixel (the "serrilhado" on
            // diagonals). Then flip + rotate into shape space (mirror of shader):
            //   1. centred uv from fractional centre, 2. flip, 3. rotate, 4. un-centre.
            let mut uc = ((world_x as f32) - stamp.position_world[0]) * inv_size;
            let mut vc = ((world_y as f32) - stamp.position_world[1]) * inv_size;
            uc *= flip_x_sign;
            vc *= flip_y_sign;
            let ur = uc * cos_r - vc * sin_r;
            let vr = uc * sin_r + vc * cos_r;
            let u = ur + 0.5;
            let v = vr + 0.5;
            let shape_alpha = crate::library::shape_alpha_for_slot(shape_slot, u, v, aa);
            if shape_alpha < (1.0 / 255.0) {
                continue;
            }
            let mut combined_alpha = (color_alpha * opacity * flow * shape_alpha).clamp(0.0, 1.0);
            if combined_alpha < (1.0 / 255.0) {
                continue;
            }
            let world_cx = world_x as f32 + 0.5;
            let world_cy = world_y as f32 + 0.5;
            // Procedural grain modulates coverage by the paper/fibre texture.
            combined_alpha *= grain_factor(stamp, world_cx, world_cy, u, v);
            // Global canvas paper tooth (world-space, pressure-aware) — the texture
            // that reads as "real media" on every stroke.
            combined_alpha *= paper_tooth_factor(world_cx, world_cy, stamp.pressure, paper_grain);
            let pix = world_y as usize * width as usize + world_x as usize;
            // Stroke-coverage side output (build-up + edges): accumulate this dab's
            // per-pixel deposit so the edge settle can locate the stroke even when
            // there is no wash composite. Monotonic Porter-Duff "over" of coverage.
            if let Some(cov) = coverage.as_deref_mut() {
                let c = cov[pix];
                cov[pix] = c + combined_alpha * (1.0 - c);
            }
            let idx = pix * 4;
            // Decode dst RGBA8 → straight LINEAR. The canvas is sRGB-encoded
            // (the sprite it mirrors uploads as `Rgba8UnormSrgb`), so
            // alpha-over compositing must run in LINEAR light — decode RGB
            // through the IEC-61966 sRGB transfer (the SAME `ph2d-color` fn
            // the render/replay/swatch path uses, so a painted stroke matches
            // its swatch). Alpha is NOT gamma-encoded → straight `/255`.
            // Straight→premul follows for correct alpha-over; premul→straight
            // + re-encode to sRGB happens on write below. (gamma fix 2026-05-31)
            let dst_straight = [
                srgb_to_linear_byte(canvas[idx]),
                srgb_to_linear_byte(canvas[idx + 1]),
                srgb_to_linear_byte(canvas[idx + 2]),
                canvas[idx + 3] as f32 / 255.0,
            ];
            let dst_alpha = dst_straight[3];
            let dst_premul = [
                dst_straight[0] * dst_alpha,
                dst_straight[1] * dst_alpha,
                dst_straight[2] * dst_alpha,
                dst_alpha,
            ];
            let src_premul = [
                rgb_clamped[0] * combined_alpha,
                rgb_clamped[1] * combined_alpha,
                rgb_clamped[2] * combined_alpha,
                combined_alpha,
            ];
            // Mixbox (pigment_mode==1): the deposited pigment MIXES subtractively
            // with the wet backdrop (mirror of `stamp.wgsl`). The canvas colour =
            // `mixbox_lerp(backdrop, brush, deposit)`, coverage accumulates
            // alpha-over; `deposit` = share of the final pigment that is NEW (1 over
            // blank → pure brush; <1 over paint → an even mix → yellow over blue =
            // green). Bypasses the linear rendering-mode colour blend.
            let result = if let Some(ref prep) = pigment {
                let out_a = combined_alpha + dst_alpha * (1.0 - combined_alpha);
                let deposit = combined_alpha / out_a.max(1e-4);
                let mixed = crate::pigment_mix::mix_prepared(
                    prep,
                    [dst_straight[0], dst_straight[1], dst_straight[2]],
                    deposit,
                );
                [mixed[0] * out_a, mixed[1] * out_a, mixed[2] * out_a, out_a]
            } else {
                apply_rendering_mode(mode, src_premul, dst_premul, wet)
            };
            // NaN guard (paridade D-2.F7).
            let result = [
                if result[0].is_nan() { 0.0 } else { result[0] },
                if result[1].is_nan() { 0.0 } else { result[1] },
                if result[2].is_nan() { 0.0 } else { result[2] },
                if result[3].is_nan() { 0.0 } else { result[3] },
            ];
            // result é premul; clamp + un-premul para gravar straight u8.
            let out_premul = [
                result[0].clamp(0.0, 1.0),
                result[1].clamp(0.0, 1.0),
                result[2].clamp(0.0, 1.0),
                result[3].clamp(0.0, 1.0),
            ];
            let out_alpha = out_premul[3];
            let out_straight = if out_alpha > 1e-6 {
                [
                    (out_premul[0] / out_alpha).clamp(0.0, 1.0),
                    (out_premul[1] / out_alpha).clamp(0.0, 1.0),
                    (out_premul[2] / out_alpha).clamp(0.0, 1.0),
                    out_alpha,
                ]
            } else {
                [0.0, 0.0, 0.0, 0.0]
            };
            // T3.7 alpha-lock (§2.10): paint only within existing alpha. A
            // fully-transparent destination pixel is left untouched, and the
            // destination alpha is held constant — only the color blends.
            if alpha_lock && dst_alpha <= 0.0 {
                continue;
            }
            // Re-encode RGB linear → sRGB byte (mirror of the read decode), so
            // the canvas stays sRGB for the `Rgba8UnormSrgb` sprite + the
            // replay path. Alpha is straight light → plain quantize, no gamma.
            canvas[idx] = linear_to_srgb_byte(out_straight[0]);
            canvas[idx + 1] = linear_to_srgb_byte(out_straight[1]);
            canvas[idx + 2] = linear_to_srgb_byte(out_straight[2]);
            if !alpha_lock {
                canvas[idx + 3] = (out_straight[3] * 255.0 + 0.5) as u8;
            }
        }
    }
}

/// **W5 Mixbox wash model** — composite a whole pigment stroke against a
/// fixed `backdrop` snapshot, capping coverage at `opacity_cap`.
///
/// ## Why this exists (the green bug)
///
/// The per-dab [`apply_stamps_with_options`] path composites each stamp over
/// the *live* canvas. Within one stroke, overlapping dabs therefore build up:
/// yellow over (yellow over (yellow over blue)) → the deposit `t` climbs to ~1
/// and the Mixbox mix converges to **pure brush colour** (yellow). The green
/// only survives in the thin feathered fringe where a single dab lands at
/// `t≈0.5`. That is exactly the "paint yellow over blue → still yellow/grey,
/// no green" report.
///
/// Real media don't behave that way: one pass of a 50%-loaded brush deposits
/// *half* its pigment and stops — the result is a stable 50/50 mix (green),
/// no matter how many dabs overlap inside that single stroke. That is the
/// **wash** model (Photoshop/Krita "build-up off"): `flow`×shape accumulates a
/// per-pixel **coverage** ∈[0,1], and `opacity` is the **cap** on how much of
/// the stroke's pigment reaches the canvas. Each dab re-composites the pixel
/// from the *pre-stroke backdrop* (never the accumulating result), so coverage
/// is monotonic and the mix is stable: `mix(backdrop, brush, opacity·coverage)`.
///
/// `backdrop` is the canvas as it was at `begin_stroke` (the tool already keeps
/// this snapshot for undo — `pending_pre_stroke`, reused here at zero extra
/// cost). `coverage` is a per-pixel `f32` buffer, zeroed at `begin_stroke`,
/// owned by the stroke. Opacity must NOT be pre-baked into the stamp colour
/// alpha on this path (the tool skips that bake for pigment strokes) — it is
/// applied once, here, as the cap.
/// **Wet Mix mixer-brush reservoir** (W7, ADR-0097). The pigment the brush
/// carries as it travels: evolves dab-to-dab (picks up canvas colour → smear)
/// and depletes (the `load` runs out → the trail fades). Persists across dabs
/// WITHIN one stroke; owned by the tool, seeded at `begin_stroke`
/// (`color` = brush colour, `load` = Charge), dropped at `end_stroke`. The
/// reservoir is updated **once per dab** (not per footprint pixel) — one pickup
/// at the dab centre, then the same reservoir colour deposits over the footprint.
#[derive(Clone, Debug, PartialEq)]
pub struct WetState {
    /// Reservoir pigment in **linear sRGB** (matches `wash_color`).
    pub color: [f32; 3],
    /// Remaining paint ∈ [0, 1]; seeded from `wet_mix.load` (Charge).
    pub load: f32,
}

/// Snapshot of the Wet Mix sliders that drive the per-dab pickup/deposit.
/// Passed **by value** (not through the frozen 96 B `Stamp`) so the ABI is
/// untouched. `None` wet argument ⇒ the wash path is byte-for-byte the legacy
/// default brush. Grade/Blur are W7 phase 2 (not here). W7.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WetMixConfig {
    /// Water mixed in → transparency (lowers deposit). `[0,1]`.
    pub dilution: f32,
    /// Deposit-rate multiplier (how much loaded paint sticks). `[0,1]`.
    pub attack: f32,
    /// How strongly the brush picks up / smears canvas colour. `[0,1]`.
    pub pull: f32,
    /// Per-dab randomisation of `dilution`. `[0,1]`.
    pub wetness_jitter: f32,
    /// Texture chunkiness/contrast (W7 phase 2). `0.5` = neutral. `[0,1]`.
    pub grade: f32,
    /// Spread/softening of the laid paint vs the canvas underneath. `[0,1]`.
    pub blur: f32,
    /// Per-dab randomisation of `blur`. `[0,1]`.
    pub blur_jitter: f32,
}

/// Charge depletion per unit of per-dab deposit intensity. Tuned so a full
/// Charge (`load = 1`) lays a long trail before drying, while a low Charge dries
/// out within ~1 brush diameter. The only free tuning knob of W7 (design §risco 3).
const WET_DEPLETE_K: f32 = 0.06;

/// Deterministic `[0,1)` hash of a dab's world position — HR-5 safe (no RNG
/// state, same bits cross-OS). Drives `wetness_jitter` per dab.
#[inline]
fn dab_hash01(x: f32, y: f32) -> f32 {
    let mut h = x.to_bits().wrapping_mul(0x9E37_79B1) ^ y.to_bits().wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0x00FF_FFFF) as f32 / (0x0100_0000u32 as f32)
}

/// Max backdrop-blur radius (px) at `blur = 1` — bounds the per-dab cost. The
/// wet "Blur" softens the seam between laid paint and the canvas underneath by
/// compositing over a box-blurred backdrop (W7 phase 2).
const MAX_WET_BLUR_RADIUS: i32 = 3;

/// Grade contrast strength: at `grade = 1` the texture contrast is pushed to
/// `1 + GRADE_K` around its 0.5 midpoint (chunkier); `grade = 0.5` is neutral.
const WET_GRADE_K: f32 = 1.6;

/// Wet Mix **Grade**: reshape the per-dab texture factor's contrast by scaling
/// its *valleys* (the `1 − tex` depth below full coverage). `grade > 0.5`
/// deepens the texture (chunkier); `grade < 0.5` fills it in (smoother). Pivots
/// at `tex = 1`, so a textureless brush (`tex ≈ 1`) is unaffected at any grade,
/// and `grade = 0.5` is the exact identity (default ⇒ no change to legacy).
#[inline]
fn apply_grade(tex: f32, grade: f32) -> f32 {
    let k = 1.0 + (grade.clamp(0.0, 1.0) - 0.5) * 2.0 * WET_GRADE_K;
    (1.0 - (1.0 - tex) * k).clamp(0.0, 1.0)
}

/// Smudge pickup radius as a fraction of the dab footprint (Krita "Smudge
/// Radius" is a % of brush size; MyPaint defaults to the dab radius). Capped for
/// per-dab cost.
const SMUDGE_RADIUS_FRAC: f32 = 0.5;
const MAX_SMUDGE_RADIUS: i32 = 16;

/// Box-average an RGBA8 slice (straight → linear) over a square of `radius`,
/// clamped to the canvas. Used both for the wet **Blur** (over the pre-stroke
/// backdrop) and the **Pull** pickup (over the LIVE canvas — Krita "Dulling").
#[inline]
fn box_average_linear(src: &[u8], cx: i32, cy: i32, radius: i32, w: i32, h: i32) -> [f32; 4] {
    let backdrop = src;
    let (mut r, mut g, mut b, mut a, mut n) = (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
    for yy in (cy - radius)..=(cy + radius) {
        let sy = yy.clamp(0, h - 1);
        for xx in (cx - radius)..=(cx + radius) {
            let sx = xx.clamp(0, w - 1);
            let i = (sy as usize * w as usize + sx as usize) * 4;
            r += srgb_to_linear_byte(backdrop[i]);
            g += srgb_to_linear_byte(backdrop[i + 1]);
            b += srgb_to_linear_byte(backdrop[i + 2]);
            a += backdrop[i + 3] as f32 / 255.0;
            n += 1.0;
        }
    }
    [r / n, g / n, b / n, a / n]
}

#[allow(clippy::too_many_arguments)]
pub fn apply_stamps_wash(
    canvas: &mut [u8],
    backdrop: &[u8],
    coverage: &mut [f32],
    wash_color: &mut [[f32; 3]],
    width: u32,
    height: u32,
    stamps: &[Stamp],
    opacity_cap: f32,
    pigment: bool,
    alpha_lock: bool,
    paper_grain: f32,
    wet: Option<(&mut WetState, WetMixConfig)>,
) {
    let n = (width as usize) * (height as usize);
    assert_eq!(canvas.len(), n * 4, "canvas size must match width*height*4");
    assert_eq!(
        backdrop.len(),
        n * 4,
        "backdrop size must match width*height*4"
    );
    assert_eq!(coverage.len(), n, "coverage size must match width*height");
    assert_eq!(
        wash_color.len(),
        n,
        "wash_color size must match width*height"
    );
    let cap = opacity_cap.clamp(0.0, 1.0);
    // The reservoir (if any) is mutated dab-to-dab: reborrow it for each stamp so
    // the `&mut WetState` threads through the whole stroke. `None` ⇒ legacy wash.
    let mut wet = wet;
    for stamp in stamps {
        let wet_ref = wet.as_mut().map(|(st, cfg)| (&mut **st, *cfg));
        apply_one_stamp_wash(
            canvas,
            backdrop,
            coverage,
            wash_color,
            width,
            height,
            stamp,
            cap,
            pigment,
            alpha_lock,
            paper_grain,
            wet_ref,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_one_stamp_wash(
    canvas: &mut [u8],
    backdrop: &[u8],
    coverage: &mut [f32],
    wash_color: &mut [[f32; 3]],
    width: u32,
    height: u32,
    stamp: &Stamp,
    opacity_cap: f32,
    pigment: bool,
    alpha_lock: bool,
    paper_grain: f32,
    mut wet: Option<(&mut WetState, WetMixConfig)>,
) {
    if !stamp.size_px.is_finite()
        || stamp.size_px <= 0.0
        || !stamp.position_world[0].is_finite()
        || !stamp.position_world[1].is_finite()
    {
        return;
    }
    let footprint = (stamp.size_px.ceil() as u32).min(crate::stamp::MAX_STAMP_SIZE_PX);
    if footprint == 0 {
        return;
    }
    if (stamp.flags & (crate::stamp::FLAG_HOVER_PREVIEW | crate::stamp::FLAG_PREDICTED_SAMPLE)) != 0
    {
        return;
    }
    let footprint_f = footprint as f32;
    let center_offset = (footprint_f - 1.0) * 0.5;
    let inv_size = 1.0 / stamp.size_px;
    let aa = crate::library::SHAPE_AA_PX * inv_size;
    let [l, a, b, color_alpha] = stamp.color_oklab;
    let rgb_linear = oklab_to_linear_srgb(l, a, b);
    let rgb_clamped = [
        rgb_linear[0].clamp(0.0, 1.0),
        rgb_linear[1].clamp(0.0, 1.0),
        rgb_linear[2].clamp(0.0, 1.0),
    ];
    let flow = stamp.flow.clamp(0.0, 1.0);
    let shape_slot = stamp.shape_layer;
    // **Rendering mode in the wash (option a).** Linear wash composites the brush
    // over the backdrop with the full per-mode formula (`apply_rendering_mode`) —
    // `UniformGlaze` reproduces the old over-lerp exactly (no regression), the
    // others vary the glaze/blend character. Pigment wash keeps its K-M colour but
    // applies a per-mode coverage curve (`wash_glaze_coverage`) so the glaze modes
    // still vary the wash transparency. Constant per stamp.
    let mode = RenderingMode::from_u32(stamp.rendering_mode);
    let wet_amount = stamp.wet_amount;
    // ── Wet Mix (W7, design 04): ONE reservoir update per dab ────────────────
    // **Pull = smudge (Krita "Dulling", MyPaint reservoir EMA).** Sample the
    // **LIVE canvas** (not the frozen backdrop) over an **area** under the dab
    // (the Smudge Radius — % of the footprint), so the brush drags the paint laid
    // this stroke instead of re-stamping a single stale pixel. The reservoir is an
    // EMA toward that sample by `pull`: `reservoir ← (1−pull)·reservoir +
    // pull·sample`. The deposited colour is the reservoir (the smear); the deposit
    // RATE is scaled by `attack · load · (1 − dilution)`, and `load` depletes per
    // dab (Charge). The *composite* still uses the stable pre-stroke backdrop —
    // pickup-source and composite-source are independent (Krita samples the live
    // paint device too). `wet = None` ⇒ legacy wash, byte-for-byte identical.
    let (rgb_clamped, deposit_scale, wetcfg) = match wet {
        Some((ref mut st, cfg)) => {
            let cx = (stamp.position_world[0].round() as i32).clamp(0, width as i32 - 1);
            let cy = (stamp.position_world[1].round() as i32).clamp(0, height as i32 - 1);
            let smudge_r = ((footprint as f32 * SMUDGE_RADIUS_FRAC).round() as i32)
                .clamp(1, MAX_SMUDGE_RADIUS);
            let pk = box_average_linear(canvas, cx, cy, smudge_r, width as i32, height as i32);
            let picked = [pk[0], pk[1], pk[2]];
            let k_pickup = cfg.pull.clamp(0.0, 1.0);
            st.color = [
                st.color[0] + (picked[0] - st.color[0]) * k_pickup,
                st.color[1] + (picked[1] - st.color[1]) * k_pickup,
                st.color[2] + (picked[2] - st.color[2]) * k_pickup,
            ];
            let signed = dab_hash01(stamp.position_world[0], stamp.position_world[1]) * 2.0 - 1.0;
            let dil = (cfg.dilution * (1.0 + cfg.wetness_jitter.clamp(0.0, 1.0) * signed))
                .clamp(0.0, 1.0);
            let ds = (cfg.attack.clamp(0.0, 1.0) * st.load.clamp(0.0, 1.0) * (1.0 - dil))
                .clamp(0.0, 1.0);
            (st.color, ds, Some(cfg))
        }
        None => (rgb_clamped, 1.0, None),
    };
    // Per-dab Wet Mix texture/blur params (W7 phase 2; constant across the
    // footprint). `grade` reshapes the texture contrast; `blur_radius` is the
    // backdrop box-blur the deposit composites over (jittered per dab).
    let wet_grade = wetcfg.map(|c| c.grade);
    let blur_radius = wetcfg.map_or(0, |c| {
        let signed =
            dab_hash01(stamp.position_world[0] + 1.7, stamp.position_world[1] + 3.1) * 2.0 - 1.0;
        let b = (c.blur * (1.0 + c.blur_jitter.clamp(0.0, 1.0) * signed)).clamp(0.0, 1.0);
        (b * MAX_WET_BLUR_RADIUS as f32).round() as i32
    });
    // Pigment prepared once per stamp (constant across the footprint; varies
    // stamp-to-stamp only under hue jitter, which we honour by re-preparing).
    // `None` when the brush is in linear (non-pigment) wash mode.
    let prep = pigment.then(|| crate::pigment_mix::prepare_pigment(rgb_clamped));

    let cos_r = stamp.rotation_rad.cos();
    let sin_r = -stamp.rotation_rad.sin();
    let flip_x_sign: f32 = if (stamp.flags & crate::stamp::FLAG_SHAPE_FLIP_X) != 0 {
        -1.0
    } else {
        1.0
    };
    let flip_y_sign: f32 = if (stamp.flags & crate::stamp::FLAG_SHAPE_FLIP_Y) != 0 {
        -1.0
    } else {
        1.0
    };
    let canvas_w = width as i32;
    let canvas_h = height as i32;

    for py in 0..footprint {
        for px in 0..footprint {
            // World pixel (round half-up; consecutive over the footprint grid).
            let world_x =
                (stamp.position_world[0] + (px as f32) - center_offset + 0.5).floor() as i32;
            let world_y =
                (stamp.position_world[1] + (py as f32) - center_offset + 0.5).floor() as i32;
            if world_x < 0 || world_y < 0 || world_x >= canvas_w || world_y >= canvas_h {
                continue;
            }
            // **Sub-pixel shape sample (anti-wobble)** — sample at this pixel
            // centre's offset from the dab's TRUE fractional centre, then flip +
            // rotate into shape space (mirror of `apply_one_stamp`).
            let mut uc = ((world_x as f32) - stamp.position_world[0]) * inv_size;
            let mut vc = ((world_y as f32) - stamp.position_world[1]) * inv_size;
            uc *= flip_x_sign;
            vc *= flip_y_sign;
            let ur = uc * cos_r - vc * sin_r;
            let vr = uc * sin_r + vc * cos_r;
            let u = ur + 0.5;
            let v = vr + 0.5;
            let shape_alpha = crate::library::shape_alpha_for_slot(shape_slot, u, v, aa);
            if shape_alpha < (1.0 / 255.0) {
                continue;
            }
            // Per-dab deposit RATE. The BRUSH opacity is the cap below
            // (`opacity_cap`); `stamp.opacity` here is the PER-STAMP falloff
            // taper (T1.7) — 1.0 for a non-fading brush, decaying along the
            // stroke when `stroke_path.falloff > 0`, so later dabs deposit less
            // coverage and the stroke fades. Without this the wash path (the
            // default brush) ignored falloff entirely.
            let taper = stamp.opacity.clamp(0.0, 1.0);
            // `deposit_scale` is 1.0 on the legacy path; under Wet Mix it folds in
            // attack·load·(1−dilution) so the trail thins as the reservoir runs out.
            let mut rate = (color_alpha * flow * shape_alpha * taper * deposit_scale).clamp(0.0, 1.0);
            if rate < (1.0 / 255.0) {
                continue;
            }
            let world_cx = world_x as f32 + 0.5;
            let world_cy = world_y as f32 + 0.5;
            // Procedural grain × global paper tooth = the per-dab texture factor.
            let mut tex = grain_factor(stamp, world_cx, world_cy, u, v)
                * paper_tooth_factor(world_cx, world_cy, stamp.pressure, paper_grain);
            // Wet Mix Grade (W7 phase 2): push the texture contrast (chunkiness).
            // `grade = 0.5` is the identity, so the default brush is unaffected.
            if let Some(grade) = wet_grade {
                tex = apply_grade(tex, grade);
            }
            rate *= tex;
            let pix = world_y as usize * width as usize + world_x as usize;
            let idx = pix * 4;
            // Monotonic coverage build-up (Porter-Duff "over" of dab onto the
            // stroke's own coverage). Capped implicitly at 1.0.
            let cov = coverage[pix];
            let new_cov = cov + rate * (1.0 - cov);
            coverage[pix] = new_cov;
            // Color Dynamics blend with correct Z-ORDER: the new dab paints "over"
            // the existing stroke colour proportional to its OWN per-dab coverage
            // `rate`, so within one stroke later dabs sit on top of earlier ones
            // (`wash_color ← lerp(wash_color, dab, rate)`). The previous coverage-
            // weighted average (`(wash_color·cov + dab·dcov)/new_cov`) weighted
            // EARLIER dabs by the accumulated coverage — which dominates as coverage
            // saturates — so the earlier-painted colour appeared ON TOP (inverted z,
            // user-visible with Color Dynamics / varying pressure). First touch
            // (cov≈0) seeds the colour. A single-colour stroke stays byte-identical
            // (`lerp(c, c, rate) = c`), so jittered dabs still blend smoothly (no
            // discrete discs) while keeping later-over-earlier ordering.
            let acc = if cov <= 1e-6 {
                rgb_clamped
            } else {
                [
                    wash_color[pix][0] + (rgb_clamped[0] - wash_color[pix][0]) * rate,
                    wash_color[pix][1] + (rgb_clamped[1] - wash_color[pix][1]) * rate,
                    wash_color[pix][2] + (rgb_clamped[2] - wash_color[pix][2]) * rate,
                ]
            };
            wash_color[pix] = acc;
            let color_varies = (acc[0] - rgb_clamped[0]).abs() > 1e-4
                || (acc[1] - rgb_clamped[1]).abs() > 1e-4
                || (acc[2] - rgb_clamped[2]).abs() > 1e-4;
            let brush_color = if color_varies { acc } else { rgb_clamped };
            // Decode the PRE-STROKE backdrop (straight linear) — NOT the live
            // canvas. This is what makes overlapping dabs stable instead of
            // building toward pure brush colour. Wet Mix **Blur** (W7 phase 2)
            // composites over a box-blurred backdrop so the paint seam spreads.
            let back = if blur_radius > 0 {
                box_average_linear(backdrop, world_x, world_y, blur_radius, canvas_w, canvas_h)
            } else {
                [
                    srgb_to_linear_byte(backdrop[idx]),
                    srgb_to_linear_byte(backdrop[idx + 1]),
                    srgb_to_linear_byte(backdrop[idx + 2]),
                    backdrop[idx + 3] as f32 / 255.0,
                ]
            };
            let back_a = back[3];
            if alpha_lock && back_a <= 0.0 {
                continue;
            }
            // Stroke pigment that reaches the canvas = coverage capped by opacity.
            let eff = (opacity_cap * new_cov).clamp(0.0, 1.0);
            // Composite via the brush's RENDERING MODE (option a — the wash now
            // honours the Mode cycler). PIGMENT keeps the subtractive K-M colour but
            // applies a per-mode coverage curve (glaze transparency); LINEAR runs
            // the full `apply_rendering_mode` over the backdrop. `UniformGlaze`
            // reproduces the old wash exactly in BOTH paths (no regression).
            let (mixed, out_a) = if let Some(ref prep) = prep {
                let eff_m = wash_glaze_coverage(mode, eff);
                let out_a = eff_m + back_a * (1.0 - eff_m);
                let t = (eff_m / out_a.max(1e-4)).clamp(0.0, 1.0);
                // Pigment: cached per-stamp prep on the fast path; re-prepare from
                // the blended colour only when Color Dynamics actually varied it.
                let m = if color_varies {
                    let prep_acc = crate::pigment_mix::prepare_pigment(brush_color);
                    crate::pigment_mix::mix_prepared(&prep_acc, [back[0], back[1], back[2]], t)
                } else {
                    crate::pigment_mix::mix_prepared(prep, [back[0], back[1], back[2]], t)
                };
                (m, out_a)
            } else {
                // Linear: full per-mode composite of the brush (premul at `eff`) over
                // the backdrop. UniformGlaze == the old over-lerp byte-for-byte.
                let src = [
                    brush_color[0] * eff,
                    brush_color[1] * eff,
                    brush_color[2] * eff,
                    eff,
                ];
                let dst = [back[0] * back_a, back[1] * back_a, back[2] * back_a, back_a];
                let out = apply_rendering_mode(mode, src, dst, wet_amount);
                let oa = out[3].clamp(0.0, 1.0);
                let m = if oa > 1e-6 {
                    [
                        (out[0] / oa).clamp(0.0, 1.0),
                        (out[1] / oa).clamp(0.0, 1.0),
                        (out[2] / oa).clamp(0.0, 1.0),
                    ]
                } else {
                    [0.0, 0.0, 0.0]
                };
                (m, oa)
            };
            if out_a <= 1e-6 {
                canvas[idx] = 0;
                canvas[idx + 1] = 0;
                canvas[idx + 2] = 0;
                if !alpha_lock {
                    canvas[idx + 3] = 0;
                }
                continue;
            }
            canvas[idx] = linear_to_srgb_byte(mixed[0].clamp(0.0, 1.0));
            canvas[idx + 1] = linear_to_srgb_byte(mixed[1].clamp(0.0, 1.0));
            canvas[idx + 2] = linear_to_srgb_byte(mixed[2].clamp(0.0, 1.0));
            if !alpha_lock {
                canvas[idx + 3] = (out_a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            }
        }
    }
    // Charge depletion (W7): once per dab, proportional to how much it deposited.
    // The trail fades as `load → 0`; recharge happens at the next `begin_stroke`.
    if let Some((st, _)) = wet {
        st.load = (st.load - deposit_scale * WET_DEPLETE_K).max(0.0);
    }
}

/// Per-mode coverage curve for the **pigment** wash so the glaze modes vary the
/// stroke's transparency while the subtractive K-M colour mix is unchanged.
/// `UniformGlaze` is the identity (the default — no regression). Glaze modes shift
/// opacity: Light = thinner, Intense/Heavy = progressively more solid. The two
/// Blending modes keep the coverage — their distinction is colour mixing, which the
/// K-M wash already does. (The Linear wash uses the full `apply_rendering_mode`, so
/// all six modes are fully distinct there.)
#[inline]
fn wash_glaze_coverage(mode: RenderingMode, eff: f32) -> f32 {
    let e = eff.clamp(0.0, 1.0);
    match mode {
        RenderingMode::LightGlaze => e.powf(1.6), // thinner, more transparent glaze
        RenderingMode::IntenseGlaze => e.powf(0.6), // covers faster
        RenderingMode::HeavyGlaze => e.powf(0.4), // most solid
        _ => e,                                   // Uniform Glaze / Blending = identity
    }
}

/// OKLab → linear sRGB (D65). **Coefficients idênticos ao shader** —
/// gate `shader_oklab_coefficients_bit_identical_with_rust` em
/// `stamp_pipeline.rs` prova zero ULP drift. Exposto p/ o splat da difusão
/// (W15) usar a MESMA conversão do renderer.
#[inline]
pub fn oklab_to_linear_srgb(l: f32, a: f32, b: f32) -> [f32; 3] {
    let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;
    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;
    [
        4.076_741_7 * l3 - 3.307_711_6 * m3 + 0.230_969_94 * s3,
        -1.268_438 * l3 + 2.609_757_4 * m3 - 0.341_319_38 * s3,
        -0.004_196_086_3 * l3 - 0.703_418_6 * m3 + 1.707_614_7 * s3,
    ]
}

// ── Submodules (god-module split, 2026-06-04; pure move) ──
mod blends;
use blends::*;
#[cfg(test)]
mod tests;
