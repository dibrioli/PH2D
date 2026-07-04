//! The single **inverse-warp kernel** — `out[dst] = bilinear(stroke_src, dst − Σ D(dst))`, backward-gather
//! so there are never holes. Shared by every Reshape sub-mode (only the [`DabField`] differs).
//!
//! **Anti-blur (per-stroke single resample).** Each dab does NOT re-gather the already-warped canvas
//! (which compounds bilinear softening into a smear). Instead it ACCUMULATES its displacement into a
//! per-stroke map (`deform.stroke_disp`) and re-renders the dab bbox from the frozen stroke-start pixels
//! (`deform.stroke_src`) using the TOTAL displacement — so the whole stroke costs exactly one resample per
//! texel and stays sharp. Cross-stroke, each stroke resamples the previous stroke's result once.

use super::field::DabField;
use crate::tool::PainterTool;
use std::sync::Arc;

impl PainterTool {
    /// Apply one Reshape dab of the given field at `center` (image px) with pixel `radius`. Accumulates the
    /// field into the per-stroke displacement map, then re-renders the dab bbox from `stroke_src`. No-op
    /// outside a sized canvas or before the stroke buffers exist. With `D = 0` everywhere the render
    /// resolves each `dst` to itself → **byte-identical**.
    pub(super) fn warp_apply_dab(&mut self, field: &DabField, center: [f32; 2], radius: f32) {
        let Some(bbox) = self.dab_bbox(center, radius) else {
            return;
        };
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if self.paint.deform.pre.len() != n * 4 || self.paint.deform.disp.len() != n {
            return; // session not set up (unsized canvas)
        }
        let freeze = self.deform_freeze_effective();
        let inverted = self.deform_freeze_inverted();

        // Pass 1 (immutable self): compute each texel's displacement contribution, scaled down where the
        // Freeze mask protects it (coverage → keep-original). Collected into a bbox-local buffer so the
        // mutable accumulate/render passes don't fight the `selection_coverage_at` borrow.
        let mut adds: Vec<[f32; 2]> = Vec::with_capacity((bbox.w * bbox.h) as usize);
        for ry in 0..bbox.h {
            let dy = bbox.y + ry;
            for rx in 0..bbox.w {
                let dx = bbox.x + rx;
                let mut d = field.at([dx as f32, dy as f32]);
                if freeze {
                    let mut keep = f32::from(self.selection_coverage_at(dx, dy)) / 255.0;
                    if inverted {
                        keep = 1.0 - keep;
                    }
                    let allow = 1.0 - keep;
                    d[0] *= allow;
                    d[1] *= allow;
                }
                adds.push(d);
            }
        }

        // Pass 2 (mutable): fold the contributions into the SESSION displacement map, then render the bbox
        // from the pristine `pre` using the TOTAL displacement (one resample per texel → sharp, no compound
        // blur, and cross-stroke it still samples the pristine source).
        let src = Arc::clone(&self.paint.deform.pre);
        let disp = Arc::make_mut(&mut self.paint.deform.disp);
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        for ry in 0..bbox.h {
            let dy = bbox.y + ry;
            for rx in 0..bbox.w {
                let dx = bbox.x + rx;
                let gi = (dy * w + dx) as usize;
                let a = adds[(ry * bbox.w + rx) as usize];
                let d = &mut disp[gi];
                d[0] += a[0];
                d[1] += a[1];
                let px = bilinear_clamped(&src, w, h, dx as f32 - d[0], dy as f32 - d[1]);
                let b = gi * 4;
                buf[b..b + 4].copy_from_slice(&px);
            }
        }
        self.mark_dirty(bbox);
    }
}

/// Bilinear-sample straight-alpha RGBA8 `canvas` (`w×h`) at fractional `(x, y)`, clamping to the edge
/// (extend, never wrap → no holes). Interpolates in **premultiplied** space then un-premultiplies, so a
/// transparent texel's (arbitrary) RGB never bleeds a dark fringe near an alpha edge — the speckle a
/// straight per-channel lerp produced. At integer coords the fractions are `0`, so it returns the exact
/// texel (identity-safe). Canvas is straight RGBA (the shell premultiplies on GPU upload).
#[inline]
pub(super) fn bilinear_clamped(canvas: &[u8], w: u32, h: u32, x: f32, y: f32) -> [u8; 4] {
    if w == 0 || h == 0 {
        return [0, 0, 0, 0];
    }
    let x0f = x.floor();
    let y0f = y.floor();
    let fx = x - x0f;
    let fy = y - y0f;
    let (wi, hi) = (w as i64, h as i64);
    let x0 = (x0f as i64).clamp(0, wi - 1);
    let y0 = (y0f as i64).clamp(0, hi - 1);
    let x1 = (x0 + 1).clamp(0, wi - 1);
    let y1 = (y0 + 1).clamp(0, hi - 1);
    let stride = w as usize * 4;
    // A corner as PREMULTIPLIED `[r·a, g·a, b·a, a]` (rgb on the `0..=255` scale, `a` raw).
    let corner = |xi: i64, yi: i64| -> [f32; 4] {
        let b = yi as usize * stride + xi as usize * 4;
        let a = f32::from(canvas[b + 3]);
        let s = a / 255.0;
        [
            f32::from(canvas[b]) * s,
            f32::from(canvas[b + 1]) * s,
            f32::from(canvas[b + 2]) * s,
            a,
        ]
    };
    let lerp = |p: [f32; 4], q: [f32; 4], t: f32| {
        [
            p[0] + (q[0] - p[0]) * t,
            p[1] + (q[1] - p[1]) * t,
            p[2] + (q[2] - p[2]) * t,
            p[3] + (q[3] - p[3]) * t,
        ]
    };
    let top = lerp(corner(x0, y0), corner(x1, y0), fx);
    let bot = lerp(corner(x0, y1), corner(x1, y1), fx);
    let m = lerp(top, bot, fy); // [premR, premG, premB, A]
    let a = m[3];
    if a <= 0.0 {
        return [0, 0, 0, 0];
    }
    let inv = 255.0 / a; // un-premultiply
    [
        (m[0] * inv).round().clamp(0.0, 255.0) as u8,
        (m[1] * inv).round().clamp(0.0, 255.0) as u8,
        (m[2] * inv).round().clamp(0.0, 255.0) as u8,
        a.round().clamp(0.0, 255.0) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bilinear_at_integer_coords_is_exact() {
        // 2×2 image, distinct pixels; sampling exactly on a texel returns it byte-for-byte (identity).
        let w = 2;
        let h = 2;
        let px = [
            10, 20, 30, 40, // (0,0)
            50, 60, 70, 80, // (1,0)
            90, 100, 110, 120, // (0,1)
            130, 140, 150, 160, // (1,1)
        ];
        assert_eq!(bilinear_clamped(&px, w, h, 0.0, 0.0), [10, 20, 30, 40]);
        assert_eq!(bilinear_clamped(&px, w, h, 1.0, 0.0), [50, 60, 70, 80]);
        assert_eq!(bilinear_clamped(&px, w, h, 1.0, 1.0), [130, 140, 150, 160]);
        // Out of bounds clamps to the edge texel.
        assert_eq!(bilinear_clamped(&px, w, h, -5.0, -5.0), [10, 20, 30, 40]);
        assert_eq!(bilinear_clamped(&px, w, h, 9.0, 9.0), [130, 140, 150, 160]);
    }

    #[test]
    fn bilinear_midpoint_of_opaque_is_the_average() {
        // Two OPAQUE texels: premultiplied == straight, so the midpoint is the plain average.
        let w = 2;
        let h = 1;
        let px = [0, 0, 0, 255, 100, 100, 100, 255];
        assert_eq!(bilinear_clamped(&px, w, h, 0.5, 0.0), [50, 50, 50, 255]);
    }

    #[test]
    fn bilinear_across_an_alpha_edge_has_no_dark_fringe() {
        // A fully-transparent texel with GARBAGE rgb (255,0,0) next to an opaque black one. A straight
        // per-channel lerp would bleed the red in; premultiplied resampling keeps the colour clean — the
        // midpoint alpha is 50% but the RGB is the opaque texel's, NOT muddied toward the transparent rgb.
        let w = 2;
        let h = 1;
        let px = [
            255, 0, 0, 0, /* transparent, junk rgb */
            0, 0, 0, 255, /* opaque black */
        ];
        let m = bilinear_clamped(&px, w, h, 0.5, 0.0);
        assert_eq!(m[3], 128, "alpha is the average (50%)");
        assert!(
            m[0] < 8 && m[1] < 8 && m[2] < 8,
            "no red fringe from the transparent texel: got {m:?}"
        );
    }
}
