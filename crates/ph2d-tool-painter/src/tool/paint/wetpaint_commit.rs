//! Doc 21 — the Wet Paint **commit deposit**: the ONE door through which every
//! re-stamping method's final dab list enters the fluid, exactly once, at its
//! commit gesture (pen-up for DragDot/Anchored; Enter/Apply for the shape
//! editors; the method-switch bake). The authoring preview is flat and
//! UN-owned ([`PainterTool::wet_owns_the_dabs`]); [`PainterTool::stamp_drag_preview`]
//! stashes the exact batch it painted (`pending_deposit`), and this door peels
//! the flat pixels, re-arms the session guard, and replays the stash through
//! the full [`PainterTool::stamp_dabs`] dispatcher under `deposit_pass` — so
//! Tiling, `dab_groups` RNG identity, the wrapper bypass and the wet arm all
//! ride the existing code, and the deposit IS the preview by construction.
//! Sibling of `wetpaint.rs` / `wetpaint_settings.rs` (workspace file-LOC cap).

use super::*;

impl PainterTool {
    /// Doc 21 law D: the sim HOLDS while a flat authoring preview is live —
    /// DERIVED, no state, no release door. In Wet Paint `drag_preview` is
    /// always the flat re-stamp record (the watercolor previews require
    /// Paint mode; the incremental methods never set it), so `Some` here is
    /// exactly "the artist is authoring". The tick asks this after its
    /// guard: zero steps, zero composites, zero knob reconciles while held —
    /// the water (and its drying) FREEZES under the author's hand, which is
    /// the owner's model and also the only consistent one (a composite
    /// copies `sess.base` verbatim where pigment α = 0 and would erase the
    /// preview inside its dirty rect: a torn state).
    pub(super) fn wet_authoring_hold(&self) -> bool {
        matches!(self.paint.paint_mode, PaintMode::WetPaint) && self.paint.drag_preview.is_some()
    }

    /// Doc 21 law D: mark the write we JUST made to `canvas_rgba` as OURS, so
    /// the canvas-identity guard does not read an authoring re-stamp (or a
    /// peel) as a foreign mutation and kill the live water. The mirror of the
    /// composite's own re-arm (`wetpaint_composite`'s tail). Deliberately a
    /// no-op for the eraser: a flat erase IS a foreign edit to the water —
    /// the session must die at the next guard (W2.6 extended, the watercolor
    /// stance). Foreign swaps (undo, layer switch, fill) never call this and
    /// still die at the guard — that asymmetry is the entire design.
    pub(super) fn wetpaint_rearm_after_own_write(&mut self) {
        if !self.paint.eraser
            && let Some(sess) = self.paint.wetpaint.session.as_mut()
        {
            sess.canvas = Arc::clone(&self.canvas_rgba);
        }
    }

    /// The commit-deposit door (doc 21 law C). Order is load-bearing:
    /// **peel → re-arm → deposit** — without the re-arm the guard kills the
    /// live session and the deposit lazily creates a fresh one: visually
    /// similar, but nothing FUSES with the old water (the silent theft of
    /// the wet-on-wet differential, gate G4's mutation). The peel runs FIRST
    /// so the flat sketch is never inside the session's frozen base — paint
    /// never exists twice.
    pub(super) fn wetpaint_commit_deposit(&mut self) {
        let dabs = std::mem::take(&mut self.paint.wetpaint.pending_deposit);
        if dabs.is_empty() || self.paint.eraser {
            return;
        }
        // (1) Peel the flat preview back to pristine.
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        // (2) Our write — the live session survives it.
        self.wetpaint_rearm_after_own_write();
        // (3) The one replay. `deposit_pass` is written in exactly these two
        // adjacent statements around the single `stamp_dabs` call — nowhere
        // else in the codebase; a leak is I2 resurrected (doc 21 §F), so no
        // early return may ever be inserted between them.
        self.paint.wetpaint.deposit_pass = true;
        self.stamp_dabs(&dabs);
        self.paint.wetpaint.deposit_pass = false;
        // (4) The preview's relief dies with the preview (belt — the wet
        // mode lays none, gate G12) and the sculpt session with it.
        self.reset_stroke_height();
        self.drop_live_relief();
        self.end_sculpt_session();
        // (5) Close the engine stroke: lanes end, the sim resumes next tick
        // and the fresh pigment flows with the old water. Idempotent for
        // `paint_end`'s own later call.
        self.wetpaint_stroke_end();
        // (6) Return the cleared allocation to the stash slot.
        let mut dabs = dabs;
        dabs.clear();
        self.paint.wetpaint.pending_deposit = dabs;
    }
}

#[cfg(test)]
mod tests;
