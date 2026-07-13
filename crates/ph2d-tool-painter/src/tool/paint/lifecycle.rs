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

        // ── The three subsystems that were born AFTER this reset was written (2026-07-02) and never
        //    registered here. Each one keeps canvas-sized state that OUTLIVES the gesture, so each one
        //    walked straight onto the next sprite (sweep, 2026-07-12). Their own guards check only the
        //    LENGTH of their buffers — which MATCHES when the two sprites happen to be the same size, so
        //    "same-size sprites" is the case that corrupts in silence. See `BUGS_painter.md` #13.

        // The wet session: `canvas_wet` is the one canvas-sized buffer that survives pen-up (it dries on
        // the heartbeat, over ~10 s). Rebinding a BIGGER sprite inside that window made the next tick's
        // `dry_canvas_wet` slice the old, smaller map with the new sprite's stride → SIGSEGV. This is the
        // same teardown that undo/redo already runs for exactly this reason ("the canvas identity changed,
        // so the moisture map is stale") — a document rebind IS the canvas identity changing. It drops the
        // moisture map and the session buffers; the BAKED washes stay in `canvas_rgba` (no pixels touched).
        self.dry_session_now();

        // The Deform session: `deform.pre` is a "pristine" snapshot of the OLD sprite's pixels, validated
        // only by its LENGTH — so a same-size sprite kept it, and the next reshape resampled the new canvas
        // from the OLD sprite's pixels (the flood-the-new-sprite class this very reset was written to kill).
        self.end_deform_session();

        // The **Sculpt** session is the same hazard on the height channel, and the paragraph above is its
        // repro verbatim. `sculpt.pre` is the OLD sprite's relief, and the two things guarding it are the
        // two this reset exists because they do not hold across a rebind: the active-layer id (`LayerStack`
        // restarts `next_id` at 1, so the ids COLLIDE by construction) and a LENGTH check (which matches
        // whenever the two sprites are the same size — the case that corrupts in silence).
        //
        // A COMMITTED sculpt stroke ends its own session, so the surviving carrier is an **open shape**:
        // carve with a Curve, bind a same-size sprite B (the shape is discarded above, the session is not),
        // and — *without painting a thing* — drag the Radius slider. `refresh_live_sculpt` passes both
        // guards and writes A's frozen relief onto B's height plane. A pen-down on B would have cured it (it
        // opens a fresh session), so the hazard window is exactly "switch sprite, then touch the card": the
        // one order in which nothing warns you. And a knob edit records no undo entry, so there is no way
        // back. (`discard_open_shape` above drops the SHAPE; only this drops the session it was carving in.)
        self.end_sculpt_session();

        // The pixel Selection: tool-global (it is NOT in `StashedDoc`, which stashes the LAYER selection),
        // and `selection_restricts_paint()` asks only "is the mask non-empty?". So the new sprite silently
        // inherited the old one's selection and every stroke outside it was reverted — the "it just doesn't
        // paint and I don't know why" report. Cleared field-by-field rather than via `clear_selection()`:
        // that one records a structural UNDO entry, which must not land on the document being unbound.
        self.paint.selection_active = false;
        self.paint.selection_edit_mode = false;
        self.paint.selection_grab = None;
        self.paint.selection_mask = std::sync::Arc::new(Vec::new());
        self.paint.selection_crisp = std::sync::Arc::new(Vec::new());
        self.paint.selection_shapes.clear();
        self.reset_selection_offset();
    }
}
