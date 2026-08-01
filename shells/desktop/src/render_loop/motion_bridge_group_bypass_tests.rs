//! Group-bypass gates — the cook-time rewire. Declared by `motion_bridge_group_bypass` as a
//! `#[path]` sibling, so `super` is that module (its `cook_graph` and helpers are in scope).

use super::*;
use crate::motion_state::MotionState;
use ph2d_motion_doc::Subgraph;
use ph2d_nodegraph::graph::{Edge, NodeId};

/// `A -> [group{B,C}: B -> C] -> D`. The group's input slot 0 is fed by A; its output slot 0
/// (from C) feeds D. Returns `(motion, sid, A, B, C, D)`.
fn chain_through_a_group() -> (MotionState, u32, NodeId, NodeId, NodeId, NodeId) {
    let mut m = MotionState::new();
    let a = m.doc.graph.add_node("motion.grid");
    let b = m.doc.graph.add_node("motion.clone");
    let c = m.doc.graph.add_node("motion.clone");
    let d = m.doc.graph.add_node("motion.clone");
    for (from, to) in [(a, b), (b, c), (c, d)] {
        m.doc
            .graph
            .connect(Edge {
                from: (from, 0),
                to: (to, 0),
                delayed: false,
            })
            .unwrap();
    }
    m.doc.subgraphs.push(Subgraph {
        id: 1,
        parent: None,
        x: 0.0,
        y: 0.0,
        title: "Rig".into(),
    });
    m.doc.members.insert(b, 1);
    m.doc.members.insert(c, 1);
    (m, 1, a, b, c, d)
}

fn has_edge(g: &ph2d_nodegraph::graph::Graph, from: NodeId, to: NodeId) -> bool {
    g.edges()
        .iter()
        .any(|e| e.from == (from, 0) && e.to == (to, 0))
}

/// **Bypassing a group short-circuits input[0] → output[0], and skips the interior.** With the
/// group muted, the consumer of its output slot 0 (D) reads the source of its input slot 0 (A)
/// directly — the interior (B, C) no longer reaches outside. The document graph is UNTOUCHED (the
/// interior must survive an un-mute); only the throwaway cook graph is rewired. FALSIFIED by not
/// redirecting (D loses its input), by not cutting C -> D (the interior still feeds out), or by
/// editing `doc.graph` instead of the clone.
#[test]
fn bypassing_a_group_passes_input0_to_output0_and_skips_the_interior() {
    let (mut m, sid, a, _b, c, d) = chain_through_a_group();

    // Not muted: the cook sees the document graph byte for byte (the common case).
    assert!(cook_graph(&m).is_none(), "an un-muted graph is not rewired");

    m.doc.set_subgraph_bypassed(sid, true);
    let g = cook_graph(&m).expect("a bypassed group rewires the cook graph");
    assert!(
        has_edge(&g, a, d),
        "D now reads A -- input[0] passes through"
    );
    assert!(!has_edge(&g, c, d), "the interior (C) no longer feeds D");

    // The DOCUMENT is intact: the interior wiring survives, ready for an un-mute.
    assert!(
        has_edge(&m.doc.graph, c, d) && !has_edge(&m.doc.graph, a, d),
        "the document graph is untouched -- only the cook clone is rewired"
    );

    // Un-mute: back to the byte-identical common case.
    m.doc.set_subgraph_bypassed(sid, false);
    assert!(cook_graph(&m).is_none(), "un-muting stops rewiring");
}

/// **Only output slot 0 passes through; every other boundary output goes Empty** (Houdini's
/// `bypass_outputs`, one level up). The group{B,C} is fed by A into both; B feeds E, C feeds D.
/// Slots sort by `(node, port)`, so slot 0 is B's output — E reads A — and C's output is slot 1,
/// so D is left with NOTHING. FALSIFIED by not reconnecting slot 0 (E loses A), or by reconnecting
/// EVERY output to the source (D would wrongly read A too — the "only slot 0" guard).
#[test]
fn only_output_slot_0_passes_through_the_rest_go_empty() {
    let mut m = MotionState::new();
    let a = m.doc.graph.add_node("motion.grid");
    let b = m.doc.graph.add_node("motion.clone");
    let c = m.doc.graph.add_node("motion.clone");
    let d = m.doc.graph.add_node("motion.clone");
    let e = m.doc.graph.add_node("motion.clone");
    // A feeds both interior nodes; B feeds E, C feeds D (two boundary outputs).
    for (from, to) in [(a, b), (a, c), (b, e), (c, d)] {
        m.doc
            .graph
            .connect(Edge {
                from: (from, 0),
                to: (to, 0),
                delayed: false,
            })
            .unwrap();
    }
    m.doc.subgraphs.push(Subgraph {
        id: 1,
        parent: None,
        x: 0.0,
        y: 0.0,
        title: "Rig".into(),
    });
    m.doc.members.insert(b, 1);
    m.doc.members.insert(c, 1);
    m.doc.set_subgraph_bypassed(1, true);

    let g = cook_graph(&m).expect("bypassed");
    // Slot 0 (B) passes A through to its consumer E.
    assert!(has_edge(&g, a, e), "output slot 0 (via B) passes A to E");
    // Slot 1 (C) goes Empty: D reads neither A nor C.
    assert!(
        !has_edge(&g, a, d) && !has_edge(&g, c, d),
        "output slot 1 (via C) is Empty -- only slot 0 passes through"
    );
}
