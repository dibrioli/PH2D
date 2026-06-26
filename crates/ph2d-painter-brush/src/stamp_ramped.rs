//! Cached **Colour-Ramp** stamp blit — the Shape Colour Ramp indexed by the dab coverage.
//!
//! The colour ramp paints `ramp[coverage]` per texel; for the **Shape** ramp (no Grain) that coverage
//! is exactly the falloff × silhouette weight the [`crate::stamp::StampMask`] already caches. So instead
//! of recomputing the silhouette per pixel per dab (the per-pixel [`crate::stamp_dab_ramped`] path), we
//! reuse the cached mask + the scale-blit and only do a 256-entry LUT lookup per texel — a Shape-ramp
//! stroke is then as cheap as a plain cached stamp. The blit matches the per-pixel no-Grain ramp branch
//! (its no-accumulate-cap finalize, all 3 alpha modes) up to the cache's bilinear approximation, the
//! same trade the non-ramp cache already makes (Enio 2026-06-26).

use crate::blend::BrushBlend;
use crate::dab::{DirtyRect, encode, parallel_band_stamp, ramp_sample, stamp_rgba};
use crate::ramp_alpha::RampAlphaMode;
use crate::spec::BrushSpec;
use crate::stamp::{StampMask, sample_mask};

/// Read-only per-dab blit context for the colour-ramp path: the cached coverage mask, but each texel's
/// colour comes from `ramp[coverage]` (the alpha acts per `alpha_mode`) instead of one brush colour.
struct RampedBlitCtx<'a> {
    mask: &'a StampMask,
    ramp: &'a [[f32; 4]],
    alpha_mode: RampAlphaMode,
    blend: BrushBlend,
    cx: f32,
    cy: f32,
    inv_radius: f32,
    coverage: f32,
    preserve_alpha: bool,
    x0: i64,
    x1: i64,
    stride: usize,
}

/// Like [`crate::stamp::blit_stamp`] but the per-texel colour is `ramp[coverage]` — the **Shape Colour
/// Ramp** (indexed by the dab coverage the mask stores). `coverage` folds in Flow × Strength exactly as
/// `blit_stamp`. The caller routes here only for the cacheable, no-accumulate-cap, no-per-dab-rotation,
/// no-Grain cases (else the per-pixel [`crate::stamp_dab_ramped`]). Returns the touched [`DirtyRect`].
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn blit_stamp_ramped(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    radius: f32,
    mask: &StampMask,
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
    ramp: &[[f32; 4]],
    alpha_mode: RampAlphaMode,
) -> Option<DirtyRect> {
    debug_assert!(
        buf.len() >= (width as usize) * (height as usize) * 4,
        "buffer too small"
    );
    let coverage =
        coverage.clamp(0.0, 1.0) * spec.flow.clamp(0.0, 1.0) * spec.strength.clamp(0.0, 1.0);
    if coverage <= 0.0 || width == 0 || height == 0 || radius <= 0.0 {
        return None;
    }
    let (cx, cy) = (center[0], center[1]);
    let x0 = (cx - radius).floor().max(0.0) as i64;
    let y0 = (cy - radius).floor().max(0.0) as i64;
    let x1 = ((cx + radius).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + radius).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let stride = (width as usize) * 4;
    let ctx = RampedBlitCtx {
        mask,
        ramp,
        alpha_mode,
        blend: spec.blend,
        cx,
        cy,
        inv_radius: 1.0 / radius,
        coverage,
        preserve_alpha,
        x0,
        x1,
        stride,
    };
    let touched = parallel_band_stamp(buf, y0, y1, x0, x1, stride, |dst, band_y0| {
        blit_band_ramped(&ctx, dst, band_y0)
    });
    if !touched {
        return None;
    }
    Some(DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}

/// Blit the cached coverage mask into the band, colouring each texel by `ramp[coverage]` — the exact
/// no-accumulate-cap finalize of the per-pixel [`crate::dab`] ramp branch (the 3 alpha modes).
fn blit_band_ramped(ctx: &RampedBlitCtx, dst: &mut [u8], band_y0: i64) -> bool {
    let mut touched = false;
    for r in 0..dst.len() / ctx.stride {
        let py = band_y0 + r as i64;
        let v = ((py as f32 + 0.5) - ctx.cy) * ctx.inv_radius;
        let row = r * ctx.stride;
        for px in ctx.x0..ctx.x1 {
            let u = ((px as f32 + 0.5) - ctx.cx) * ctx.inv_radius;
            let cov = sample_mask(ctx.mask, u, v);
            if cov <= 0.0 {
                continue;
            }
            let c = ramp_sample(ctx.ramp, cov);
            let color = [c[0], c[1], c[2]];
            let i = row + (px as usize) * 4;
            let prev = [
                f32::from(dst[i]) / 255.0,
                f32::from(dst[i + 1]) / 255.0,
                f32::from(dst[i + 2]) / 255.0,
                f32::from(dst[i + 3]) / 255.0,
            ];
            let out = if matches!(ctx.alpha_mode, RampAlphaMode::TextureAlpha) {
                // Stamp the ramp as RGBA: translucent ramp areas LOWER the sprite's own alpha.
                let m = cov * ctx.coverage;
                if m <= 0.0 {
                    continue;
                }
                stamp_rgba(prev, color, c[3], m)
            } else {
                // None: coverage only. Strength: the ramp alpha scales coverage (paints weaker).
                let w = if matches!(ctx.alpha_mode, RampAlphaMode::Strength) {
                    cov * c[3]
                } else {
                    cov
                };
                let a = if ctx.preserve_alpha {
                    w * ctx.coverage * prev[3]
                } else {
                    w * ctx.coverage
                };
                if a <= 0.0 {
                    continue;
                }
                crate::blend::blend_over(ctx.blend, prev, color, a)
            };
            dst[i] = encode(out[0]);
            dst[i + 1] = encode(out[1]);
            dst[i + 2] = encode(out[2]);
            dst[i + 3] = encode(out[3]);
            touched = true;
        }
    }
    touched
}
