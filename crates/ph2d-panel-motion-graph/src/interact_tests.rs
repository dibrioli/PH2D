//! Unit tests for [`super`] (`interact.rs`) — extracted to a sibling module
//! (`#[path]`) so the gesture-dispatch source stays under the 600-LOC panel
//! cap. Pure relocation of the `#[cfg(test)] mod tests` block — no test changed.
use super::*;
use crate::snapshot::{GraphNodeView, GraphViewSnapshot, PortView, drain_intents};
use ph2d_a11y::NodeId as A11yNodeId;
use ph2d_editor_core::interaction::GestureMods;
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
use ph2d_nodegraph::port::{Clock, Dim, Domain};

const RECT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 800.0,
    h: 600.0,
};
// Scene half of the split (unused by the node/socket/menu tests; a valid
// arg for `apply_gesture`).
const CENTER: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 800.0,
    h: 300.0,
};

fn port(domain: Domain) -> PortView {
    PortView {
        name: "p",
        domain,
        dim: Dim::Scalar,
        clock: Clock::Frame,
    }
}

/// A → B with a matching output/input (`Instances/Scalar/Frame`), B's input
/// socket 0 at screen (200, 37) under the identity view.
fn two_node_snapshot() -> GraphViewSnapshot {
    let node = |id: u32, x: f32, ins: Vec<PortView>, outs: Vec<PortView>| GraphNodeView {
        id,
        display_name: "n".into(),
        category: NodeUiCategory::Utility,
        silhouette: NodeSilhouette::Rect,
        x,
        y: 0.0,
        inputs: ins,
        outputs: outs,
    };
    GraphViewSnapshot {
        nodes: vec![
            node(1, 0.0, vec![], vec![port(Domain::Instances)]),
            node(2, 200.0, vec![port(Domain::Instances)], vec![]),
        ],
        edges: vec![],
        backdrops: vec![],
    }
}

/// Two nodes (1 at x=0, 2 at x=600) and a backdrop that frames only the FIRST —
/// so a drag that carried node 2 as well would be caught. The region sits at the
/// canvas origin so its header is ON-screen: a header panned off-canvas is clipped
/// away by `hits` (an invisible target must never stay clickable), and the
/// click-through test below would then be measuring the clip, not the body.
fn backdrop_snapshot() -> GraphViewSnapshot {
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

fn gesture(kind: GraphHitKind, phase: GesturePhase, x: f32, y: f32) -> GraphGesture {
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

// ── Backdrops (F2) ───────────────────────────────────────────────────────────

/// **The point of a backdrop**: dragging its header carries the nodes it FRAMES —
/// and only those. Node 1 is inside the region, node 2 (at x = 600) is far
/// outside. FALSIFIED if the drag moved the region alone (a backdrop that slides
/// out from under its group), or if it swept up node 2 (a group with no edges).
#[test]
fn dragging_a_backdrop_header_carries_the_nodes_it_frames() {
    let _ = drain_intents();
    let snap = backdrop_snapshot();
    let mut st = MotionGraphPanelState::default();
    let hit = GraphHitKind::Backdrop { id: 9 };

    apply_gesture(
        &mut st,
        gesture(hit, GesturePhase::Begin, 10.0, 0.0),
        RECT,
        CENTER,
        &snap,
    );
    apply_gesture(
        &mut st,
        gesture(hit, GesturePhase::Update, 40.0, 20.0),
        RECT,
        CENTER,
        &snap,
    );
    apply_gesture(
        &mut st,
        gesture(hit, GesturePhase::End, 40.0, 20.0),
        RECT,
        CENTER,
        &snap,
    );

    assert_eq!(
        drain_intents(),
        vec![
            GraphIntent::BeginDrag,
            GraphIntent::MoveBackdrop {
                id: 9,
                dx: 30.0,
                dy: 20.0
            },
            GraphIntent::MoveNodes {
                nodes: vec![1],
                dx: 30.0,
                dy: 20.0
            },
            GraphIntent::EndDrag,
        ],
        "the region and its framed node move together, as ONE undo step"
    );
    assert_eq!(st.selected_backdrop, Some(9), "the header selects it");
}

/// **Either** bottom gripper resizes, and the intent says WHICH corner was grabbed
/// (the shell anchors the opposite edge to it). The framed nodes do NOT move — a
/// resize changes what the region covers; that is the whole point of a corner.
#[test]
fn dragging_either_gripper_resizes_without_moving_the_nodes() {
    for left in [false, true] {
        let _ = drain_intents();
        let snap = backdrop_snapshot();
        let mut st = MotionGraphPanelState::default();
        let hit = GraphHitKind::BackdropResize {
            id: crate::backdrop::resize_handle(9, left),
        };

        apply_gesture(
            &mut st,
            gesture(hit, GesturePhase::Begin, 280.0, 200.0),
            RECT,
            CENTER,
            &snap,
        );
        apply_gesture(
            &mut st,
            gesture(hit, GesturePhase::Update, 300.0, 210.0),
            RECT,
            CENTER,
            &snap,
        );
        let intents = drain_intents();
        assert_eq!(
            intents,
            vec![
                GraphIntent::BeginDrag,
                GraphIntent::ResizeBackdrop {
                    id: 9,
                    left,
                    dx: 20.0,
                    dy: 10.0
                },
            ],
            "the corner grabbed rides the intent (left = {left})"
        );
        assert!(
            !intents
                .iter()
                .any(|i| matches!(i, GraphIntent::MoveNodes { .. })),
            "a resize never drags the group along"
        );
    }
}

/// The Backdrop chip frames the SELECTION when there is one (Nuke's behaviour):
/// the emitted rect contains both selected nodes.
#[test]
fn the_backdrop_chip_wraps_the_selection() {
    let _ = drain_intents();
    let snap = backdrop_snapshot();
    let mut st = MotionGraphPanelState::default();
    st.selected.insert(1);
    st.selected.insert(2);

    apply_gesture(
        &mut st,
        gesture(
            GraphHitKind::Chrome {
                id: crate::paint_chrome::CHROME_BACKDROP,
            },
            GesturePhase::Click,
            0.0,
            0.0,
        ),
        RECT,
        CENTER,
        &snap,
    );

    match drain_intents()[..] {
        [GraphIntent::AddBackdrop { x, y, w, h }] => {
            let framed = crate::snapshot::GraphBackdropView {
                id: 0,
                x,
                y,
                w,
                h,
                color: 0,
                title: String::new(),
            };
            assert!(
                snap.nodes
                    .iter()
                    .all(|n| crate::backdrop::frames_node(&framed, n)),
                "the new region frames every selected node"
            );
        }
        ref other => panic!("expected one AddBackdrop, got {other:?}"),
    }
}

/// With nothing selected the chip drops a default block instead (the same button,
/// the second behaviour) — never a zero-size region at the origin.
#[test]
fn the_backdrop_chip_with_no_selection_drops_a_default_block() {
    let _ = drain_intents();
    let snap = backdrop_snapshot();
    let mut st = MotionGraphPanelState::default();

    apply_gesture(
        &mut st,
        gesture(
            GraphHitKind::Chrome {
                id: crate::paint_chrome::CHROME_BACKDROP,
            },
            GesturePhase::Click,
            0.0,
            0.0,
        ),
        RECT,
        CENTER,
        &snap,
    );
    match drain_intents()[..] {
        [GraphIntent::AddBackdrop { w, h, .. }] => {
            assert_eq!((w, h), (crate::backdrop::NEW_W, crate::backdrop::NEW_H));
        }
        ref other => panic!("expected one AddBackdrop, got {other:?}"),
    }
}

/// Delete is never ambiguous: node and backdrop selection are mutually exclusive,
/// so with a backdrop selected Delete removes the REGION — and the nodes it framed
/// stay (a backdrop owns nothing).
#[test]
fn delete_with_a_backdrop_selected_removes_only_the_backdrop() {
    let _ = drain_intents();
    let mut st = MotionGraphPanelState {
        selected_backdrop: Some(9),
        ..Default::default()
    };

    apply_key(&mut st, GraphKey::Delete, RECT);

    assert_eq!(drain_intents(), vec![GraphIntent::DeleteBackdrop { id: 9 }]);
    assert_eq!(st.selected_backdrop, None);
}

/// Selecting a node clears the backdrop selection (and vice-versa) — the params
/// panel shows ONE subject, and Delete must know what it is deleting.
#[test]
fn selecting_a_node_clears_the_backdrop_selection() {
    let _ = drain_intents();
    let snap = backdrop_snapshot();
    let mut st = MotionGraphPanelState {
        selected_backdrop: Some(9),
        ..Default::default()
    };

    apply_gesture(
        &mut st,
        gesture(
            GraphHitKind::Node { node: 1 },
            GesturePhase::Begin,
            5.0,
            5.0,
        ),
        RECT,
        CENTER,
        &snap,
    );
    assert_eq!(st.selected_backdrop, None, "the node took the selection");
    assert!(st.selected.contains(&1));
    let _ = drain_intents();
}

/// **The body is click-through.** Only the header and the gripper register hit
/// rects; nothing covers the middle of the region. FALSIFIED by the bug that makes
/// a grouping tool unusable: a body rect would swallow every click and box-select
/// aimed at the nodes it frames.
#[test]
fn the_backdrop_body_registers_no_hit_rect() {
    let snap = backdrop_snapshot();
    let b = &snap.backdrops[0];
    let view = View::new(RECT, crate::state::ViewState::default());
    let mut hits: Vec<(A11yNodeId, GraphHitKind, Rect)> = Vec::new();
    crate::hits::push_backdrop_hits(&mut hits, b, &view, RECT);

    assert_eq!(
        hits.len(),
        3,
        "exactly the header and the two corner grippers"
    );
    // The centre of the body — where a framed node sits — is covered by neither.
    let (cx, cy) = view.pt(b.x + b.w * 0.5, b.y + b.h * 0.5);
    assert!(
        !hits.iter().any(|(_, _, r)| r.contains(cx, cy)),
        "no hit rect covers the body: clicks reach the nodes beneath"
    );
}

// ── Buttons: middle pans, left selects (Enio, smoke 2026-07-12) ──────────────

/// **The middle button pans — from anywhere**, including over a card (the graph
/// slides under the cursor; it does not grab the node). FALSIFIED if the pan were
/// still bound to the left button, or only worked over empty canvas.
#[test]
fn a_middle_drag_pans_from_anywhere_even_over_a_card() {
    let _ = drain_intents();
    let snap = backdrop_snapshot();
    let mut st = MotionGraphPanelState::default();
    let mut g = gesture(
        GraphHitKind::Node { node: 1 },
        GesturePhase::Begin,
        100.0,
        100.0,
    );
    g.button = PointerButton::Middle;

    apply_gesture(&mut st, g, RECT, CENTER, &snap);
    g.phase = GesturePhase::Update;
    g.x = 140.0;
    g.y = 130.0;
    apply_gesture(&mut st, g, RECT, CENTER, &snap);

    assert_eq!(
        (st.view.pan_x, st.view.pan_y),
        (40.0, 30.0),
        "the view panned"
    );
    assert!(
        st.selected.is_empty(),
        "a middle-drag never selects the card"
    );
    assert!(
        drain_intents().is_empty(),
        "and never moves it (no doc edit)"
    );
}

/// **The left button rubber-band selects.** A band swept over node 1 (at x = 0)
/// but not node 2 (at x = 600) takes exactly node 1 — and the view does NOT pan,
/// which is what the left button used to do.
#[test]
fn a_left_drag_on_empty_canvas_band_selects_what_it_touches() {
    let _ = drain_intents();
    let snap = backdrop_snapshot();
    let mut st = MotionGraphPanelState::default();
    let bg = GraphHitKind::Background;

    apply_gesture(
        &mut st,
        gesture(bg, GesturePhase::Begin, 400.0, 300.0),
        RECT,
        CENTER,
        &snap,
    );
    // Sweep back over node 1's card (its rect starts at the origin).
    apply_gesture(
        &mut st,
        gesture(bg, GesturePhase::Update, 10.0, 10.0),
        RECT,
        CENTER,
        &snap,
    );
    assert!(
        matches!(st.interaction, Interaction::BoxSelect { .. }),
        "the left-drag is a band, not a pan"
    );
    apply_gesture(
        &mut st,
        gesture(bg, GesturePhase::End, 10.0, 10.0),
        RECT,
        CENTER,
        &snap,
    );

    assert_eq!(
        st.selected.iter().copied().collect::<Vec<_>>(),
        vec![1],
        "exactly the card the band touched"
    );
    assert_eq!(
        (st.view.pan_x, st.view.pan_y),
        (0.0, 0.0),
        "the left button no longer pans"
    );
}

/// Shift makes the band ADDITIVE (it unions); without Shift it replaces.
#[test]
fn shift_makes_the_band_additive() {
    let _ = drain_intents();
    let mut snap = backdrop_snapshot();
    snap.nodes[1].x = 300.0; // bring node 2 within reach of a second band
    let bg = GraphHitKind::Background;

    // Band 1 (plain): grabs node 1 only.
    let mut st = MotionGraphPanelState::default();
    for (phase, x, y) in [
        (GesturePhase::Begin, 200.0, 200.0),
        (GesturePhase::Update, 10.0, 10.0),
        (GesturePhase::End, 10.0, 10.0),
    ] {
        apply_gesture(&mut st, gesture(bg, phase, x, y), RECT, CENTER, &snap);
    }
    assert_eq!(st.selected.iter().copied().collect::<Vec<_>>(), vec![1]);

    // Band 2 over node 2, with Shift → both are selected.
    for (phase, x, y) in [
        (GesturePhase::Begin, 290.0, 5.0),
        (GesturePhase::Update, 500.0, 60.0),
        (GesturePhase::End, 500.0, 60.0),
    ] {
        let mut g = gesture(bg, phase, x, y);
        g.mods.shift = true;
        apply_gesture(&mut st, g, RECT, CENTER, &snap);
    }
    assert_eq!(
        st.selected.iter().copied().collect::<Vec<_>>(),
        vec![1, 2],
        "Shift unions instead of replacing"
    );

    // A third band, WITHOUT Shift, over nothing → the selection is replaced (empty).
    for (phase, x, y) in [
        (GesturePhase::Begin, 700.0, 500.0),
        (GesturePhase::Update, 780.0, 560.0),
        (GesturePhase::End, 780.0, 560.0),
    ] {
        apply_gesture(&mut st, gesture(bg, phase, x, y), RECT, CENTER, &snap);
    }
    assert!(
        st.selected.is_empty(),
        "a plain band replaces the selection"
    );
    let _ = drain_intents();
}
