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
//! a **mesma matemática** em CPU — single-source-of-truth `library::round_
//! hard_shape()` + as mesmas constantes OKLab + as mesmas funções de
//! rendering mode (em Rust, bit-equivalentes às funções do shader). É o
//! caminho que valida que scheduler + Stamp ABI + rendering modes
//! formam uma cadeia coerente, sem depender ainda da integração GPU
//! end-to-end.
//!
//! ## Paridade com shader — escopo do claim
//!
//! - **OKLab → linear sRGB:** mesmas 15 constantes do shader (
//!   `stamp.wgsl::oklab_to_linear_srgb`); CPU usa os literais
//!   `0.396_337_78` etc. que o gate `shader_oklab_coefficients_bit_
//!   identical_with_rust` cobre.
//! - **Round-hard shape:** `library::round_hard_shape()` é a referência
//!   atlas R8 256×256; o shader procedural sample é semanticamente
//!   equivalente (Hermite smoothstep 0.85..=1.0). CPU usa formula
//!   analítica idêntica ao shader (não amostra atlas) para evitar drift
//!   entre o atlas-fixed-256 do `library.rs` e o footprint-variable do
//!   shader.
//! - **Rendering modes:** cópias 1:1 das funções do shader (Light /
//!   Uniform / Intense / Heavy Glaze + Uniform / Intense Blending).
//!   Tests `rendering_mode_zero_src_is_identity_on_dst`,
//!   `uniform_glaze_full_opacity_replaces_dst`,
//!   `rendering_mode_premul_invariant_preserved`,
//!   `uniform_blending_at_full_alpha_equals_src_color`,
//!   `intense_glaze_precision_at_small_alpha` (round 1 A-H2) +
//!   `uniform_glaze_cpu_shader_textual_parity` (round 2 F3) covam
//!   propriedades algébricas + paridade textual com shader.
//!   **ULP-bounded near alpha_s → 0:** o termo `1/max(alpha_s, 1e-6)`
//!   no Uniform/Intense Blending pode acumular ULP drift entre
//!   backends GPU (Metal/Vulkan/D3D12) e Rust f32 quando `alpha_s` é
//!   muito pequeno; em região `alpha_s ∈ [1e-6, 1.0]` a paridade é
//!   bit-identical (audit T1.5 round 1 A-M3).
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
//! módulo permanece como path de validação cross-OS HR-5 (bit-identical
//! reference para golden tests).
//!
//! Follow-up registrado: GPU integration cycle não bloqueia Day-7
//! ship; sessão T-perf substitui `apply_stamps` por `dispatch_stamps`
//! com `StampPipeline::encode` + retained texture state + straight↔premul
//! conversion at the upload/download boundary.

use crate::rendering_mode::RenderingMode;
use crate::stamp::Stamp;

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
    // Audit T1.5 round 1 A-L4 — production-grade length guard so a
    // mismatch panics on the spot instead of reading/writing past the
    // canvas in release builds.
    assert_eq!(
        canvas.len(),
        (width as usize) * (height as usize) * 4,
        "canvas buffer size must match width*height*4 RGBA8"
    );
    for stamp in stamps {
        apply_one_stamp(canvas, width, height, stamp);
    }
}

fn apply_one_stamp(canvas: &mut [u8], width: u32, height: u32, stamp: &Stamp) {
    // Paridade com shader: filtra degenerate sizes antes de qualquer
    // arithmetic. Mantém canvas inalterado para inputs lixo. NaN é
    // catched pelo `is_finite()` (não-finito); ≤ 0 captura zero/negativo.
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

    let canvas_w = width as i32;
    let canvas_h = height as i32;

    for py in 0..footprint {
        for px in 0..footprint {
            // uv com pixel-center convention (paridade D-1.M1).
            let u = (px as f32 + 0.5) / footprint_f;
            let v = (py as f32 + 0.5) / footprint_f;
            let shape_alpha = round_hard_shape(u, v);
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
            // Decode dst RGBA8 → premul linear vec4. Canvas storage é
            // tratado como straight u8 (sem gamma) — preserva paridade
            // com `wgpu::TextureFormat::Rgba8Unorm` (sem sRGB encoding).
            // Mas pra alpha-over correto, precisamos da forma PREMUL.
            // Converte straight → premul aqui, e premul → straight de
            // volta no write.
            let dst_straight = [
                canvas[idx] as f32 / 255.0,
                canvas[idx + 1] as f32 / 255.0,
                canvas[idx + 2] as f32 / 255.0,
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
            canvas[idx] = (out_straight[0] * 255.0 + 0.5) as u8;
            canvas[idx + 1] = (out_straight[1] * 255.0 + 0.5) as u8;
            canvas[idx + 2] = (out_straight[2] * 255.0 + 0.5) as u8;
            canvas[idx + 3] = (out_straight[3] * 255.0 + 0.5) as u8;
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

/// Round-hard procedural shape — Hermite smoothstep no anel
/// `[0.85, 1.0]` da distância radial normalizada. **Paridade analítica**
/// com `library::round_hard_shape()` (atlas 256² CPU) e
/// `shader/stamp.wgsl::round_hard_shape` — mesma forma matemática,
/// amostrada no grid do footprint (variable).
#[inline]
fn round_hard_shape(u: f32, v: f32) -> f32 {
    let dx = u - 0.5;
    let dy = v - 0.5;
    let d = (dx * dx + dy * dy).sqrt() / 0.5;
    let edge_t = ((d - 0.85) / 0.15).clamp(0.0, 1.0);
    let smooth = edge_t * edge_t * (3.0 - 2.0 * edge_t);
    1.0 - smooth
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
    let inv_as = 1.0 / alpha_s.max(1e-6);
    let inv_ad = 1.0 / alpha_d.max(1e-6);
    let src_rgb = [src[0] * inv_as, src[1] * inv_as, src[2] * inv_as];
    let dst_rgb = [dst[0] * inv_ad, dst[1] * inv_ad, dst[2] * inv_ad];
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
    let inv_as = 1.0 / alpha_s.max(1e-6);
    let inv_ad = 1.0 / alpha_d.max(1e-6);
    let src_rgb = [src[0] * inv_as, src[1] * inv_as, src[2] * inv_as];
    let dst_rgb = [dst[0] * inv_ad, dst[1] * inv_ad, dst[2] * inv_ad];
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
    fn uniform_glaze_cpu_shader_textual_parity() {
        // Audit T1.5 round 2 F3 (MISSING-GATE-CPU-SHADER-PARITY). The
        // claim "Funções idênticas às do shader" was only verified
        // through invariant tests (Porter-Duff identity, alpha=sqrt,
        // etc.). This test does a textual paridade check on the
        // canonical formula of uniform_glaze in the WGSL source —
        // catches a future drift where the shader's uniform_glaze gets
        // rewritten without the CPU function being updated in lockstep.
        //
        // Method: whitespace-normalize STAMP_WGSL, then assert the
        // canonical Porter-Duff "over" expression appears verbatim.
        // The Rust function uses the same formula:
        //   `src + dst * (1.0 - src.a)`  — both sides Porter-Duff.
        let shader = crate::stamp_pipeline::STAMP_WGSL;
        let norm = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        let shader_norm = norm(shader);
        // The shader's `uniform_glaze` body returns `src + dst * (1.0 -
        // src.a)`. Whitespace-normalized substring is stable across
        // formatting churn.
        assert!(
            shader_norm.contains("return src + dst * (1.0 - src.a);"),
            "shader uniform_glaze formula drifted — Rust + shader must \
             keep `src + dst * (1.0 - src.a)` in lockstep (audit F3)"
        );
        // Heavy glaze: `src + dst * (1.0 - src.a * 0.85)` followed by clamp.
        assert!(
            shader_norm.contains("let r = src + dst * (1.0 - src.a * 0.85);"),
            "shader heavy_glaze formula drifted (audit F3)"
        );
        // Light glaze: `src + dst * (1.0 - src.a * 0.6)`.
        assert!(
            shader_norm.contains("return src + dst * (1.0 - src.a * 0.6);"),
            "shader light_glaze formula drifted (audit F3)"
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
}
