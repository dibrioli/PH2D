//! Subgraph gesture gates (Motion Nodes doc 57) — the panel half of the seam.
//!
//! They assert the EXACT list of intents a gesture drains, which is the only thing
//! the panel is allowed to say about the document. A gesture that pushed nothing (or
//! pushed the wrong verb) is a click the artist made and the editor threw away — and
//! it would not be a compile error anywhere.

use super::tests::{CENTER, RECT, gesture, two_node_snapshot};
use super::*;
use crate::interact::GroupVerb;
use crate::snapshot::{Crumb, GraphIntent, NodeViewKind, drain_intents};

/// The two-node scene with node 2 replaced by a COLLAPSED CARD (view id tagged), and
/// the view standing at the root.
fn card_snapshot() -> GraphViewSnapshot {
    let mut snap = two_node_snapshot();
    snap.nodes[1].id = crate::snapshot::SUBGRAPH_VIEW_TAG | 3;
    snap.nodes[1].kind = NodeViewKind::Subgraph;
    snap
}

/// The same scene seen from INSIDE subgraph 3, with the breadcrumb the shell would
/// have published.
fn inside_snapshot() -> GraphViewSnapshot {
    let mut snap = two_node_snapshot();
    snap.level = Some(3);
    snap.breadcrumb = vec![
        Crumb {
            level: None,
            title: "Root".into(),
        },
        Crumb {
            level: Some(3),
            title: "Forces".into(),
        },
    ];
    snap
}

/// **Double-click a collapsed card → go inside it.** Houdini's gesture, and the one
/// thing the stack drawn on the card is promising.
#[test]
fn double_clicking_a_card_enters_it_and_a_node_is_untouched() {
    let snap = card_snapshot();
    let card = snap.nodes[1].id;
    let mut st = MotionGraphPanelState::default();

    apply_gesture(
        &mut st,
        gesture(
            GraphHitKind::Node { node: card as u64 },
            GesturePhase::DoubleClick,
            10.0,
            10.0,
        ),
        RECT,
        CENTER,
        &snap,
    );
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::EnterSubgraph { id: 3 }],
        "the card opens, and the id it opens is the SUBGRAPH's (the tag is stripped)"
    );
    assert_eq!(st.interaction, Interaction::Idle, "the drag is disarmed");

    // The same gesture on an ordinary node says nothing at all: a node has no inside,
    // and the double-click that splices a reroute belongs to WIRES.
    apply_gesture(
        &mut st,
        gesture(
            GraphHitKind::Node { node: 1 },
            GesturePhase::DoubleClick,
            10.0,
            10.0,
        ),
        RECT,
        CENTER,
        &snap,
    );
    assert!(drain_intents().is_empty());
}

/// **Ctrl+G collapses the selection** — and with nothing selected it is inert rather
/// than minting an empty group nobody asked for.
#[test]
fn ctrl_g_groups_the_selection_and_is_inert_when_there_is_none() {
    let snap = two_node_snapshot();
    let mut st = MotionGraphPanelState::default();

    apply_key(&mut st, GraphKey::Group, RECT, &snap);
    assert!(
        drain_intents().is_empty(),
        "nothing selected, nothing to collapse"
    );

    st.selected.insert(1);
    st.selected.insert(2);
    apply_key(&mut st, GraphKey::Group, RECT, &snap);
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::GroupSelection { nodes: vec![1, 2] }]
    );
    assert!(
        st.selected.is_empty(),
        "the shell hands back the new card as the selection (it minted the id)"
    );
}

/// **Ctrl+Alt+G dissolves.** With a card selected it dissolves that one; with nothing
/// selected while standing inside a group, it dissolves the room you are in (Nuke's
/// Expand Group from the inside).
#[test]
fn ctrl_alt_g_ungroups_the_card_or_the_room_you_are_in() {
    let snap = card_snapshot();
    let card = snap.nodes[1].id;
    let mut st = MotionGraphPanelState::default();
    st.selected.insert(card);

    apply_key(&mut st, GraphKey::Ungroup, RECT, &snap);
    assert_eq!(drain_intents(), vec![GraphIntent::Ungroup { id: 3 }]);

    // Nothing selected, standing INSIDE 3 → dissolve 3.
    let mut st = MotionGraphPanelState::default();
    apply_key(&mut st, GraphKey::Ungroup, RECT, &inside_snapshot());
    assert_eq!(drain_intents(), vec![GraphIntent::Ungroup { id: 3 }]);

    // Nothing selected, at the root → nothing to dissolve.
    let mut st = MotionGraphPanelState::default();
    apply_key(&mut st, GraphKey::Ungroup, RECT, &two_node_snapshot());
    assert!(drain_intents().is_empty());
}

/// **The breadcrumb is the way out** — every crumb walks to its level, and the root
/// crumb walks all the way out.
#[test]
fn clicking_a_crumb_walks_to_that_level() {
    let snap = inside_snapshot();
    let mut st = MotionGraphPanelState::default();

    apply_gesture(
        &mut st,
        gesture(
            GraphHitKind::Chrome {
                id: crate::paint_chrome::CHROME_CRUMB_BASE,
            },
            GesturePhase::Click,
            10.0,
            10.0,
        ),
        RECT,
        CENTER,
        &snap,
    );
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::GoToLevel { level: None }],
        "the first crumb is the root"
    );

    apply_gesture(
        &mut st,
        gesture(
            GraphHitKind::Chrome {
                id: crate::paint_chrome::CHROME_CRUMB_BASE + 1,
            },
            GesturePhase::Click,
            10.0,
            10.0,
        ),
        RECT,
        CENTER,
        &snap,
    );
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::GoToLevel { level: Some(3) }]
    );
}

/// **The chip is ONE button with BOTH verbs** (Enio, smoke 2026-07-13: *"o botão de
/// agrupar deveria ser usado para desagrupar tb"*) — and it draws the icon of the verb
/// it will actually perform, so it never lies about what a press will do.
#[test]
fn the_group_chip_groups_or_ungroups_depending_on_what_is_selected() {
    let press = |st: &mut MotionGraphPanelState, snap: &GraphViewSnapshot| {
        apply_gesture(
            st,
            gesture(
                GraphHitKind::Chrome {
                    id: crate::paint_chrome::CHROME_GROUP,
                },
                GesturePhase::Click,
                10.0,
                10.0,
            ),
            RECT,
            CENTER,
            snap,
        );
    };

    // Nothing selected: the chip is INERT, and says so.
    let plain = two_node_snapshot();
    let mut st = MotionGraphPanelState::default();
    assert_eq!(super::subgraph_gesture::verb(&st), GroupVerb::Inert);
    press(&mut st, &plain);
    assert!(drain_intents().is_empty());

    // Nodes selected: it groups.
    st.selected.insert(2);
    assert_eq!(super::subgraph_gesture::verb(&st), GroupVerb::Group);
    press(&mut st, &plain);
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::GroupSelection { nodes: vec![2] }]
    );

    // A CARD selected: the same chip now ungroups it.
    let cards = card_snapshot();
    let mut st = MotionGraphPanelState::default();
    st.selected.insert(cards.nodes[1].id);
    assert_eq!(super::subgraph_gesture::verb(&st), GroupVerb::Ungroup);
    press(&mut st, &cards);
    assert_eq!(drain_intents(), vec![GraphIntent::Ungroup { id: 3 }]);
}

/// **A ghost is never rubber-banded.** It lives on another level, and a band that swept
/// one up would let a Delete reach across the boundary into a canvas the artist is not
/// even looking at.
#[test]
fn a_box_select_never_picks_up_a_ghost() {
    let mut snap = two_node_snapshot();
    snap.nodes[1].kind = NodeViewKind::Ghost;
    let view = crate::geom::View::new(RECT, crate::state::ViewState::default());
    // A band over the WHOLE canvas: it touches both cards.
    let hit = crate::geom::nodes_in_box(&snap, &view, RECT);
    assert_eq!(hit, vec![1], "the ghost is not selectable, the node is");
}
