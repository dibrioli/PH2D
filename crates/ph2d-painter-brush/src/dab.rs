//! Stamp one brush dab into an RGBA8 layer buffer.
//!
//! Behavioural reference (clean-room, no code copied): Blender
//! `editors/sculpt_paint/mesh/paint_image_2d.cc` (the soft 2D brush that walks the dab's bounding
//! box, weights each texel by the falloff mask, and blends) + `paint_image_2d_curve_mask.cc` (the
//! per-texel falloff weight). Distance is measured from the dab centre to each **pixel centre**
//! (`px + 0.5`), matching the texel-centre convention.

use crate::spec::BrushSpec;
use crate::texture::TexDabBasis;

/// Axis-aligned region of the buffer touched by a dab, in pixels (half-open: `[x, x+w)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Stamp one dab centred at `center` (image-space pixel coordinates) into `buf`.
///
/// `buf` is straight-alpha RGBA8, row-major, `width * height * 4` bytes, in the layer's native
/// space. `coverage` is the dab's overall opacity in `[0, 1]` — the stroke engine folds pressure
/// dynamics and the per-stroke strength cap into it; the brush's [`BrushSpec::flow`] and falloff
/// are applied here. Returns the touched [`DirtyRect`], or `None` if the dab is fully off-canvas
/// or has zero coverage.
///
/// When `preserve_alpha` is set ("alpha lock" / "preserve transparency"), each pixel's coverage is
/// scaled by the destination's existing alpha, so paint only lands where the layer already has
/// opacity (it recolours the shape without growing it).
///
/// Panics in debug if `buf` is smaller than `width * height * 4`.
///
/// This is the texture-free fast path; it delegates to [`stamp_dab_textured`] with no texture.
#[must_use]
pub fn stamp_dab(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
) -> Option<DirtyRect> {
    stamp_dab_textured(
        buf,
        width,
        height,
        center,
        spec,
        coverage,
        preserve_alpha,
        None,
    )
}

/// As [`stamp_dab`], but the brush texture ([`BrushSpec::texture`]) modulates each texel's coverage
/// when `tex` is `Some` and a texture kind is assigned. `tex` is the per-dab texture frame resolved
/// once by [`crate::texture::dab_basis`] (rotation + per-dab random offset); passing `None` — or an
/// inactive texture — reproduces [`stamp_dab`] exactly. See [`crate::texture`].
// Mirrors [`stamp_dab`]'s established positional signature plus the per-dab texture frame; bundling
// the buffer/geometry into a struct here would diverge from the sibling fast path for no real gain.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn stamp_dab_textured(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
    tex: Option<&TexDabBasis>,
) -> Option<DirtyRect> {
    debug_assert!(
        buf.len() >= (width as usize) * (height as usize) * 4,
        "buffer too small"
    );
    // Per-dab opacity = the stroke's coverage × the brush's Flow (per-dab build-up)
    // × Strength (overall opacity). Both default to 1.0. (A true per-stroke
    // accumulation cap for Strength — the "Accumulate off" model — is a later
    // refinement; today it scales the dab, which composes with Flow.)
    let coverage =
        coverage.clamp(0.0, 1.0) * spec.flow.clamp(0.0, 1.0) * spec.strength.clamp(0.0, 1.0);
    if coverage <= 0.0 || width == 0 || height == 0 {
        return None;
    }
    let radius = spec.clamped_radius();
    let (cx, cy) = (center[0], center[1]);

    // Bounding box of the dab, clamped to the canvas (half-open on the max side).
    let x0 = (cx - radius).floor().max(0.0) as i64;
    let y0 = (cy - radius).floor().max(0.0) as i64;
    let x1 = ((cx + radius).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + radius).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }

    let inv_radius = 1.0 / radius;
    let color = spec.color;
    let blend = spec.blend;
    // Texture frame, only when a texture is actually assigned (else the per-pixel sample is skipped
    // entirely so the texture-free path costs nothing).
    let tex = tex.filter(|_| spec.texture.is_active());
    let mut touched = false;

    for py in y0..y1 {
        let dy = (py as f32 + 0.5) - cy;
        let row = (py as usize) * (width as usize) * 4;
        for px in x0..x1 {
            let dx = (px as f32 + 0.5) - cx;
            let t = (dx * dx + dy * dy).sqrt() * inv_radius;
            let mut w = spec.falloff_weight(t);
            // The brush texture multiplies the falloff mask, exactly as the falloff multiplies
            // coverage — so a texel value of 0 paints nothing, 1 paints the full falloff weight.
            if let Some(b) = tex {
                w *= crate::texture::sample(&spec.texture, b, px, py, center, radius);
            }
            if w <= 0.0 {
                continue;
            }
            let i = row + (px as usize) * 4;
            let dst = [
                f32::from(buf[i]) / 255.0,
                f32::from(buf[i + 1]) / 255.0,
                f32::from(buf[i + 2]) / 255.0,
                f32::from(buf[i + 3]) / 255.0,
            ];
            // Alpha lock: gate coverage by the destination's existing alpha so paint
            // recolours the shape without extending it.
            let a = if preserve_alpha {
                w * coverage * dst[3]
            } else {
                w * coverage
            };
            if a <= 0.0 {
                continue;
            }
            let out = crate::blend::blend_over(blend, dst, color, a);
            buf[i] = encode(out[0]);
            buf[i + 1] = encode(out[1]);
            buf[i + 2] = encode(out[2]);
            buf[i + 3] = encode(out[3]);
            touched = true;
        }
    }

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

#[inline]
fn encode(v: f32) -> u8 {
    // Round-to-nearest, clamped. (No gamma here — the buffer is already in its native space.)
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blend::BrushBlend;
    use crate::falloff::Falloff;

    fn solid(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
        rgba.iter()
            .copied()
            .cycle()
            .take((width * height * 4) as usize)
            .collect()
    }

    #[test]
    fn dab_paints_center_and_reports_rect() {
        let (w, h) = (32, 32);
        let mut buf = solid(w, h, [255, 255, 255, 255]); // white, opaque
        let spec = BrushSpec {
            radius_px: 6.0,
            color: [0.0, 0.0, 0.0], // black
            blend: BrushBlend::Mix,
            falloff: Falloff::Constant, // hard so the centre is fully painted
            ..Default::default()
        };
        let rect = stamp_dab(&mut buf, w, h, [16.0, 16.0], &spec, 1.0, false).expect("painted");
        // Centre pixel went black.
        let i = ((16 * w + 16) * 4) as usize;
        assert_eq!(&buf[i..i + 4], &[0, 0, 0, 255]);
        // Rect contains the centre and is within the canvas.
        assert!(rect.x <= 16 && rect.y <= 16);
        assert!(rect.x + rect.w <= w && rect.y + rect.h <= h);
        // A pixel well outside the radius is untouched (still white).
        let far = ((2 * w + 2) * 4) as usize;
        assert_eq!(&buf[far..far + 4], &[255, 255, 255, 255]);
    }

    #[test]
    fn dab_off_canvas_is_none() {
        let (w, h) = (16, 16);
        let mut buf = solid(w, h, [0, 0, 0, 0]);
        let spec = BrushSpec {
            radius_px: 3.0,
            ..Default::default()
        };
        assert!(stamp_dab(&mut buf, w, h, [-100.0, -100.0], &spec, 1.0, false).is_none());
    }

    #[test]
    fn zero_coverage_is_none() {
        let (w, h) = (16, 16);
        let mut buf = solid(w, h, [10, 20, 30, 255]);
        let spec = BrushSpec::default();
        assert!(stamp_dab(&mut buf, w, h, [8.0, 8.0], &spec, 0.0, false).is_none());
        // Buffer untouched.
        assert_eq!(buf[0..4], [10, 20, 30, 255]);
    }

    #[test]
    fn soft_falloff_fades_from_center_to_rim() {
        let (w, h) = (40, 40);
        let mut buf = solid(w, h, [255, 255, 255, 255]);
        let spec = BrushSpec {
            radius_px: 14.0,
            color: [0.0, 0.0, 0.0],
            blend: BrushBlend::Mix,
            falloff: Falloff::Smooth,
            ..Default::default()
        };
        stamp_dab(&mut buf, w, h, [20.0, 20.0], &spec, 1.0, false).expect("painted");
        let val = |x: u32, y: u32| buf[((y * w + x) * 4) as usize];
        // Centre darker than a point near the rim.
        assert!(
            val(20, 20) < val(20, 32),
            "centre should be darker than the rim"
        );
    }

    #[test]
    fn strength_scales_dab_opacity() {
        // Full strength → black; half strength → mid-grey over white (Mix at a=0.5).
        let (w, h) = (16, 16);
        let hard = |strength: f32| BrushSpec {
            radius_px: 5.0,
            color: [0.0, 0.0, 0.0],
            blend: BrushBlend::Mix,
            falloff: Falloff::Constant,
            hardness: 1.0,
            strength,
            ..Default::default()
        };
        let mut full = solid(w, h, [255, 255, 255, 255]);
        stamp_dab(&mut full, w, h, [8.0, 8.0], &hard(1.0), 1.0, false).expect("painted");
        assert_eq!(full[((8 * w + 8) * 4) as usize], 0, "full strength = black");

        let mut half = solid(w, h, [255, 255, 255, 255]);
        stamp_dab(&mut half, w, h, [8.0, 8.0], &hard(0.5), 1.0, false).expect("painted");
        let v = half[((8 * w + 8) * 4) as usize];
        assert!(
            (120..=136).contains(&v),
            "half strength ≈ mid-grey, got {v}"
        );
    }

    #[test]
    fn preserve_alpha_paints_only_into_existing_alpha() {
        let (w, h) = (8, 8);
        // Left half opaque white, right half fully transparent.
        let mut buf = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..4 {
                let i = ((y * w + x) * 4) as usize;
                buf[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
        let spec = BrushSpec {
            radius_px: 6.0,
            color: [0.0, 0.0, 0.0],
            blend: BrushBlend::Mix,
            falloff: Falloff::Constant,
            ..Default::default()
        };
        // Cover the whole buffer with alpha-lock on.
        stamp_dab(&mut buf, w, h, [4.0, 4.0], &spec, 1.0, true).expect("painted");
        let alpha = |x: u32, y: u32| buf[((y * w + x) * 4 + 3) as usize];
        let rgb = |x: u32, y: u32| {
            let i = ((y * w + x) * 4) as usize;
            [buf[i], buf[i + 1], buf[i + 2]]
        };
        // Opaque side recoloured to black, alpha preserved.
        assert_eq!(rgb(1, 4), [0, 0, 0], "opaque side recoloured");
        assert_eq!(alpha(1, 4), 255, "opaque alpha preserved");
        // Transparent side untouched — no alpha created.
        assert_eq!(alpha(6, 4), 0, "alpha-lock did not paint into transparency");
    }
}
