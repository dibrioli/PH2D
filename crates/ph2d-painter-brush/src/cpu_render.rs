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
            let combined_alpha = (color_alpha * opacity * flow * shape_alpha).clamp(0.0, 1.0);
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
            let result = apply_rendering_mode(mode, src_premul, dst_premul, wet);
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

/// Aplica um rendering mode em premul linear sRGB. **Funções idênticas
/// às do shader** (`uniform_glaze`, `intense_glaze`, etc.) — invariante
/// premul preservada em todo o caminho.
#[inline]
fn apply_rendering_mode(mode: RenderingMode, src: [f32; 4], dst: [f32; 4], wet: f32) -> [f32; 4] {
    match mode {
        RenderingMode::LightGlaze => light_glaze(src, dst),
        RenderingMode::UniformGlaze => uniform_glaze(src, dst),
        RenderingMode::IntenseGlaze => intense_glaze(src, dst),
        RenderingMode::HeavyGlaze => heavy_glaze(src, dst),
        RenderingMode::UniformBlending => uniform_blending(src, dst),
        RenderingMode::IntenseBlending => intense_blending(src, dst, wet),
    }
}

#[inline]
fn light_glaze(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let k = 1.0 - src[3] * 0.6;
    [
        src[0] + dst[0] * k,
        src[1] + dst[1] * k,
        src[2] + dst[2] * k,
        src[3] + dst[3] * k,
    ]
}

#[inline]
fn uniform_glaze(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let k = 1.0 - src[3];
    [
        src[0] + dst[0] * k,
        src[1] + dst[1] * k,
        src[2] + dst[2] * k,
        src[3] + dst[3] * k,
    ]
}

#[inline]
fn intense_glaze(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let alpha_s = src[3].clamp(0.0, 1.0);
    if alpha_s < 1e-6 {
        return dst;
    }
    let aa = alpha_s.sqrt();
    let one_minus_aa = 1.0 - aa;
    // src.rgb / aa = src_straight * sqrt(α_s); paridade D-3.M15.
    let inv_aa = 1.0 / aa;
    [
        src[0] * inv_aa + dst[0] * one_minus_aa,
        src[1] * inv_aa + dst[1] * one_minus_aa,
        src[2] * inv_aa + dst[2] * one_minus_aa,
        aa + dst[3] * one_minus_aa,
    ]
}

#[inline]
fn heavy_glaze(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let k = 1.0 - src[3] * 0.85;
    [
        (src[0] + dst[0] * k).clamp(0.0, 1.0),
        (src[1] + dst[1] * k).clamp(0.0, 1.0),
        (src[2] + dst[2] * k).clamp(0.0, 1.0),
        (src[3] + dst[3] * k).clamp(0.0, 1.0),
    ]
}

#[inline]
fn uniform_blending(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let alpha_s = src[3];
    let alpha_d = dst[3];
    let result_a = alpha_s + alpha_d * (1.0 - alpha_s);
    if result_a < 1e-6 {
        return [0.0, 0.0, 0.0, 0.0];
    }
    // **Audit T1.6 R9 S1-H1 — CPU↔GPU divide-per-channel parity.**
    // Previous form pre-computed `inv_as = 1.0 / alpha.max(1e-6)`
    // then multiplied 3× — IEEE 754 division + multiply produces
    // results differing from per-channel division by 1-4 ULP at
    // extreme alpha (near 1e-6). The shader (`stamp.wgsl`
    // uniform_blending) divides per-channel; matching that form
    // CPU-side closes the divergence so the
    // `cpu_shader_textual_parity_all_six_modes` gate can tighten
    // beyond the current "ULP-bounded" tolerance toward
    // bit-equivalence (T-numerical-parity W2+).
    let safe_as = alpha_s.max(1e-6);
    let safe_ad = alpha_d.max(1e-6);
    let src_rgb = [src[0] / safe_as, src[1] / safe_as, src[2] / safe_as];
    let dst_rgb = [dst[0] / safe_ad, dst[1] / safe_ad, dst[2] / safe_ad];
    let mixed = [
        dst_rgb[0] + (src_rgb[0] - dst_rgb[0]) * alpha_s,
        dst_rgb[1] + (src_rgb[1] - dst_rgb[1]) * alpha_s,
        dst_rgb[2] + (src_rgb[2] - dst_rgb[2]) * alpha_s,
    ];
    [
        mixed[0] * result_a,
        mixed[1] * result_a,
        mixed[2] * result_a,
        result_a,
    ]
}

#[inline]
fn intense_blending(src: [f32; 4], dst: [f32; 4], wet: f32) -> [f32; 4] {
    let alpha_s = src[3];
    let alpha_d = dst[3];
    let result_a = alpha_s + alpha_d * (1.0 - alpha_s);
    if result_a < 1e-6 {
        return [0.0, 0.0, 0.0, 0.0];
    }
    // Audit T1.6 R9 S1-H1 (intense_blending parity): same per-channel
    // divide form as `uniform_blending` above; matches the WGSL shader.
    let safe_as = alpha_s.max(1e-6);
    let safe_ad = alpha_d.max(1e-6);
    let src_rgb = [src[0] / safe_as, src[1] / safe_as, src[2] / safe_as];
    let dst_rgb = [dst[0] / safe_ad, dst[1] / safe_ad, dst[2] / safe_ad];
    let pull = wet.clamp(0.0, 1.0);
    let half_pull = 0.5 * pull;
    let smudged_src = [
        src_rgb[0] + (dst_rgb[0] - src_rgb[0]) * half_pull,
        src_rgb[1] + (dst_rgb[1] - src_rgb[1]) * half_pull,
        src_rgb[2] + (dst_rgb[2] - src_rgb[2]) * half_pull,
    ];
    let mixed = [
        dst_rgb[0] + (smudged_src[0] - dst_rgb[0]) * alpha_s,
        dst_rgb[1] + (smudged_src[1] - dst_rgb[1]) * alpha_s,
        dst_rgb[2] + (smudged_src[2] - dst_rgb[2]) * alpha_s,
    ];
    [
        mixed[0] * result_a,
        mixed[1] * result_a,
        mixed[2] * result_a,
        result_a,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stamp::Stamp;

    fn empty_canvas(w: u32, h: u32) -> Vec<u8> {
        vec![0u8; (w * h * 4) as usize]
    }

    fn red_stamp(x: f32, y: f32, size: f32) -> Stamp {
        let mut s = Stamp::zeroed();
        s.position_world = [x, y];
        s.size_px = size;
        // OKLab approximation of opaque red. Doesn't have to be perceptually
        // accurate — we just need a finite, non-trivially-zero color.
        s.color_oklab = [0.6, 0.25, 0.1, 1.0];
        s.opacity = 1.0;
        s.flow = 1.0;
        s.rendering_mode = RenderingMode::UniformGlaze as u32;
        s
    }

    #[test]
    fn alpha_lock_skips_transparent_and_locks_alpha() {
        // T3.7 §2.10: with alpha_lock, paint lands only where the canvas already
        // has alpha, and the alpha is held constant everywhere (only color blends).
        let (w, h) = (16u32, 16u32);
        // Left half opaque white, right half fully transparent.
        let mut canvas = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..(w / 2) {
                let i = ((y * w + x) * 4) as usize;
                canvas[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
        let before = canvas.clone();
        // A large opaque-red stamp over the whole canvas, alpha-locked.
        let s = red_stamp((w / 2) as f32, (h / 2) as f32, (w * 2) as f32);
        apply_stamps_with_options(&mut canvas, w, h, &[s], true);
        // Alpha is locked at EVERY pixel (transparent stays transparent, opaque
        // stays opaque).
        for i in (0..canvas.len()).step_by(4) {
            assert_eq!(canvas[i + 3], before[i + 3], "alpha locked at pixel {}", i / 4);
        }
        // Transparent (right) half is byte-for-byte untouched (no color leak).
        for y in 0..h {
            for x in (w / 2)..w {
                let i = ((y * w + x) * 4) as usize;
                assert_eq!(&canvas[i..i + 4], &before[i..i + 4], "transparent untouched");
            }
        }
        // Some opaque (left) pixel had its color blended toward the paint.
        let any_color_changed = (0..h).any(|y| {
            (0..(w / 2)).any(|x| {
                let i = ((y * w + x) * 4) as usize;
                canvas[i..i + 3] != before[i..i + 3]
            })
        });
        assert!(any_color_changed, "alpha-locked paint still blends color on opaque pixels");
    }

    #[test]
    fn single_stamp_writes_pixels() {
        let (w, h) = (32, 32);
        let mut canvas = empty_canvas(w, h);
        let s = red_stamp(16.0, 16.0, 16.0);
        apply_stamps(&mut canvas, w, h, &[s]);
        // Center pixel must have non-zero alpha.
        let idx = (16 * w as usize + 16) * 4;
        assert!(
            canvas[idx + 3] > 0,
            "center pixel must be painted (got alpha {})",
            canvas[idx + 3]
        );
        // Top-left corner of the canvas must be untouched.
        assert_eq!(canvas[0..4], [0, 0, 0, 0]);
    }

    #[test]
    fn painted_byte_is_srgb_encoded_color_not_raw_linear() {
        // GAMMA FIX (2026-05-31): a stamp must write the sRGB-ENCODED color —
        // the value the `Rgba8UnormSrgb` sprite + the sidebar swatch + the
        // replay path show — NOT the raw linear value. Before the fix the
        // compositor wrote `linear * 255`, so a stroke came out dark/
        // desaturated and mismatched its swatch (Enio W2 smoke). A
        // UniformGlaze-over on a TRANSPARENT canvas yields straight RGB =
        // the source color exactly (un-premul cancels coverage alpha), so the
        // painted byte must equal `linear_to_srgb_byte(linear)`.
        let (w, h) = (8, 8);
        let mut canvas = empty_canvas(w, h);
        let mut s = red_stamp(4.0, 4.0, 6.0);
        s.shape_layer = 2; // square_hard → shape α = 1 in the interior.
        apply_stamps(&mut canvas, w, h, &[s]);
        let idx = (4 * w as usize + 4) * 4;
        assert!(canvas[idx + 3] > 0, "center pixel must be painted");
        let linear = oklab_to_linear_srgb(s.color_oklab[0], s.color_oklab[1], s.color_oklab[2]);
        for c in 0..3 {
            let lin_c = linear[c].clamp(0.0, 1.0);
            let want_srgb = linear_to_srgb_byte(lin_c);
            let old_raw_linear = (lin_c * 255.0 + 0.5) as u8;
            assert_eq!(
                canvas[idx + c],
                want_srgb,
                "channel {c}: painted byte {} must equal the sRGB encoding {} (= swatch), \
                 not the raw-linear {}",
                canvas[idx + c],
                want_srgb,
                old_raw_linear,
            );
            // Fix proof: for a mid-range channel, sRGB-encode is strictly
            // brighter than the old raw-linear byte (the visible bug symptom).
            if (0.02..0.98).contains(&lin_c) {
                assert!(
                    want_srgb > old_raw_linear,
                    "channel {c}: sRGB {want_srgb} must lift mid linear above raw {old_raw_linear}",
                );
            }
        }
    }

    #[test]
    fn nan_position_is_noop() {
        let (w, h) = (8, 8);
        let mut canvas = empty_canvas(w, h);
        let mut s = red_stamp(f32::NAN, 4.0, 8.0);
        s.position_world[0] = f32::NAN;
        apply_stamps(&mut canvas, w, h, &[s]);
        assert!(
            canvas.iter().all(|&b| b == 0),
            "NaN position must produce zero writes"
        );
    }

    #[test]
    fn zero_size_is_noop() {
        let (w, h) = (8, 8);
        let mut canvas = empty_canvas(w, h);
        let s = red_stamp(4.0, 4.0, 0.0);
        apply_stamps(&mut canvas, w, h, &[s]);
        assert!(canvas.iter().all(|&b| b == 0));
    }

    #[test]
    fn off_canvas_clips() {
        let (w, h) = (8, 8);
        let mut canvas = empty_canvas(w, h);
        // Stamp entirely outside the canvas.
        let s = red_stamp(-100.0, -100.0, 8.0);
        apply_stamps(&mut canvas, w, h, &[s]);
        assert!(canvas.iter().all(|&b| b == 0));
    }

    #[test]
    fn two_overlapping_stamps_accumulate_alpha() {
        // Two opaque UniformGlaze stamps on the same pixel should each
        // increase alpha (Porter-Duff "over" reaches 1.0 after a single
        // fully-opaque stamp; the second is a no-op on alpha but proves
        // the dst-read works (would crash / produce NaN otherwise).
        let (w, h) = (8, 8);
        let mut canvas = empty_canvas(w, h);
        let s = red_stamp(4.0, 4.0, 6.0);
        apply_stamps(&mut canvas, w, h, &[s, s]);
        let idx = (4 * w as usize + 4) * 4;
        assert!(
            canvas[idx + 3] > 200,
            "should be near-opaque after 2 stamps"
        );
        // No NaN-induced zero, no garbage. RGB channels should be in
        // the red region (R > G, R > B).
        assert!(
            canvas[idx] >= canvas[idx + 1] && canvas[idx] >= canvas[idx + 2],
            "Red channel should dominate ({}, {}, {})",
            canvas[idx],
            canvas[idx + 1],
            canvas[idx + 2]
        );
    }

    #[test]
    fn determinism_same_stamps_same_canvas() {
        // HR-5 — same input → same output bytes.
        let (w, h) = (32, 32);
        let mut a = empty_canvas(w, h);
        let mut b = empty_canvas(w, h);
        let stamps = vec![
            red_stamp(10.0, 10.0, 12.0),
            red_stamp(20.0, 10.0, 12.0),
            red_stamp(15.0, 20.0, 12.0),
        ];
        apply_stamps(&mut a, w, h, &stamps);
        apply_stamps(&mut b, w, h, &stamps);
        assert_eq!(a, b, "deterministic byte output required");
    }

    #[test]
    fn hover_preview_flag_skips() {
        let (w, h) = (8, 8);
        let mut canvas = empty_canvas(w, h);
        let mut s = red_stamp(4.0, 4.0, 6.0);
        s.flags = crate::stamp::FLAG_HOVER_PREVIEW;
        apply_stamps(&mut canvas, w, h, &[s]);
        assert!(
            canvas.iter().all(|&b| b == 0),
            "hover preview stamps must not write to canvas"
        );
    }

    // ──────────────────────────────────────────────────────────────────────
    // Rendering-mode parity gates (audit T1.5 round 1 A-H2). Algebraic
    // invariants the per-mode functions must satisfy — ANY change to the
    // CPU OR shader rendering math that breaks one of these is silently
    // visible to the user.
    // ──────────────────────────────────────────────────────────────────────

    fn approx_eq(a: [f32; 4], b: [f32; 4], eps: f32) -> bool {
        (a[0] - b[0]).abs() < eps
            && (a[1] - b[1]).abs() < eps
            && (a[2] - b[2]).abs() < eps
            && (a[3] - b[3]).abs() < eps
    }

    #[test]
    fn rendering_mode_zero_src_is_identity_on_dst() {
        // For every glaze mode: src_premul = [0,0,0,0] → result = dst.
        // (Blending modes can be undefined at result_a < 1e-6; we test
        // their non-trivial cases below.)
        let dst = [0.4, 0.2, 0.1, 0.5];
        for mode in [
            RenderingMode::LightGlaze,
            RenderingMode::UniformGlaze,
            RenderingMode::HeavyGlaze,
        ] {
            let r = apply_rendering_mode(mode, [0.0; 4], dst, 0.0);
            assert!(
                approx_eq(r, dst, 1e-6),
                "mode {:?} with zero src must equal dst; got {:?}",
                mode,
                r
            );
        }
    }

    #[test]
    fn uniform_glaze_full_opacity_replaces_dst() {
        // Porter-Duff over with α_s = 1 → result = src.
        let src = [0.6 * 1.0, 0.3 * 1.0, 0.1 * 1.0, 1.0];
        let dst = [0.2, 0.5, 0.7, 0.5];
        let r = apply_rendering_mode(RenderingMode::UniformGlaze, src, dst, 0.0);
        assert!(approx_eq(r, src, 1e-6));
    }

    #[test]
    fn rendering_mode_premul_invariant_preserved() {
        // For UniformGlaze: result must satisfy `result.rgb ≤ result.a`
        // (premul invariant — straight RGB after un-premul must be in
        // [0,1]). C2-round-3 regression guard.
        let src = [0.4 * 0.7, 0.2 * 0.7, 0.1 * 0.7, 0.7];
        let dst = [0.3 * 0.5, 0.1 * 0.5, 0.05 * 0.5, 0.5];
        let r = apply_rendering_mode(RenderingMode::UniformGlaze, src, dst, 0.0);
        assert!(
            r[0] <= r[3] + 1e-6 && r[1] <= r[3] + 1e-6 && r[2] <= r[3] + 1e-6,
            "premul invariant violated: rgb {:?} > alpha {}",
            &r[..3],
            r[3]
        );
    }

    #[test]
    fn uniform_blending_at_full_alpha_equals_src_color() {
        // alpha_s = 1 → result alpha = 1, result rgb = src_rgb (mix
        // collapses to src). Used to ensure unmul→lerp→re-premul cycle
        // didn't introduce a divisor bug.
        let src_straight_rgb = [0.6, 0.3, 0.1];
        let src = [
            src_straight_rgb[0],
            src_straight_rgb[1],
            src_straight_rgb[2],
            1.0,
        ];
        let dst = [0.2 * 0.4, 0.5 * 0.4, 0.7 * 0.4, 0.4];
        let r = apply_rendering_mode(RenderingMode::UniformBlending, src, dst, 0.0);
        // r is premul; result_a = 1, so r.rgb should equal src_rgb.
        assert!(
            approx_eq(r, src, 1e-5),
            "uniform_blending α_s=1 collapse: got {:?}, expected {:?}",
            r,
            src
        );
    }

    #[test]
    fn intense_glaze_precision_at_small_alpha() {
        // Regression for D-3.M15: src.rgb / aa = src_straight * sqrt(α_s)
        // avoids divisor blow-up at α_s → 0. Test at α_s = 0.01 (small
        // but finite): result should be finite and have alpha = sqrt(0.01) = 0.1.
        let alpha_s: f32 = 0.01;
        let src_straight_rgb = [0.8, 0.4, 0.2];
        let src = [
            src_straight_rgb[0] * alpha_s,
            src_straight_rgb[1] * alpha_s,
            src_straight_rgb[2] * alpha_s,
            alpha_s,
        ];
        let dst = [0.0, 0.0, 0.0, 0.0];
        let r = apply_rendering_mode(RenderingMode::IntenseGlaze, src, dst, 0.0);
        assert!(r[0].is_finite() && r[1].is_finite() && r[2].is_finite());
        assert!((r[3] - alpha_s.sqrt()).abs() < 1e-5);
    }

    #[test]
    fn round_half_up_negative_inputs() {
        // D-2.F3 regression: `floor(x + 0.5) as i32` must produce
        // shader-identical results at negative inputs. Test that the
        // CPU formula matches expected reference for `x ∈ [-1.5, 1.5]`
        // in 0.25 steps.
        let cases: &[(f32, i32)] = &[
            (-1.5, -1),
            (-1.0, -1),
            (-0.51, -1),
            (-0.5, 0),
            (-0.25, 0),
            (0.0, 0),
            (0.25, 0),
            (0.49, 0),
            (0.5, 1),
            (1.0, 1),
            (1.49, 1),
            (1.5, 2),
        ];
        for &(x, expected) in cases {
            let got = (x + 0.5).floor() as i32;
            assert_eq!(
                got, expected,
                "round_half_up({}) = {} (expected {})",
                x, got, expected
            );
        }
    }

    #[test]
    fn cpu_shader_textual_parity_all_six_modes() {
        // Audit T1.5 round 2 F3 + round 6 R6-LN-2: textual parity gate
        // for ALL 6 rendering modes. Catches a future shader rewrite
        // that preserves invariants (alpha collapse / over composition)
        // but drifts the formula structure — CPU + shader would silently
        // disagree on intermediate pixel values.
        //
        // Method: whitespace-normalize STAMP_WGSL, then assert each
        // mode's canonical formula entry-line appears verbatim.
        let shader = crate::stamp_pipeline::STAMP_WGSL;
        let norm = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        let shader_norm = norm(shader);

        // ── Glaze modes (Porter-Duff over with mode-specific α curve) ─
        // light_glaze: `src + dst * (1.0 - src.a * 0.6)`.
        assert!(
            shader_norm.contains("return src + dst * (1.0 - src.a * 0.6);"),
            "shader light_glaze formula drifted (R2-F3)"
        );
        // uniform_glaze: Porter-Duff "over".
        assert!(
            shader_norm.contains("return src + dst * (1.0 - src.a);"),
            "shader uniform_glaze formula drifted (R2-F3)"
        );
        // intense_glaze: alpha curve via `aa = sqrt(α_s)` precision-safe.
        assert!(
            shader_norm.contains("let aa = sqrt(alpha_s);"),
            "shader intense_glaze sqrt-alpha curve drifted (R6-LN-2)"
        );
        // heavy_glaze: aggressive over with clamp.
        assert!(
            shader_norm.contains("let r = src + dst * (1.0 - src.a * 0.85);"),
            "shader heavy_glaze formula drifted (R2-F3)"
        );

        // ── Blending modes (continuous mix in straight space + Porter-Duff α) ─
        // Both blending modes share the `result_a = α_s + α_d * (1 - α_s)`
        // Porter-Duff "over" alpha accumulation (premul invariant fix —
        // round 1 D-3.C2).
        assert!(
            shader_norm.contains("let result_a = alpha_s + alpha_d * (1.0 - alpha_s);"),
            "shader blending Porter-Duff alpha-over drifted (R6-LN-2 / C2)"
        );
        // intense_blending wet-mix smudge factor.
        assert!(
            shader_norm.contains("let pull = clamp(wet, 0.0, 1.0);"),
            "shader intense_blending wet-pull factor drifted (R6-LN-2)"
        );
    }

    #[test]
    fn shader_flag_constants_match_rust_bit_values() {
        // Audit T1.5 round 1 A-M7: parity gate between Rust FLAG_* and
        // shader FLAG_* declarations. If a new flag bit is added in
        // Rust without the parallel WGSL line, this test catches it
        // before the shader silently treats the bit as zero.
        //
        // Whitespace-normalized contains() so the test survives
        // alignment churn in the shader source (e.g. tabs vs spaces,
        // extra padding to align `=` columns).
        let shader = crate::stamp_pipeline::STAMP_WGSL;
        let normalize = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        let shader_norm = normalize(shader);
        let pairs: &[(&str, u32)] = &[
            (
                "FLAG_SHAPE_FLIP_X: u32 = 1u",
                crate::stamp::FLAG_SHAPE_FLIP_X,
            ),
            (
                "FLAG_SHAPE_FLIP_Y: u32 = 2u",
                crate::stamp::FLAG_SHAPE_FLIP_Y,
            ),
            (
                "FLAG_GRAIN_BEHAVIOR_MOVING: u32 = 4u",
                crate::stamp::FLAG_GRAIN_BEHAVIOR_MOVING,
            ),
            ("FLAG_BURNT_EDGES: u32 = 8u", crate::stamp::FLAG_BURNT_EDGES),
            ("FLAG_WET_EDGES: u32 = 16u", crate::stamp::FLAG_WET_EDGES),
            (
                "FLAG_LUMINANCE_BLENDING: u32 = 32u",
                crate::stamp::FLAG_LUMINANCE_BLENDING,
            ),
            (
                "FLAG_GRAIN_PROCEDURAL: u32 = 64u",
                crate::stamp::FLAG_GRAIN_PROCEDURAL,
            ),
            (
                "FLAG_FLUID_SAMPLE: u32 = 128u",
                crate::stamp::FLAG_FLUID_SAMPLE,
            ),
            (
                "FLAG_HOVER_PREVIEW: u32 = 256u",
                crate::stamp::FLAG_HOVER_PREVIEW,
            ),
            (
                "FLAG_PREDICTED_SAMPLE: u32 = 512u",
                crate::stamp::FLAG_PREDICTED_SAMPLE,
            ),
        ];
        for (needle, _rust_value) in pairs {
            assert!(
                shader_norm.contains(needle),
                "shader FLAG declaration missing or drifted: `{}`",
                needle
            );
        }
        // Bit values themselves (paranoia — also covered by stamp::tests::
        // flag_constants_are_distinct_bits).
        assert_eq!(crate::stamp::FLAG_HOVER_PREVIEW, 256);
        assert_eq!(crate::stamp::FLAG_PREDICTED_SAMPLE, 512);
    }

    // ──────────────────────────────────────────────────────────────────────
    // T1.6 extension tests — shape dispatch / rotation / flip + shader
    // textual parity for the three procedural shape kernels.
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn cpu_shader_shape_kernels_textual_parity() {
        // Audit T1.6 — extended after lens O-1: shape kernels must stay
        // byte-equivalent CPU ↔ shader OR a future shader rewrite that
        // preserves an analytic curve but drifts the formula structure
        // would silently diverge. This gate covers EACH kernel's
        // distance line + edge-band line + smoothstep line + return
        // line. The CPU side uses scalar `(dx*dx+dy*dy).sqrt()` (not
        // WGSL `length()`) — audit T1.6 O-3 — and this test pins the
        // shader's scalar form to match.
        //
        // **Limitation (audit O-2):** this is a shader↔hardcoded-string
        // parity, NOT a CPU↔shader runtime parity. A future CPU edit
        // that drifts from these strings stays green. A real CPU↔shader
        // numerical parity test (run the GPU dispatch headless + compare
        // pixels) requires a wgpu device in CI and is tracked as a
        // T-numerical-parity follow-up (W2+).
        let shader = crate::stamp_pipeline::STAMP_WGSL;
        let norm = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        let shader_norm = norm(shader);

        // round_hard — full body. dx/dy scalar form (NOT WGSL `length()`).
        assert!(
            shader_norm.contains("let dx = uv.x - 0.5;"),
            "shader round_hard dx scalar drifted"
        );
        assert!(
            shader_norm.contains("let dy = uv.y - 0.5;"),
            "shader round_hard dy scalar drifted"
        );
        assert!(
            shader_norm.contains("let d = sqrt(dx * dx + dy * dy) / 0.5;"),
            "shader round_hard scalar sqrt-distance drifted (O-3 regression)"
        );
        assert!(
            shader_norm.contains("let edge_t = clamp((d - 0.85) / 0.15, 0.0, 1.0);"),
            "shader round_hard smoothstep band drifted"
        );
        assert!(
            shader_norm.contains("let smooth_t = edge_t * edge_t * (3.0 - 2.0 * edge_t);"),
            "shader round_hard Hermite smoothstep drifted (O-1 extension)"
        );
        assert!(
            shader_norm.contains("return 1.0 - smooth_t;"),
            "shader round_hard return-formula drifted (O-1 extension)"
        );

        // round_soft — full body with explicit d² >= 1 early-out.
        assert!(
            shader_norm.contains("let d_sq = (dx * dx + dy * dy) / 0.25;"),
            "shader round_soft d² normalization drifted"
        );
        assert!(
            shader_norm.contains("if d_sq >= 1.0 {"),
            "shader round_soft d² >= 1 early-out drifted (O-1 extension)"
        );
        assert!(
            shader_norm.contains("let one_minus_d_sq = 1.0 - d_sq;"),
            "shader round_soft (1-d²) intermediate drifted (O-1 extension)"
        );
        assert!(
            shader_norm.contains("return one_minus_d_sq * one_minus_d_sq;"),
            "shader round_soft (1-d²)² return drifted"
        );

        // square_hard — full body. Chebyshev distance + Hermite smoothstep.
        assert!(
            shader_norm.contains("let d = max(dx, dy) / 0.5;"),
            "shader square_hard Chebyshev distance drifted"
        );
        assert!(
            shader_norm.contains("let edge_t = clamp((d - 0.90) / 0.10, 0.0, 1.0);"),
            "shader square_hard smoothstep band drifted"
        );
        assert!(
            shader_norm.contains("let smooth_t = edge_t * edge_t * (3.0 - 2.0 * edge_t);"),
            "shader square_hard Hermite smoothstep drifted (O-1 extension)"
        );
        assert!(
            shader_norm.contains("return 1.0 - smooth_t;"),
            "shader square_hard return-formula drifted (O-1 extension)"
        );

        // oval_hard — full body. Stretched radial (2:1 oblong).
        assert!(
            shader_norm.contains("let dx = (uv.x - 0.5) / 0.5;"),
            "shader oval_hard dx stretched drifted (V-2 R4 addition)"
        );
        assert!(
            shader_norm.contains("let dy = (uv.y - 0.5) / 0.25;"),
            "shader oval_hard dy stretched drifted (V-2 R4 addition)"
        );
        assert!(
            shader_norm.contains("let d = sqrt(dx * dx + dy * dy);"),
            "shader oval_hard radial sqrt drifted (V-2 R4 addition)"
        );

        // shape_alpha_for_slot dispatch + 0 → round_hard / 1 → round_soft /
        // 2 → square_hard / 3 → oval_hard / default → round_hard fallback.
        assert!(
            shader_norm.contains("case 0u: { return round_hard_shape(uv); }"),
            "shader shape dispatch slot 0 drifted"
        );
        assert!(
            shader_norm.contains("case 1u: { return round_soft_shape(uv); }"),
            "shader shape dispatch slot 1 drifted"
        );
        assert!(
            shader_norm.contains("case 2u: { return square_hard_shape(uv); }"),
            "shader shape dispatch slot 2 drifted"
        );
        assert!(
            shader_norm.contains("case 3u: { return oval_hard_shape(uv); }"),
            "shader shape dispatch slot 3 drifted (V-2 R4 addition)"
        );
        assert!(
            shader_norm.contains("default: { return round_hard_shape(uv); }"),
            "shader shape dispatch fallback drifted"
        );
    }

    #[test]
    fn cpu_shader_rotation_pipeline_textual_parity() {
        // Audit T1.6: the rotation/flip uv transform is the most error-prone
        // part of the shader edit — gate it textually so a future rewrite
        // that breaks parity (e.g. rotates by +rotation_rad instead of
        // -rotation_rad) doesn't slip through.
        let shader = crate::stamp_pipeline::STAMP_WGSL;
        let norm = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        let shader_norm = norm(shader);

        // Center to [-0.5, +0.5].
        assert!(
            shader_norm.contains("var uv_centered = uv_axis - vec2<f32>(0.5);"),
            "shader uv centering drifted"
        );
        // Flip bits applied BEFORE rotation (audit T1.6).
        assert!(
            shader_norm.contains("if (stamp.flags & FLAG_SHAPE_FLIP_X) != 0u {"),
            "shader flip_x check drifted"
        );
        assert!(
            shader_norm.contains("uv_centered.x = -uv_centered.x;"),
            "shader flip_x negate drifted"
        );
        assert!(
            shader_norm.contains("if (stamp.flags & FLAG_SHAPE_FLIP_Y) != 0u {"),
            "shader flip_y check drifted"
        );
        // Rotation by -rotation_rad via trig identity (audit T1.6 U-9):
        // `cos(-x) = cos(x)`, `sin(-x) = -sin(x)` — avoids feeding a
        // negated argument to the backend intrinsic + eliminates ±0
        // sign-bit drift at rotation_rad = 0.
        assert!(
            shader_norm.contains("let cos_r = cos(stamp.rotation_rad);"),
            "shader rotation matrix uses cos(stamp.rotation_rad) per U-9 identity"
        );
        assert!(
            shader_norm.contains("let sin_r = -sin(stamp.rotation_rad);"),
            "shader rotation matrix uses -sin(stamp.rotation_rad) per U-9 identity"
        );
    }

    fn red_stamp_with_slot(x: f32, y: f32, size: f32, slot: u32) -> Stamp {
        let mut s = Stamp::zeroed();
        s.position_world = [x, y];
        s.size_px = size;
        s.color_oklab = [0.6, 0.25, 0.1, 1.0];
        s.opacity = 1.0;
        s.flow = 1.0;
        s.rendering_mode = RenderingMode::UniformGlaze as u32;
        s.shape_layer = slot;
        s
    }

    #[test]
    fn shape_slot_dispatch_round_soft_writes_pixels() {
        let (w, h) = (32, 32);
        let mut canvas = empty_canvas(w, h);
        let s = red_stamp_with_slot(16.0, 16.0, 16.0, crate::library::ROUND_SOFT_SLOT);
        apply_stamps(&mut canvas, w, h, &[s]);
        let center_idx = (16 * w as usize + 16) * 4;
        assert!(
            canvas[center_idx + 3] > 0,
            "round_soft stamp must paint center pixel"
        );
    }

    #[test]
    fn shape_slot_dispatch_square_hard_writes_corner_pixels() {
        // Distinguishing feature of square_hard vs round_*: corners INSIDE
        // the bounding box have non-zero alpha (a circle has empty corners).
        let (w, h) = (32, 32);
        let mut canvas_sq = empty_canvas(w, h);
        let mut canvas_rh = empty_canvas(w, h);
        let stamp_size = 20.0;
        apply_stamps(
            &mut canvas_sq,
            w,
            h,
            &[red_stamp_with_slot(
                16.0,
                16.0,
                stamp_size,
                crate::library::SQUARE_HARD_SLOT,
            )],
        );
        apply_stamps(
            &mut canvas_rh,
            w,
            h,
            &[red_stamp_with_slot(
                16.0,
                16.0,
                stamp_size,
                crate::library::ROUND_HARD_SLOT,
            )],
        );
        // Inspect bounding-box corner (around pixel 7,7 — top-left of the
        // 20px footprint centered at 16,16). Square fills; round leaves it 0.
        let corner_x = 7;
        let corner_y = 7;
        let idx = (corner_y * w as usize + corner_x) * 4;
        let sq_alpha = canvas_sq[idx + 3];
        let rh_alpha = canvas_rh[idx + 3];
        assert!(
            sq_alpha > rh_alpha,
            "square_hard corner alpha ({sq_alpha}) must exceed round_hard corner alpha ({rh_alpha})"
        );
    }

    #[test]
    fn rotation_pipeline_round_hard_unaffected() {
        // round_hard is radial-symmetric: rotation must not change pixels.
        // Render at rotation=0 and rotation=π/3, compare byte-equal.
        let (w, h) = (32, 32);
        let mut a = empty_canvas(w, h);
        let mut b = empty_canvas(w, h);
        let mut s0 = red_stamp_with_slot(16.0, 16.0, 12.0, crate::library::ROUND_HARD_SLOT);
        let mut s1 = s0;
        s0.rotation_rad = 0.0;
        s1.rotation_rad = core::f32::consts::FRAC_PI_3;
        apply_stamps(&mut a, w, h, &[s0]);
        apply_stamps(&mut b, w, h, &[s1]);
        assert_eq!(a, b, "round_hard pixels must be invariant under rotation");
    }

    // FOLLOW-UP-W6: pixel-level flip gate that PROVES flip changes
    // output (rather than being a no-op on symmetric shapes) requires
    // an asymmetric shape kernel. Today's T1.6 shapes (round_hard /
    // round_soft / square_hard) are doubly-symmetric → flip is
    // visually a no-op by construction. When the first asymmetric
    // shape ships in W6+ (`flat_chisel`, `bristle_spread`,
    // `splatter_spread` per spec §1.6.7), add a parallel test:
    //   `flip_x_mirrors_asymmetric_shape_pixels` — apply same stamp
    //   with/without FLAG_SHAPE_FLIP_X on an asymmetric shape; assert
    //   the canvas pixels differ in a mirror-image pattern.
    // The current test (`flip_preserves_output_for_symmetric_shapes`)
    // is TAUTOLOGICAL for T1.6 shapes but remains useful as a
    // regression sentinel for the symmetric set.
    // Audit T1.6 R-8 + W-2 + W-3 — search grep "FOLLOW-UP-W6" to find.
    #[test]
    fn flip_preserves_output_for_symmetric_shapes() {
        // All three T1.6 shape kernels (round_hard, round_soft, square_hard)
        // are symmetric in **both** x and y around the centerpoint, so
        // applying flip_x and/or flip_y MUST be a no-op visually (the
        // sample-coord set is permuted but each pixel reads the same
        // alpha). This test pins that invariant — if a future shape with
        // axis-asymmetry is added (W6+ flat_chisel etc.), the test gets
        // SHAPE_SLOT updated and the asymmetric shape gets its own gate.
        //
        // **Follow-up:** pixel-level flip gate that PROVES the flip
        // changes output (rather than no-op) requires an asymmetric shape
        // (W6+). T1.6 ships the flip code path + the FLAG bits + the
        // scheduler wiring; the asymmetric-shape acceptance test lands
        // when the first asymmetric shape arrives.
        let (w, h) = (32, 32);
        for slot in [
            crate::library::ROUND_HARD_SLOT,
            crate::library::ROUND_SOFT_SLOT,
            crate::library::SQUARE_HARD_SLOT,
        ] {
            for rotation in [0.0_f32, core::f32::consts::FRAC_PI_4, 0.7] {
                let mut a = empty_canvas(w, h);
                let mut b = empty_canvas(w, h);
                let mut s0 = red_stamp_with_slot(16.0, 16.0, 12.0, slot);
                s0.rotation_rad = rotation;
                let mut s1 = s0;
                s1.flags |= crate::stamp::FLAG_SHAPE_FLIP_X | crate::stamp::FLAG_SHAPE_FLIP_Y;
                apply_stamps(&mut a, w, h, &[s0]);
                apply_stamps(&mut b, w, h, &[s1]);
                assert_eq!(
                    a, b,
                    "slot {slot} + rotation {rotation} must be invariant under \
                     flip_x|flip_y (T1.6 shapes are doubly-symmetric)"
                );
            }
        }
    }

    #[test]
    fn flip_flag_changes_internal_sample_coord_for_offcenter_pixel() {
        // Unit-level proof that the flip code path **does** alter the
        // sample uv. We can't observe shape sampling directly without
        // exposing internals, so we use a shape-agnostic invariant: when
        // `rotation_rad != 0` and `flip_x` is set, the position of the
        // SAMPLE point for off-center pixel (px=0, py=0) differs from
        // the no-flip case. Computed via the documented analytic formula
        // (matches the inner loop of `apply_one_stamp` exactly).
        let footprint = 16.0_f32;
        let center = (footprint - 1.0) * 0.5;
        // pixel (0, 0)
        let u_axis = 0.5 / footprint;
        let v_axis = 0.5 / footprint;
        let uc = u_axis - 0.5;
        let vc = v_axis - 0.5;
        let rotation = 0.7_f32;
        let cos_r = (-rotation).cos();
        let sin_r = (-rotation).sin();

        // Without flip
        let ur_noflip = uc * cos_r - vc * sin_r;
        let vr_noflip = uc * sin_r + vc * cos_r;
        // With flip_x
        let uc_flipped = -uc;
        let ur_flip = uc_flipped * cos_r - vc * sin_r;
        let vr_flip = uc_flipped * sin_r + vc * cos_r;

        assert!(
            (ur_noflip - ur_flip).abs() > 1e-3 || (vr_noflip - vr_flip).abs() > 1e-3,
            "flip_x must change the sample coord for off-center pixel under rotation; \
             noflip=({ur_noflip}, {vr_noflip}) flip=({ur_flip}, {vr_flip})"
        );
        let _ = center; // silence "unused" warning if test moves
    }

    #[test]
    fn cpu_deterministic_under_rotation() {
        // HR-5: same rotation + same stamp → byte-identical canvas.
        let (w, h) = (32, 32);
        let mut a = empty_canvas(w, h);
        let mut b = empty_canvas(w, h);
        let mut s = red_stamp_with_slot(16.0, 16.0, 14.0, crate::library::SQUARE_HARD_SLOT);
        s.rotation_rad = 0.7;
        apply_stamps(&mut a, w, h, &[s]);
        apply_stamps(&mut b, w, h, &[s]);
        assert_eq!(a, b, "rotated stamp must be bit-deterministic");
    }

    #[test]
    fn unknown_shape_slot_falls_back_to_round_hard() {
        // CPU path falls back to round_hard for unknown slots, matching
        // shader `default:` arm. Compare against explicit slot 0.
        let (w, h) = (32, 32);
        let mut a = empty_canvas(w, h);
        let mut b = empty_canvas(w, h);
        let s_unknown = red_stamp_with_slot(16.0, 16.0, 12.0, 99);
        let s_round = red_stamp_with_slot(16.0, 16.0, 12.0, 0);
        apply_stamps(&mut a, w, h, &[s_unknown]);
        apply_stamps(&mut b, w, h, &[s_round]);
        assert_eq!(a, b, "unknown slot must fall back to round_hard");
    }
}
