//! Seam tests for the F2 graph edits (Ctrl+D duplicate, the knife). Declared by
//! the parent as a `#[path]` sibling, so `super` is `render_loop::motion_bridge`.

use super::edit;
use crate::motion_state::MotionState;
use ph2d_editor::ToastQueue;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A fresh doc: `grid → move → output`, plus a stray `tint` fed by the grid.
/// Returns the ids in that order.
fn scene(motion: &mut MotionState) -> (NodeId, NodeId, NodeId, NodeId) {
    motion.doc.graph = Graph::new();
    let g = &mut motion.doc.graph;
    let (grid, mv, out, tint) = (
        g.add_node("motion.grid"),
        g.add_node("motion.move"),
        g.add_node("motion.output"),
        g.add_node("motion.tint"),
    );
    for (from, to) in [
        ((grid, 0), (mv, 0)),
        ((mv, 0), (out, 0)),
        ((grid, 0), (tint, 0)),
    ] {
        g.connect(Edge {
            from,
            to,
            delayed: false,
        })
        .unwrap();
    }
    (grid, mv, out, tint)
}

/// Ctrl+D copies the nodes **with their params and text params**, and re-creates
/// the wires that ran BETWEEN them — offset, so the copies read as new cards.
///
/// FALSIFIED three ways: params dropped (the copy is a factory-reset node) · the
/// internal wire dropped (a duplicated chain arrives in pieces) · an EXTERNAL wire
/// copied (the copy would be silently spliced into somebody else's upstream, which
/// is not what a duplicate is).
#[test]
fn duplicate_copies_params_and_the_wires_between_the_copies_only() {
    let mut motion = MotionState::new();
    let (grid, mv, _out, _tint) = scene(&mut motion);
    motion.doc.graph.set_param(mv, "dx", 3.5);
    let before = motion.doc.graph.nodes().len();

    // Duplicate grid + move (the chain), NOT output.
    let copies: Vec<NodeId> = edit::duplicate(&mut motion, vec![grid.0, mv.0])
        .into_iter()
        .map(NodeId)
        .collect();

    let g = &motion.doc.graph;
    assert_eq!(g.nodes().len(), before + 2, "two copies");
    assert_eq!(copies.len(), 2, "and they came back as the new selection");

    let copy_of_move = copies
        .iter()
        .find(|id| g.node(**id).unwrap().type_name == "motion.move")
        .expect("the move was copied");
    assert_eq!(
        g.node_param_overrides(*copy_of_move).unwrap().get("dx"),
        Some(&3.5),
        "the params rode along"
    );
    // The internal wire (grid → move) exists between the COPIES.
    let copy_of_grid = copies.iter().find(|id| *id != copy_of_move).unwrap();
    assert_eq!(
        g.input_edge(*copy_of_move, 0).map(|(from, _, _)| from),
        Some(*copy_of_grid),
        "the wire between the copies was re-created"
    );
    // And the copy of `move` does NOT feed the original output (no external wire).
    assert!(
        !g.edges()
            .iter()
            .any(|e| e.from.0 == *copy_of_move && e.to.0 == _out),
        "an external wire is never copied"
    );
}

/// A duplicated node's text param (a `motion.expression` formula) survives — the
/// text channel is a second map, and copying only the f32 params would silently
/// hand back a formula-less expression node.
#[test]
fn duplicate_carries_the_text_param_too() {
    let mut motion = MotionState::new();
    motion.doc.graph = Graph::new();
    let ex = motion.doc.graph.add_node("motion.expression");
    motion.doc.graph.set_text_param(ex, "expr", "sin(t) * a");

    let copy = NodeId(edit::duplicate(&mut motion, vec![ex.0])[0]);
    assert_eq!(
        motion
            .doc
            .graph
            .node_text_param_overrides(copy)
            .and_then(|m| m.get("expr"))
            .map(String::as_str),
        Some("sin(t) * a"),
        "the formula rode along"
    );
}

/// Ctrl+D is ONE undo step, however many nodes it copied.
#[test]
fn duplicate_is_one_undo_step() {
    let mut motion = MotionState::new();
    let (grid, mv, _, _) = scene(&mut motion);
    let before = motion.doc.graph.nodes().len();

    edit::duplicate(&mut motion, vec![grid.0, mv.0]);
    let back = motion.history.undo(&motion.doc).expect("one step");
    assert_eq!(
        back.graph.nodes().len(),
        before,
        "one Ctrl+Z takes both back"
    );
}

/// **The knife is one undo step for the whole stroke** — a blade that cut three
/// wires and needed three Ctrl+Z would be a trap.
#[test]
fn the_knife_cuts_every_crossed_wire_in_one_undo_step() {
    let mut motion = MotionState::new();
    let (_grid, mv, _out, tint) = scene(&mut motion);
    let mut toasts = ToastQueue::default();
    let before = motion.doc.graph.edges().len();

    // Cut the two wires the grid feeds (into `move` and into `tint`).
    edit::cut_wires(&mut motion, &mut toasts, vec![(mv.0, 0), (tint.0, 0)]);
    assert_eq!(motion.doc.graph.edges().len(), before - 2, "both wires cut");

    let back = motion.history.undo(&motion.doc).expect("one step");
    assert_eq!(
        back.graph.edges().len(),
        before,
        "one Ctrl+Z restores the whole stroke"
    );
}

/// The knife may not cut the sequential-node state loop: that wiring is the
/// editor's plumbing, not a wire the artist drew. The document (the authority)
/// refuses it and says so — a SECOND barrier, since the panel already skips `pre`
/// edges when testing the stroke.
#[test]
fn the_knife_refuses_managed_state_wiring() {
    let mut motion = MotionState::new();
    motion.doc.graph = Graph::new();
    // `motion.spring` is a sequential node: dropping it plumbs its `pre` self-loop.
    let pre = motion.doc.clone();
    let spring = motion.doc.graph.add_node("motion.spring");
    super::plumbing::reconcile_after(&mut motion.doc.graph, &motion.registry, &pre.graph);
    let edges = motion.doc.graph.edges().len();
    assert!(edges > 0, "the state self-loop was plumbed");

    let mut toasts = ToastQueue::default();
    edit::cut_wires(&mut motion, &mut toasts, vec![(spring.0, 1)]); // port 1 = `state`

    assert_eq!(
        motion.doc.graph.edges().len(),
        edges,
        "the managed state wire survived the blade"
    );
}
