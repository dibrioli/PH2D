//! **Sharp-corner Selection offset** (Enio 2026-07-05: "como você resolveu o offset das quinas para o
//! Stroke, faça o mesmo para Selection"). The former SDF grow/shrink rounded every corner BY CONSTRUCTION
//! (Euclidean dilation = Minkowski sum with a disc: a square grown by `d` gets radius-`d` corner arcs).
//! This module mirrors the STROKE pipeline instead:
//!
//! 1. Trace the pre-offset source crisp into closed contours — outer boundaries AND **holes**
//!    ([`super::selection_trace::trace_all_contours_with_holes`]; an SDF saw holes implicitly, a
//!    per-contour offset must see each boundary).
//! 2. **Refit** each contour ([`super::curve_refit::refit_closed_spine`]) — corner cusps are re-anchored
//!    on the true edge-line intersection, so the offset miters RAZOR corners exactly like the stroke.
//! 3. Calibrate each contour's GROW direction numerically (offset a copy by +2px, compare |signed area|)
//!    — traced windings vary, so the sign is measured, not assumed.
//! 4. Per offset level: CAD parallel-curve offset ([`super::curve_offset::offset_curve_refined`] —
//!    miter convex / split+Trim concave) of every contour — outers by `+level` (the selection grows),
//!    holes by `−level` (the hole SHRINKS) — then scanline-fill each and combine by **signed coverage**
//!    (outer +1, hole −1): a pixel is selected iff its count ≥ 1. Union/nesting semantics match a vector
//!    app's compound-path Offset Path.
//!
//! `level == 0` short-circuits to the source verbatim (byte-identical neutral). Pure geometry +
//! transcendental-free (HR-5); free fns + one `PainterTool` composer.

use super::PainterTool;
use super::{curve_geom, curve_offset, curve_refit, curve_trim};

/// Fit tolerance (px) for the traced source contours — same tightness as the Merge refit, so the
/// level-0 boundary stays within a pixel of the source mask.
const SOURCE_REFIT_TOL_PX: f32 = 1.0;
/// Probe distance (px) for the numeric grow-direction calibration.
const GROW_PROBE_PX: f32 = 2.0;

/// One boundary of the offset source: a corner-true closed curve + how to offset it.
pub(super) struct OffsetContour {
    pub points: Vec<[f32; 2]>,
    pub handles: Vec<[[f32; 2]; 2]>,
    /// `false` = outer boundary (fills +1), `true` = hole (fills −1, offsets opposite).
    pub is_hole: bool,
    /// Multiplier that makes `offset_curve_refined(…, level * grow, …)` GROW the enclosed region for
    /// positive `level` — measured per contour (traced windings vary).
    pub grow: f32,
}

/// Trace + refit + calibrate the source crisp into offset-ready contours. Empty when nothing is inside.
pub(super) fn build_offset_contours(crisp: &[u8], w: usize, h: usize) -> Vec<OffsetContour> {
    let mut out = Vec::new();
    for (mut contour, is_hole) in super::selection_trace::trace_all_contours_with_holes(crisp, w, h)
    {
        if contour.len() >= 2 && contour[0] == contour[contour.len() - 1] {
            contour.pop(); // the tracer closes the loop; the refit wants the open ring
        }
        let Some(r) = curve_refit::refit_closed_spine(&contour, SOURCE_REFIT_TOL_PX) else {
            continue; // sub-8-sample sliver — contributes nothing offsetable
        };
        let grow = measure_grow_sign(&r.points, &r.handles);
        out.push(OffsetContour {
            points: r.points,
            handles: r.handles,
            is_hole,
            grow,
        });
    }
    out
}

/// The offset sign that GROWS this contour's enclosed region: offset a probe copy by `+GROW_PROBE_PX`
/// and compare absolute signed areas. Transcendental-free (shoelace + the offset's own math).
fn measure_grow_sign(points: &[[f32; 2]], handles: &[[[f32; 2]; 2]]) -> f32 {
    let base = flat_area(points, handles, 0.0);
    let grown = flat_area(points, handles, GROW_PROBE_PX);
    if grown >= base { 1.0 } else { -1.0 }
}

/// |signed area| of the contour offset by `d` (shoelace over the flattened spine).
fn flat_area(points: &[[f32; 2]], handles: &[[[f32; 2]; 2]], d: f32) -> f32 {
    let (op, oh, _) = curve_offset::offset_curve_refined(points, handles, d, true);
    let mut spine = Vec::new();
    curve_geom::flatten_spine(&op, &oh, true, &mut spine);
    let n = spine.len();
    if n < 3 {
        return 0.0;
    }
    let mut a = 0.0f32;
    for i in 0..n {
        let p = spine[i];
        let q = spine[(i + 1) % n];
        a += p[0] * q[1] - q[0] * p[1];
    }
    (a * 0.5).abs()
}

impl PainterTool {
    /// The effective selection mask at offset `level` px, composed from the corner-true contours: each
    /// contour CAD-offset (outers `+level`, holes `−level`), trimmed, scanline-filled, and combined by
    /// signed coverage (≥ 1 selected). `level == 0` returns the source verbatim (byte-identical neutral).
    pub(super) fn selection_offset_mask_at(&self, level: f32) -> Vec<u8> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let n = w * h;
        let src = &self.paint.selection_offset_source;
        if level == 0.0 || self.paint.selection_offset_curves.is_empty() {
            return if src.len() == n {
                (**src).clone()
            } else {
                vec![0u8; n]
            };
        }
        let mut count = vec![0i16; n];
        for c in self.paint.selection_offset_curves.iter() {
            // Outers grow the SELECTION by `level`; a hole's enclosed region shrinks by it.
            let d = level * c.grow * if c.is_hole { -1.0 } else { 1.0 };
            let (op, oh, _) = curve_offset::offset_curve_refined(&c.points, &c.handles, d, true);
            let mut spine = Vec::new();
            curve_geom::flatten_spine(&op, &oh, true, &mut spine);
            if spine.len() < 3 {
                continue; // the contour collapsed at this level (fully eroded)
            }
            // SIGNED-WINDING fill: a deep offset folds ears at concave splits, and no iterative trim can
            // reliably peel them (a compound of main+ears fools any area ranking). The winding rule is
            // exact instead: the MAIN region winds WITH the source orientation, every folded ear winds
            // AGAINST it (or cancels) — fill where the winding sign matches, no trim at all.
            let src_positive = curve_trim::loop_sign_positive(&c.points);
            let fill = fill_winding_signed(&spine, src_positive, w, h);
            let sign: i16 = if c.is_hole { -1 } else { 1 };
            for (acc, &f) in count.iter_mut().zip(fill.iter()) {
                if f >= 128 {
                    *acc += sign;
                }
            }
        }
        count
            .into_iter()
            .map(|c| if c >= 1 { 255 } else { 0 })
            .collect()
    }
}

/// Scanline NONZERO-WINDING fill of a closed polyline keeping only the pixels whose winding SIGN matches
/// the source loop's orientation (`src_positive` = its shoelace sign; in the y-DOWN image space a
/// shoelace-positive loop reads `wind < 0` under the down=+1 crossing convention): the exact region of a
/// deeply-offset loop — the main region winds once WITH the source, folded ears wind AGAINST it. Pixel
/// centres (`x+0.5, y+0.5`), directional crossings (down `+1`, up `−1`). Transcendental-free.
fn fill_winding_signed(spine: &[[f32; 2]], src_positive: bool, w: usize, h: usize) -> Vec<u8> {
    let mut out = vec![0u8; w * h];
    let n = spine.len();
    if n < 3 {
        return out;
    }
    let mut xs: Vec<(f32, i32)> = Vec::new();
    for y in 0..h {
        let sy = y as f32 + 0.5;
        xs.clear();
        for i in 0..n {
            let a = spine[i];
            let b = spine[(i + 1) % n];
            if (a[1] <= sy) != (b[1] <= sy) {
                let t = (sy - a[1]) / (b[1] - a[1]);
                xs.push((a[0] + t * (b[0] - a[0]), if b[1] > a[1] { 1 } else { -1 }));
            }
        }
        if xs.is_empty() {
            continue;
        }
        xs.sort_by(|p, q| p.0.total_cmp(&q.0));
        let mut wind = 0i32;
        let mut k = 0usize;
        for x in 0..w {
            let sx = x as f32 + 0.5;
            while k < xs.len() && xs[k].0 <= sx {
                wind += xs[k].1;
                k += 1;
            }
            let inside = if src_positive { wind < 0 } else { wind > 0 };
            if inside {
                out[y * w + x] = 255;
            }
        }
    }
    out
}
