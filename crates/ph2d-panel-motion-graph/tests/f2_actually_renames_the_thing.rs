//! **F2 names the thing** (doc 61) — through the REAL paint and the REAL keyboard.
//!
//! The sibling of `the_add_menu_actually_adds_a_node`, and it exists for the same reason: the last
//! widget this panel grew had unit tests and no *paint* test, and it shipped a box you could not
//! click ([[feedback_a_click_is_a_press_that_drifted]]). What can go wrong with a rename box is not
//! "does `arm` set the field" — it is:
//!
//! - it never paints, so there is nothing to type into;
//! - it takes the keyboard and **never gives it back**, and from then on every shortcut in the
//!   editor types a letter into a buffer nobody can see;
//! - it commits the name into the wrong one of three id spaces (a document routinely has a node 3
//!   *and* a subgraph 3 *and* a backdrop 3).
//!
//! So this gate paints the panel for real, presses F2 for real, and reads what comes out of the
//! intent channel.

use ph2d_editor_core::interaction::{
    GestureMods, GesturePhase, GraphGesture, GraphHitKind, GraphKey, InteractiveState, WidgetEvent,
};
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::screens::layout::{CenterSplit, HeroLayout};
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerButton;
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
use ph2d_panel_motion_graph::{
    GraphBackdropView, GraphIntent, GraphNodeView, GraphViewSnapshot, MotionGraphPanel,
    MotionGraphPanelState, NodeViewKind, RenameTarget, SUBGRAPH_VIEW_TAG, drain_intents,
    request_graph_selection, set_current_motion_graph,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// The layout the Motion tool actually runs under: the centre is SPLIT, and the graph gets the
/// bottom half. Without the split the graph's rect is zero-sized and its paint returns before
/// drawing anything — which is why this panel had no paint gate at all until doc 59.
fn layout() -> HeroLayout {
    HeroLayout::for_viewport_split(
        VIEWPORT,
        false,
        ph2d_editor_core::screens::layout::RAIL_W,
        CenterSplit::Horizontal {
            t: CenterSplit::T_DEFAULT,
        },
    )
}

fn card(id: u32, name: &str, kind: NodeViewKind, x: f32) -> GraphNodeView {
    GraphNodeView {
        id,
        kind,
        display_name: name.into(),
        category: NodeUiCategory::Utility,
        silhouette: NodeSilhouette::Rect,
        x,
        y: 0.0,
        inputs: Vec::new(),
        outputs: Vec::new(),
        readout: None,
        count: None,
        hot: false,
        is_sink: false,
        preview: None,
    }
}

/// A scene with one of each: a plain card (unnamed), a named card, a collapsed group card, and a
/// backdrop — the three id spaces, all holding the same number 1, which is the point.
fn scene() {
    set_current_motion_graph(Some(GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![
            card(1, "Move", NodeViewKind::Node, 0.0),
            card(2, "The Sky", NodeViewKind::Node, 260.0),
            card(
                1 | SUBGRAPH_VIEW_TAG,
                "Age & Fade",
                NodeViewKind::Subgraph,
                520.0,
            ),
        ],
        edges: Vec::new(),
        backdrops: vec![GraphBackdropView {
            id: 1,
            x: -40.0,
            y: -120.0,
            w: 900.0,
            h: 90.0,
            color: 0,
            title: "The Snow".into(),
        }],
        probe: None,
        now: 0.0,
    }));
}

/// Paint, so the panel adopts the pending selection and the view has a zoom to draw at.
fn frame(host: &mut MockPanelHost, state: &mut MotionGraphPanelState) {
    let _ = host.paint_with_layout::<MotionGraphPanel>(state, layout(), VIEWPORT);
}

/// The open box's widget id and the text in it — `None` if no box is open.
fn box_text(host: &MockPanelHost) -> Option<(ph2d_a11y::NodeId, String)> {
    let id = host.store().focus_id()?;
    match host.store().get(id) {
        Some(InteractiveState::TextInput { text, .. }) => Some((id, text.clone())),
        _ => None,
    }
}

fn press_f2(host: &mut MockPanelHost) {
    host.store_mut().push_graph_key(GraphKey::Rename);
}

/// **Select the backdrop the way a hand does** — by pressing its header. There is no shell-to-panel
/// channel for a backdrop selection (`set_graph_backdrop_selection` is the panel *publishing* its
/// own, and it overwrites whatever a test wrote there on the next paint), so a test that reached for
/// one would be testing its own variable.
fn press_backdrop_header(host: &mut MockPanelHost, id: u32) {
    host.store_mut().push_graph_gesture(GraphGesture {
        surface: ph2d_editor_core::ids::MOTION_GRAPH_PANEL,
        kind: GraphHitKind::Backdrop { id: id as u64 },
        phase: GesturePhase::Begin,
        x: 100.0,
        y: 500.0,
        button: PointerButton::Primary,
        mods: GestureMods::default(),
    });
}

/// The gesture an artist makes: select a card, press F2, type, press Enter.
#[test]
fn f2_over_a_card_opens_a_box_that_names_it() {
    scene();
    let mut host = MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    request_graph_selection(vec![1]);
    frame(&mut host, &mut state);
    let _ = drain_intents();

    press_f2(&mut host);
    frame(&mut host, &mut state);

    // The box opened, took the keyboard, and is SEEDED with what the card says today — which is
    // what the artist is looking at, so it is what they expect to be editing.
    let (id, text) = box_text(&host).expect("F2 must open a box that holds the keyboard");
    assert_eq!(
        text, "Move",
        "an unnamed card seeds with what it SAYS (its type), not with an empty field"
    );

    // Type into the box the way the dispatcher does: the STORE owns the buffer.
    host.store_mut().register(
        id,
        InteractiveState::TextInput {
            state: ph2d_editor_core::widget::TextInputState::Focused,
            text: "Up To The Sky".into(),
            caret: 13,
            selection_anchor: None,
        },
    );
    MotionGraphPanel::apply_event(&mut state, &mut host, WidgetEvent::Submit(id));

    let named = drain_intents()
        .into_iter()
        .find_map(|i| match i {
            GraphIntent::Rename { target, name } => Some((target, name)),
            _ => None,
        })
        .expect("Enter must commit the name");
    assert_eq!(named, (RenameTarget::Node(1), "Up To The Sky".to_string()));
}

/// **The keyboard comes back.** The bug this panel has already shipped once: a field that keeps
/// focus after its box is gone swallows every shortcut in the editor — `A` stops opening the
/// add-menu and starts typing an "a" into a buffer nobody can see.
#[test]
fn closing_the_box_gives_the_keyboard_back() {
    scene();
    let mut host = MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    request_graph_selection(vec![1]);
    frame(&mut host, &mut state);
    let _ = drain_intents();

    // Esc — the box goes and the name does not change.
    press_f2(&mut host);
    frame(&mut host, &mut state);
    let (id, _) = box_text(&host).expect("the box has the keyboard");
    MotionGraphPanel::apply_event(&mut state, &mut host, WidgetEvent::Cancel(id));
    frame(&mut host, &mut state);
    assert_eq!(
        host.store().focus_id(),
        None,
        "Esc closed the box but the field kept the keyboard - from here every editor shortcut \
         types a letter into it"
    );
    assert!(
        !drain_intents()
            .iter()
            .any(|i| matches!(i, GraphIntent::Rename { .. })),
        "Esc keeps the old name: it must not commit"
    );

    // …and the same after a COMMIT, which is the other way out.
    press_f2(&mut host);
    frame(&mut host, &mut state);
    let (id, _) = box_text(&host).expect("re-armed");
    MotionGraphPanel::apply_event(&mut state, &mut host, WidgetEvent::Submit(id));
    frame(&mut host, &mut state);
    assert_eq!(host.store().focus_id(), None, "Enter gives it back too");
}

/// **Three id spaces, three targets.** There is a node 1, a subgraph 1 and a backdrop 1 in this
/// scene at the same time — which is the common case, not a contrived one — so a rename that
/// carried a bare `u32` would be a coin toss about what it just renamed.
#[test]
fn the_same_id_in_three_spaces_renames_the_right_one() {
    scene();
    let mut host = MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();

    for (select, expect) in [
        (Some(1u32), RenameTarget::Node(1)),
        (Some(1 | SUBGRAPH_VIEW_TAG), RenameTarget::Subgraph(1)),
        (None, RenameTarget::Backdrop(1)),
    ] {
        match select {
            Some(id) => {
                request_graph_selection(vec![id]);
            }
            None => {
                request_graph_selection(vec![]);
                frame(&mut host, &mut state);
                press_backdrop_header(&mut host, 1);
            }
        }
        frame(&mut host, &mut state);
        let _ = drain_intents();

        press_f2(&mut host);
        frame(&mut host, &mut state);
        let (id, _) = box_text(&host).unwrap_or_else(|| panic!("no box opened for {expect:?}"));
        MotionGraphPanel::apply_event(&mut state, &mut host, WidgetEvent::Submit(id));

        let got = drain_intents()
            .into_iter()
            .find_map(|i| match i {
                GraphIntent::Rename { target, .. } => Some(target),
                _ => None,
            })
            .expect("committed");
        assert_eq!(got, expect, "the name landed in the wrong id space");
        frame(&mut host, &mut state); // hand the keyboard back before the next round
    }
}

/// A rename is a question about ONE name. Nothing selected, or many things selected, and F2 is
/// inert — rather than guessing which of the cards you meant.
#[test]
fn f2_with_no_single_subject_does_nothing() {
    scene();
    let mut host = MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();

    for selection in [vec![], vec![1, 2]] {
        request_graph_selection(selection.clone());
        frame(&mut host, &mut state);
        press_f2(&mut host);
        frame(&mut host, &mut state);
        assert!(
            box_text(&host).is_none(),
            "F2 opened a box with {} things selected - which one is it naming?",
            selection.len()
        );
    }
}

/// A card that already has a name seeds with **that**, not with its type — otherwise the second
/// rename of a node silently throws the first one away.
#[test]
fn renaming_a_named_card_seeds_with_its_name() {
    scene();
    let mut host = MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    request_graph_selection(vec![2]); // the card called "The Sky"
    frame(&mut host, &mut state);

    press_f2(&mut host);
    frame(&mut host, &mut state);
    let (_, text) = box_text(&host).expect("the box opened");
    assert_eq!(text, "The Sky");
}
