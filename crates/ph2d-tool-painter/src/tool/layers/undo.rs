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
        let (selection_mask, selection_active, selection_crisp, selection_feather) =
            self.selection_for_snapshot();
        let deform = self.warp_for_snapshot();
        let sculpt = self.sculpt_for_snapshot();
        crate::undo::ModelSnapshot {
            layers: self.layers.clone(),
            images: self.images.clone(),
            heights: self.heights.clone(),
            covers: self.covers.clone(),
            mats: self.mats.clone(),
            canvas_rgba: Arc::clone(&self.canvas_rgba),
            selection: self.selection.clone(),
            // Layer ops carry no open shape / live preview; the shape paths override these via
            // `capture_shape_model` (see `tool::paint::shape_snapshot`).
            shape: None,
            offset_norm: self.shape_offset_norm(),
            offset_base_px: self.shape_offset_base_px(),
            preview_patch: None,
            // Layer ops carry no parked stroke shapes; the shape paths override this via `capture_shape_model`.
            parked_shapes: Vec::new(),
            // The ACTIVE shape's boolean op (captured always — cheap; makes the op-cycle tap undoable).
            active_op: self.active_op_wire(),
            mask_scratch,
            mask_scratch_target,
            selection_mask,
            selection_active,
            selection_crisp,
            selection_feather,
            // The parametric shape list is the source of truth the mask derives from — capture it so undo
            // restores the editable shapes (curve points/handles, box params), not just the raster mask.
            selection_shapes: self.selection_shapes_snapshot(),
            deform,
            sculpt,
        }
    }

    /// Reinstall a model snapshot (a structural undo/redo). Restores the layer
    /// tree (incl. the active target), the per-layer pixel store, the active
    /// working buffer, and the panel selection, then refreshes every derived
    /// cache so the composite + GPU preview rebuild.
    pub(crate) fn restore_model(&mut self, m: crate::undo::ModelSnapshot) {
        self.layers = m.layers;
        self.images = m.images;
        self.heights = m.heights;
        self.covers = m.covers;
        self.mats = m.mats;
        // The live-edit buffer describes a stroke on a ground that no longer exists — forget it, or the
        // next Depth drag would rebuild an undone stroke out of thin air.
        self.drop_live_relief();
        // And the in-progress ENVELOPE too. The shape editors reset it before each re-stamp, so when
        // the restored snapshot still carries a shape, `restore_shape_overlay` below rebuilds it in
        // lock-step with the pixels. But the LAST undo restores a snapshot with NO shape — that path
        // only clears the editors, and the previous re-stamp's envelope survived with no pigment
        // under it: the light kept shading a Line whose colour was gone (Enio, live smoke
        // 2026-07-15: "desfez a cor mas restou o relevo"). A gesture cannot be in flight during an
        // undo, so there is never a live envelope this reset could legitimately lose.
        self.reset_stroke_height();
        self.canvas_rgba = m.canvas_rgba;
        self.selection = m.selection;
        // Reinstate the Mask brush scratch + target so an undo/redo across a mask stroke restores the
        // live mask-in-progress in lock-step with the pixels (the composite rebuild below reads it).
        self.restore_mask_scratch(m.mask_scratch, m.mask_scratch_target);
        // Reinstate the parametric selection SHAPES (the source of truth) BEFORE the mask, so the editable
        // curve points/handles + box params roll back with the mask and the next recompose can't regenerate
        // the undone edit (Enio 2026-07-03).
        self.restore_selection_shapes(m.selection_shapes);
        // Reinstate the Selection mask + active flag so an undo/redo across a selection edit restores the
        // selected region in lock-step with the pixels (ADR-0103).
        self.restore_selection(
            m.selection_mask,
            m.selection_active,
            m.selection_crisp,
            m.selection_feather,
        );
        self.set_shape_offset_norm(m.offset_norm);
        self.set_shape_offset_base_px(m.offset_base_px);
        // Reinstate the ACTIVE shape's boolean op so undoing a centre-square op-cycle tap rolls it back.
        self.set_active_op_wire(m.active_op);
        // The **Sculpt** session — and it must land BEFORE the shape overlay below, which RE-STAMPS.
        //
        // The sculpt writes the layer's relief LIVE, so a snapshot captures an already-carved plane. Two
        // things therefore have to be true when that re-stamp runs, and only the second is obvious:
        //
        //   1. `heights` is the snapshot's (done above), and
        //   2. the SESSION is the snapshot's too — because the re-stamp restores the window from the
        //      session's frozen `pre` before re-accumulating.
        //
        // Restore the session *after* the re-stamp and (2) is false: the re-stamp finds no session, opens a
        // fresh one, and freezes the **carved** plane as its source — then runs the kernel on top of it.
        // Undo, redo, and the ridge has been smoothed twice; the pixels are perfect the whole time, so
        // nothing else in the system so much as blinks. (`docs/Painter/18…` §10.4; the same
        // source-of-truth-before-its-derivative ordering the selection shapes get, four lines up.)
        self.restore_sculpt(m.sculpt);
        // Reinstate (or clear) the open shape overlay: peel the snapshot canvas back to its pristine
        // baseline (strip the preview patch) and re-stamp the editor's geometry, so dots + pixels stay in
        // sync. A `None` shape just clears the editors. See `tool::paint::shape_snapshot`.
        self.restore_shape_overlay(m.shape, m.preview_patch, m.parked_shapes);
        // Every layer's pixels may have changed identity → bump all content
        // versions so the GPU compositor re-uploads each slice, and drop the CPU
        // composite cache + bump the panel `layers_revision`.
        // Reinstate the Deform session (displacement + `pre` + active) in lock-step with the pixels, so
        // undoing a deform stroke rolls the warp back AND keeps Reconstruct able to un-warp what remains.
        self.restore_deform(m.deform);
        self.bump_all_layer_pixels();
        self.invalidate_composite();
        // Watercolor: the canvas identity just changed under the wet session — its moisture map (+ frozen
        // base + union buffers) is now stale (the overlay showed the undone stroke's damp; the guard Arc no
        // longer matches). Tear it down so undo/redo clears the wetness (Enio 2026-07-11). No-op when dry.
        self.dry_session_now();
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

    /// [`Self::commit_structural_edit`] for a COALESCIBLE action (see [`crate::undo::CoalesceKind`]): a run
    /// of repeated same-kind actions collapses into one undo entry spanning first-before → latest-after.
    pub(crate) fn commit_structural_edit_coalesced(
        &mut self,
        kind: crate::undo::CoalesceKind,
        before: crate::undo::ModelSnapshot,
    ) {
        let after = self.snapshot_model();
        self.undo.record_structural_coalesced(kind, before, after);
    }
}
