//! Guards for the rewiring gestures (F2, doc 45). `super` is `motion_bridge::rewire`.

use super::*;
use crate::motion_state::MotionState;
use ph2d_editor::ToastQueue;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// `grid -> move -> output`. The wire under test lands on `move`'s input (node 1, port 0).
fn wired() -> MotionState {
    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let out = g.add_node("motion.output");
    for (from, to) in [(grid, mv), (mv, out)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .expect("wire");
    }
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 4.0);
    motion.doc.graph = g;
    motion.sinks = vec![out];
    motion
}

/// What the sink renders — the oracle for "the splice changed nothing".
fn rendered(motion: &mut MotionState) -> Vec<[f32; 2]> {
    motion.pump.mark_dirty();
    motion.pump.pump(
        &motion.doc.graph,
        &motion.registry,
        &motion.sinks,
        0,
        0.0,
        motion.default_uv_rect,
        motion.default_size,
    );
    motion.pump.instances.iter().map(|i| i.world_pos).collect()
}

fn source_of(motion: &MotionState, to: u32, port: u16) -> Option<(u32, u16)> {
    motion
        .doc
        .graph
        .edges()
        .iter()
        .find(|e| e.to.0.0 == to && e.to.1 == port && !e.delayed)
        .map(|e| (e.from.0.0, e.from.1))
}

/// **A spliced reroute changes NOTHING about what is rendered** — and that is what makes the
/// gesture safe to reach for while tidying a live scene. The dot goes into the chain, the
/// graph re-cooks (it is a node), and every instance lands exactly where it did.
///
/// FALSIFIED by a reroute that is not a true pass-through (drops a column, re-orders, or
/// emits its own stream).
#[test]
fn splicing_a_reroute_does_not_move_a_single_pixel() {
    let mut motion = wired();
    let before = rendered(&mut motion);
    assert_eq!(before.len(), 12, "3x4 instances render");

    let mut toasts = ToastQueue::default();
    splice_reroute(&mut motion, &mut toasts, 1, 0, 100.0, 50.0);

    assert_eq!(rendered(&mut motion), before, "the render is untouched");
    assert_eq!(motion.doc.graph.nodes().len(), 4, "the dot is IN the graph");
    // …and it is spliced INTO the chain: grid → dot → move.
    let dot = source_of(&motion, 1, 0)
        .expect("the wire into `move` still exists")
        .0;
    assert_ne!(dot, 0, "…but its source is the DOT now, not the grid");
    assert_eq!(
        source_of(&motion, dot, 0),
        Some((0, 0)),
        "grid feeds the dot"
    );
    assert_eq!(
        motion.doc.graph.node(NodeId(dot)).unwrap().type_name,
        "util.reroute",
        "the type was chosen from the WIRE, not from a menu"
    );
}

/// The splice picks the reroute that fits the wire's own port type — a VALUE wire gets the
/// value reroute, not the instance-stream one (which would not even validate).
#[test]
fn the_splice_picks_the_reroute_that_fits_the_wire() {
    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let lfo = g.add_node("value.lfo");
    let drive = g.add_node("motion.drive"); // takes a VALUE on some input
    let value_port = motion
        .registry
        .resolve(ph2d_nodegraph::node::NodeTypeId::of("motion.drive"))
        .unwrap()
        .manifest()
        .inputs
        .iter()
        .position(|p| {
            p.ty == ph2d_nodegraph::port::PortType::new(
                ph2d_nodegraph::port::Domain::Instances,
                ph2d_nodegraph::port::Dim::Scalar,
                ph2d_nodegraph::port::Clock::Frame,
            )
        })
        .expect("drive takes a value") as u16;
    g.connect(Edge {
        from: (lfo, 0),
        to: (drive, value_port),
        delayed: false,
    })
    .expect("wire");
    motion.doc.graph = g;

    let mut toasts = ToastQueue::default();
    splice_reroute(&mut motion, &mut toasts, drive.0, value_port, 0.0, 0.0);

    let dot = source_of(&motion, drive.0, value_port)
        .expect("still wired")
        .0;
    assert_eq!(
        motion.doc.graph.node(NodeId(dot)).unwrap().type_name,
        "util.reroute_value",
        "a value wire got the VALUE reroute"
    );
    assert!(motion.doc.graph.validate(&motion.registry).is_ok());
}

/// **A grabbed wire-end MOVES; it does not copy.** Drop it on another input and the old one
/// is empty — one undo step for the pair.
///
/// FALSIFIED by emitting `Connect` alone (the old wire would still be there: the artist
/// would have *duplicated* a wire by trying to move it).
#[test]
fn moving_a_wire_end_leaves_the_old_input_empty() {
    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let a = g.add_node("motion.move");
    let b = g.add_node("motion.scale");
    g.connect(Edge {
        from: (grid, 0),
        to: (a, 0),
        delayed: false,
    })
    .expect("wire");
    motion.doc.graph = g;

    let mut toasts = ToastQueue::default();
    move_wire_end(&mut motion, &mut toasts, grid.0, 0, a.0, 0, Some((b.0, 0)));

    assert_eq!(source_of(&motion, a.0, 0), None, "the old input is empty");
    assert_eq!(
        source_of(&motion, b.0, 0),
        Some((grid.0, 0)),
        "it landed on b"
    );

    // ONE undo puts the WHOLE move back — unplug and plug were one step.
    let back = motion.history.undo(&motion.doc).expect("one undo step");
    motion.doc = back;
    assert_eq!(source_of(&motion, a.0, 0), Some((grid.0, 0)), "back on a");
    assert_eq!(source_of(&motion, b.0, 0), None, "and off b");
}

/// **A refused landing keeps the ORIGINAL wire.** The artist asked to move a wire somewhere
/// it cannot go; the answer is "no" — not "the wire you had is gone too".
///
/// FALSIFIED by disconnecting first and connecting after: the wire would be destroyed by a
/// gesture that was only ever asking to move it.
#[test]
fn a_refused_move_destroys_nothing() {
    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid"); // an INSTANCE stream
    let mv = g.add_node("motion.move");
    let drive = g.add_node("motion.drive");
    g.connect(Edge {
        from: (grid, 0),
        to: (mv, 0),
        delayed: false,
    })
    .expect("wire");
    motion.doc.graph = g;

    // Find a VALUE input on `drive` — an instance stream cannot land there.
    let value_port = motion
        .registry
        .resolve(ph2d_nodegraph::node::NodeTypeId::of("motion.drive"))
        .unwrap()
        .manifest()
        .inputs
        .iter()
        .position(|p| p.ty.dim == ph2d_nodegraph::port::Dim::Scalar)
        .expect("drive takes a value") as u16;

    let mut toasts = ToastQueue::default();
    move_wire_end(
        &mut motion,
        &mut toasts,
        grid.0,
        0,
        mv.0,
        0,
        Some((drive.0, value_port)),
    );

    assert_eq!(
        source_of(&motion, mv.0, 0),
        Some((grid.0, 0)),
        "the original wire survived the refusal"
    );
    assert_eq!(
        source_of(&motion, drive.0, value_port),
        None,
        "and it did not land"
    );
    assert!(motion.doc.graph.validate(&motion.registry).is_ok());
}

/// Dropped on empty canvas, the wire is simply unplugged. Dropped back where it came from,
/// nothing happened at all — and nothing is pushed onto the undo stack for a gesture that
/// changed nothing.
#[test]
fn dropped_in_space_unplugs_and_dropped_home_does_nothing() {
    let mut motion = wired();
    let mut toasts = ToastQueue::default();

    // Dropped back home: nothing happened, and nothing was pushed onto the undo stack.
    move_wire_end(&mut motion, &mut toasts, 0, 0, 1, 0, Some((1, 0)));
    assert_eq!(source_of(&motion, 1, 0), Some((0, 0)), "still wired");
    assert!(!motion.history.can_undo(), "a no-op mints no undo step");

    // Dropped in space: unplugged — and THAT is undoable.
    move_wire_end(&mut motion, &mut toasts, 0, 0, 1, 0, None);
    assert_eq!(
        source_of(&motion, 1, 0),
        None,
        "dropped in space: unplugged"
    );
    assert!(motion.history.can_undo());
}
