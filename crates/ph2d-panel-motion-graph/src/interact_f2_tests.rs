//! F2 interaction tests, part 1 — **backdrops** and the **button convention** (middle pans,
//! left selects). Split from `interact_tests` for the panel LOC cap; `super` is `interact`,
//! so the shared fixtures come in with it.

use super::tests::{CENTER, RECT, backdrop_snapshot, gesture, two_node_snapshot};
use super::*;
use crate::snapshot::drain_intents;
use ph2d_a11y::NodeId as A11yNodeId;

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

/// **The Arrange chip asks the shell to lay the graph out.** Clicking it pushes ONE
/// `ArrangeLayout` intent and nothing else — the panel never touches positions
/// itself (the shell owns the document and the undo bracket). FALSIFIED by dropping
/// the dispatch arm (the chip becomes a dead button — painted, hittable, inert).
#[test]
fn the_arrange_chip_asks_the_shell_to_lay_the_graph_out() {
    let _ = drain_intents();
    let snap = two_node_snapshot();
    let mut st = MotionGraphPanelState::default();

    apply_gesture(
        &mut st,
        gesture(
            GraphHitKind::Chrome {
                id: crate::paint_chrome::CHROME_ARRANGE,
            },
            GesturePhase::Click,
            0.0,
            0.0,
        ),
        RECT,
        CENTER,
        &snap,
    );

    assert_eq!(
        drain_intents(),
        vec![GraphIntent::ArrangeLayout],
        "the arrange chip emits exactly one layout intent"
    );
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

    apply_key(&mut st, GraphKey::Delete, RECT, &two_node_snapshot());

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
