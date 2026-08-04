//! Unit tests for [`super`] (`interact.rs`) — extracted to a sibling module
//! (`#[path]`) so the gesture-dispatch source stays under the 600-LOC panel
//! cap. Pure relocation of the `#[cfg(test)] mod tests` block — no test changed.
use super::*;
use crate::snapshot::{GraphEdgeView, GraphNodeView, GraphViewSnapshot, PortView, drain_intents};
use ph2d_a11y::NodeId as A11yNodeId;
use ph2d_editor_core::interaction::GestureMods;
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
use ph2d_nodegraph::port::{Clock, Dim, Domain};

// The selection-family gestures (Ctrl+A / Invert / Linked / box-subtract) live in a CHILD
// module (this file was itself at the panel LOC cap); it reads this file's fixture helpers
// through `use super::*`.
#[path = "interact_select_tests.rs"]
mod select_tests;

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
        kind: crate::snapshot::NodeViewKind::Node,
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
        bypassed: false,
        inert: false,
        thumbnail: None,
    };
    GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
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

/// A node builder with an explicit kind, for the node-body-drop gates.
pub(super) fn body_node(
    id: u32,
    x: f32,
    kind: crate::snapshot::NodeViewKind,
    ins: Vec<PortView>,
    outs: Vec<PortView>,
) -> GraphNodeView {
    GraphNodeView {
        kind,
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
        bypassed: false,
        inert: false,
        thumbnail: None,
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
fn socket_drag_into_empty_space_opens_the_palette() {
    let _ = drain_intents();
    crate::snapshot::set_current_node_catalog(Vec::new());
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
    // No local menu and no premature connect — a loose end dropped in space just asks the shell to
    // open the palette in smart-connect mode (the actual wire happens when the shell routes the pick).
    assert!(st.menu.is_none());
    let intents = drain_intents();
    assert!(
        matches!(
            intents.as_slice(),
            [GraphIntent::OpenLibrary {
                connect_from: Some((1, 0)),
                splice: None,
                ..
            }]
        ),
        "the drop opens the palette in smart-connect mode: {intents:?}"
    );
}

#[test]
fn delete_key_emits_delete_selection_and_is_idempotent() {
    let _ = drain_intents();
    let mut st = MotionGraphPanelState::default();
    st.selected.extend([1, 2]);
    apply_key(&mut st, GraphKey::Delete, RECT, &two_node_snapshot());
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::DeleteSelection { nodes: vec![1, 2] }]
    );
    assert!(st.selected.is_empty());
    // A second Delete (double-dispatch) with the now-empty selection is inert.
    apply_key(&mut st, GraphKey::Delete, RECT, &two_node_snapshot());
    assert!(drain_intents().is_empty());
}

/// **H switches the selection off, then on** (bypass/mute), by the rove scope rule: with anything
/// un-muted it mutes ALL; with everything already muted it un-mutes; a mixed selection resolves to
/// "off". Nothing selected → inert (no intent). FALSIFIED by reading `on` from a fixed side, or by
/// dropping the empty-selection guard.
#[test]
fn h_switches_the_selection_off_then_on() {
    let _ = drain_intents();
    let mut st = MotionGraphPanelState::default();
    st.selected.extend([1, 2]);

    // Nothing muted → H mutes ALL selected.
    apply_key(&mut st, GraphKey::Bypass, RECT, &two_node_snapshot());
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::SetBypass {
            nodes: vec![1, 2],
            on: true
        }]
    );

    // Both already muted → H un-mutes.
    let mut snap = two_node_snapshot();
    for n in &mut snap.nodes {
        n.bypassed = true;
    }
    apply_key(&mut st, GraphKey::Bypass, RECT, &snap);
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::SetBypass {
            nodes: vec![1, 2],
            on: false
        }]
    );

    // Mixed (one muted, one not) → H mutes ALL (the rove idiom, not a per-node toggle).
    let mut snap = two_node_snapshot();
    snap.nodes[0].bypassed = true;
    apply_key(&mut st, GraphKey::Bypass, RECT, &snap);
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::SetBypass {
            nodes: vec![1, 2],
            on: true
        }]
    );

    // Nothing selected → inert.
    st.selected.clear();
    apply_key(&mut st, GraphKey::Bypass, RECT, &two_node_snapshot());
    assert!(drain_intents().is_empty());
}

/// **Ctrl+C copies the selection; Ctrl+V pastes regardless of it** (the graph clipboard).
/// Copy carries the selection into a `CopySelection` intent and is INERT with nothing
/// selected (a copy of nothing is not an operation). Paste ALWAYS emits `Paste` — it
/// depends on what was copied, not on the current selection — so it fires even when
/// nothing is selected. FALSIFIED by copy emitting on an empty selection, or paste
/// carrying a selection guard it must not have.
#[test]
fn copy_and_paste_keys_emit_their_intents() {
    let _ = drain_intents();
    let mut st = MotionGraphPanelState::default();
    st.selected.extend([1, 2]);

    apply_key(&mut st, GraphKey::Copy, RECT, &two_node_snapshot());
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::CopySelection { nodes: vec![1, 2] }]
    );

    // Paste fires with a selection present...
    apply_key(&mut st, GraphKey::Paste, RECT, &two_node_snapshot());
    assert_eq!(drain_intents(), vec![GraphIntent::Paste]);

    // ...and, crucially, with NONE — the clipboard is what it acts on.
    st.selected.clear();
    apply_key(&mut st, GraphKey::Paste, RECT, &two_node_snapshot());
    assert_eq!(drain_intents(), vec![GraphIntent::Paste]);

    // Copy of nothing is inert (no clipboard write requested).
    apply_key(&mut st, GraphKey::Copy, RECT, &two_node_snapshot());
    assert!(drain_intents().is_empty());
}

/// **Ctrl+X cuts: copy THEN delete, in that order, one gesture.** It emits the two
/// intents it composes — `CopySelection` FIRST (so the clip captures the nodes while
/// they still exist), then `DeleteSelection` — and clears the selection like Delete.
/// The ORDER is load-bearing: swap it and the delete runs before the copy reads. Cut
/// of nothing is inert. FALSIFIED by the wrong order, by dropping either half (which
/// would make Cut a bare Copy or a bare Delete), or by dropping the empty-selection
/// guard.
#[test]
fn cut_key_emits_copy_then_delete_in_order() {
    let _ = drain_intents();
    let mut st = MotionGraphPanelState::default();
    st.selected.extend([1, 2]);

    apply_key(&mut st, GraphKey::Cut, RECT, &two_node_snapshot());
    assert_eq!(
        drain_intents(),
        vec![
            GraphIntent::CopySelection { nodes: vec![1, 2] },
            GraphIntent::DeleteSelection { nodes: vec![1, 2] },
        ],
        "cut is copy-then-delete, and the copy must run first"
    );
    assert!(
        st.selected.is_empty(),
        "cut clears the selection like delete"
    );

    // Cut of nothing is inert (double-dispatch safe, and there is nothing to carry).
    apply_key(&mut st, GraphKey::Cut, RECT, &two_node_snapshot());
    assert!(drain_intents().is_empty());
}

/// **The double-dispatch is collapsed.** The graph's keys reach the store TWICE per
/// press (focus gate + cursor router), so a non-idempotent verb like Paste would run
/// twice — two copies from one Ctrl+V, exactly the bug this fixes. An adjacent repeat
/// is the artifact and is dropped; distinct verbs and non-adjacent repeats survive.
/// FALSIFIED by dropping the collapse (a doubled Paste passes through).
#[test]
fn a_doubled_key_press_collapses_to_one() {
    // One Ctrl+V arrives as two adjacent Pastes → one paste.
    assert_eq!(
        dedup_double_dispatch(vec![GraphKey::Paste, GraphKey::Paste]),
        vec![GraphKey::Paste],
        "one press is one paste, not two"
    );
    // Distinct verbs in a frame both survive.
    assert_eq!(
        dedup_double_dispatch(vec![GraphKey::Copy, GraphKey::Paste]),
        vec![GraphKey::Copy, GraphKey::Paste]
    );
    // A repeat that is NOT adjacent (two real presses) is preserved — only the
    // back-to-back double is the artifact.
    assert_eq!(
        dedup_double_dispatch(vec![GraphKey::Paste, GraphKey::Copy, GraphKey::Paste]),
        vec![GraphKey::Paste, GraphKey::Copy, GraphKey::Paste]
    );
}

#[test]
fn right_click_background_opens_the_palette() {
    let _ = drain_intents();
    let mut st = MotionGraphPanelState::default();
    // R-press on empty canvas asks the shell to open the full-screen palette at the cursor (plain
    // library — no wire context). Movement-independent (on Begin), and NO local dropdown: the pick
    // (add at the spawn) happens shell-side when it routes the palette choice.
    let mut rc = gesture(GraphHitKind::Background, GesturePhase::Begin, 120.0, 90.0);
    rc.button = PointerButton::Secondary;
    apply_gesture(&mut st, rc, RECT, CENTER, &two_node_snapshot());
    assert!(
        st.menu.is_none(),
        "no local dropdown - the palette is the shell's"
    );
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::OpenLibrary {
            x: 120.0, // identity view → graph == screen
            y: 90.0,
            connect_from: None,
            splice: None,
            compatible: Vec::new(),
        }],
    );
}

#[test]
fn right_press_over_a_node_opens_menu_and_release_keeps_it() {
    let _ = drain_intents();
    let mut st = MotionGraphPanelState::default();
    // A right-press over a node opens its ACTIONS menu (doc 62) and SELECTS the node it
    // asks about — like the backdrop's right-press. Movement-independent (over any hit,
    // on the press), and the node is not dragged.
    let mut down = gesture(
        GraphHitKind::Node { node: 7 },
        GesturePhase::Begin,
        300.0,
        150.0,
    );
    down.button = PointerButton::Secondary;
    apply_gesture(&mut st, down, RECT, CENTER, &two_node_snapshot());
    assert!(
        matches!(
            st.menu.as_ref().map(|m| &m.body),
            Some(crate::state::MenuBody::NodeActions { .. })
        ),
        "right-press over a node opens its actions menu"
    );
    assert!(
        st.selected.contains(&7),
        "the node it asked about is selected"
    );
    // A right-release classified as End (the click drifted) must NOT dismiss.
    let mut up = gesture(
        GraphHitKind::Node { node: 7 },
        GesturePhase::End,
        305.0,
        152.0,
    );
    up.button = PointerButton::Secondary;
    apply_gesture(&mut st, up, RECT, CENTER, &two_node_snapshot());
    assert!(st.menu.is_some(), "the right-release keeps the menu open");
}

/// **The header toggle flips the stamp above/below** (doc 86) — a Click on a `PreviewToggle`
/// hit reaches the dispatch and moves this node's preview. FALSIFIED by dropping the arm: the
/// click would fall to the no-op `_` and the preview would never move.
#[test]
fn a_click_on_the_preview_toggle_flips_the_position() {
    use crate::state::PreviewPos;
    let mut st = MotionGraphPanelState::default();
    assert_eq!(st.preview_position(5), PreviewPos::Below, "starts Below");
    let g = gesture(
        GraphHitKind::PreviewToggle { node: 5 },
        GesturePhase::Click,
        10.0,
        10.0,
    );
    apply_gesture(&mut st, g, RECT, CENTER, &two_node_snapshot());
    assert_eq!(
        st.preview_position(5),
        PreviewPos::Above,
        "the click moved it up"
    );
}

/// **A Click on the ⚠ inert badge asks the shell to fix it** (ADR-0155) — the gesture
/// reaches the dispatch and pushes exactly one `FixInert` intent carrying the node. The
/// panel forwards; the shell (which has the graph) decides fix-vs-explain. FALSIFIED by
/// dropping the arm: the click would fall to the no-op `_` and nothing would be pushed.
#[test]
fn a_click_on_the_inert_badge_pushes_a_fix_intent() {
    let _ = drain_intents(); // isolate this test thread's intent queue
    let mut st = MotionGraphPanelState::default();
    let g = gesture(
        GraphHitKind::InertBadge { node: 7 },
        GesturePhase::Click,
        10.0,
        10.0,
    );
    apply_gesture(&mut st, g, RECT, CENTER, &two_node_snapshot());
    let got = drain_intents();
    assert!(
        matches!(got.as_slice(), [GraphIntent::FixInert { node: 7 }]),
        "the click pushed exactly FixInert(7), got {got:?}"
    );
}
