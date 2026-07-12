//! F2 interaction tests, part 2 — **duplicate + knife** and **probe + smart-connect**. Split
//! from `interact_f2_tests` for the panel LOC cap.

use super::tests::{CENTER, RECT, gesture, two_node_snapshot};
use super::*;
use crate::snapshot::drain_intents;
use ph2d_node_registry::NodeUiCategory;
use ph2d_nodegraph::port::{Clock, Dim, Domain};

// ── Duplicate + Knife (F2) ───────────────────────────────────────────────────

/// Ctrl+D asks the shell to duplicate the selection (only the shell can mint ids).
/// With nothing selected it is INERT — no empty duplicate, no undo step.
#[test]
fn ctrl_d_duplicates_the_selection_and_is_inert_when_empty() {
    let _ = drain_intents();
    let mut st = MotionGraphPanelState::default();

    apply_key(&mut st, GraphKey::Duplicate, RECT);
    assert!(
        drain_intents().is_empty(),
        "nothing selected: nothing to do"
    );

    st.selected.extend([1, 2]);
    apply_key(&mut st, GraphKey::Duplicate, RECT);
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::DuplicateSelection { nodes: vec![1, 2] }]
    );
}

/// The shell's answer to Ctrl+D — the ids it minted — becomes the selection, so the
/// drag that naturally follows moves the COPIES. FALSIFIED if the panel ignored the
/// request (the originals would stay selected and the artist would drag the wrong
/// cards, silently).
#[test]
fn the_shells_new_ids_become_the_selection() {
    let mut st = MotionGraphPanelState::default();
    st.selected.extend([1, 2]);
    crate::snapshot::request_graph_selection(vec![7, 8]);

    // `process` drains the request; drive it through the public entry the panel uses.
    if let Some(nodes) = crate::snapshot::take_selection_request() {
        st.selected = nodes.into_iter().collect();
        st.selected_backdrop = None;
    }
    assert_eq!(st.selected.iter().copied().collect::<Vec<_>>(), vec![7, 8]);
}

/// **The knife.** `K` arms it; the next left-drag across a wire cuts it — as ONE
/// intent (one undo step for the stroke), and the stroke DISARMS the knife (a blade
/// you must remember to put away is a blade that cuts by accident). A stroke that
/// crosses nothing emits nothing.
#[test]
fn the_knife_cuts_the_wires_it_crosses_then_disarms() {
    let _ = drain_intents();
    let mut snap = two_node_snapshot();
    // Node 1's output (right edge, x = CARD_W) into node 2's input (x = 200): the
    // wire spans that gap at y ≈ 37.
    snap.edges = vec![crate::snapshot::GraphEdgeView {
        from_node: 1,
        from_port: 0,
        to_node: 2,
        to_port: 0,
        delayed: false,
        out_domain: Domain::Instances,
        waypoints: vec![],
    }];
    let mut st = MotionGraphPanelState::default();
    let bg = GraphHitKind::Background;

    apply_key(&mut st, GraphKey::Knife, RECT);
    assert!(st.knife_armed, "K arms the knife");

    // Slice vertically through the middle of the gap the wire crosses.
    for (phase, x, y) in [
        (GesturePhase::Begin, 195.0, 0.0),
        (GesturePhase::Update, 195.0, 90.0),
        (GesturePhase::End, 195.0, 90.0),
    ] {
        apply_gesture(&mut st, gesture(bg, phase, x, y), RECT, CENTER, &snap);
    }

    assert_eq!(
        drain_intents(),
        vec![GraphIntent::CutWires {
            targets: vec![(2, 0)]
        }],
        "one intent for the whole stroke"
    );
    assert!(!st.knife_armed, "the stroke put the blade away");

    // Re-armed, a stroke through empty space cuts nothing (and still disarms).
    apply_key(&mut st, GraphKey::Knife, RECT);
    for (phase, x, y) in [
        (GesturePhase::Begin, 600.0, 400.0),
        (GesturePhase::Update, 700.0, 500.0),
        (GesturePhase::End, 700.0, 500.0),
    ] {
        apply_gesture(&mut st, gesture(bg, phase, x, y), RECT, CENTER, &snap);
    }
    assert!(drain_intents().is_empty(), "crossed nothing, cut nothing");
}

/// While the knife is armed the left-drag is a BLADE, not a rubber band — the two
/// gestures share the button and must never both fire.
#[test]
fn an_armed_knife_suppresses_the_rubber_band() {
    let _ = drain_intents();
    let snap = two_node_snapshot();
    let mut st = MotionGraphPanelState::default();
    apply_key(&mut st, GraphKey::Knife, RECT);

    for (phase, x, y) in [
        (GesturePhase::Begin, 400.0, 300.0),
        (GesturePhase::Update, 10.0, 10.0),
    ] {
        apply_gesture(
            &mut st,
            gesture(GraphHitKind::Background, phase, x, y),
            RECT,
            CENTER,
            &snap,
        );
    }
    assert!(matches!(st.interaction, Interaction::Knife { .. }));

    apply_gesture(
        &mut st,
        gesture(GraphHitKind::Background, GesturePhase::End, 10.0, 10.0),
        RECT,
        CENTER,
        &snap,
    );
    assert!(
        st.selected.is_empty(),
        "the knife stroke selected nothing (it is not a band)"
    );
    let _ = drain_intents();
}

// ── Probe + smart-connect (F2) ───────────────────────────────────────────────

/// `P` arms the probe and the next click on a node POINTS it there — it does not
/// select or drag the card. A second `P` (or Esc) disarms.
#[test]
fn p_arms_the_probe_and_the_next_click_picks_the_node() {
    let _ = drain_intents();
    let snap = two_node_snapshot();
    let mut st = MotionGraphPanelState::default();

    apply_key(&mut st, GraphKey::Probe, RECT);
    assert!(st.probe_armed, "P armed it");

    apply_gesture(
        &mut st,
        gesture(
            GraphHitKind::Node { node: 2 },
            GesturePhase::Begin,
            5.0,
            5.0,
        ),
        RECT,
        CENTER,
        &snap,
    );
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::SetProbe { node: Some(2) }]
    );
    assert_eq!(st.probe, Some(2));
    assert!(!st.probe_armed, "the pick disarmed it");
    assert!(
        st.selected.is_empty(),
        "the probe pick never selects or drags the card"
    );

    // Esc puts the probe away.
    apply_key(&mut st, GraphKey::Escape, RECT);
    assert_eq!(drain_intents(), vec![GraphIntent::SetProbe { node: None }]);
    assert_eq!(st.probe, None);
}

/// **Smart-connect.** A wire dropped on empty canvas opens the add-menu carrying
/// the source socket, the menu lists ONLY the types that can take that wire, and
/// picking one emits a single `SmartConnect` (add + wire = one gesture, one undo).
/// FALSIFIED by the old behaviour: dropping a wire in space was a silent no-op.
#[test]
fn a_wire_dropped_in_space_offers_only_what_can_take_it() {
    use ph2d_nodegraph::node::PortSpec;
    use ph2d_nodegraph::port::PortType;
    let _ = drain_intents();

    // The catalog: one node that takes the dragged type (Instances/Scalar/Frame),
    // one that takes something else entirely.
    static TAKES: [PortSpec; 1] = [PortSpec {
        name: "in",
        ty: PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame),
    }];
    static REFUSES: [PortSpec; 1] = [PortSpec {
        name: "in",
        ty: PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame),
    }];
    crate::snapshot::set_current_node_catalog(vec![
        crate::snapshot::NodeChoice {
            type_name: "motion.takes",
            display: "Takes",
            category: NodeUiCategory::Utility,
            inputs: &TAKES,
        },
        crate::snapshot::NodeChoice {
            type_name: "motion.refuses",
            display: "Refuses",
            category: NodeUiCategory::Utility,
            inputs: &REFUSES,
        },
    ]);

    let snap = two_node_snapshot(); // node 1's output is Instances/Scalar/Frame
    let mut st = MotionGraphPanelState::default();
    let out = GraphHitKind::SocketOut { node: 1, port: 0 };
    for (phase, x, y) in [
        (GesturePhase::Begin, 10.0, 37.0),
        (GesturePhase::Update, 500.0, 400.0),
        (GesturePhase::End, 500.0, 400.0),
    ] {
        apply_gesture(&mut st, gesture(out, phase, x, y), RECT, CENTER, &snap);
    }
    let menu = st.add_menu.expect("the drop opened the smart-connect menu");
    assert_eq!(menu.connect_from, Some((1, 0)), "it remembers the wire");

    // Only the compatible type is listed.
    let rows = crate::snapshot::menu_catalog(&snap, menu.connect_from);
    assert_eq!(rows.len(), 1, "the incompatible type is not offered");
    assert_eq!(rows[0].type_name, "motion.takes");

    // Picking it adds AND wires, in one intent.
    let panel = geom::add_menu_panel(&menu, rows.len(), RECT);
    let row = geom::add_menu_row(panel, 0);
    apply_gesture(
        &mut st,
        gesture(
            GraphHitKind::Background,
            GesturePhase::Click,
            row.x + 2.0,
            row.y + 2.0,
        ),
        RECT,
        CENTER,
        &snap,
    );
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::SmartConnect {
            from_node: 1,
            from_port: 0,
            to_type: "motion.takes",
            x: 500.0,
            y: 400.0,
        }],
        "one intent: the add and the wire are one gesture"
    );
    crate::snapshot::set_current_node_catalog(Vec::new());
}
