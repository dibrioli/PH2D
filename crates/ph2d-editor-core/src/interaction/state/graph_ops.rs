//! Motion Nodes M0.T2 — graph-surface dispatch channel on [`WidgetStore`].
//!
//! Editor-core knows no graph semantics: the pointer/key dispatch stashes typed
//! [`GraphGesture`]s / [`GraphKey`]s / [`GraphZoom`]s here and the motion-graph
//! panel drains + interprets them each frame. This mirrors the curve-point-drag
//! channel (`curve_point_drag`, `set/take_curve_point_drag`) and the panel-scroll
//! scalar channel (`panel_scroll`), scoped to graph surfaces.

use super::*;
use crate::interaction::types::{GestureMods, GraphGesture, GraphHitKind, GraphKey, GraphZoom};

impl WidgetStore {
    // ── Pointer gestures ─────────────────────────────────────────────────
    /// Stash one graph pointer gesture (dispatch → panel).
    pub fn push_graph_gesture(&mut self, gesture: GraphGesture) {
        self.graph_gestures.push(gesture);
    }

    /// Drain this frame's graph gestures. `collect()` retains the store's Vec
    /// capacity (the drain keeps the allocation), so a steady drag reuses it.
    pub fn drain_graph_gestures(&mut self) -> std::vec::Drain<'_, GraphGesture> {
        self.graph_gestures.drain(..)
    }

    /// If `id`'s state is an [`InteractiveState::GraphSurface`], return its
    /// `(surface parent, hit kind)`. Editor-core copies both out and never
    /// dereferences the opaque handles inside `kind`.
    pub fn graph_surface_at_id(&self, id: NodeId) -> Option<(NodeId, GraphHitKind)> {
        match self.get(id) {
            Some(InteractiveState::GraphSurface { parent, kind, .. }) => Some((*parent, *kind)),
            _ => None,
        }
    }

    /// Mark whether the active graph capture has moved (set on Update; read on
    /// Up to choose End vs Click).
    pub fn set_graph_moved(&mut self, moved: bool) {
        self.graph_moved = moved;
    }

    /// Read + reset the "graph capture moved" flag.
    pub fn take_graph_moved(&mut self) -> bool {
        std::mem::take(&mut self.graph_moved)
    }

    /// Arm the double-click flag on a graph Down (from
    /// [`WidgetStore::record_pointer_down`]). The graph's Down captures the pointer and
    /// returns EARLY — past the general double-click detection — so without this a graph
    /// surface could never see a double-click at all, whatever a panel asked for.
    pub fn set_graph_double(&mut self, double: bool) {
        self.graph_double = double;
    }

    /// Read + reset the flag. The Up uses it to choose `DoubleClick` over `Click`.
    pub fn take_graph_double(&mut self) -> bool {
        std::mem::take(&mut self.graph_double)
    }

    // ── Anchored zoom ────────────────────────────────────────────────────
    /// Accumulate a wheel-notch delta for `surface`'s anchored zoom, updating
    /// the anchor to the latest cursor. The panel drains + applies it.
    pub fn add_graph_zoom(&mut self, surface: NodeId, delta: f32, anchor_x: f32, anchor_y: f32) {
        let z = self.graph_zoom.entry(surface).or_insert(GraphZoom {
            delta: 0.0,
            anchor_x,
            anchor_y,
        });
        z.delta += delta;
        z.anchor_x = anchor_x;
        z.anchor_y = anchor_y;
    }

    /// Drain the accumulated zoom for `surface` (removes the entry).
    pub fn take_graph_zoom(&mut self, surface: NodeId) -> Option<GraphZoom> {
        self.graph_zoom.remove(&surface)
    }

    // ── Canvas registry (wheel hit-test) ─────────────────────────────────
    /// Republish `surface`'s canvas rect (panel, each frame) so `dispatch_wheel`
    /// can tell when the cursor is over a graph.
    pub fn set_graph_canvas(&mut self, surface: NodeId, rect: Rect) {
        self.graph_canvas.insert(surface, rect);
    }

    /// Forget every registered graph canvas (panel calls this before re-adding
    /// the visible ones, so a hidden graph leaves no stale rect).
    pub fn clear_graph_canvas(&mut self) {
        self.graph_canvas.clear();
    }

    /// The graph surface whose canvas contains `(x, y)`, if any.
    pub fn graph_surface_at(&self, x: f32, y: f32) -> Option<NodeId> {
        self.graph_canvas
            .iter()
            .find(|(_, r)| r.contains(x, y))
            .map(|(id, _)| *id)
    }

    // ── Keyboard focus + commands ────────────────────────────────────────
    /// Set (or clear) the graph surface that holds keyboard focus. `dispatch_key`
    /// routes graph shortcuts only while this is `Some` and no text widget is
    /// focused.
    pub fn set_graph_focused(&mut self, surface: Option<NodeId>) {
        self.graph_focused = surface;
    }

    /// The graph surface that currently holds keyboard focus, if any.
    pub fn graph_focused(&self) -> Option<NodeId> {
        self.graph_focused
    }

    /// Stash one graph keyboard command (dispatch → panel).
    pub fn push_graph_key(&mut self, key: GraphKey) {
        self.graph_keys.push(key);
    }

    /// Drain this frame's graph key commands (capacity-retaining, like
    /// [`Self::drain_graph_gestures`]).
    pub fn drain_graph_keys(&mut self) -> std::vec::Drain<'_, GraphKey> {
        self.graph_keys.drain(..)
    }

    // ── Modifiers ────────────────────────────────────────────────────────
    /// Snapshot the cached pointer modifiers for a gesture (pointer events carry
    /// none natively; the shell pushes these on `ModifiersChanged`).
    pub fn gesture_mods(&self) -> GestureMods {
        GestureMods {
            shift: self.shift_held,
            cmd: self.cmd_held,
            alt: self.alt_held,
        }
    }

    /// Cache the latest Alt state (mirror of `set_shift_held`/`set_cmd_held`).
    pub fn set_alt_held(&mut self, held: bool) {
        self.alt_held = held;
    }

    /// Latest cached Alt state.
    pub fn alt_held(&self) -> bool {
        self.alt_held
    }
}
