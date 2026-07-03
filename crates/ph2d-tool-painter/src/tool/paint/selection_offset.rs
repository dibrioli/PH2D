//! **Selection Offset** (ADR-0103 Amendment 3, Enio 2026-07-03) — grow/shrink the selection boundary with
//! the same parallel-offset idea as the stroke Offset, reusing the Apply / Apply & Keep verbs:
//!
//! * **Before the first Apply & Keep** the slider offsets the WHOLE selection (a plain grow/shrink), exactly
//!   like the stroke Offset moves the whole curve.
//! * **Apply & Keep** freezes the current offset line as a ring boundary and re-centres the slider, so a new
//!   sweep starts from that boundary. Each successive band ALTERNATES: the first frozen band is *protected*
//!   (deselected), the next *paint* (selected), the next protected … — concentric intercalated rings. Going
//!   the opposite way (inward) mirrors it (first band protected, then paint, …).
//! * **Apply** bakes the current (possibly ringed) selection as a plain `Raster` shape and leaves offset mode.
//!
//! The geometry is a **signed-distance offset** of the pre-offset crisp mask (`selection_offset_source`): a
//! band boundary at distance `d` px is `{ sdf <= d }` (dilate for `d>0`, erode for `d<0`). This is the mask
//! analogue of the stroke's parallel-curve offset and works for EVERY selection kind (ellipse / polygon /
//! freehand / composite / Automatic raster), not just a single curve. The SDF is an exact Euclidean distance
//! transform (Felzenszwalb–Huttenlocher, `sqrt`-only — HR-5-lean) cached off the source and thresholded per
//! frame. `norm == 0.5` in plain mode is byte-identical to the un-offset mask (`{sdf<=0}` == the inside set).

use super::PainterTool;
use super::selection_shapes::{SelectionEntry, SelectionShape};
use std::sync::Arc;

/// Largest offset (image px) the `0..1` Offset slider reaches at either extreme (`0` or `1`). Selections
/// span the document, so the range is wider than the stroke Offset's 100 px.
const SEL_OFFSET_MAX_PX: f32 = 200.0;

impl PainterTool {
    /// The Offset slider position (`0..1`, `0.5` = no offset) — read by the panel snapshot.
    #[must_use]
    pub fn selection_offset(&self) -> f32 {
        self.paint.selection_offset_norm
    }

    /// The slider's own contribution mapped to px: `(norm−0.5)·2·MAX`, `0` at the centred `0.5` track.
    fn selection_slider_offset_px(&self) -> f32 {
        (self.paint.selection_offset_norm - 0.5) * 2.0 * SEL_OFFSET_MAX_PX
    }

    /// The signed-distance levels (px from the base boundary) of EVERY offset line the overlay must draw
    /// explicitly while in ring mode: each frozen ring boundary PLUS the live line being swept. Empty in
    /// plain mode (there the offset IS the mask edge, already shown as marching ants). Drawing every frozen
    /// boundary keeps each selection line permanently visible — a protected band adds no selected pixels, so
    /// without this its edges vanish in the transition area once Apply & Keep re-centres the slider.
    #[must_use]
    pub(super) fn selection_offset_contour_levels(&self) -> Vec<f32> {
        if !self.paint.selection_offset_active {
            return Vec::new();
        }
        let mut levels = self.paint.selection_offset_rings.clone();
        let live = self.selection_slider_offset_px();
        if live != 0.0 {
            let last = self
                .paint
                .selection_offset_rings
                .last()
                .copied()
                .unwrap_or(0.0);
            levels.push(last + live);
        }
        levels
    }

    /// Read access to the cached SDF (may be empty when the offset is neutral). The overlay uses it to trace
    /// the live offset contour.
    pub(super) fn selection_offset_sdf(&self) -> &[f32] {
        &self.paint.selection_offset_sdf
    }

    /// Set the Offset slider (`0..1`) and re-derive the effective selection. A live preview — no undo entry
    /// (the whole drag folds into the single structural step committed by Apply / Apply & Keep, mirroring the
    /// stroke Offset). No-op without a live selection to offset.
    pub fn set_selection_offset(&mut self, norm: f32) {
        if self.paint.selection_offset_source.is_empty() {
            // Capture the current crisp as the offset source the first time the slider is touched (so a
            // freshly-drawn selection, whose mask came from a gesture rather than a recompose, can offset).
            let src = Arc::clone(&self.paint.selection_crisp);
            if src.is_empty() {
                return;
            }
            self.paint.selection_offset_source = src;
            self.paint.selection_offset_sdf = Arc::new(Vec::new());
        }
        self.paint.selection_offset_norm = norm.clamp(0.0, 1.0);
        self.apply_selection_offset();
    }

    /// **Apply & Keep**: freeze the current offset line and keep sweeping the next band. The first press
    /// (plain mode) bakes the grown/shrunk selection as the new base and enters ring mode; later presses push
    /// the live offset as a ring boundary. Re-centres the slider. One structural undo entry.
    pub fn selection_offset_apply_keep(&mut self) {
        if self.paint.selection_offset_source.is_empty() || !self.paint.selection_active {
            return;
        }
        let before = self.snapshot_model();
        let live = self.selection_slider_offset_px();
        if !self.paint.selection_offset_active {
            // Plain mode → bake the grown selection as the base (a Raster shape), then enter ring mode so the
            // NEXT sweep draws alternating bands off THIS boundary.
            self.bake_offset_into_shapes();
            self.paint.selection_offset_active = true;
            self.paint.selection_offset_rings.clear();
        } else {
            // Ring mode → push the live line as a frozen boundary (cumulative from the base boundary).
            let base = self
                .paint
                .selection_offset_rings
                .last()
                .copied()
                .unwrap_or(0.0);
            self.paint.selection_offset_rings.push(base + live);
        }
        self.paint.selection_offset_norm = 0.5;
        self.apply_selection_offset();
        self.commit_structural_edit(before);
    }

    /// **Apply**: bake the current (possibly ringed) selection as a plain `Raster` shape and leave offset
    /// mode (slider re-centred, rings dropped). One structural undo entry.
    pub fn selection_offset_apply(&mut self) {
        if self.paint.selection_offset_source.is_empty() || !self.paint.selection_active {
            return;
        }
        let before = self.snapshot_model();
        self.bake_offset_into_shapes();
        self.reset_selection_offset();
        self.recompose_selection_mask();
        self.commit_structural_edit(before);
    }

    /// Drop the offset state (slider centred, rings cleared, ring mode off, source/SDF released). Called by
    /// Apply and by any NEW selection gesture / Clear so a fresh selection starts from a clean offset.
    pub(super) fn reset_selection_offset(&mut self) {
        self.paint.selection_offset_norm = 0.5;
        self.paint.selection_offset_active = false;
        self.paint.selection_offset_rings.clear();
        self.paint.selection_offset_source = Arc::new(Vec::new());
        self.paint.selection_offset_sdf = Arc::new(Vec::new());
    }

    /// `true` while an offset is engaged (source captured) — the panel shows the Apply / Apply & Keep buttons
    /// as active and the offset is live.
    #[must_use]
    pub fn selection_offset_engaged(&self) -> bool {
        !self.paint.selection_offset_source.is_empty()
    }

    /// Refresh the pre-offset source (called by `recompose_selection_mask` after rebuilding the shapes' crisp)
    /// and clear the cached SDF. In ring mode the base is already baked into the shapes, so the recomposed
    /// crisp IS the correct offset source.
    pub(super) fn set_selection_offset_source(&mut self, crisp: &[u8]) {
        self.paint.selection_offset_source = Arc::new(crisp.to_vec());
        self.paint.selection_offset_sdf = Arc::new(Vec::new());
    }

    /// Bake the current effective selection (whatever the offset currently shows) as a single `Raster` entry,
    /// replacing the shape list AND becoming the new pre-offset source (so ring sweeps offset off THIS frozen
    /// boundary). Keeps the effective mask unchanged; clears the SDF cache so it recomputes off the new base.
    fn bake_offset_into_shapes(&mut self) {
        let crisp = Arc::clone(&self.paint.selection_crisp);
        self.paint.selection_shapes = vec![SelectionEntry {
            shape: SelectionShape::Raster {
                crisp: Arc::clone(&crisp),
            },
            op: 0,
        }];
        self.paint.selection_offset_source = crisp;
        self.paint.selection_offset_sdf = Arc::new(Vec::new());
    }

    /// Recompute the effective selection crisp from the source SDF + the ring stack + the live slider, then
    /// install it (Feather re-derives on top). The single funnel the offset drives. When the offset is
    /// neutral (plain mode, centred slider) this is byte-identical to the source.
    pub(super) fn apply_selection_offset(&mut self) {
        let n = (self.source_size.0 as usize) * (self.source_size.1 as usize);
        if n == 0 || self.paint.selection_offset_source.len() != n {
            return;
        }
        let live = self.selection_slider_offset_px();
        // Neutral fast path: plain mode + centred slider ⇒ the un-offset source verbatim (no SDF needed).
        if !self.paint.selection_offset_active && live == 0.0 {
            let src = (*self.paint.selection_offset_source).clone();
            self.set_selection_from_crisp(src);
            return;
        }
        self.ensure_selection_sdf();
        let sdf = Arc::clone(&self.paint.selection_offset_sdf);
        if sdf.len() != n {
            return;
        }
        let mut eff = vec![0u8; n];
        if !self.paint.selection_offset_active {
            // Plain grow/shrink: the whole selection is `{ sdf <= live }`.
            for i in 0..n {
                if sdf[i] <= live {
                    eff[i] = 255;
                }
            }
        } else {
            // Ring mode: levels = [0, ring1, ring2, …]; band `k` (level[k-1]→level[k]) is PAINT iff `k` even
            // (band 0 = the interior, paint). Plus the LIVE band being swept at index `rings.len()+1`.
            let rings = self.paint.selection_offset_rings.clone();
            let mut levels = vec![0.0f32];
            levels.extend_from_slice(&rings);
            // Band 0 — the interior up to the first level: paint.
            for i in 0..n {
                if sdf[i] <= levels[0] {
                    eff[i] = 255;
                }
            }
            for k in 1..levels.len() {
                let paint = k.is_multiple_of(2);
                paint_band(&mut eff, &sdf, levels[k - 1], levels[k], paint);
            }
            // Live band: swept from the outermost frozen boundary by the current slider.
            let last = levels[levels.len() - 1];
            let live_idx = rings.len() + 1;
            paint_band(
                &mut eff,
                &sdf,
                last,
                last + live,
                live_idx.is_multiple_of(2),
            );
        }
        self.set_selection_from_crisp(eff);
    }

    /// Fill the lazy SDF cache from the source crisp if empty — an exact Euclidean distance transform,
    /// signed (`<0` inside the source, `>0` outside).
    fn ensure_selection_sdf(&mut self) {
        if !self.paint.selection_offset_sdf.is_empty() {
            return;
        }
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let src = Arc::clone(&self.paint.selection_offset_source);
        if w == 0 || h == 0 || src.len() != w * h {
            return;
        }
        self.paint.selection_offset_sdf = Arc::new(signed_distance(&src, w, h));
    }
}

/// Set the shell between offset levels `lo` and `hi` (whichever order) to selected (`paint`) or deselected.
/// The shell is `{ sdf in (min(lo,hi), max(lo,hi)] }` — the ring the two boundaries enclose.
fn paint_band(eff: &mut [u8], sdf: &[f32], lo: f32, hi: f32, paint: bool) {
    let (a, b) = (lo.min(hi), lo.max(hi));
    let v = if paint { 255 } else { 0 };
    for i in 0..eff.len().min(sdf.len()) {
        if sdf[i] > a && sdf[i] <= b {
            eff[i] = v;
        }
    }
}

/// Signed Euclidean distance (image px) of a binary coverage mask (`>=128` = inside): `<0` inside, `>0`
/// outside, magnitude = distance to the boundary. Two exact distance transforms differenced. The minimum
/// magnitude is `1`, so `{ sdf <= 0 }` is exactly the inside set (offset `0` is a no-op).
fn signed_distance(cov: &[u8], w: usize, h: usize) -> Vec<f32> {
    let inside: Vec<bool> = cov.iter().map(|&c| c >= 128).collect();
    // Distance to the nearest INSIDE pixel (0 on inside) and to the nearest OUTSIDE pixel (0 on outside).
    let d_in = edt(&inside, w, h);
    let outside: Vec<bool> = inside.iter().map(|&b| !b).collect();
    let d_out = edt(&outside, w, h);
    (0..w * h).map(|i| d_in[i] - d_out[i]).collect()
}

/// Exact Euclidean distance transform: distance from every pixel to the nearest `feature` pixel (`0` at a
/// feature). Felzenszwalb–Huttenlocher (2012): the 1-D lower-envelope pass down columns then across rows on
/// squared distances, `sqrt` at the end. `O(w·h)`, transcendental-free bar the final `sqrt` (HR-5-lean).
fn edt(feature: &[bool], w: usize, h: usize) -> Vec<f32> {
    const INF: f32 = 1e20;
    // Column pass: squared distance within each column.
    let mut g = vec![0.0f32; w * h];
    let mut col = vec![0.0f32; h];
    for x in 0..w {
        for y in 0..h {
            col[y] = if feature[y * w + x] { 0.0 } else { INF };
        }
        let d = edt_1d(&col);
        for y in 0..h {
            g[y * w + x] = d[y];
        }
    }
    // Row pass: fold the column distances across each row.
    let mut out = vec![0.0f32; w * h];
    let mut row = vec![0.0f32; w];
    for y in 0..h {
        row.copy_from_slice(&g[y * w..y * w + w]);
        let d = edt_1d(&row);
        for x in 0..w {
            out[y * w + x] = d[x].max(0.0).sqrt();
        }
    }
    out
}

/// 1-D squared-distance transform of a cost sample `f` (Felzenszwalb–Huttenlocher lower-envelope of the
/// parabolas `(q - x)² + f[q]`). Returns the squared distance at each index.
fn edt_1d(f: &[f32]) -> Vec<f32> {
    let n = f.len();
    let mut d = vec![0.0f32; n];
    if n == 0 {
        return d;
    }
    let mut v = vec![0usize; n]; // parabola vertices in the lower envelope
    let mut z = vec![0.0f32; n + 1]; // envelope breakpoints
    let mut k = 0usize;
    v[0] = 0;
    z[0] = f32::NEG_INFINITY;
    z[1] = f32::INFINITY;
    for q in 1..n {
        // Intersection of the parabola at q with the current top parabola v[k].
        let mut s = intersect(f, q, v[k]);
        while s <= z[k] {
            k -= 1;
            s = intersect(f, q, v[k]);
        }
        k += 1;
        v[k] = q;
        z[k] = s;
        z[k + 1] = f32::INFINITY;
    }
    k = 0;
    for (q, out) in d.iter_mut().enumerate() {
        while z[k + 1] < q as f32 {
            k += 1;
        }
        let dq = q as f32 - v[k] as f32;
        *out = dq * dq + f[v[k]];
    }
    d
}

/// The x where the parabolas rooted at `q` and `r` (heights `f[q]`, `f[r]`) intersect — the envelope
/// breakpoint. `r < q`, so the denominator is positive.
fn intersect(f: &[f32], q: usize, r: usize) -> f32 {
    let (qf, rf) = (q as f32, r as f32);
    ((f[q] + qf * qf) - (f[r] + rf * rf)) / (2.0 * qf - 2.0 * rf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_distance_is_negative_inside_and_zero_offset_is_the_inside_set() {
        // 7×7 with a 3×3 filled centre. Inside pixels get sdf<0, outside >0, and {sdf<=0} == the inside set.
        let (w, h) = (7, 7);
        let mut cov = vec![0u8; w * h];
        for y in 2..5 {
            for x in 2..5 {
                cov[y * w + x] = 255;
            }
        }
        let sdf = signed_distance(&cov, w, h);
        for i in 0..w * h {
            let inside = cov[i] >= 128;
            assert_eq!(
                sdf[i] <= 0.0,
                inside,
                "pixel {i}: sdf {} vs inside {inside}",
                sdf[i]
            );
        }
    }

    #[test]
    fn dilation_grows_and_erosion_shrinks_the_area() {
        let (w, h) = (21, 21);
        let mut cov = vec![0u8; w * h];
        for y in 8..13 {
            for x in 8..13 {
                cov[y * w + x] = 255;
            }
        }
        let sdf = signed_distance(&cov, w, h);
        let base = cov.iter().filter(|&&c| c >= 128).count();
        let grown = sdf.iter().filter(|&&d| d <= 3.0).count();
        let shrunk = sdf.iter().filter(|&&d| d <= -1.5).count();
        assert!(grown > base, "dilate grows: {grown} > {base}");
        assert!(
            shrunk < base && shrunk > 0,
            "erode shrinks: {shrunk} < {base}"
        );
    }
}
