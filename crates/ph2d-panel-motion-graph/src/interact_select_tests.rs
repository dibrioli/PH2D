//! **Selection-family gesture tests** (Ctrl+A / Select Inverse / Select Linked /
//! Ctrl-box subtract) — split from `interact_tests` for the panel LOC cap. A CHILD of
//! `interact_tests` (`#[path]`), so `use super::*` reaches its shared fixture helpers
//! (`two_node_snapshot`, `gesture`, `body_node`, `RECT`, `CENTER`) — the door stays one.

use super::*;

/// **Ctrl+A selects every node at the current level** — the universal select-all, and a backdrop
/// (a separate subject) is dropped from the selection. FALSIFIED if the verb is inert: the
/// selection stays empty and the stray backdrop survives.
#[test]
fn select_all_selects_every_node_at_this_level() {
    // A stray backdrop selection, to prove it clears.
    let mut st = MotionGraphPanelState {
        selected_backdrop: Some(9),
        ..Default::default()
    };
    apply_key(&mut st, GraphKey::SelectAll, RECT, &two_node_snapshot());
    let mut got: Vec<u32> = st.selected.iter().copied().collect();
    got.sort_unstable();
    assert_eq!(got, vec![1, 2], "every node at this level is selected");
    assert_eq!(
        st.selected_backdrop, None,
        "a backdrop is a separate subject, so it clears"
    );
}

/// **Ctrl+I inverts the level selection** — every node NOT selected becomes selected and vice
/// versa, level-scoped like Ctrl+A, and a stray backdrop (a separate subject) clears. Inverting a
/// second time returns the original set, proving it is a true flip, not a one-shot select-all.
/// FALSIFIED three ways: an inert handler leaves {1}; a select-ALL leaves {1,2}; and keeping the
/// selected side (dropping only the rest) also leaves {1}.
#[test]
fn select_invert_flips_the_level_selection() {
    let mut st = MotionGraphPanelState {
        selected_backdrop: Some(9), // a stray backdrop, to prove it clears
        ..Default::default()
    };
    st.selected.insert(1);

    apply_key(&mut st, GraphKey::SelectInvert, RECT, &two_node_snapshot());
    let got: Vec<u32> = st.selected.iter().copied().collect();
    assert_eq!(got, vec![2], "node 1 dropped out, node 2 came in");
    assert_eq!(
        st.selected_backdrop, None,
        "a backdrop is a separate subject, so it clears"
    );

    // A second invert returns the original — it is a flip, not a one-shot select-all.
    apply_key(&mut st, GraphKey::SelectInvert, RECT, &two_node_snapshot());
    let got: Vec<u32> = st.selected.iter().copied().collect();
    assert_eq!(got, vec![1], "inverting twice is a round-trip");
}

/// **Ctrl+L grows the selection to the whole connected island, and only that island** — flood-fill
/// out along edges (Select Linked). Selecting the island's SINK (node 3) proves the walk is
/// UNDIRECTED: it must travel BACKWARD along `2->3` and `1->2` to reach 2 and 1. FALSIFIED three
/// ways: an inert handler leaves the selection at {3}; a forward-only walk from a sink reaches
/// nothing; a select-ALL would drag in the unrelated island {4, 5}.
#[test]
fn select_linked_grows_to_the_connected_island_only() {
    use crate::snapshot::NodeViewKind;
    let edge = |from: u32, to: u32| GraphEdgeView {
        from_node: from,
        from_port: 0,
        to_node: to,
        to_port: 0,
        delayed: false,
        out_domain: Domain::Instances,
    };
    let snap = GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: (1..=5)
            .map(|id| {
                body_node(
                    id,
                    id as f32 * 100.0,
                    NodeViewKind::Node,
                    Vec::new(),
                    Vec::new(),
                )
            })
            .collect(),
        // Island A: 1 -> 2 -> 3. Island B: 4 -> 5.
        edges: vec![edge(1, 2), edge(2, 3), edge(4, 5)],
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    };
    let mut st = MotionGraphPanelState::default();
    st.selected.insert(3); // the SINK of island A
    apply_key(&mut st, GraphKey::SelectLinked, RECT, &snap);
    let mut got: Vec<u32> = st.selected.iter().copied().collect();
    got.sort_unstable();
    assert_eq!(
        got,
        vec![1, 2, 3],
        "the whole island A joined; island B (4-5) is untouched"
    );
}

/// **Ctrl-drag a box REMOVES the covered nodes from the selection** — refine a select-all / linked
/// island down without re-picking. Node 2's card is at x 200..390; a Ctrl-box over [195, 400]
/// covers it but not node 1 ([0, 190]). FALSIFIED if the subtract branch is ignored: a plain /
/// additive band would instead leave node 2 selected (or reselect only it).
#[test]
fn ctrl_box_drag_subtracts_the_covered_nodes() {
    let snap = two_node_snapshot();
    let mut st = MotionGraphPanelState::default();
    st.selected.extend([1, 2]);
    for phase in [GesturePhase::Begin, GesturePhase::Update, GesturePhase::End] {
        let (x, y) = if phase == GesturePhase::Begin {
            (195.0, 10.0)
        } else {
            (400.0, 300.0)
        };
        let mut g = gesture(GraphHitKind::Background, phase, x, y);
        g.mods.cmd = true; // Ctrl held → the band subtracts
        apply_gesture(&mut st, g, RECT, CENTER, &snap);
    }
    let mut got: Vec<u32> = st.selected.iter().copied().collect();
    got.sort_unstable();
    assert_eq!(
        got,
        vec![1],
        "node 2 (under the box) was deselected; node 1 (outside) stays"
    );
}
