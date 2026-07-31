//! Interaction tests — **the wire dropped on a node BODY** (both directions) and the
//! helper that builds a body-sized card. Split from `interact_tests` for the panel LOC
//! cap, the same cut `interact_f2_tests` made; `super` is `interact`, so the shared
//! fixtures come in with it.

use super::tests::{CENTER, RECT, body_node, gesture, port};
use super::*;
use crate::snapshot::{GraphEdgeView, GraphViewSnapshot, PortView, drain_intents};
use ph2d_nodegraph::port::{Clock, Dim, Domain};

/// **A wire dropped on a node's BODY connects to its first FREE, type-compatible input** — the
/// forgiving "drop on the node" of Blender / Nuke, so the artist need not hit the exact socket
/// (doc 63.3). B has three inputs: input0 compatible but OCCUPIED, input1 free but a DIFFERENT
/// type, input2 free + compatible — the drop must skip the first two. FALSIFIED two ways at once:
/// ignoring occupancy lands on input0, ignoring the type check lands on input1.
#[test]
fn a_wire_dropped_on_a_node_body_takes_its_first_free_compatible_input() {
    let _ = drain_intents();
    use crate::snapshot::NodeViewKind;
    let incompat = PortView {
        name: "p",
        domain: Domain::Instances,
        dim: Dim::Vec2, // differs from the source's Scalar → not compatible
        clock: Clock::Frame,
    };
    let snap = GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![
            body_node(
                1,
                0.0,
                NodeViewKind::Node,
                vec![],
                vec![port(Domain::Instances)],
            ),
            body_node(
                2,
                200.0,
                NodeViewKind::Node,
                vec![port(Domain::Instances), incompat, port(Domain::Instances)],
                vec![],
            ),
        ],
        // input0 is occupied by a wire from some node.
        edges: vec![GraphEdgeView {
            from_node: 3,
            from_port: 0,
            to_node: 2,
            to_port: 0,
            delayed: false,
            out_domain: Domain::Instances,
        }],
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    };
    let mut st = MotionGraphPanelState::default();
    let out = GraphHitKind::SocketOut { node: 1, port: 0 };
    apply_gesture(
        &mut st,
        gesture(out, GesturePhase::Begin, 10.0, 37.0),
        RECT,
        CENTER,
        &snap,
    );
    // Drop on B's BODY (295, 50): 95 px from any socket, well outside the 22 px magnet.
    apply_gesture(
        &mut st,
        gesture(out, GesturePhase::Update, 295.0, 50.0),
        RECT,
        CENTER,
        &snap,
    );
    apply_gesture(
        &mut st,
        gesture(out, GesturePhase::End, 295.0, 50.0),
        RECT,
        CENTER,
        &snap,
    );
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::Connect {
            from_node: 1,
            from_port: 0,
            to_node: 2,
            to_port: 2,
        }],
        "the drop skipped the occupied input0 and the incompatible input1, landing on input2"
    );
}

/// **The node-body drop skips a collapsed CARD and the wire's own SOURCE** — a card's hidden
/// ports go through the port menu (doc 57), and dropping a wire on the node it came FROM would
/// self-connect. FALSIFIED by dropping either guard: without the source exclusion the drop lands
/// on the source's own first input; without the Subgraph skip it lands inside the card.
#[test]
fn the_node_body_drop_skips_a_collapsed_card_and_its_own_source() {
    use crate::snapshot::NodeViewKind;
    let snap = GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![
            // Source (id 1) has a free compatible input — so ONLY the exclusion stops a self-drop.
            body_node(
                1,
                0.0,
                NodeViewKind::Node,
                vec![port(Domain::Instances)],
                vec![port(Domain::Instances)],
            ),
            // A collapsed card (id 2) with a free compatible input.
            body_node(
                2,
                200.0,
                NodeViewKind::Subgraph,
                vec![port(Domain::Instances)],
                Vec::new(),
            ),
        ],
        edges: Vec::new(),
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    };
    let view = View::new(RECT, crate::state::ViewState::default());
    assert_eq!(
        drop_gesture::node_body_target(&snap, &view, 1, 0, 95.0, 50.0),
        None,
        "dropping a wire on its own source's body is not a self-connect"
    );
    assert_eq!(
        drop_gesture::node_body_target(&snap, &view, 1, 0, 295.0, 50.0),
        None,
        "a collapsed card is left to its port menu, not the body drop"
    );
}

/// **A backward wire (dragged out of an empty input) dropped on a node's BODY takes that node's
/// first compatible OUTPUT** — the mirror of the forward drop-on-node, so a backward drag is as
/// forgiving as a forward one. Node 2's output socket is at x=390; a drop at (295, 30) is 95 px
/// away, outside the magnet, but inside its card body. FALSIFIED if the node-body branch is
/// missing: the backward drop over a plain node is a silent no-op.
#[test]
fn a_backward_wire_dropped_on_a_node_body_takes_its_first_compatible_output() {
    let _ = drain_intents();
    use crate::snapshot::NodeViewKind;
    let snap = GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![
            // Node 1: an EMPTY input to drag a wire backwards out of.
            body_node(
                1,
                0.0,
                NodeViewKind::Node,
                vec![port(Domain::Instances)],
                Vec::new(),
            ),
            // Node 2: a compatible OUTPUT — the backward wire lands on its body.
            body_node(
                2,
                200.0,
                NodeViewKind::Node,
                Vec::new(),
                vec![port(Domain::Instances)],
            ),
        ],
        edges: Vec::new(),
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    };
    let mut st = MotionGraphPanelState::default();
    let in_sock = GraphHitKind::SocketIn { node: 1, port: 0 };
    apply_gesture(
        &mut st,
        gesture(in_sock, GesturePhase::Begin, 0.0, 37.0),
        RECT,
        CENTER,
        &snap,
    );
    // Drop on node 2's BODY (295, 30) — 95 px from its output socket, outside the magnet.
    apply_gesture(
        &mut st,
        gesture(in_sock, GesturePhase::Update, 295.0, 30.0),
        RECT,
        CENTER,
        &snap,
    );
    apply_gesture(
        &mut st,
        gesture(in_sock, GesturePhase::End, 295.0, 30.0),
        RECT,
        CENTER,
        &snap,
    );
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::Connect {
            from_node: 2,
            from_port: 0,
            to_node: 1,
            to_port: 0,
        }],
        "the backward wire took node 2's first compatible output"
    );
}
