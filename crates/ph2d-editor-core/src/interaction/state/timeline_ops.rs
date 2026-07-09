//! Timeline dope-sheet dispatch channel on [`WidgetStore`] (W2.E5b).
//!
//! Editor-core knows no timeline semantics: the pointer/wheel dispatch stashes
//! typed [`TimelineGesture`]s / [`TimelineZoom`]s here and the timeline panel
//! drains + interprets them each frame (which diamond was hit, drag-to-move,
//! clear-on-empty, anchored zoom + pan of the time axis). Lean mirror of the
//! graph-surface channel (`graph_ops`); keyboard (Delete/undo) is handled
//! shell-side against the panel selection.

use super::*;
use crate::interaction::types::{TimelineGesture, TimelineHitKind, TimelineZoom};

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

    /// Mark whether the active timeline capture has moved (set on Update; read on
    /// Up to choose End vs Click).
    pub fn set_timeline_moved(&mut self, moved: bool) {
        self.timeline_moved = moved;
    }

    /// Read + reset the "timeline capture moved" flag.
    pub fn take_timeline_moved(&mut self) -> bool {
        std::mem::take(&mut self.timeline_moved)
    }

    // ── Anchored zoom + pan (wheel) ──────────────────────────────────────
    /// Accumulate a wheel notch for `surface`: `zoom` notches drive the anchored
    /// zoom, `pan` notches slide the time axis. The anchor follows the latest
    /// cursor. The panel drains + applies it.
    pub fn add_timeline_zoom(&mut self, surface: NodeId, zoom: f32, pan: f32, anchor_x: f32) {
        let z = self.timeline_zoom.entry(surface).or_default();
        z.zoom_delta += zoom;
        z.pan_delta += pan;
        z.anchor_x = anchor_x;
    }

    /// Drain the accumulated wheel for `surface` (removes the entry).
    pub fn take_timeline_zoom(&mut self, surface: NodeId) -> Option<TimelineZoom> {
        self.timeline_zoom.remove(&surface)
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
