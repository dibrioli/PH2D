//! Timeline dope-sheet dispatch channel on [`WidgetStore`] (W2.E5b).
//!
//! Editor-core knows no timeline semantics: the pointer/wheel dispatch stashes
//! typed [`TimelineGesture`]s / [`TimelineWheel`]s here and the timeline panel
//! drains + interprets them each frame (which diamond was hit, drag-to-move,
//! clear-on-empty, anchored zoom + pan of the time axis). Lean mirror of the
//! graph-surface channel (`graph_ops`); keyboard (Delete/undo) is handled
//! shell-side against the panel selection.

use super::*;
use crate::interaction::types::{TimelineGesture, TimelineHitKind, TimelineWheel};

/// Pointer travel (logical px, Chebyshev) from the Down point before a timeline
/// gesture counts as a drag rather than a tap.
pub const TIMELINE_DRAG_SLOP_PX: f32 = 4.0;

impl WidgetStore {
    /// Stash one timeline pointer gesture (dispatch → panel).
    pub fn push_timeline_gesture(&mut self, gesture: TimelineGesture) {
        self.timeline_gestures.push(gesture);
    }

    /// Drain this frame's timeline gestures. `collect()` retains the store's Vec
    /// capacity (the drain keeps the allocation), so a steady drag reuses it.
    pub fn drain_timeline_gestures(&mut self) -> std::vec::Drain<'_, TimelineGesture> {
        self.timeline_gestures.drain(..)
    }

    /// If `id`'s state is an [`InteractiveState::TimelineSurface`], return its
    /// `(surface parent, hit kind)`. Editor-core copies both out and never
    /// dereferences the opaque handles inside `kind`.
    pub fn timeline_surface_at_id(&self, id: NodeId) -> Option<(NodeId, TimelineHitKind)> {
        match self.get(id) {
            Some(InteractiveState::TimelineSurface { parent, kind, .. }) => Some((*parent, *kind)),
            _ => None,
        }
    }

    /// Arm a fresh timeline capture at the Down point: not yet a drag, and this
    /// is the origin the slop is measured from.
    pub fn begin_timeline_press(&mut self, x: f32, y: f32) {
        self.timeline_press = (x, y);
        self.timeline_moved = false;
    }

    /// Promote the capture to a DRAG once the pointer has travelled past
    /// [`TIMELINE_DRAG_SLOP_PX`] from the Down point. Called on every move.
    ///
    /// Without the slop, one pixel of hand tremor between press and release makes
    /// Up choose `End` over `Click`, and every tap target that only answers to a
    /// click (the row twirl) becomes a coin flip.
    pub fn note_timeline_pointer(&mut self, x: f32, y: f32) {
        if !self.timeline_moved {
            let (px, py) = self.timeline_press;
            let travel = (x - px).abs().max((y - py).abs());
            self.timeline_moved = travel > TIMELINE_DRAG_SLOP_PX;
        }
    }

    /// Read + reset the "timeline capture moved" flag (Up chooses End vs Click).
    pub fn take_timeline_moved(&mut self) -> bool {
        std::mem::take(&mut self.timeline_moved)
    }

    // ── Wheel: anchored zoom + pan + scroll ──────────────────────────────
    /// Accumulate a frame's wheel for `surface` across its three axes (zoom /
    /// horizontal pan / vertical scroll). The anchor follows the latest cursor.
    /// The panel drains + applies it.
    pub fn add_timeline_wheel(
        &mut self,
        surface: NodeId,
        zoom: f32,
        pan: f32,
        scroll: f32,
        anchor_x: f32,
    ) {
        let w = self.timeline_wheel.entry(surface).or_default();
        w.zoom_delta += zoom;
        w.pan_delta += pan;
        w.scroll_delta += scroll;
        w.anchor_x = anchor_x;
    }

    /// Drain the accumulated wheel for `surface` (removes the entry).
    pub fn take_timeline_wheel(&mut self, surface: NodeId) -> Option<TimelineWheel> {
        self.timeline_wheel.remove(&surface)
    }

    // ── Time-axis rect registry (wheel hit-test) ─────────────────────────
    /// Republish `surface`'s time-axis rect (panel, each frame) so
    /// `dispatch_wheel` can tell when the cursor is over the dope-sheet.
    pub fn set_timeline_canvas(&mut self, surface: NodeId, rect: Rect) {
        self.timeline_canvas.insert(surface, rect);
    }

    /// Forget every registered timeline canvas (the panel calls this while
    /// hidden, so a closed timeline leaves no stale wheel-zoom rect).
    pub fn clear_timeline_canvas(&mut self) {
        self.timeline_canvas.clear();
    }

    /// The timeline surface whose time-axis rect contains `(x, y)`, if any.
    pub fn timeline_surface_at(&self, x: f32, y: f32) -> Option<NodeId> {
        self.timeline_canvas
            .iter()
            .find(|(_, r)| r.contains(x, y))
            .map(|(id, _)| *id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tap_that_trembles_within_the_slop_is_still_a_click() {
        // Without this, a twirl (which only answers to Click) is a coin flip: the
        // dispatch used to call the capture "moved" on the first pixel of travel.
        let mut store = WidgetStore::with_capacity(0);
        store.begin_timeline_press(100.0, 50.0);
        store.note_timeline_pointer(101.0, 52.0);
        store.note_timeline_pointer(100.0, 47.0);
        assert!(!store.take_timeline_moved(), "still a tap");
    }

    #[test]
    fn travelling_past_the_slop_makes_it_a_drag_for_good() {
        let mut store = WidgetStore::with_capacity(0);
        store.begin_timeline_press(100.0, 50.0);
        store.note_timeline_pointer(100.0 + TIMELINE_DRAG_SLOP_PX + 0.1, 50.0);
        // Coming back inside the slop does NOT demote it — a drag stays a drag.
        store.note_timeline_pointer(100.0, 50.0);
        assert!(store.take_timeline_moved());
        assert!(!store.take_timeline_moved(), "the flag resets on read");
    }

    #[test]
    fn each_press_rearms_the_slop_from_its_own_origin() {
        let mut store = WidgetStore::with_capacity(0);
        store.begin_timeline_press(0.0, 0.0);
        store.note_timeline_pointer(500.0, 0.0);
        assert!(store.take_timeline_moved());
        store.begin_timeline_press(500.0, 0.0);
        store.note_timeline_pointer(501.0, 0.0);
        assert!(!store.take_timeline_moved(), "measured from the new Down");
    }
}
