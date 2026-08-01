//! Guards for the CONTEXT menus (doc 62) — right-click a node, a group card, or a backdrop.
//! `super` is `interact`. Split from `interact_menu_tests` (the add-menu scroll/search tests)
//! for the panel LOC cap; both are children of `interact`, so `super::menu`, `super::tests` and
//! the re-exported `resolve_menu` / `geom` resolve the same from here.

use super::tests::{RECT, gesture, two_node_snapshot};
use super::*;
use crate::snapshot::{GraphIntent, drain_intents};
use crate::state::MenuBody;

/// **Right-clicking a node opens its actions, and a pick runs the keyboard verb** (doc 62) —
/// the missing case of the context-dependent right-press (backdrop → tints, wire → splice,
/// node → actions). Opening SELECTS the node it asks about; picking a row dispatches through
/// the SAME `apply_key` the shortcut uses, so the menu cannot drift from the key. FALSIFIED:
/// the right-press not opening node-actions (or not selecting the node), or a row running the
/// wrong verb (e.g. the Delete row not deleting).
#[test]
fn right_clicking_a_node_opens_its_actions_and_a_pick_runs_the_verb() {
    use crate::state::NodeAction;
    let _ = drain_intents();
    let snap = two_node_snapshot();
    let mut st = MotionGraphPanelState::default();

    // Right-press a node that is NOT selected: opens the node-actions menu AND selects it.
    let mut g = gesture(
        GraphHitKind::Node { node: 1 },
        GesturePhase::Begin,
        RECT.x + 30.0,
        RECT.y + 30.0,
    );
    g.button = PointerButton::Secondary;
    super::menu::open_on_right_press(&mut st, g, RECT, &snap);
    assert!(
        matches!(
            st.menu.as_ref().map(|m| &m.body),
            Some(MenuBody::NodeActions { multi: false, group: false })
        ),
        "a node right-press opens its actions (single subject, a plain node)"
    );
    assert_eq!(
        st.selected.iter().copied().collect::<Vec<_>>(),
        vec![1],
        "and selects the node it asked about"
    );

    // Resolve the Delete row (NodeAction index 3, single selection → all rows shown) →
    // GraphKey::Delete on the selection.
    let menu = st.menu.take().expect("the menu is open");
    let panel = geom::menu_panel(&menu, NodeAction::visible(false, false).len(), RECT);
    let del = geom::menu_row(&menu, panel, 3, 0.0);
    resolve_menu(
        &mut st,
        &menu,
        RECT,
        &snap,
        del.x + del.w * 0.5,
        del.y + del.h * 0.5,
    );
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::DeleteSelection { nodes: vec![1] }],
        "the Delete row ran the Delete verb on the selected node"
    );
}

/// **A multi-selection is not offered single-subject verbs** (Enio, smoke: *"para múltiplos
/// nós opções como rename não podem aparecer"*). Rename asks for one name; with many selected
/// there is none, so the row drops out — while the verbs that act on a whole selection stay.
/// A right-press within a multi-selection keeps it, and opens the menu in `multi` mode.
/// FALSIFIED by `requires_single` returning false (Rename would linger as an inert row).
#[test]
fn the_node_menu_hides_rename_for_a_multi_selection() {
    use crate::state::NodeAction;
    let single = NodeAction::visible(false, false);
    let multi = NodeAction::visible(true, false);
    assert!(single.contains(&NodeAction::Rename), "one node: rename is offered");
    assert!(!multi.contains(&NodeAction::Rename), "many nodes: rename drops out");
    assert!(
        multi.contains(&NodeAction::Delete) && multi.contains(&NodeAction::Bypass),
        "the verbs that act on a whole selection stay"
    );

    // Right-clicking one of several selected nodes keeps the selection → the menu is `multi`.
    let snap = two_node_snapshot();
    let mut st = MotionGraphPanelState::default();
    st.selected.extend([1, 2]);
    let mut g = gesture(
        GraphHitKind::Node { node: 1 },
        GesturePhase::Begin,
        RECT.x + 30.0,
        RECT.y + 30.0,
    );
    g.button = PointerButton::Secondary;
    super::menu::open_on_right_press(&mut st, g, RECT, &snap);
    assert!(
        matches!(
            st.menu.as_ref().map(|m| &m.body),
            Some(MenuBody::NodeActions { multi: true, group: false })
        ),
        "a right-press inside a multi-selection keeps it and opens in multi mode"
    );
}

/// **A group card's menu adds the two verbs unique to a group** — Enter (walk in) and Ungroup
/// (dissolve). A plain node's menu shows neither, so its list is unchanged; a card's menu leads
/// with Enter and tails with Ungroup, around the shared edits. Each row still routes through its
/// ONE door: Enter through `subgraph_gesture::enter` (the double-click's), Ungroup through
/// `apply_key(GraphKey::Ungroup)` (Ctrl+Alt+G's) — both on the untagged subgraph id.
/// FALSIFIED by `requires_group` returning false (a plain node would be offered them), by the
/// `group` flag never being set at open (a card's menu would omit them), or by Enter's `graph_key`
/// resolving to a verb instead of the enter door (the pick would not navigate).
#[test]
fn the_group_card_menu_adds_enter_and_ungroup() {
    use crate::state::NodeAction;

    // MODEL: a plain node is offered neither group verb; a card leads with Enter, tails with
    // Ungroup, and keeps the shared rows.
    let plain = NodeAction::visible(false, false);
    assert!(
        !plain.contains(&NodeAction::Enter) && !plain.contains(&NodeAction::Ungroup),
        "a plain node's menu has no group-only verbs"
    );
    let card_rows = NodeAction::visible(false, true);
    assert_eq!(card_rows.first(), Some(&NodeAction::Enter), "a card leads with Enter");
    assert_eq!(card_rows.last(), Some(&NodeAction::Ungroup), "a card tails with Ungroup");
    assert!(
        card_rows.contains(&NodeAction::Rename) && card_rows.contains(&NodeAction::Bypass),
        "the shared edits are still there"
    );

    // A collapsed card: node 3 as a tagged Subgraph view.
    let mut snap = two_node_snapshot();
    snap.nodes[1].id = crate::snapshot::SUBGRAPH_VIEW_TAG | 3;
    snap.nodes[1].kind = crate::snapshot::NodeViewKind::Subgraph;
    let cardid = snap.nodes[1].id;

    let open = || -> MotionGraphPanelState {
        let mut st = MotionGraphPanelState::default();
        let mut g = gesture(
            GraphHitKind::Node {
                node: cardid as u64,
            },
            GesturePhase::Begin,
            RECT.x + 30.0,
            RECT.y + 30.0,
        );
        g.button = PointerButton::Secondary;
        super::menu::open_on_right_press(&mut st, g, RECT, &snap);
        st
    };

    // SEAM: a right-press on the card opens its actions in `group` mode.
    let st0 = open();
    assert!(
        matches!(
            st0.menu.as_ref().map(|m| &m.body),
            Some(MenuBody::NodeActions { multi: false, group: true })
        ),
        "a right-press on a card opens its actions in group mode"
    );

    // The Enter row (index 0) walks INTO the subgraph — the untagged id, the double-click's door.
    let mut st = open();
    let _ = drain_intents();
    let menu = st.menu.take().expect("the menu is open");
    let rows = NodeAction::visible(false, true).len();
    let panel = geom::menu_panel(&menu, rows, RECT);
    let r0 = geom::menu_row(&menu, panel, 0, 0.0);
    resolve_menu(&mut st, &menu, RECT, &snap, r0.x + r0.w * 0.5, r0.y + r0.h * 0.5);
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::EnterSubgraph { id: 3 }],
        "the Enter row entered the subgraph"
    );

    // The Ungroup row (last) dissolves it — the untagged id, Ctrl+Alt+G's door.
    let mut st = open();
    let _ = drain_intents();
    let menu = st.menu.take().expect("the menu is open");
    let panel = geom::menu_panel(&menu, rows, RECT);
    let rl = geom::menu_row(&menu, panel, rows - 1, 0.0);
    resolve_menu(&mut st, &menu, RECT, &snap, rl.x + rl.w * 0.5, rl.y + rl.h * 0.5);
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::Ungroup { id: 3 }],
        "the Ungroup row dissolved the group"
    );
}

/// **A backdrop's menu offers Rename and Delete, not just its colour** (doc 62). A backdrop is a
/// labelled object, but its right-click used to be a bare tint palette; F2 renamed it and Del
/// removed it with no menu presence. Now the two verbs are appended AFTER the swatches, routed
/// through the SAME `apply_key` the keys use — the tints still set their colour by index, so the
/// palette is unchanged. FALSIFIED by dropping the appended rows (the two verbs vanish), by the
/// resolve boundary being wrong (the Rename row would set a colour), or by either verb pointing at
/// the wrong `apply_key` (Rename would not arm the box / Delete would not remove the backdrop).
#[test]
fn the_backdrop_menu_offers_rename_and_delete() {
    use crate::snapshot::{RenameTarget, menu_rows};
    let snap = super::tests::backdrop_snapshot();
    let bd = 9u32;

    let open = || -> MotionGraphPanelState {
        let mut st = MotionGraphPanelState::default();
        let mut g = gesture(
            GraphHitKind::Backdrop { id: bd as u64 },
            GesturePhase::Begin,
            RECT.x + 10.0,
            RECT.y + 10.0,
        );
        g.button = PointerButton::Secondary;
        super::menu::open_on_right_press(&mut st, g, RECT, &snap);
        st
    };
    let resolve_row = |st: &mut MotionGraphPanelState, i: usize| {
        let menu = st.menu.take().expect("the backdrop menu is open");
        let n = menu_rows(&snap, &menu).len();
        let panel = geom::menu_panel(&menu, n, RECT);
        let r = geom::menu_row(&menu, panel, i, 0.0);
        resolve_menu(st, &menu, RECT, &snap, r.x + r.w * 0.5, r.y + r.h * 0.5);
    };

    // PRESENCE: the eight tint swatches, then Rename, then Delete.
    let acts = crate::state::BACKDROP_ACTIONS;
    let st = open();
    let menu = st.menu.as_ref().expect("the backdrop menu is open");
    let rows = menu_rows(&snap, menu);
    assert_eq!(rows.len(), 8 + acts.len(), "eight tints, then the two actions");
    assert_eq!(rows[rows.len() - 2].label, acts[0].0, "Rename after the tints");
    assert_eq!(rows[rows.len() - 1].label, acts[1].0, "Delete last");

    // REGRESSION: a tint row still sets that colour by its index.
    let mut st = open();
    let _ = drain_intents();
    resolve_row(&mut st, 2);
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::SetBackdropColor { id: bd, color: 2 }],
        "a tint row still sets its colour"
    );

    // Rename (the row after the tints) arms the box on THIS backdrop — no edit yet.
    let mut st = open();
    let _ = drain_intents();
    resolve_row(&mut st, 8);
    assert_eq!(
        st.rename.as_ref().map(|r| r.target),
        Some(RenameTarget::Backdrop(bd)),
        "the Rename row armed the box on the backdrop"
    );
    assert!(drain_intents().is_empty(), "arming the rename box is not itself an edit");

    // Delete (the last row) removes the backdrop.
    let mut st = open();
    let _ = drain_intents();
    resolve_row(&mut st, 9);
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::DeleteBackdrop { id: bd }],
        "the Delete row removed the backdrop"
    );
}

