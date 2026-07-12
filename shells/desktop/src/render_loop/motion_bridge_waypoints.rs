//! Waypoint intents (Motion Nodes F2, doc 44) — the shell half of the graph panel's wire
//! routing, split out of `motion_bridge` (shell LOC cap). Sibling of `motion_bridge_backdrops`,
//! and for the same reason.
//!
//! Waypoints are **UI-only decoration**, exactly like the backdrops: they live on the
//! `MotionDoc` beside the graph, they serialize with it, and **nothing here ever calls
//! `mark_dirty`** — a waypoint changes how a wire is *drawn* and nothing about what the
//! graph *computes*, so re-cooking on a waypoint edit would be pure waste (and would make
//! dragging a routing dot stutter a 79-node graph). The `is_dirty` guard the backdrops
//! brought (doc 35) proves it, and a test holds it.
//!
//! They ARE undoable (they are document state): the drag arrives bracketed by
//! `BeginDrag`/`EndDrag` (one step for the whole gesture), and add/remove push their own.
//!
//! ## The wire is named by its INPUT
//!
//! An input port holds at most one edge — the graph's own invariant, the one
//! `GraphIntent::Disconnect { to_node, to_port }` already leans on. So `(to_node, to_port)`
//! names a wire exactly. **Cut the wire and its waypoints go with it** ([`prune`]): routing
//! points for a wire that no longer exists are not decoration, they are litter — and they
//! would silently reattach themselves to the next wire dropped on that input.

use super::MotionState;
use ph2d_motion_doc::Waypoints;

/// The routing points of the wire landing on `(to_node, to_port)`, if any.
fn find(motion: &mut MotionState, to_node: u32, to_port: u16) -> Option<&mut Waypoints> {
    motion
        .doc
        .waypoints
        .iter_mut()
        .find(|w| w.to_node == to_node && w.to_port == to_port)
}

/// Add a routing point to a wire, at `index` in its order. One undo step; no re-cook.
///
/// `index` is clamped: it comes from the panel's hit geometry, and a stale snapshot (a
/// gesture that lands the frame after the wire changed) must not panic the editor.
pub(super) fn add(
    motion: &mut MotionState,
    to_node: u32,
    to_port: u16,
    index: usize,
    x: f32,
    y: f32,
) {
    let pre = motion.doc.clone();
    match find(motion, to_node, to_port) {
        Some(w) => {
            let i = index.min(w.points.len());
            w.points.insert(i, (x, y));
        }
        None => motion.doc.waypoints.push(Waypoints {
            to_node,
            to_port,
            points: vec![(x, y)],
        }),
    }
    motion.history.push_undo(pre);
}

/// Move a routing point by a graph-space delta (live, each frame of the drag). The undo
/// bracket is opened by the panel's `BeginDrag`, so this pushes none of its own.
pub(super) fn translate(
    motion: &mut MotionState,
    to_node: u32,
    to_port: u16,
    index: usize,
    dx: f32,
    dy: f32,
) {
    if let Some(w) = find(motion, to_node, to_port)
        && let Some(p) = w.points.get_mut(index)
    {
        p.0 += dx;
        p.1 += dy;
    }
}

/// Remove a routing point. One undo step; no re-cook. A wire whose last point is removed
/// drops its whole record, so a straight wire leaves nothing behind in the document.
pub(super) fn remove(motion: &mut MotionState, to_node: u32, to_port: u16, index: usize) {
    let pre = motion.doc.clone();
    let mut changed = false;
    if let Some(w) = find(motion, to_node, to_port)
        && index < w.points.len()
    {
        w.points.remove(index);
        changed = true;
    }
    if changed {
        motion.doc.waypoints.retain(|w| !w.points.is_empty());
        motion.history.push_undo(pre);
    }
}

/// **Drop the routing of every wire that no longer exists.** Called after any edit that can
/// remove an edge (disconnect, knife, delete-selection, undo of a connect).
///
/// Routing points for a dead wire are not decoration, they are litter — and worse, they
/// would silently reattach to the next wire dropped on that same input, so a fresh
/// connection would come out mysteriously bent. Pruning is part of the SAME undo step as the
/// edit that killed the wire (it is called before the snapshot is pushed by the caller, or
/// inside its bracket), so one Ctrl+Z brings the wire and its routing back together.
pub(super) fn prune(motion: &mut MotionState) {
    let edges = motion.doc.graph.edges().to_vec();
    motion.doc.waypoints.retain(|w| {
        edges
            .iter()
            .any(|e| e.to.0.0 == w.to_node && e.to.1 == w.to_port && !e.delayed)
    });
}

/// The panel's view of a wire's routing, stamped onto the snapshot the panel is about to
/// receive — the same shape the backdrops take (`snapshot_from` only sees the graph, and the
/// waypoints live on the document).
pub(super) fn stamp(motion: &MotionState, snap: &mut ph2d_panel_motion_graph::GraphViewSnapshot) {
    for e in &mut snap.edges {
        if let Some(w) = motion
            .doc
            .waypoints
            .iter()
            .find(|w| w.to_node == e.to_node && w.to_port == e.to_port)
        {
            e.waypoints = w.points.clone();
        }
    }
}

#[cfg(test)]
#[path = "motion_bridge_waypoint_tests.rs"]
mod tests;
