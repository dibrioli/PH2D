//! Blur (soften) one dab — average the canvas neighbourhood under the brush and blend it back.
//!
//! **Clean-room** behaviour of Blender's 2D image-paint Soften brush
//! (`editors/sculpt_paint/mesh/paint_image_2d.cc`: `paint_2d_lift_soften` convolves the footprint with
//! a kernel — Gaussian by default — then composites the blurred buffer over the canvas with
//! `IMB_BLEND_INTERPOLATE`, i.e. `lerp(canvas, blurred, mask × strength)`). This is the
//! community-standard blur brush (GIMP/Krita/Photoshop): a separable convolution blended back by the
//! dab mask. Only the algorithm is ported, never the code.
//!
//! Two deliberate departures from Blender, both required here:
//! - **Kernel:** Blender's Gaussian uses `exp()`, which is libm-variable across platforms and would
//!   break the replay-hash determinism gate (HR-5). We use the **binomial (Pascal) kernel** — the
//!   deterministic, transcendental-free Gaussian approximation (integer recurrence, separable H×V).
//! - **Kernel radius:** Blender's is a fixed per-brush setting independent of size. We have no UI
//!   control for it, so it's derived from the brush radius (community consensus ≈⅓ · radius, capped),
//!   which also makes the blur scale with the brush like every other slot.
//!
//! Blur is **stationary per dab** (unlike Smear it needs no motion): a single click softens the
//! footprint, and dense dabs along a stroke build up progressively. The neighbourhood is read into a
//! snapshot BEFORE any write, so a dab never feeds its own output back into its own reads.

use crate::dab::DirtyRect;
use crate::spec::BrushSpec;
use crate::stamp::{StampMask, sample_mask};

/// Kernel radius as a fraction of the dab radius (community consensus for a size-proportional blur).
const BLUR_KERNEL_FRAC: f32 = 0.34;
/// Hard cap on the kernel radius (px) — bounds the per-dab convolution cost for huge brushes; the blur
/// still builds up over repeated dabs. A perf follow-up (SAT / cached region) can lift it.
const BLUR_KERNEL_MAX: usize = 32;

/// The blur kernel radius (px) for a dab of `radius`: `round(FRAC · radius)`, clamped to `1..=MAX`.
#[must_use]
pub(crate) fn kernel_radius(radius: f32) -> usize {
    ((radius * BLUR_KERNEL_FRAC).round() as i64).clamp(1, BLUR_KERNEL_MAX as i64) as usize
}

/// Normalized 1D **binomial** weights for kernel radius `k` (side `2k+1`) — the transcendental-free
/// Gaussian approximation (`C(2k, i)` via the multiplicative recurrence, then scaled to sum `1`). σ ≈
/// `√(k/2)`. Separable: applied horizontally then vertically gives the isotropic 2D kernel.
fn binomial_weights(k: usize) -> Vec<f32> {
    let n = 2 * k;
    let mut w = Vec::with_capacity(n + 1);
    let mut c = 1.0f64; // C(n, 0)
    w.push(1.0f32);
    for i in 1..=n {
        c = c * (n - i + 1) as f64 / i as f64; // C(n, i) = C(n, i-1) · (n-i+1)/i
        w.push(c as f32);
    }
    let inv: f32 = 1.0 / w.iter().sum::<f32>();
    for x in &mut w {
        *x *= inv;
    }
    w
}

/// Read `(sx, sy)` mapping for one apron sample: toroidal (`rem_euclid`) on a `wrap` axis (seamless
/// Tiling), else clamped to the nearest edge (extend — so a border neither fades the alpha nor bleeds
/// transparent RGB into the blur).
#[inline]
fn src_coord(v: i64, span: i64, wrap: bool) -> i64 {
    if wrap {
        v.rem_euclid(span)
    } else {
        v.clamp(0, span - 1)
    }
}

/// Blur the canvas region `[min_x, min_x+bw) × [min_y, min_y+bh)` with a binomial kernel of radius `k`,
/// returning `bw·bh` **straight** RGBA (`f32`, `0..=255`), row-major. The convolution runs in
/// **premultiplied** space (RGB × α, blur, then un-premultiply) so transparent neighbours don't bleed
/// their arbitrary RGB into the result — the standard fringe-free image blur. The apron (`±k`) is read
/// with [`src_coord`] (wrap / clamp). Snapshots the source first, so callers may write the region after.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub(crate) fn blur_region(
    buf: &[u8],
    fw: i64,
    fh: i64,
    min_x: i64,
    min_y: i64,
    bw: usize,
    bh: usize,
    k: usize,
    wrap: [bool; 2],
) -> Vec<[f32; 4]> {
    let w = binomial_weights(k);
    let ap_w = bw + 2 * k;
    let ap_h = bh + 2 * k;
    // Apron in PREMULTIPLIED RGBA (rgb·α with α as a 0..1 fraction; alpha kept as a 0..255 byte value).
    let mut apron = vec![[0f32; 4]; ap_w * ap_h];
    for j in 0..ap_h {
        let sy = src_coord(min_y + j as i64 - k as i64, fh, wrap[1]);
        for i in 0..ap_w {
            let sx = src_coord(min_x + i as i64 - k as i64, fw, wrap[0]);
            let si = ((sy * fw + sx) * 4) as usize;
            let a = f32::from(buf[si + 3]);
            let af = a / 255.0;
            apron[j * ap_w + i] = [
                f32::from(buf[si]) * af,
                f32::from(buf[si + 1]) * af,
                f32::from(buf[si + 2]) * af,
                a,
            ];
        }
    }
    // Horizontal pass: apron (ap_w × ap_h) → hbuf (bw × ap_h).
    let mut hbuf = vec![[0f32; 4]; bw * ap_h];
    for j in 0..ap_h {
        let row = j * ap_w;
        for i in 0..bw {
            let mut acc = [0f32; 4];
            for (t, &wt) in w.iter().enumerate() {
                let s = apron[row + i + t];
                acc[0] += s[0] * wt;
                acc[1] += s[1] * wt;
                acc[2] += s[2] * wt;
                acc[3] += s[3] * wt;
            }
            hbuf[j * bw + i] = acc;
        }
    }
    // Vertical pass: hbuf (bw × ap_h) → out (bw × bh), then un-premultiply to straight RGBA.
    let mut out = vec![[0f32; 4]; bw * bh];
    for j in 0..bh {
        for i in 0..bw {
            let mut acc = [0f32; 4];
            for (t, &wt) in w.iter().enumerate() {
                let s = hbuf[(j + t) * bw + i];
                acc[0] += s[0] * wt;
                acc[1] += s[1] * wt;
                acc[2] += s[2] * wt;
                acc[3] += s[3] * wt;
            }
            let a = acc[3];
            let inv = if a > 1e-4 { 255.0 / a } else { 0.0 };
            out[j * bw + i] = [acc[0] * inv, acc[1] * inv, acc[2] * inv, a];
        }
    }
    out
}

/// Blend the blurred region into `buf` over the footprint bbox, weighting each pixel by `weight(i, j)`
/// (the dab mask × strength; `0` skips). `dest = lerp(dest, blurred, w)` per straight channel — the
/// `IMB_BLEND_INTERPOLATE` composite Blender's soften uses. Shared with the canvas-fixed Grain path
/// ([`crate::blur_grain`]), which supplies a per-pixel silhouette × Grain weight closure.
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) fn blend_blurred(
    buf: &mut [u8],
    fw: i64,
    min_x: i64,
    min_y: i64,
    bw: usize,
    bh: usize,
    blurred: &[[f32; 4]],
    mut weight: impl FnMut(usize, usize) -> f32,
) {
    for j in 0..bh {
        let py = min_y + j as i64;
        for i in 0..bw {
            let w = weight(i, j);
            if w <= 0.0 {
                continue;
            }
            let px = min_x + i as i64;
            let src = blurred[j * bw + i];
            let di = ((py * fw + px) * 4) as usize;
            for c in 0..4 {
                let d = f32::from(buf[di + c]);
                buf[di + c] = (d + (src[c] - d) * w).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// The footprint bbox at `center` (radius `radius`), clamped half-open to the canvas. `pad` extends the
/// high edge (`+1`) for mask sampling. `None` when fully off-canvas. Shared with [`crate::blur_grain`].
pub(crate) fn footprint_bbox(
    center: [f32; 2],
    radius: f32,
    fw: i64,
    fh: i64,
    pad: i64,
) -> Option<(i64, i64, usize, usize)> {
    let min_x = (center[0] - radius).floor().max(0.0) as i64;
    let min_y = (center[1] - radius).floor().max(0.0) as i64;
    let max_x = ((center[0] + radius).ceil() as i64 + pad).min(fw);
    let max_y = ((center[1] + radius).ceil() as i64 + pad).min(fh);
    if max_x <= min_x || max_y <= min_y {
        return None;
    }
    Some((
        min_x,
        min_y,
        (max_x - min_x) as usize,
        (max_y - min_y) as usize,
    ))
}

/// Blur one dab centred at `center`, weighted by the brush falloff (with `spec.hardness`) × `strength`.
/// The round-falloff path (no Shape / Grain / flatten) — the [`crate::smear_dab`] sibling for softening.
///
/// `buf` is straight-alpha RGBA8, `width · height · 4` bytes. `strength` in `[0, 1]` is the blend-back
/// amount (`0` = untouched, `1` = fully blurred at the dab centre). `wrap` (Tiling) makes the
/// neighbourhood read toroidal per axis, so blurring across a tiled seam pulls the opposite edge in
/// instead of the border. Returns `None` for zero strength/radius or a fully off-canvas footprint.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn blur_dab(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    strength: f32,
    wrap: [bool; 2],
) -> Option<DirtyRect> {
    let radius = spec.clamped_radius();
    let strength = strength.clamp(0.0, 1.0);
    if strength <= 0.0 || radius <= 0.0 {
        return None;
    }
    let (fw, fh) = (width as i64, height as i64);
    let (min_x, min_y, bw, bh) = footprint_bbox(center, radius, fw, fh, 0)?;
    let k = kernel_radius(radius);
    let blurred = blur_region(buf, fw, fh, min_x, min_y, bw, bh, k, wrap);
    let inv_r = 1.0 / radius;
    blend_blurred(buf, fw, min_x, min_y, bw, bh, &blurred, |i, j| {
        let dx = (min_x + i as i64) as f32 + 0.5 - center[0];
        let dy = (min_y + j as i64) as f32 + 0.5 - center[1];
        let t = (dx * dx + dy * dy).sqrt() * inv_r;
        if t >= 1.0 {
            0.0
        } else {
            spec.falloff_weight(t) * strength
        }
    });
    Some(DirtyRect {
        x: min_x as u32,
        y: min_y as u32,
        w: bw as u32,
        h: bh as u32,
    })
}

/// Blur one dab using a pre-rendered [`StampMask`] as the per-pixel weight — so the brush **Shape**
/// (silhouette), **Grain**, and **dab flatten/rotate** all shape the blur exactly as they shape a
/// painted dab (the mask bakes silhouette × Grain × flatten/rotate). The [`crate::blit_stamp`] sibling
/// for softening: `radius` is the dab radius (the mask spans `[-1, 1]²`), `strength` scales the
/// blend-back. Reads the neighbourhood first, so overlapping read/write never feeds back.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn blur_blit_stamp(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    radius: f32,
    mask: &StampMask,
    strength: f32,
    wrap: [bool; 2],
) -> Option<DirtyRect> {
    let strength = strength.clamp(0.0, 1.0);
    if strength <= 0.0 || radius <= 0.0 {
        return None;
    }
    let (fw, fh) = (width as i64, height as i64);
    let (min_x, min_y, bw, bh) = footprint_bbox(center, radius, fw, fh, 1)?;
    let k = kernel_radius(radius);
    let blurred = blur_region(buf, fw, fh, min_x, min_y, bw, bh, k, wrap);
    let inv_r = 1.0 / radius;
    blend_blurred(buf, fw, min_x, min_y, bw, bh, &blurred, |i, j| {
        let u = ((min_x + i as i64) as f32 + 0.5 - center[0]) * inv_r;
        let v = ((min_y + j as i64) as f32 + 0.5 - center[1]) * inv_r;
        sample_mask(mask, u, v) * strength
    });
    Some(DirtyRect {
        x: min_x as u32,
        y: min_y as u32,
        w: bw as u32,
        h: bh as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn px(buf: &[u8], w: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * w + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    /// A hard black|white vertical edge softens into a ramp: the column just inside the white side
    /// loses brightness (the black neighbour is averaged in), proving the neighbourhood convolves.
    #[test]
    fn softens_a_hard_edge() {
        let (w, h) = (32u32, 16u32);
        let mut buf = vec![255u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..16 {
                let i = ((y * w + x) * 4) as usize;
                buf[i..i + 3].copy_from_slice(&[0, 0, 0]); // left half black, opaque
            }
        }
        let spec = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            ..Default::default()
        };
        let before = px(&buf, w, 16, 8); // first white column
        let dirty = blur_dab(&mut buf, w, h, [16.0, 8.0], &spec, 1.0, [false, false])
            .expect("in-bounds blur paints");
        let after = px(&buf, w, 16, 8);
        assert!(
            after[0] < before[0] && after[0] > 0,
            "the edge softened toward grey: {before:?} → {after:?}"
        );
        assert_eq!(after[3], 255, "opaque canvas stays opaque");
        assert!(dirty.w > 0 && dirty.h > 0);
    }

    /// Strength is the blend-back amount: a higher strength moves the pixel further from the original
    /// toward the fully-blurred value.
    #[test]
    fn strength_scales_the_blend() {
        let make = || {
            let (w, h) = (32u32, 16u32);
            let mut buf = vec![255u8; (w * h * 4) as usize];
            for y in 0..h {
                for x in 0..16 {
                    let i = ((y * w + x) * 4) as usize;
                    buf[i..i + 3].copy_from_slice(&[0, 0, 0]);
                }
            }
            (w, h, buf)
        };
        let spec = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            ..Default::default()
        };
        let (w, h, mut lo) = make();
        let _ = blur_dab(&mut lo, w, h, [16.0, 8.0], &spec, 0.25, [false, false]);
        let (_, _, mut hi) = make();
        let _ = blur_dab(&mut hi, w, h, [16.0, 8.0], &spec, 1.0, [false, false]);
        // The white column starts at 255; the blurred value is lower, so more strength ⇒ lower.
        assert!(
            px(&hi, w, 16, 8)[0] < px(&lo, w, 16, 8)[0],
            "higher strength blurs more"
        );
    }

    #[test]
    fn fully_offscreen_is_none() {
        let (w, h) = (16u32, 16u32);
        let mut buf = vec![255u8; (w * h * 4) as usize];
        let spec = BrushSpec {
            radius_px: 3.0,
            ..Default::default()
        };
        assert!(blur_dab(&mut buf, w, h, [-50.0, -50.0], &spec, 1.0, [false, false]).is_none());
    }

    /// The masked path with a default round brush's StampMask (== its falloff) blurs like the round
    /// path — this is the path that also carries Shape / Grain / flatten via the baked mask.
    #[test]
    fn masked_blur_uses_the_stamp_mask() {
        let (w, h) = (32u32, 16u32);
        let mut buf = vec![255u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..16 {
                let i = ((y * w + x) * 4) as usize;
                buf[i..i + 3].copy_from_slice(&[0, 0, 0]);
            }
        }
        let spec = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            ..Default::default()
        };
        let mask = crate::render_stamp_mask(&spec, None, None, None, 64);
        let before = px(&buf, w, 16, 8);
        let dirty = blur_blit_stamp(&mut buf, w, h, [16.0, 8.0], 8.0, &mask, 1.0, [false, false])
            .expect("masked blur paints");
        assert!(
            px(&buf, w, 16, 8)[0] < before[0],
            "masked blur softened the edge"
        );
        assert!(dirty.w > 0 && dirty.h > 0);
    }

    /// Tiling: a blur at the left seam reads the OPPOSITE (right) edge instead of clamping, so a
    /// colour that differs across the seam bleeds through — proving the wrap is seamless.
    #[test]
    fn wrapping_blur_reads_across_the_seam() {
        // Left edge column red, everything else blue, all opaque. A blur at x=0 with wrap pulls the
        // right-edge blue across the seam; without wrap it only sees the local red/blue.
        let (w, h) = (16u32, 8u32);
        let mut buf = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                let c = if x == 0 { [255, 0, 0] } else { [0, 0, 255] };
                buf[i..i + 3].copy_from_slice(&c);
                buf[i + 3] = 255;
            }
        }
        let spec = BrushSpec {
            radius_px: 3.0,
            hardness: 1.0,
            ..Default::default()
        };
        let mut wrapped = buf.clone();
        let _ = blur_dab(&mut wrapped, w, h, [0.0, 4.0], &spec, 1.0, [true, false]);
        let mut plain = buf.clone();
        let _ = blur_dab(&mut plain, w, h, [0.0, 4.0], &spec, 1.0, [false, false]);
        // The seam pixel differs between wrap (sees far edge) and no-wrap (clamps the near edge).
        assert_ne!(
            px(&wrapped, w, 0, 4),
            px(&plain, w, 0, 4),
            "the wrapped blur reads across the seam, the clamped one doesn't"
        );
    }
}
