//! Transient-edit lifecycle — the single "abandon everything in-progress" reset run at each document
//! (re)bind and on deactivate. A submodule of `paint` so it can reach `PaintState`'s private fields.
//!
//! **Why this exists (Enio 2026-07-02 bug):** binding a new sprite (`set_source` / `restore_doc`) swaps
//! the document — pixels, layers, undo — but historically left every *in-progress* canvas operation
//! untouched. So deleting a sprite, then selecting another, carried a pending Fill ColorDrop, an armed
//! Eyedropper, the Mask scratch, the Inpaint marks, and the Drag-Dot restore record straight onto the new
//! sprite. Symptoms: the pending Fill floods the new sprite BLACK (its `refill_from_snapshot` runs against
//! the new canvas with the brush colour, and even stamps the old sprite's snapshot over a same-size one);
//! the armed Eyedropper swallows the next Down so "nothing paints". This reset closes that whole class.

use crate::tool::PainterTool;

impl PainterTool {
    /// Abandon EVERY in-progress / pending canvas edit. Called at the start of each document (re)bind
    /// (`set_source` / `restore_doc`) and on `on_deactivate`, so switching or deleting a sprite never
    /// carries a half-finished operation onto the next one. Touches only transient state — the document
    /// (pixels / layers / undo / composite caches) is the caller's job.
    ///
    /// Note: the pending Fill is DROPPED, not committed — committing a stale ColorDrop onto the newly
    /// bound sprite is exactly the black-flood bug. The pixels being replaced by the caller, no restore
    /// is needed (unlike [`Self::fill_cancel`]).
    pub(crate) fn reset_transient_edit_state(&mut self) {
        self.discard_open_shape(); // Curve / Ellipse / Line / Polygon point editors
        self.abandon_pending_fill(); // Fill ColorDrop: fill_seed / fill_snapshot / fill_last_rect
        self.paint.stroke = None; // an in-progress brush stroke (mid-gesture rebind)
        self.paint.stroke_undo = None; // its pending undo snapshot (of the OLD document)
        self.paint.drag_preview = None; // Drag-Dot restore record — holds OLD-canvas pixels
        self.paint.inpaint_mask = Vec::new(); // Inpaint defect marks (rebuilt on the next paint_begin)
        self.paint.mask_scratch_rgba = std::sync::Arc::new(Vec::new()); // Mask scratch of the old sprite
        self.paint.mask_scratch_target = None;
        self.paint.eyedropper_armed = false; // don't leave a colour pick armed on the new sprite
    }
}
