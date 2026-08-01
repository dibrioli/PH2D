//! Wire-interaction gates — alt-click Disconnect and left-click SELECTION. `super` is `interact`.
//! Split from `interact_tests` for the panel LOC cap; both are children of `interact`, so the
//! fixtures in `super::tests` resolve the same from here.

use super::tests::{CENTER, RECT, gesture, two_node_snapshot};
use super::*;
use crate::snapshot::{GraphIntent, drain_intents};
use std::collections::BTreeSet;

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
    // A plain press now SELECTS the wire (it used to be inert) — view-state, so no intent.
    apply_gesture(&mut st, g, RECT, CENTER, &snap);
    assert!(
        drain_intents().is_empty(),
        "selecting a wire is not a doc edit"
    );
    assert_eq!(
        st.selected_wires,
        BTreeSet::from([(2, 0)]),
        "the plain press selected the wire"
    );
    // Alt-press: disconnect the edge into (2, 0). The alt arm is tried first, so it wins over
    // the plain-press select arm.
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

/// **A left-click selects a wire, Delete disconnects it, and the three selections are exclusive.**
/// A plain press on a wire (was inert) makes it the sole subject; `Delete` emits exactly the
/// alt-click `Disconnect` (the wire's unique target input) and clears it; and selecting a wire
/// clears any node/backdrop selection and vice versa (one subject at a time). This is the visible
/// click-then-Delete affordance the alt-click never had. FALSIFIED by the plain press not
/// selecting, by Delete not routing the wire to Disconnect, or by the selections not clearing
/// each other.
#[test]
fn a_left_click_selects_a_wire_and_delete_disconnects_it() {
    let _ = drain_intents();
    let snap = two_node_snapshot();
    let press_wire = |st: &mut MotionGraphPanelState| {
        let g = gesture(
            GraphHitKind::Wire {
                edge: crate::paint::wire_handle(2, 0),
            },
            GesturePhase::Begin,
            100.0,
            37.0,
        );
        apply_gesture(st, g, RECT, CENTER, &snap);
    };

    // Plain press selects the wire; Delete disconnects exactly it and clears the selection.
    let mut st = MotionGraphPanelState::default();
    press_wire(&mut st);
    assert_eq!(
        st.selected_wires,
        BTreeSet::from([(2, 0)]),
        "the plain press selected the wire"
    );
    apply_key(&mut st, GraphKey::Delete, RECT, &snap);
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::Disconnect {
            to_node: 2,
            to_port: 0,
        }],
        "Delete on a selected wire is the alt-click Disconnect"
    );
    assert!(
        st.selected_wires.is_empty(),
        "Delete cleared the wire selection"
    );

    // Selecting a wire clears a node selection...
    let mut st = MotionGraphPanelState::default();
    st.selected.extend([1, 2]);
    press_wire(&mut st);
    assert!(
        st.selected.is_empty() && st.selected_wires == BTreeSet::from([(2, 0)]),
        "the wire is now the sole subject"
    );

    // ...and pressing a NODE clears the wire.
    let node = gesture(
        GraphHitKind::Node { node: 1 },
        GesturePhase::Begin,
        30.0,
        30.0,
    );
    apply_gesture(&mut st, node, RECT, CENTER, &snap);
    assert!(
        st.selected_wires.is_empty() && st.selected.contains(&1),
        "the node is now the sole subject"
    );
}

/// **Shift builds a multi-wire selection, Shift toggles a wire back off, and Delete disconnects
/// EVERY selected wire in one gesture.** A plain press replaces the whole set with the pressed
/// wire; Shift adds/removes; and Delete drains the set to one `Disconnect` apiece (a bundle of
/// edges dropped together, one undo step). FALSIFIED by Shift not accumulating (the set never
/// grows past one — the single-wire model), or by Delete disconnecting only the first wire.
#[test]
fn shift_click_builds_a_multi_wire_selection_and_delete_disconnects_all() {
    let _ = drain_intents();
    let snap = two_node_snapshot();
    let mut st = MotionGraphPanelState::default();
    // The selection is keyed on the wire's TARGET (decoded from the handle), so distinct targets
    // are distinct wires regardless of the fixture's edge list.
    let press = |st: &mut MotionGraphPanelState, wire: (u32, u16), shift: bool| {
        let mut g = gesture(
            GraphHitKind::Wire {
                edge: crate::paint::wire_handle(wire.0, wire.1),
            },
            GesturePhase::Begin,
            100.0,
            37.0,
        );
        g.mods.shift = shift;
        apply_gesture(st, g, RECT, CENTER, &snap);
    };

    // Plain press selects one; Shift-press ADDS a second (the set grows past one).
    press(&mut st, (2, 0), false);
    press(&mut st, (3, 0), true);
    assert_eq!(
        st.selected_wires,
        BTreeSet::from([(2, 0), (3, 0)]),
        "Shift built a two-wire selection"
    );

    // Shift-pressing an already-selected wire TOGGLES it back off (the node idiom).
    press(&mut st, (3, 0), true);
    assert_eq!(
        st.selected_wires,
        BTreeSet::from([(2, 0)]),
        "Shift toggled the second wire off"
    );

    // Rebuild the pair and Delete: one Disconnect per selected wire, then the set clears.
    press(&mut st, (3, 0), true);
    apply_key(&mut st, GraphKey::Delete, RECT, &snap);
    let mut got = drain_intents();
    got.sort_by_key(|i| match i {
        GraphIntent::Disconnect { to_node, .. } => *to_node,
        _ => u32::MAX,
    });
    assert_eq!(
        got,
        vec![
            GraphIntent::Disconnect {
                to_node: 2,
                to_port: 0,
            },
            GraphIntent::Disconnect {
                to_node: 3,
                to_port: 0,
            },
        ],
        "Delete disconnected EVERY selected wire"
    );
    assert!(
        st.selected_wires.is_empty(),
        "Delete cleared the multi-wire selection"
    );

    // A PLAIN press replaces the whole set with just the pressed wire.
    press(&mut st, (2, 0), false);
    press(&mut st, (3, 0), true);
    press(&mut st, (2, 0), false);
    assert_eq!(
        st.selected_wires,
        BTreeSet::from([(2, 0)]),
        "a plain press replaced the multi-selection"
    );
}
