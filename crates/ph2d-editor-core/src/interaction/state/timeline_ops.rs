//! Timeline dope-sheet dispatch channel on [`WidgetStore`] (W2.E5b).
//!
//! Editor-core knows no timeline semantics: the pointer dispatch stashes typed
//! [`TimelineGesture`]s here and the timeline panel drains + interprets them each
//! frame (which diamond was hit, drag-to-move, clear-on-empty). Lean mirror of
//! the graph-surface channel (`graph_ops`) — pointer gestures only; zoom/pan is
//! E6 and keyboard (Delete) is handled shell-side against the panel selection.

use super::*;
use crate::interaction::types::{TimelineGesture, TimelineHitKind};

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
}
