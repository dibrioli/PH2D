//! Structural UNDO / SNAPSHOT — capture the editable model, reinstall a
//! snapshot (a structural undo/redo) and record a transition. `impl PainterTool`
//! (one of several blocks in this crate). Split out of the former
//! `tool/layers.rs` god-file (pure move). NB: `crate::undo` is the undo
//! controller crate-module; this `tool::layers::undo` only holds the tool-side
//! glue and refers to it fully-qualified.

use super::super::*;

impl PainterTool {
    /// Capture the full editable model for transactional (structural) undo —
    /// see [`crate::undo::ModelSnapshot`]. `canvas_rgba` is `Arc`-shared (cheap);
    /// `images` deep-copies the non-active layers (a rare, user-paced cost).
    pub(crate) fn snapshot_model(&self) -> crate::undo::ModelSnapshot {
        let (mask_scratch, mask_scratch_target) = self.mask_scratch_for_snapshot();
        let (selection_mask, selection_active) = self.selection_for_snapshot();
        crate::undo::ModelSnapshot {
            layers: self.layers.clone(),
            images: self.images.clone(),
            canvas_rgba: Arc::clone(&self.canvas_rgba),
            selection: self.selection.clone(),
            // Layer ops carry no open shape / live preview; the shape paths override these via
            // `capture_shape_model` (see `tool::paint::shape_snapshot`).
            shape: None,
            offset_norm: self.shape_offset_norm(),
            offset_base_px: self.shape_offset_base_px(),
            preview_patch: None,
            mask_scratch,
            mask_scratch_target,
            selection_mask,
            selection_active,
        }
    }

    /// Reinstall a model snapshot (a structural undo/redo). Restores the layer
    /// tree (incl. the active target), the per-layer pixel store, the active
    /// working buffer, and the panel selection, then refreshes every derived
    /// cache so the composite + GPU preview rebuild.
    pub(crate) fn restore_model(&mut self, m: crate::undo::ModelSnapshot) {
        self.layers = m.layers;
        self.images = m.images;
        self.canvas_rgba = m.canvas_rgba;
        self.selection = m.selection;
        // Reinstate the Mask brush scratch + target so an undo/redo across a mask stroke restores the
        // live mask-in-progress in lock-step with the pixels (the composite rebuild below reads it).
        self.restore_mask_scratch(m.mask_scratch, m.mask_scratch_target);
        // Reinstate the Selection mask + active flag so an undo/redo across a selection edit restores the
        // selected region in lock-step with the pixels (ADR-0103).
        self.restore_selection(m.selection_mask, m.selection_active);
        self.set_shape_offset_norm(m.offset_norm);
        self.set_shape_offset_base_px(m.offset_base_px);
        // Reinstate (or clear) the open shape overlay: peel the snapshot canvas back to its pristine
        // baseline (strip the preview patch) and re-stamp the editor's geometry, so dots + pixels stay in
        // sync. A `None` shape just clears the editors. See `tool::paint::shape_snapshot`.
        self.restore_shape_overlay(m.shape, m.preview_patch);
        // Every layer's pixels may have changed identity → bump all content
        // versions so the GPU compositor re-uploads each slice, and drop the CPU
        // composite cache + bump the panel `layers_revision`.
        self.bump_all_layer_pixels();
        self.invalidate_composite();
        self.preview_dirty = true;
    }

    /// Record a STRUCTURAL transition from `before` (captured at the edit's start)
    /// to the CURRENT model, making the edit undoable in chronological order. Every
    /// structural site (add/delete/duplicate layer, mask/adjustment create, active
    /// switch) calls this with the model it snapshotted before mutating.
    pub(crate) fn commit_structural_edit(&mut self, before: crate::undo::ModelSnapshot) {
        let after = self.snapshot_model();
        self.undo.record_structural(before, after);
    }
}
