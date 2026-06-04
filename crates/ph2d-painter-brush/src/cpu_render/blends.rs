//! RenderingMode dispatch + glaze/blending kernels, split out of the
//! former `cpu_render.rs` (pure mechanical move).

use super::*;

/// Aplica um rendering mode em premul linear sRGB. **Funções idênticas
/// às do shader** (`uniform_glaze`, `intense_glaze`, etc.) — invariante
/// premul preservada em todo o caminho.
#[inline]
pub(crate) fn apply_rendering_mode(
    mode: RenderingMode,
    src: [f32; 4],
    dst: [f32; 4],
    wet: f32,
) -> [f32; 4] {
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
pub(crate) fn light_glaze(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let k = 1.0 - src[3] * 0.6;
    [
        src[0] + dst[0] * k,
        src[1] + dst[1] * k,
        src[2] + dst[2] * k,
        src[3] + dst[3] * k,
    ]
}

#[inline]
pub(crate) fn uniform_glaze(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let k = 1.0 - src[3];
    [
        src[0] + dst[0] * k,
        src[1] + dst[1] * k,
        src[2] + dst[2] * k,
        src[3] + dst[3] * k,
    ]
}

#[inline]
pub(crate) fn intense_glaze(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
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
pub(crate) fn heavy_glaze(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let k = 1.0 - src[3] * 0.85;
    [
        (src[0] + dst[0] * k).clamp(0.0, 1.0),
        (src[1] + dst[1] * k).clamp(0.0, 1.0),
        (src[2] + dst[2] * k).clamp(0.0, 1.0),
        (src[3] + dst[3] * k).clamp(0.0, 1.0),
    ]
}

#[inline]
pub(crate) fn uniform_blending(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
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
pub(crate) fn intense_blending(src: [f32; 4], dst: [f32; 4], wet: f32) -> [f32; 4] {
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
