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

// ── Wiring INTO a closed group (doc 57 §5) ──────────────────────────────────

/// The two hidden ports of card 3 — one input (node 7) and one output (node 8), both of
/// the type `two_node_snapshot`'s wires speak, plus one of a type they do NOT.
fn publish_hidden() {
    use crate::snapshot::{HiddenPorts, PortChoice, set_card_hidden_ports};
    use ph2d_nodegraph::port::{Clock, Dim, Domain};
    let choice = |node, port, label: &str, domain| PortChoice {
        node,
        port,
        label: label.into(),
        category: ph2d_node_registry::NodeUiCategory::Utility,
        port_type: crate::snapshot::PortView {
            name: "p",
            domain,
            dim: Dim::Scalar,
            clock: Clock::Frame,
        },
    };
    set_card_hidden_ports(
        [(
            crate::snapshot::SUBGRAPH_VIEW_TAG | 3,
            HiddenPorts {
                inputs: vec![
                    choice(7, 1, "Lifetime: Age", Domain::Instances),
                    // Same port name, WRONG type: the drop must not offer it.
                    choice(9, 0, "Colour: RGB", Domain::Field),
                ],
                outputs: vec![choice(8, 0, "Ramp: Out", Domain::Instances)],
            },
        )]
        .into_iter()
        .collect(),
    );
}

/// **A wire dropped on a closed card asks which port inside it lands on** — and then lands
/// on the REAL port, not on the card.
///
/// This is the hole the fold leaves and has to pay for: a card's sockets ARE the wires that
/// cross it, so a *new* wire has no socket to aim at. Without this the artist has to enter
/// the group to wire anything into it, which makes a closed group a place you cannot reach.
#[test]
fn a_wire_dropped_on_a_card_offers_the_ports_hidden_inside_it() {
    publish_hidden();
    let snap = card_snapshot(); // node 1 (out) at x=0; card 3 at x=200
    let mut st = MotionGraphPanelState::default();

    // Drag out of node 1's output and drop it on the CARD'S BODY (not a socket).
    let out = GraphHitKind::SocketOut { node: 1, port: 0 };
    for (phase, x, y) in [
        (GesturePhase::Begin, 10.0, 37.0),
        (GesturePhase::Update, 210.0, 20.0),
        (GesturePhase::End, 210.0, 20.0),
    ] {
        apply_gesture(&mut st, gesture(out, phase, x, y), RECT, CENTER, &snap);
    }
    assert!(
        drain_intents().is_empty(),
        "nothing is decided until the artist names the port"
    );
    let menu = st
        .menu
        .clone()
        .expect("the drop opened the card's port menu");
    let crate::state::MenuBody::CardPorts { rows, .. } = &menu.body else {
        panic!("the popup must be the card's ports, not the node library");
    };
    assert_eq!(
        rows.iter().map(|p| p.node).collect::<Vec<_>>(),
        vec![7],
        "only the port this wire can actually feed - a list of ports it would be refused \
         on is a list of disappointments"
    );

    // Pick the row → an ordinary Connect to the real node inside. The card grows the socket
    // by derivation, from the edge that now crosses it (the shell's gate proves that half).
    let panel = crate::geom::menu_panel(&menu, rows.len(), RECT);
    let row = crate::geom::menu_row(panel, 0, 0.0);
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
        vec![GraphIntent::Connect {
            from_node: 1,
            from_port: 0,
            to_node: 7,
            to_port: 1,
        }],
        "the wire lands on the REAL port inside the group, not on the card"
    );
    assert!(st.menu.is_none(), "picking closes the popup");
}

/// The same drop, drawn BACKWARDS out of an empty input: the card answers with its hidden
/// OUTPUTS. (This gesture used to be a silent no-op on a card — the artist drags a wire
/// looking for a source, drops it on the group that has one, and gets nothing.)
#[test]
fn a_backwards_wire_dropped_on_a_card_offers_the_outputs_hidden_inside_it() {
    publish_hidden();
    let mut snap = card_snapshot();
    // Node 1 needs an INPUT to drag backwards out of; give it one, of the wires' type.
    snap.nodes[0].inputs = vec![super::tests::port(ph2d_nodegraph::port::Domain::Instances)];
    let mut st = MotionGraphPanelState::default();

    let inp = GraphHitKind::SocketIn { node: 1, port: 0 };
    for (phase, x, y) in [
        (GesturePhase::Begin, 2.0, 37.0),
        (GesturePhase::Update, 210.0, 20.0),
        (GesturePhase::End, 210.0, 20.0),
    ] {
        apply_gesture(&mut st, gesture(inp, phase, x, y), RECT, CENTER, &snap);
    }
    let menu = st.menu.clone().expect("the backwards drop opened the menu");
    let crate::state::MenuBody::CardPorts { rows, forward, .. } = &menu.body else {
        panic!("the popup must be the card's ports");
    };
    assert!(!forward, "the wire is hunting for a SOURCE");
    assert_eq!(rows.iter().map(|p| p.node).collect::<Vec<_>>(), vec![8]);

    let panel = crate::geom::menu_panel(&menu, rows.len(), RECT);
    let row = crate::geom::menu_row(panel, 0, 0.0);
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
        vec![GraphIntent::Connect {
            from_node: 8,
            from_port: 0,
            to_node: 1,
            to_port: 0,
        }],
        "the group's hidden output feeds the input that went looking for it"
    );
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
