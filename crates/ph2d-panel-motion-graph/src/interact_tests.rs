//! Unit tests for [`super`] (`interact.rs`) — extracted to a sibling module
//! (`#[path]`) so the gesture-dispatch source stays under the 600-LOC panel
//! cap. Pure relocation of the `#[cfg(test)] mod tests` block — no test changed.
use super::*;
use crate::snapshot::{GraphNodeView, GraphViewSnapshot, PortView, drain_intents};
use ph2d_a11y::NodeId as A11yNodeId;
use ph2d_editor_core::interaction::GestureMods;
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
use ph2d_nodegraph::port::{Clock, Dim, Domain};

pub(super) const RECT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 800.0,
    h: 600.0,
};
// Scene half of the split (unused by the node/socket/menu tests; a valid
// arg for `apply_gesture`).
pub(super) const CENTER: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 800.0,
    h: 300.0,
};

pub(super) fn port(domain: Domain) -> PortView {
    PortView {
        name: "p",
        domain,
        dim: Dim::Scalar,
        clock: Clock::Frame,
    }
}

/// A → B with a matching output/input (`Instances/Scalar/Frame`), B's input
/// socket 0 at screen (200, 37) under the identity view.
pub(super) fn two_node_snapshot() -> GraphViewSnapshot {
    let node = |id: u32, x: f32, ins: Vec<PortView>, outs: Vec<PortView>| GraphNodeView {
        id,
        display_name: "n".into(),
        category: NodeUiCategory::Utility,
        silhouette: NodeSilhouette::Rect,
        x,
        y: 0.0,
        inputs: ins,
        outputs: outs,
        readout: None,
        count: None,
        hot: false,
        is_sink: false,
        preview: None,
    };
    GraphViewSnapshot {
        nodes: vec![
            node(1, 0.0, vec![], vec![port(Domain::Instances)]),
            node(2, 200.0, vec![port(Domain::Instances)], vec![]),
        ],
        edges: vec![],
        backdrops: vec![],
        probe: None,
        now: 0.0,
    }
}

/// Two nodes (1 at x=0, 2 at x=600) and a backdrop that frames only the FIRST —
/// so a drag that carried node 2 as well would be caught. The region sits at the
/// canvas origin so its header is ON-screen: a header panned off-canvas is clipped
/// away by `hits` (an invisible target must never stay clickable), and the
/// click-through test below would then be measuring the clip, not the body.
pub(super) fn backdrop_snapshot() -> GraphViewSnapshot {
    let mut snap = two_node_snapshot();
    snap.nodes[1].x = 600.0;
    snap.backdrops = vec![crate::snapshot::GraphBackdropView {
        id: 9,
        x: 0.0,
        y: 0.0,
        w: 300.0,
        h: 240.0,
        color: 0,
        title: "Group".into(),
    }];
    snap
}

pub(super) fn gesture(kind: GraphHitKind, phase: GesturePhase, x: f32, y: f32) -> GraphGesture {
    GraphGesture {
        surface: A11yNodeId(0),
        kind,
        phase,
        x,
        y,
        button: PointerButton::Primary,
        mods: GestureMods::default(),
    }
}

#[test]
fn socket_drag_over_compatible_input_emits_connect() {
    let _ = drain_intents(); // isolate this test thread's intent queue
    let snap = two_node_snapshot();
    let mut st = MotionGraphPanelState::default();
    let out = GraphHitKind::SocketOut { node: 1, port: 0 };
    // Begin on A's output, drag to B's input (200, 37), release there.
    apply_gesture(
        &mut st,
        gesture(out, GesturePhase::Begin, 10.0, 37.0),
        RECT,
        CENTER,
        &snap,
    );
    apply_gesture(
        &mut st,
        gesture(out, GesturePhase::Update, 200.0, 37.0),
        RECT,
        CENTER,
        &snap,
    );
    // The live ghost snapped to a compatible target.
    assert!(matches!(
        st.interaction,
        Interaction::DrawWire {
            target: Some((2, 0, true)),
            ..
        }
    ));
    apply_gesture(
        &mut st,
        gesture(out, GesturePhase::End, 200.0, 37.0),
        RECT,
        CENTER,
        &snap,
    );
    let intents = drain_intents();
    assert_eq!(
        intents,
        vec![GraphIntent::Connect {
            from_node: 1,
            from_port: 0,
            to_node: 2,
            to_port: 0,
        }]
    );
    assert!(matches!(st.interaction, Interaction::Idle));
}

#[test]
fn socket_drag_into_empty_space_emits_nothing() {
    let _ = drain_intents();
    let snap = two_node_snapshot();
    let mut st = MotionGraphPanelState::default();
    let out = GraphHitKind::SocketOut { node: 1, port: 0 };
    apply_gesture(
        &mut st,
        gesture(out, GesturePhase::Begin, 10.0, 37.0),
        RECT,
        CENTER,
        &snap,
    );
    apply_gesture(
        &mut st,
        gesture(out, GesturePhase::Update, 500.0, 500.0),
        RECT,
        CENTER,
        &snap,
    );
    apply_gesture(
        &mut st,
        gesture(out, GesturePhase::End, 500.0, 500.0),
        RECT,
        CENTER,
        &snap,
    );
    assert!(drain_intents().is_empty());
}

#[test]
fn alt_click_on_wire_emits_disconnect() {
    let _ = drain_intents();
    let snap = two_node_snapshot();
    let mut st = MotionGraphPanelState::default();
    let handle = crate::paint::wire_handle(2, 0);
    let mut g = gesture(
        GraphHitKind::Wire { edge: handle },
        GesturePhase::Begin,
        100.0,
        37.0,
    );
    // Plain press: inert.
    apply_gesture(&mut st, g, RECT, CENTER, &snap);
    assert!(drain_intents().is_empty());
    // Alt-press: disconnect the edge into (2, 0).
    g.mods.alt = true;
    apply_gesture(&mut st, g, RECT, CENTER, &snap);
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::Disconnect {
            to_node: 2,
            to_port: 0,
        }]
    );
}

#[test]
fn delete_key_emits_delete_selection_and_is_idempotent() {
    let _ = drain_intents();
    let mut st = MotionGraphPanelState::default();
    st.selected.extend([1, 2]);
    apply_key(&mut st, GraphKey::Delete, RECT);
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::DeleteSelection { nodes: vec![1, 2] }]
    );
    assert!(st.selected.is_empty());
    // A second Delete (double-dispatch) with the now-empty selection is inert.
    apply_key(&mut st, GraphKey::Delete, RECT);
    assert!(drain_intents().is_empty());
}

#[test]
fn right_click_background_opens_menu_then_left_pick_adds_node() {
    let _ = drain_intents();
    crate::snapshot::set_current_node_catalog(vec![crate::snapshot::NodeChoice {
        type_name: "motion.grid",
        display: "Grid",
        category: NodeUiCategory::Source,
        inputs: &[],
    }]);
    let mut st = MotionGraphPanelState::default();
    // R-press opens the menu at the cursor (on Begin, movement-independent).
    let mut rc = gesture(GraphHitKind::Background, GesturePhase::Begin, 120.0, 90.0);
    rc.button = PointerButton::Secondary;
    apply_gesture(&mut st, rc, RECT, CENTER, &two_node_snapshot());
    let menu = st.add_menu.expect("menu opened");
    assert_eq!(menu.spawn, (120.0, 90.0)); // identity view → graph == screen
    // Left-click the first (only) row → AddNode at the spawn point.
    let panel = geom::add_menu_panel(&menu, 1, RECT);
    let row = geom::add_menu_row(panel, 0);
    let pick = gesture(
        GraphHitKind::Background,
        GesturePhase::Click,
        row.x + 2.0,
        row.y + 2.0,
    );
    apply_gesture(&mut st, pick, RECT, CENTER, &two_node_snapshot());
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::AddNode {
            type_name: "motion.grid",
            x: 120.0,
            y: 90.0,
        }]
    );
    assert!(st.add_menu.is_none()); // picking closes the menu
}

#[test]
fn right_press_over_a_node_opens_menu_and_release_keeps_it() {
    let _ = drain_intents();
    let mut st = MotionGraphPanelState::default();
    // A right-press whose hit resolved to a node still opens the add-menu
    // (movement-independent, over any hit) — the node is not selected/dragged.
    let mut down = gesture(
        GraphHitKind::Node { node: 7 },
        GesturePhase::Begin,
        300.0,
        150.0,
    );
    down.button = PointerButton::Secondary;
    apply_gesture(&mut st, down, RECT, CENTER, &two_node_snapshot());
    assert!(
        st.add_menu.is_some(),
        "right-press opens the menu over a node"
    );
    assert!(st.selected.is_empty(), "the node is not selected");
    // A right-release classified as End (the click drifted) must NOT dismiss.
    let mut up = gesture(
        GraphHitKind::Node { node: 7 },
        GesturePhase::End,
        305.0,
        152.0,
    );
    up.button = PointerButton::Secondary;
    apply_gesture(&mut st, up, RECT, CENTER, &two_node_snapshot());
    assert!(
        st.add_menu.is_some(),
        "the right-release keeps the menu open"
    );
}
