//! The **Selection** tool (ADR-0103) — a document-wide selection mask that gates every paint op to the
//! selected region, integrated into the painter's single interleaved undo/redo queue.
//!
//! The mask is a single-channel coverage buffer (`w*h` bytes, `0` = outside / `255` = inside; Feather
//! softens the edge), held in [`super::PaintState`]. It is the SELECTION analogue of the Mask brush's
//! `mask_scratch` (see [`super::mask`]): captured into / restored from the `ModelSnapshot` in lock-step
//! with the pixels, and applied at stamp time by reverting texels OUTSIDE the selection to their
//! pre-stamp values ([`Self::restore_deselected_region`], mirror of `restore_protected_region`).
//!
//! This file holds the CORE: state accessors, snapshot/restore, the paint gate, and the Feather / Edit-flag
//! / overlay-opacity view setters. The creation gestures live in [`super::selection_input`], rasterization
//! in [`super::selection_raster`], the overlay + panel routing in [`super::selection_overlay`], Edit mode in
//! [`super::selection_edit`], the shape-list model in [`super::selection_shapes`], and Wave-5 actions in
//! [`super::selection_actions`].

use super::{PainterTool, Region};
use std::sync::Arc;

impl PainterTool {
    /// `true` while a selection is live (even an empty one, which paints nothing).
    #[must_use]
    pub fn selection_active(&self) -> bool {
        self.paint.selection_active
    }

    /// Selection coverage (`0..=255`) at image texel `(x, y)`; `0` when out of bounds / no selection. Read
    /// by the overlay (marching ants / hatching) and tests.
    #[must_use]
    pub fn selection_coverage_at(&self, x: u32, y: u32) -> u8 {
        let w = self.source_size.0;
        if w == 0 {
            return 0;
        }
        self.paint
            .selection_mask
            .get((y * w + x) as usize)
            .copied()
            .unwrap_or(0)
    }

    /// Snapshot the selection state for the undo model — the buffers live in `PaintState` (private to
    /// `tool::paint`), so the general `snapshot_model` reaches them through here. `Arc`-shared, cheap clone.
    pub(crate) fn selection_for_snapshot(&self) -> (Arc<Vec<u8>>, bool, Arc<Vec<u8>>, f32) {
        (
            Arc::clone(&self.paint.selection_mask),
            self.paint.selection_active,
            Arc::clone(&self.paint.selection_crisp),
            self.paint.selection_feather,
        )
    }

    /// Reinstate the selection state from an undo model (structural undo/redo), keeping the selected region
    /// (effective + crisp accumulator + Feather amount) in sync with the restored layers/pixels.
    pub(crate) fn restore_selection(
        &mut self,
        mask: Arc<Vec<u8>>,
        active: bool,
        crisp: Arc<Vec<u8>>,
        feather: f32,
    ) {
        self.paint.selection_mask = mask;
        self.paint.selection_active = active;
        self.paint.selection_crisp = crisp;
        self.paint.selection_feather = feather;
        // The on-canvas overlay (marching ants / hatching) is derived from this, so refresh.
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

    /// Replace the selection with a filled rectangle in image px (fully inside = 255). Records ONE structural
    /// undo entry so it joins the single interleaved queue. Seeds the shape list with one Rect entry.
    pub fn set_rect_selection(&mut self, x: u32, y: u32, rw: u32, rh: u32) {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let before = self.snapshot_model();
        let mut crisp = vec![0u8; (w as usize) * (h as usize)];
        let x1 = (x + rw).min(w);
        let y1 = (y + rh).min(h);
        for yy in y.min(h)..y1 {
            for xx in x.min(w)..x1 {
                crisp[(yy * w + xx) as usize] = 255;
            }
        }
        self.paint.selection_shapes = vec![super::selection_shapes::SelectionEntry {
            shape: super::selection_shapes::SelectionShape::Rect {
                a: [x as f32, y as f32],
                b: [x1 as f32, y1 as f32],
            },
            op: 0,
        }];
        self.set_selection_from_crisp(crisp);
        self.commit_structural_edit(before);
    }

    /// **Clear** (deselect) — no active selection, painting unrestricted again. Records one structural undo
    /// entry (no-op when there is nothing selected). Also drops the shape list + any Edit session.
    pub fn clear_selection(&mut self) {
        if !self.paint.selection_active {
            return;
        }
        let before = self.snapshot_model();
        if self.paint.selection_edit_mode {
            self.paint.selection_edit_mode = false;
            self.discard_open_shape();
            if let Some(m) = self.paint.selection_edit_saved_method.take() {
                self.paint.brush.stroke_method = m;
            }
            self.paint.selection_edit_idx = None;
        }
        self.paint.selection_active = false;
        self.paint.selection_mask = Arc::new(Vec::new());
        self.paint.selection_crisp = Arc::new(Vec::new());
        self.paint.selection_shapes.clear();
        self.invalidate_composite();
        self.commit_structural_edit(before);
    }

    /// Set the **Feather** amount (`0..1`) and re-derive the effective mask from the crisp accumulator.
    pub fn set_selection_feather(&mut self, f: f32) {
        self.paint.selection_feather = f.clamp(0.0, 1.0);
        self.derive_effective();
    }
    /// The Feather amount (`0..1`).
    #[must_use]
    pub fn selection_feather(&self) -> f32 {
        self.paint.selection_feather
    }

    /// `true` while the selection boundary is in editable-Shape mode.
    #[must_use]
    pub fn selection_edit_mode(&self) -> bool {
        self.paint.selection_edit_mode
    }

    /// Set the selection-overlay hatch opacity (`0..1`, a view preference).
    pub fn set_selection_overlay_opacity(&mut self, o: f32) {
        self.paint.selection_overlay_opacity = o.clamp(0.0, 1.0);
        self.invalidate_composite();
    }
    /// The selection-overlay hatch opacity (`0..1`).
    #[must_use]
    pub fn selection_overlay_opacity(&self) -> f32 {
        self.paint.selection_overlay_opacity
    }

    /// Clip `canvas_rgba` to the active selection: revert each texel toward `orig` by `1 - coverage`, so an
    /// edit that wrote outside the selection is undone (feathered edges blend). No-op without a selection.
    /// Used by paths that bypass the per-dab stamp gate — the Fill flood and (via the scratch) the Mask.
    pub(super) fn clip_canvas_to_selection(&mut self, orig: &[u8]) {
        if !self.selection_restricts_paint() {
            return;
        }
        let mask = Arc::clone(&self.paint.selection_mask);
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let n = mask.len().min(buf.len() / 4).min(orig.len() / 4);
        for i in 0..n {
            let keep = f32::from(mask[i]) / 255.0;
            if keep >= 1.0 {
                continue;
            }
            let b = i * 4;
            for c in 0..4 {
                let painted = f32::from(buf[b + c]);
                let o = f32::from(orig[b + c]);
                buf[b + c] = (painted * keep + o * (1.0 - keep)).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}
