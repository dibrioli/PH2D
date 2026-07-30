//! Seam tests for **replace-on-drop** (doc 63.2) — the shell half of a wire dropped on an
//! already-fed input. Declared by the parent as a `#[path]` sibling, so `super` is
//! `render_loop::motion_bridge`.

use super::connect;
use crate::motion_state::MotionState;
use ph2d_editor::ToastQueue;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// `a → move.0`, plus a free second source `b`. Both are `motion.grid`, so both feed a
/// deformer's input 0 the same well-typed way — the swap is the only difference between them.
/// The raw `connect` pushes NO undo step, so `can_undo()` starts false.
fn two_sources_one_input(motion: &mut MotionState) -> (NodeId, NodeId, NodeId) {
    motion.doc.graph = Graph::new();
    let g = &mut motion.doc.graph;
    let a = g.add_node("motion.grid");
    let b = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    g.connect(Edge {
        from: (a, 0),
        to: (mv, 0),
        delayed: false,
    })
    .unwrap();
    (a, b, mv)
}

/// **A wire dropped on an occupied input SWAPS what feeds it** (Blender / Nuke / Houdini), rather
/// than earning the "input already wired" refusal — the natural completion of the magnetic drop,
/// which now lands on occupied sockets far more often. FALSIFIED by dropping the disconnect: the
/// connect then hits `InputAlreadyConnected`, the input stays fed by `a`, and `b` never lands.
#[test]
fn a_wire_dropped_on_an_occupied_input_replaces_what_feeds_it() {
    let mut motion = MotionState::new();
    let (a, b, mv) = two_sources_one_input(&mut motion);
    let mut toasts = ToastQueue::new();

    // Drop b's wire onto mv's input 0 — already fed by a.
    connect::apply_connect(&mut motion, &mut toasts, b.0, 0, mv.0, 0);

    let g = &motion.doc.graph;
    assert_eq!(
        g.input_edge(mv, 0).map(|(from, _, _)| from),
        Some(b),
        "the drop swapped the input's feeder to b"
    );
    assert!(
        !g.edges().iter().any(|e| e.from.0 == a && e.to.0 == mv),
        "and a's old wire is gone — one edge per input"
    );
    assert_eq!(toasts.len(), 0, "a swap is not a refusal");
}

/// **An expert `pre` (feedback) edge is NOT swapped** — a forward drop onto it earns the ordinary
/// occupied refusal, exactly the edge `clear_managed_pre_at` deliberately preserves. FALSIFIED by
/// widening the swap to any occupant (dropping the `!delayed` guard): the pre would be replaced.
#[test]
fn a_forward_drop_does_not_replace_an_expert_pre_edge() {
    let mut motion = MotionState::new();
    motion.doc.graph = Graph::new();
    let g = &mut motion.doc.graph;
    let a = g.add_node("motion.grid");
    let b = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    // A DELAYED edge feeds mv.0 — an expert feedback wire on a plain deformer, which
    // `clear_managed_pre_at` does not touch (mv is not an engine-managed feedback port).
    g.connect(Edge {
        from: (a, 0),
        to: (mv, 0),
        delayed: true,
    })
    .unwrap();
    let mut toasts = ToastQueue::new();

    connect::apply_connect(&mut motion, &mut toasts, b.0, 0, mv.0, 0);

    let g = &motion.doc.graph;
    let (from, _, delayed) = g.input_edge(mv, 0).expect("mv.0 is still fed");
    assert_eq!(
        (from, delayed),
        (a, true),
        "the expert pre survived the drop"
    );
    assert!(
        !g.edges().iter().any(|e| e.from.0 == b),
        "and b's forward wire was refused, not swapped in"
    );
    assert_eq!(toasts.len(), 1, "with the ordinary occupied refusal");
}

/// **Re-dropping the wire already there is a no-op** — no toast, and no spurious undo step for a
/// gesture that changed nothing. FALSIFIED by dropping the early return: the wire would be
/// disconnected and reconnected, committing an identical graph and pushing an undo step.
#[test]
fn re_dropping_the_wire_already_there_is_a_no_op() {
    let mut motion = MotionState::new();
    let (a, _b, mv) = two_sources_one_input(&mut motion);
    assert!(!motion.history.can_undo(), "raw setup pushes no undo step");
    let mut toasts = ToastQueue::new();

    // Drop a's wire onto mv.0 — where a already feeds it.
    connect::apply_connect(&mut motion, &mut toasts, a.0, 0, mv.0, 0);

    assert!(
        !motion.history.can_undo(),
        "re-dropping the exact wire is a no-op — no spurious undo step"
    );
    assert_eq!(toasts.len(), 0, "and says nothing");
    assert_eq!(
        motion.doc.graph.input_edge(mv, 0).map(|(from, _, _)| from),
        Some(a),
        "the wire is exactly as it was"
    );
}
