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
        .into_values()
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

    let copy = edit::duplicate(&mut motion, vec![ex.0])[&ex];
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

/// **Smart-connect through the shell**: the node is added at the drop point AND
/// wired to the socket the drag came from — in ONE undo step, because it was one
/// gesture. FALSIFIED if the node landed unconnected (the artist drew a wire and
/// got an orphan) or if it took two Ctrl+Z to take back.
#[test]
fn smart_connect_adds_and_wires_in_one_undo_step() {
    let mut motion = MotionState::new();
    motion.doc.graph = Graph::new();
    let grid = motion.doc.graph.add_node("motion.grid");
    let mut toasts = ToastQueue::default();

    edit::smart_connect(
        &mut motion,
        &mut toasts,
        grid.0,
        0,
        "motion.move",
        300.0,
        80.0,
    );

    let g = &motion.doc.graph;
    let target = g
        .nodes()
        .iter()
        .find(|n| n.type_name == "motion.move")
        .expect("the picked node was added");
    assert_eq!(g.pos(target.id).map(|p| (p.x, p.y)), Some((300.0, 80.0)));
    assert_eq!(
        g.input_edge(target.id, 0).map(|(from, _, _)| from),
        Some(grid),
        "and it arrived WIRED to the socket the drag came from"
    );

    let back = motion.history.undo(&motion.doc).expect("one step");
    assert!(
        back.graph
            .nodes()
            .iter()
            .all(|n| n.type_name != "motion.move"),
        "one Ctrl+Z takes back the whole gesture"
    );
}

/// The probe reads the node the artist pointed at — through the pump's OWN cook, so
/// it can never report a different tick than the one on screen. A `motion.grid`
/// carries instances, so the readout counts them.
#[test]
fn the_probe_reads_the_node_it_points_at() {
    let mut motion = MotionState::new();
    motion.doc.graph = Graph::new();
    let grid = motion.doc.graph.add_node("motion.grid");
    motion.doc.graph.set_param(grid, "rows", 4.0);
    motion.doc.graph.set_param(grid, "cols", 5.0);
    motion.probe = Some(grid);

    let view = edit::sample_probe(&mut motion, 0.0, None).expect("the probe read the node");
    assert_eq!(view.node, grid.0);
    assert_eq!(view.label, "instances");
    assert_eq!(view.value, Some(20.0), "a 4×5 grid carries 20 instances");
    assert_eq!(view.samples, vec![20.0], "and the ring started filling");

    // A second tick appends (the sparkline is a history, not a single reading).
    let view = edit::sample_probe(&mut motion, 1.0 / 60.0, None).unwrap();
    assert_eq!(view.samples.len(), 2);
}

/// A probe pointed at a node that is then DELETED clears itself — a readout of a
/// node that no longer exists is a lie, and the panel would draw it beside nothing.
#[test]
fn the_probe_lets_go_of_a_deleted_node() {
    let mut motion = MotionState::new();
    motion.doc.graph = Graph::new();
    let grid = motion.doc.graph.add_node("motion.grid");
    motion.probe = Some(grid);
    assert!(edit::sample_probe(&mut motion, 0.0, None).is_some());

    motion.doc.graph.remove_node(grid);
    assert!(edit::sample_probe(&mut motion, 0.0, None).is_none());
    assert!(motion.probe.is_none(), "the probe let go");
}

/// **A GPU-resident cook leaves the probe with nothing to say — and the probe must
/// SAY that, rather than cook the answer on the CPU.**
///
/// The strong half of this gate is not the `None`: it is that the pump's memo is
/// still EMPTY afterwards. `sample_probe`'s `cook()` is the cheap memo lookup its
/// doc-comment describes only while the CPU pump is the one driving; under a GPU
/// cook the memo is empty, so the SAME call silently becomes a full CPU cook of the
/// chain — every frame, beside a GPU already drawing it (~50 ms at the emitter
/// demo's 1,2 M) — and returns a simulation whose `pre` state was never marched.
/// Asserting the memo stayed empty is what pins "did not cook"; asserting only the
/// `None` would stay green with the cook back in.
#[test]
fn a_gpu_resident_cook_leaves_the_probe_with_no_reading_and_does_not_cook() {
    let mut motion = MotionState::new();
    motion.doc.graph = Graph::new();
    let grid = motion.doc.graph.add_node("motion.grid");
    motion.doc.graph.set_param(grid, "rows", 4.0);
    motion.doc.graph.set_param(grid, "cols", 5.0);
    motion.probe = Some(grid);
    // Stale CPU history, exactly as a run that HAD been on the CPU would hold.
    motion.probe_ring.push(123.0);
    motion.gpu_live = true;

    let view = edit::sample_probe(&mut motion, 0.0, None).expect("the probe card stays put");
    assert_eq!(
        view.value, None,
        "no reading, rather than a stale or invented one"
    );
    assert!(
        view.samples.is_empty(),
        "the CPU sparkline does not survive a GPU cook"
    );
    assert!(
        motion.pump.cook.peek(grid).is_none(),
        "the probe COOKED under a GPU-resident frame — the memo filled"
    );
}

/// The control for the gate above: on the CPU path the probe is untouched. Without
/// it, deleting the probe outright would leave that gate green
/// ([[feedback_absence_gate_needs_a_presence_sibling]]) — and the CPU path is the
/// one an artist uses today, so a regression there is the expensive one.
#[test]
fn the_cpu_path_still_reads_and_still_fills_the_memo() {
    let mut motion = MotionState::new();
    motion.doc.graph = Graph::new();
    let grid = motion.doc.graph.add_node("motion.grid");
    motion.doc.graph.set_param(grid, "rows", 4.0);
    motion.doc.graph.set_param(grid, "cols", 5.0);
    motion.probe = Some(grid);
    motion.gpu_live = false;

    let view = edit::sample_probe(&mut motion, 0.0, None).expect("a reading");
    assert_eq!(view.value, Some(20.0), "the CPU path still answers");
    assert!(!view.samples.is_empty(), "and still feeds the sparkline");
    assert!(
        motion.pump.cook.peek(grid).is_some(),
        "the CPU path DOES cook — which is what makes the memo assertion above mean something"
    );
}
