//! The **Selection** tool (ADR-0103) — a document-wide selection mask that gates every paint op to the
//! selected region, integrated into the painter's single interleaved undo/redo queue.
//!
//! The mask is a single-channel coverage buffer (`w*h` bytes, `0` = outside / `255` = inside; Feather
//! softens the edge), held in [`super::PaintState`]. It is the SELECTION analogue of the Mask brush's
//! `mask_scratch` (see [`super::mask`]): captured into / restored from the `ModelSnapshot` in lock-step
//! with the pixels, and applied at stamp time by reverting texels OUTSIDE the selection to their
//! pre-stamp values ([`Self::restore_deselected_region`], mirror of `restore_protected_region`).
//!
//! Wave 1 lays the state + undo + gate + a minimal rectangular seed. The on-canvas creation MODES
//! (Automatic / Freehand / Rectangle / Ellipse) and boolean operators build on this in Wave 2.

use super::{PainterTool, Region};
use std::sync::Arc;

impl PainterTool {
    /// `true` while a selection is live (even an empty one, which paints nothing).
    #[must_use]
    pub fn selection_active(&self) -> bool {
        self.paint.selection_active
    }

    /// Snapshot the selection mask + active flag for the undo model — the mask lives in `PaintState`
    /// (private to `tool::paint`), so the general `snapshot_model` reaches it through here. `Arc`-shared,
    /// so the clone is cheap.
    pub(crate) fn selection_for_snapshot(&self) -> (Arc<Vec<u8>>, bool) {
        (Arc::clone(&self.paint.selection_mask), self.paint.selection_active)
    }

    /// Reinstate the selection mask + active flag from an undo model (structural undo/redo), keeping the
    /// selected region in sync with the restored layers/pixels.
    pub(crate) fn restore_selection(&mut self, mask: Arc<Vec<u8>>, active: bool) {
        self.paint.selection_mask = mask;
        self.paint.selection_active = active;
        // The on-canvas overlay (marching ants / hatching, Wave 4) is derived from this, so refresh.
        self.invalidate_composite();
    }

    /// Whether an active selection should RESTRICT painting to its region right now. False when there is
    /// no selection (paint everywhere) or the buffer is unsized.
    pub(super) fn selection_restricts_paint(&self) -> bool {
        self.paint.selection_active && !self.paint.selection_mask.is_empty()
    }

    /// Restore the DESELECTED texels of `region` from `before` after a stamp: blend the pre-stamp pixel
    /// back by `1 - coverage`, so a fully-selected texel (coverage = 1) keeps the fresh paint and an
    /// unselected one (coverage = 0) reverts entirely. Mirror of [`Self::restore_protected_region`], but
    /// keyed on the selection coverage instead of the protection mask.
    pub(super) fn restore_deselected_region(&mut self, region: Region, before: &[u8]) {
        let (w, _h) = self.source_size;
        let mask = Arc::clone(&self.paint.selection_mask);
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let n = mask.len().min(buf.len() / 4);
        for ry in 0..region.h {
            for rx in 0..region.w {
                let gidx = ((region.y + ry) * w + (region.x + rx)) as usize;
                if gidx >= n {
                    continue;
                }
                let keep = f32::from(mask[gidx]) / 255.0; // 1 = inside (keep paint), 0 = outside (revert)
                if keep >= 1.0 {
                    continue; // fully selected → keep the fresh paint untouched
                }
                let b = gidx * 4;
                let s = ((ry * region.w + rx) * 4) as usize;
                for c in 0..4 {
                    let painted = f32::from(buf[b + c]);
                    let orig = f32::from(before[s + c]);
                    buf[b + c] = (painted * keep + orig * (1.0 - keep))
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
            }
        }
    }

    /// Ensure the selection buffer is sized to the current canvas (`w*h`, zero-filled = nothing selected).
    /// Re-allocates only when the size changed.
    pub(super) fn ensure_selection_mask(&mut self) {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let need = (w as usize) * (h as usize);
        if self.paint.selection_mask.len() != need {
            self.paint.selection_mask = Arc::new(vec![0u8; need]);
        }
    }

    /// **Wave 1 seed** — replace the selection with a filled rectangle in image px (fully inside = 255).
    /// Records ONE structural undo entry so it joins the single interleaved queue exactly like a brush
    /// stroke. Wave 2 layers the real modes + Add/Remove/Invert operators on top of this primitive.
    pub fn set_rect_selection(&mut self, x: u32, y: u32, rw: u32, rh: u32) {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let before = self.snapshot_model();
        self.ensure_selection_mask();
        let mask = Arc::make_mut(&mut self.paint.selection_mask);
        for v in mask.iter_mut() {
            *v = 0;
        }
        let x1 = (x + rw).min(w);
        let y1 = (y + rh).min(h);
        for yy in y.min(h)..y1 {
            for xx in x.min(w)..x1 {
                mask[(yy * w + xx) as usize] = 255;
            }
        }
        self.paint.selection_active = true;
        self.invalidate_composite();
        self.commit_structural_edit(before);
    }

    /// **Clear** (deselect) — no active selection, painting unrestricted again. Records one structural undo
    /// entry (no-op when there is nothing selected).
    pub fn clear_selection(&mut self) {
        if !self.paint.selection_active {
            return;
        }
        let before = self.snapshot_model();
        self.paint.selection_active = false;
        self.paint.selection_mask = Arc::new(Vec::new());
        self.invalidate_composite();
        self.commit_structural_edit(before);
    }
}
