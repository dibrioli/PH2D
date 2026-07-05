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
//! The geometry is the STROKE's parallel-curve offset applied to the mask boundary (Enio 2026-07-05 —
//! the former Euclidean-SDF grow/shrink ROUNDED every corner by construction; "como você resolveu o offset
//! das quinas para o Stroke, faça o mesmo para Selection"): the pre-offset crisp
//! (`selection_offset_source`) is traced into corner-true closed curves (outer boundaries + holes, refit
//! with razor cusps) and each offset level runs the CAD miter/split offset + Trim, composed by signed
//! coverage — see [`super::selection_offset_geom`]. Works for EVERY selection kind (the trace sees only
//! the mask). Per-level masks are cached; `norm == 0.5` in plain mode is byte-identical to the source.

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

    /// The cached effective mask at `level` (a frozen ring boundary or the live line), if this offset
    /// session computed it — the overlay draws the ring contours off these (read-only;
    /// [`Self::apply_selection_offset`] fills the cache for every level it composes).
    pub(super) fn selection_offset_level_mask(&self, level: f32) -> Option<&Arc<Vec<u8>>> {
        self.paint
            .selection_offset_level_cache
            .iter()
            .find(|(l, _)| *l == level)
            .map(|(_, m)| m)
    }

    /// Set the Offset slider (`0..1`) and re-derive the effective selection. A live preview — no undo entry
    /// (the whole drag folds into the single structural step committed by Apply / Apply & Keep, mirroring the
    /// stroke Offset). No-op without a live selection to offset.
    pub fn set_selection_offset(&mut self, norm: f32) {
        // (Re)capture the pre-offset source on ENGAGE (neutral → dragged): the crisp is pre-offset exactly
        // then, and a marquee gesture composes the crisp WITHOUT a recompose — a source captured earlier
        // (or only-when-empty) could be STALE, e.g. missing a freshly-subtracted hole (Enio 2026-07-05).
        // Mid-drag ticks and ring mode never refresh (their crisp is the OFFSET result — feedback).
        let neutral = !self.paint.selection_offset_active
            && (self.paint.selection_offset_norm - 0.5).abs() <= 1e-4;
        if neutral {
            let src = Arc::clone(&self.paint.selection_crisp);
            if src.is_empty() {
                return;
            }
            if *src != *self.paint.selection_offset_source {
                self.paint.selection_offset_source = src;
                self.paint.selection_offset_curves.clear();
                self.paint.selection_offset_level_cache.clear();
            }
        } else if self.paint.selection_offset_source.is_empty() {
            return;
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
        // Apply & Keep drops out of Edit Gizmos (Enio 2026-07-04): the frozen ring is shown by the offset
        // overlay; re-checking Edit Gizmos materialises every ring boundary into an editable curve.
        self.paint.selection_edit_mode = false;
        self.paint.selection_grab = None;
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
        self.paint.selection_offset_curves.clear();
        self.paint.selection_offset_level_cache.clear();
        self.paint.selection_ring_stack = false;
    }

    /// **Materialise** the active offset rings into editable Freehand curves — one per band boundary (level 0
    /// and each frozen ring) — so Edit Gizmos shows EVERY ring as an editable, gizmo-bearing curve that
    /// persists until Clear (Enio 2026-07-04). The intercalated fill is preserved by BAND-PARITY (see
    /// [`Self::recompose_ring_stack`]). Traces each level's contour off the cached SDF, fits it to a sparse
    /// curve, then switches to ring-stack mode. No-op without a live SDF.
    pub(super) fn materialise_offset_rings_to_curves(&mut self) {
        self.ensure_selection_offset_curves();
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let n = w * h;
        if n == 0 || self.paint.selection_offset_source.len() != n {
            return;
        }
        let mut levels = vec![0.0f32];
        levels.extend_from_slice(&self.paint.selection_offset_rings.clone());
        let mut entries = Vec::new();
        for &level in &levels {
            // The corner-true composite at this boundary → trace the outer contour → fit a sparse curve.
            let mask = self.offset_level_mask_cached(level);
            let outline = super::selection_trace::trace_selection_contour(&mask, w, h);
            if let Some(model) = super::selection_edit::to_closed_curve(&outline, &[]) {
                entries.push(SelectionEntry {
                    shape: SelectionShape::Freehand {
                        model,
                        u: [1.0, 0.0],
                    },
                    op: 0,
                });
            }
        }
        if entries.is_empty() {
            return;
        }
        self.paint.selection_shapes = entries;
        self.reset_selection_offset(); // the ring curves ARE the truth now — drop the SDF offset state
        self.paint.selection_ring_stack = true;
        self.recompose_selection_mask(); // band-parity composite of the nested curves
    }

    /// Recompose the effective mask in **ring-stack** mode: BAND-PARITY over the nested Freehand ring curves.
    /// A pixel is selected iff it is enclosed by `≡ n (mod 2)` of the `n` curves AND by at least one — so the
    /// innermost band (inside all `n`) is paint, each band outward flips, and outside every curve is deselected.
    /// This reproduces the SDF intercalation exactly while letting the user edit each ring curve's shape.
    pub(super) fn recompose_ring_stack(&mut self, n_pixels: usize) {
        let entries = self.paint.selection_shapes.clone();
        let n_curves = entries.len();
        let mut count = vec![0u16; n_pixels];
        for e in &entries {
            let region = self.rasterize_selection_shape(&e.shape);
            if region.len() != n_pixels {
                continue;
            }
            for (c, &r) in count.iter_mut().zip(region.iter()) {
                if r >= 128 {
                    *c += 1;
                }
            }
        }
        let mut crisp = vec![0u8; n_pixels];
        for (out, &c) in crisp.iter_mut().zip(count.iter()) {
            let c = c as usize;
            if c > 0 && (n_curves - c).is_multiple_of(2) {
                *out = 255;
            }
        }
        self.set_selection_offset_source(&crisp);
        self.apply_selection_offset(); // offset is neutral in ring-stack mode → installs `crisp` verbatim
    }

    /// `true` while an offset is actively in progress — ring mode (post-Apply & Keep) OR the slider is off
    /// its centred `0.5` rest. The source crisp is always populated after a recompose, so it can't signal
    /// this; the live offset is what Enter = Apply commits (Enter falls through to the stroke editor when the
    /// offset is at rest).
    #[must_use]
    pub fn selection_offset_engaged(&self) -> bool {
        self.paint.selection_active
            && (self.paint.selection_offset_active
                || (self.paint.selection_offset_norm - 0.5).abs() > 1e-4)
    }

    /// Refresh the pre-offset source (called by `recompose_selection_mask` after rebuilding the shapes' crisp)
    /// and clear the cached SDF. In ring mode the base is already baked into the shapes, so the recomposed
    /// crisp IS the correct offset source.
    pub(super) fn set_selection_offset_source(&mut self, crisp: &[u8]) {
        self.paint.selection_offset_source = Arc::new(crisp.to_vec());
        self.paint.selection_offset_curves.clear();
        self.paint.selection_offset_level_cache.clear();
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
        self.paint.selection_offset_curves.clear();
        self.paint.selection_offset_level_cache.clear();
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
        self.ensure_selection_offset_curves();
        let mut eff: Vec<u8>;
        if !self.paint.selection_offset_active {
            // Plain grow/shrink: the corner-true composite at the live level (miter corners; holes shrink).
            eff = (*self.offset_level_mask_cached(live)).clone();
        } else {
            // Ring mode: levels = [0, ring1, ring2, …]; band `k` (level[k-1]→level[k]) is PAINT iff `k` even
            // (band 0 = the interior, paint). Plus the LIVE band being swept at index `rings.len()+1`. Same
            // band-parity semantics as the former SDF, on nested corner-true level masks.
            let rings = self.paint.selection_offset_rings.clone();
            let mut levels = vec![0.0f32];
            levels.extend_from_slice(&rings);
            // Band 0 — the interior up to the first level: paint.
            eff = (*self.offset_level_mask_cached(levels[0])).clone();
            for k in 1..levels.len() {
                let paint = k.is_multiple_of(2);
                let (lo, hi) = ordered(levels[k - 1], levels[k]);
                let m_lo = self.offset_level_mask_cached(lo);
                let m_hi = self.offset_level_mask_cached(hi);
                paint_band_masks(&mut eff, &m_lo, &m_hi, paint);
            }
            // Live band: swept from the outermost frozen boundary by the current slider.
            let last = levels[levels.len() - 1];
            let live_idx = rings.len() + 1;
            let (lo, hi) = ordered(last, last + live);
            let m_lo = self.offset_level_mask_cached(lo);
            let m_hi = self.offset_level_mask_cached(hi);
            paint_band_masks(&mut eff, &m_lo, &m_hi, live_idx.is_multiple_of(2));
        }
        self.set_selection_from_crisp(eff);
    }

    /// Build the corner-true offset contours off the source crisp if empty — trace (+holes) → refit →
    /// grow-calibrate; see [`super::selection_offset_geom`]. The sharp analogue of the former SDF cache.
    fn ensure_selection_offset_curves(&mut self) {
        if !self.paint.selection_offset_curves.is_empty() {
            return;
        }
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let src = Arc::clone(&self.paint.selection_offset_source);
        if w == 0 || h == 0 || src.len() != w * h {
            return;
        }
        self.paint.selection_offset_curves =
            super::selection_offset_geom::build_offset_contours(&src, w, h);
    }

    /// The effective mask at `level`, cached per exact level: ring boundaries hit every recompose and the
    /// live level repeats while the slider rests. Pinned levels (0 + the frozen rings) never evict; the
    /// transient live levels keep only the most recent few. Dropped with the source.
    fn offset_level_mask_cached(&mut self, level: f32) -> Arc<Vec<u8>> {
        if let Some(m) = self.selection_offset_level_mask(level) {
            return Arc::clone(m);
        }
        let m = Arc::new(self.selection_offset_mask_at(level));
        self.paint
            .selection_offset_level_cache
            .push((level, Arc::clone(&m)));
        let rings = self.paint.selection_offset_rings.clone();
        let pinned = |l: f32| l == 0.0 || rings.contains(&l);
        let transient = self
            .paint
            .selection_offset_level_cache
            .iter()
            .filter(|(l, _)| !pinned(*l))
            .count();
        if transient > 3
            && let Some(pos) = self
                .paint
                .selection_offset_level_cache
                .iter()
                .position(|(l, _)| !pinned(*l))
        {
            self.paint.selection_offset_level_cache.remove(pos);
        }
        m
    }
}

/// `(min, max)` of two levels — the band the two boundaries enclose, whichever direction the sweep goes.
fn ordered(a: f32, b: f32) -> (f32, f32) {
    (a.min(b), a.max(b))
}

/// Set the shell between two NESTED level masks (`lo` ⊆ `hi`, both `0`/`255` coverage) to selected
/// (`paint`) or deselected — a pixel is in the band iff the hi-level mask covers it and the lo one doesn't.
fn paint_band_masks(eff: &mut [u8], lo: &[u8], hi: &[u8], paint: bool) {
    let v = if paint { 255 } else { 0 };
    let n = eff.len().min(lo.len()).min(hi.len());
    for i in 0..n {
        if hi[i] >= 128 && lo[i] < 128 {
            eff[i] = v;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_between_nested_masks_paints_and_unpaints_the_shell_only() {
        // lo ⊂ hi: the band is exactly hi \ lo; painting sets it, unpainting clears it, and pixels
        // outside the band are never touched.
        let lo = [0u8, 255, 0, 0];
        let hi = [0u8, 255, 255, 0];
        let mut eff = [9u8, 9, 9, 9];
        paint_band_masks(&mut eff, &lo, &hi, true);
        assert_eq!(eff, [9, 9, 255, 9], "only the shell pixel painted");
        paint_band_masks(&mut eff, &lo, &hi, false);
        assert_eq!(eff, [9, 9, 0, 9], "only the shell pixel cleared");
    }
}
