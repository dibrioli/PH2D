//! Guards for the wire routing (F2, doc 44). `super` is `motion_bridge::waypoints`.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// `grid -> move -> output`, with the wire into `move` (node 1, port 0) the one under test.
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
    motion.doc.graph = g;
    motion.sinks = vec![out];
    motion
}

fn points(motion: &MotionState, to_node: u32, to_port: u16) -> Vec<(f32, f32)> {
    motion
        .doc
        .waypoints
        .iter()
        .find(|w| w.to_node == to_node && w.to_port == to_port)
        .map(|w| w.points.clone())
        .unwrap_or_default()
}

/// **A waypoint edit NEVER re-cooks the graph** — the claim the whole design rests on.
///
/// A waypoint changes how a wire is *drawn* and nothing about what the graph *computes*, so
/// re-cooking on one would be pure waste: dragging a routing dot would re-evaluate a 79-node
/// document sixty times a second, for a curve.
///
/// This is the same `is_dirty` guard the backdrops brought (doc 35), and it is executable
/// precisely because waypoints live on the DOCUMENT and not on the `Edge` — an `Edge` field
/// would be inside the cook's fingerprint and this test could not exist.
#[test]
fn no_waypoint_edit_ever_re_cooks_the_graph() {
    let mut motion = wired();
    // Cook once, so the pump is clean.
    motion.pump.pump(
        &motion.doc.graph,
        &motion.registry,
        &motion.sinks,
        0,
        0.0,
        motion.default_uv_rect,
        motion.default_size,
    );
    assert!(!motion.pump.is_dirty(), "a fresh cook leaves it clean");

    add(&mut motion, 1, 0, 0, 50.0, 60.0);
    assert!(!motion.pump.is_dirty(), "adding a waypoint re-cooked!");
    translate(&mut motion, 1, 0, 0, 5.0, -5.0);
    assert!(!motion.pump.is_dirty(), "dragging a waypoint re-cooked!");
    remove(&mut motion, 1, 0, 0);
    assert!(!motion.pump.is_dirty(), "removing a waypoint re-cooked!");
}

/// The order the artist sees is the order the document keeps: a point inserted at an index
/// lands THERE, not at the end. FALSIFIED by a naive push — the wire would run out to the
/// last point and back, tying itself in a knot.
#[test]
fn a_point_lands_at_the_index_it_was_inserted_at() {
    let mut motion = wired();
    add(&mut motion, 1, 0, 0, 10.0, 0.0);
    add(&mut motion, 1, 0, 1, 30.0, 0.0);
    // Now insert BETWEEN them.
    add(&mut motion, 1, 0, 1, 20.0, 0.0);
    assert_eq!(
        points(&motion, 1, 0),
        vec![(10.0, 0.0), (20.0, 0.0), (30.0, 0.0)],
        "the new point sits between its neighbours"
    );

    // An out-of-range index (a gesture arriving a frame after the wire changed) clamps
    // rather than panicking the editor.
    add(&mut motion, 1, 0, 99, 40.0, 0.0);
    assert_eq!(points(&motion, 1, 0).len(), 4);
    assert_eq!(points(&motion, 1, 0)[3], (40.0, 0.0));
}

/// **Cut the wire and its routing goes with it** — in the SAME undo step, so one Ctrl+Z
/// brings the wire and its waypoints back together.
///
/// FALSIFIED by leaving the points behind: they would be litter, and worse, they would
/// silently reattach to the next wire dropped on that input — a fresh connection would come
/// out mysteriously bent by a dead wire's routing.
#[test]
fn a_cut_wire_takes_its_routing_with_it_and_undo_brings_both_back() {
    let mut motion = wired();
    add(&mut motion, 1, 0, 0, 50.0, 60.0);
    assert_eq!(points(&motion, 1, 0).len(), 1);

    // Cut the wire the way the shell does: disconnect, prune, push one undo step.
    let pre = motion.doc.clone();
    motion.doc.graph.disconnect(NodeId(1), 0).expect("cut");
    prune(&mut motion);
    motion.history.push_undo(pre);
    assert!(
        points(&motion, 1, 0).is_empty(),
        "the routing died with the wire"
    );

    // One undo brings BOTH back — they were one step.
    let restored = motion.history.undo(&motion.doc).expect("undo");
    motion.doc = restored;
    assert_eq!(
        points(&motion, 1, 0),
        vec![(50.0, 60.0)],
        "the wire and its routing come back together"
    );
}

/// The routing survives a save/load round trip — it is document state, and a document that
/// forgets how its wires were routed is a document that lost work.
#[test]
fn the_routing_round_trips_through_the_text_format() {
    let mut motion = wired();
    add(&mut motion, 1, 0, 0, 12.5, -3.25);
    add(&mut motion, 1, 0, 1, 40.0, 8.0);

    let text = motion.doc.to_text();
    assert!(text.contains("w 1 0 12.5 -3.25 40 8"), "the record: {text}");
    let back = ph2d_motion_doc::MotionDoc::from_text(&text).expect("reloads");
    assert_eq!(back.waypoints, motion.doc.waypoints);

    // A straight wire leaves NOTHING behind — removing the last point drops the record.
    remove(&mut motion, 1, 0, 1);
    remove(&mut motion, 1, 0, 0);
    assert!(motion.doc.waypoints.is_empty(), "no empty record survives");
    assert!(
        !motion.doc.to_text().contains("\nw "),
        "and none is written"
    );
}

/// The panel's snapshot carries the routing of the wire it belongs to, and only that one.
#[test]
fn the_snapshot_carries_each_wires_own_routing() {
    let mut motion = wired();
    add(&mut motion, 1, 0, 0, 50.0, 60.0);

    let mut snap = ph2d_panel_motion_graph::snapshot_from(&motion.doc.graph, &motion.registry);
    stamp(&motion, &mut snap);

    let routed = snap
        .edges
        .iter()
        .find(|e| e.to_node == 1 && e.to_port == 0)
        .expect("the wire is in the snapshot");
    assert_eq!(routed.waypoints, vec![(50.0, 60.0)]);
    let other = snap
        .edges
        .iter()
        .find(|e| e.to_node == 2)
        .expect("the other wire");
    assert!(other.waypoints.is_empty(), "routing does not leak sideways");
}
