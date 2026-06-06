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
    // Audit T1.5 round 1 A-L4 — production-grade length guard so a
    // mismatch panics on the spot instead of reading/writing past the
    // canvas in release builds.
    assert_eq!(
        canvas.len(),
        (width as usize) * (height as usize) * 4,
        "canvas buffer size must match width*height*4 RGBA8"
    );
    for stamp in stamps {
        apply_one_stamp(canvas, width, height, stamp, alpha_lock);
    }
}

/// Base spatial frequency for procedural grain — world pixels → noise space.
const GRAIN_BASE_FREQ: f32 = 0.35;
/// Fixed grain seed (per-brush variation rides in `grain_offset_uv`; v1 = 0).
const GRAIN_SEED: u32 = 0x9e37_79b9;

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

fn apply_one_stamp(canvas: &mut [u8], width: u32, height: u32, stamp: &Stamp, alpha_lock: bool) {
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
            // T1.6 shape-sample transform pipeline (mirror of shader):
            //   1. axis-aligned uv (pixel-center convention, D-1.M1)
            //   2. center to [-0.5, +0.5]
            //   3. flip per FLAG_SHAPE_FLIP_{X,Y}
            //   4. rotate by `-rotation_rad`
            //   5. un-center back to [0, 1] (outside [0,1]² → shape α=0)
            //   6. dispatch shape kernel by `stamp.shape_layer`
            let u_axis = (px as f32 + 0.5) / footprint_f;
            let v_axis = (py as f32 + 0.5) / footprint_f;
            let mut uc = u_axis - 0.5;
            let mut vc = v_axis - 0.5;
            uc *= flip_x_sign;
            vc *= flip_y_sign;
            let ur = uc * cos_r - vc * sin_r;
            let vr = uc * sin_r + vc * cos_r;
            let u = ur + 0.5;
            let v = vr + 0.5;
            let shape_alpha = crate::library::shape_alpha_for_slot(shape_slot, u, v);
            if shape_alpha < (1.0 / 255.0) {
                continue;
            }
            let mut combined_alpha = (color_alpha * opacity * flow * shape_alpha).clamp(0.0, 1.0);
            if combined_alpha < (1.0 / 255.0) {
                continue;
            }
            // Round half-up (paridade D-2.F3).
            let world_x_f = stamp.position_world[0] + (px as f32) - center_offset;
            let world_y_f = stamp.position_world[1] + (py as f32) - center_offset;
            let world_x = (world_x_f + 0.5).floor() as i32;
            let world_y = (world_y_f + 0.5).floor() as i32;
            if world_x < 0 || world_y < 0 || world_x >= canvas_w || world_y >= canvas_h {
                continue;
            }
            // Procedural grain modulates coverage by the paper/fibre texture.
            combined_alpha *= grain_factor(stamp, world_x_f, world_y_f, u, v);
            let idx = (world_y as usize * width as usize + world_x as usize) * 4;
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
#[allow(clippy::too_many_arguments)]
pub fn apply_stamps_wash(
    canvas: &mut [u8],
    backdrop: &[u8],
    coverage: &mut [f32],
    width: u32,
    height: u32,
    stamps: &[Stamp],
    opacity_cap: f32,
    pigment: bool,
    alpha_lock: bool,
) {
    let n = (width as usize) * (height as usize);
    assert_eq!(canvas.len(), n * 4, "canvas size must match width*height*4");
    assert_eq!(
        backdrop.len(),
        n * 4,
        "backdrop size must match width*height*4"
    );
    assert_eq!(coverage.len(), n, "coverage size must match width*height");
    let cap = opacity_cap.clamp(0.0, 1.0);
    for stamp in stamps {
        apply_one_stamp_wash(
            canvas, backdrop, coverage, width, height, stamp, cap, pigment, alpha_lock,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_one_stamp_wash(
    canvas: &mut [u8],
    backdrop: &[u8],
    coverage: &mut [f32],
    width: u32,
    height: u32,
    stamp: &Stamp,
    opacity_cap: f32,
    pigment: bool,
    alpha_lock: bool,
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
    let [l, a, b, color_alpha] = stamp.color_oklab;
    let rgb_linear = oklab_to_linear_srgb(l, a, b);
    let rgb_clamped = [
        rgb_linear[0].clamp(0.0, 1.0),
        rgb_linear[1].clamp(0.0, 1.0),
        rgb_linear[2].clamp(0.0, 1.0),
    ];
    let flow = stamp.flow.clamp(0.0, 1.0);
    let shape_slot = stamp.shape_layer;
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
            let u_axis = (px as f32 + 0.5) / footprint_f;
            let v_axis = (py as f32 + 0.5) / footprint_f;
            let mut uc = u_axis - 0.5;
            let mut vc = v_axis - 0.5;
            uc *= flip_x_sign;
            vc *= flip_y_sign;
            let ur = uc * cos_r - vc * sin_r;
            let vr = uc * sin_r + vc * cos_r;
            let u = ur + 0.5;
            let v = vr + 0.5;
            let shape_alpha = crate::library::shape_alpha_for_slot(shape_slot, u, v);
            if shape_alpha < (1.0 / 255.0) {
                continue;
            }
            // Per-dab deposit RATE (no opacity — opacity is the cap below).
            let mut rate = (color_alpha * flow * shape_alpha).clamp(0.0, 1.0);
            if rate < (1.0 / 255.0) {
                continue;
            }
            let world_x_f = stamp.position_world[0] + (px as f32) - center_offset;
            let world_y_f = stamp.position_world[1] + (py as f32) - center_offset;
            let world_x = (world_x_f + 0.5).floor() as i32;
            let world_y = (world_y_f + 0.5).floor() as i32;
            if world_x < 0 || world_y < 0 || world_x >= canvas_w || world_y >= canvas_h {
                continue;
            }
            // Procedural grain modulates the per-dab coverage rate.
            rate *= grain_factor(stamp, world_x_f, world_y_f, u, v);
            let pix = world_y as usize * width as usize + world_x as usize;
            let idx = pix * 4;
            // Monotonic coverage build-up (Porter-Duff "over" of dab onto the
            // stroke's own coverage). Capped implicitly at 1.0.
            let cov = coverage[pix];
            let new_cov = cov + rate * (1.0 - cov);
            coverage[pix] = new_cov;
            // Decode the PRE-STROKE backdrop (straight linear) — NOT the live
            // canvas. This is what makes overlapping dabs stable instead of
            // building toward pure brush colour.
            let back = [
                srgb_to_linear_byte(backdrop[idx]),
                srgb_to_linear_byte(backdrop[idx + 1]),
                srgb_to_linear_byte(backdrop[idx + 2]),
                backdrop[idx + 3] as f32 / 255.0,
            ];
            let back_a = back[3];
            if alpha_lock && back_a <= 0.0 {
                continue;
            }
            // Stroke pigment that reaches the canvas = coverage capped by opacity.
            let eff = (opacity_cap * new_cov).clamp(0.0, 1.0);
            let out_a = eff + back_a * (1.0 - eff);
            if out_a <= 1e-6 {
                canvas[idx] = 0;
                canvas[idx + 1] = 0;
                canvas[idx + 2] = 0;
                if !alpha_lock {
                    canvas[idx + 3] = 0;
                }
                continue;
            }
            // How much of the FINAL colour is the new brush. Pigment → subtractive
            // K-M mix; else a plain linear lerp toward the brush colour (still
            // opacity-capped — that's the wash, independent of pigment).
            let t = (eff / out_a).clamp(0.0, 1.0);
            let mixed = if let Some(ref prep) = prep {
                crate::pigment_mix::mix_prepared(prep, [back[0], back[1], back[2]], t)
            } else {
                [
                    back[0] + (rgb_clamped[0] - back[0]) * t,
                    back[1] + (rgb_clamped[1] - back[1]) * t,
                    back[2] + (rgb_clamped[2] - back[2]) * t,
                ]
            };
            canvas[idx] = linear_to_srgb_byte(mixed[0].clamp(0.0, 1.0));
            canvas[idx + 1] = linear_to_srgb_byte(mixed[1].clamp(0.0, 1.0));
            canvas[idx + 2] = linear_to_srgb_byte(mixed[2].clamp(0.0, 1.0));
            if !alpha_lock {
                canvas[idx + 3] = (out_a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            }
        }
    }
}

/// OKLab → linear sRGB (D65). **Coefficients idênticos ao shader** —
/// gate `shader_oklab_coefficients_bit_identical_with_rust` em
/// `stamp_pipeline.rs` prova zero ULP drift.
#[inline]
fn oklab_to_linear_srgb(l: f32, a: f32, b: f32) -> [f32; 3] {
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
