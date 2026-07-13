//! **The add-menu, through the PAINT.** (Enio, smoke 2026-07-13: *"não consigo inserir nenhum
//! nó ao clicar no menu"* — after the search field landed.)
//!
//! Every unit test of this menu builds its state by hand and calls `apply_gesture` directly.
//! That skips `process` — which is where the search field is opened, focused and mirrored —
//! and it skips `paint` entirely. So a menu that is **unclickable in the running app** can be
//! green in all of them, which is exactly what happened.
//!
//! This drives the real panel the way the shell does: push the gestures the dispatcher would
//! push, then `paint` (which runs `process`, which drains them) against a real `WidgetStore`.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{GestureMods, GesturePhase, GraphGesture, GraphHitKind};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::layout::{CenterSplit, HeroLayout};
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerButton;
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
use ph2d_panel_motion_graph::{
    GraphIntent, GraphNodeView, GraphViewSnapshot, MotionGraphPanel, MotionGraphPanelState,
    NodeChoice, NodeViewKind, drain_intents, first_menu_row, menu_is_open,
    set_current_motion_graph, set_current_node_catalog,
};
use ph2d_ui_testkit::MockPanelHost;

use ph2d_host::{PointerEvent, PointerKind, PointerSource};

/// One second, in nanoseconds. The gestures have to be SECONDS apart or the dispatcher reads
/// two presses as a double-click (which is a different verb, and not the one being tested).
const SEC: u128 = 1_000_000_000;

/// A real pointer event, fed to the REAL dispatcher — which is the half the hand-pushed
/// gestures skip, and therefore the half that broke.
fn pointer(kind: PointerKind, button: PointerButton, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button,
        timestamp_ns: t,
    }
}

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// The layout the Motion tool actually runs under: the centre is SPLIT, and the graph gets the
/// bottom half. Without the split the graph's rect is zero-sized and its paint returns before
/// drawing anything — which is why this panel never had a paint gate.
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

fn catalog() {
    set_current_node_catalog(vec![
        NodeChoice {
            type_name: "value.lfo",
            display: "LFO",
            category: NodeUiCategory::Source,
            inputs: &[],
        },
        NodeChoice {
            type_name: "motion.grid",
            display: "Grid",
            category: NodeUiCategory::Source,
            inputs: &[],
        },
    ]);
}

fn one_node_graph() {
    set_current_motion_graph(Some(GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![GraphNodeView {
            id: 1,
            kind: NodeViewKind::Node,
            display_name: "n".into(),
            category: NodeUiCategory::Utility,
            silhouette: NodeSilhouette::Rect,
            x: 0.0,
            y: 0.0,
            inputs: Vec::new(),
            outputs: Vec::new(),
            readout: None,
            count: None,
            hot: false,
            is_sink: false,
            preview: None,
        }],
        edges: Vec::new(),
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    }));
}

fn gesture(phase: GesturePhase, button: PointerButton, x: f32, y: f32) -> GraphGesture {
    GraphGesture {
        surface: ids::MOTION_GRAPH_PANEL,
        kind: GraphHitKind::Background,
        phase,
        x,
        y,
        button,
        mods: GestureMods::default(),
    }
}

/// **Right-click the canvas, left-click a row → that node is added.** The gesture the artist
/// makes, against the panel the artist sees.
#[test]
fn clicking_a_row_of_the_add_menu_adds_that_node() {
    catalog();
    one_node_graph();
    let mut host = MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    let _ = drain_intents();

    // Frame 1 — a right-press on empty canvas opens the menu.
    host.store_mut().push_graph_gesture(gesture(
        GesturePhase::Begin,
        PointerButton::Secondary,
        400.0,
        300.0,
    ));
    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);
    assert!(menu_is_open(&state), "the right-press opened the add-menu");

    // Frame 2 — the menu is on screen. This is ALSO the frame the search field opens and takes
    // focus on, and it is the frame the artist clicks in. Left-click the first row.
    let row = first_menu_row(&state, layout().motion_graph).expect("the menu has a first row");
    let (cx, cy) = (row.x + row.w * 0.5, row.y + row.h * 0.5);
    host.store_mut().push_graph_gesture(gesture(
        GesturePhase::Begin,
        PointerButton::Primary,
        cx,
        cy,
    ));
    host.store_mut().push_graph_gesture(gesture(
        GesturePhase::Click,
        PointerButton::Primary,
        cx,
        cy,
    ));
    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);

    let intents = drain_intents();
    assert_eq!(
        intents.len(),
        1,
        "clicking a row must add a node - got {intents:?}"
    );
    assert!(
        matches!(
            intents[0],
            GraphIntent::AddNode {
                type_name: "value.lfo",
                ..
            }
        ),
        "and it must be the row that was DRAWN first: {:?}",
        intents[0]
    );
    assert!(!menu_is_open(&state), "picking a row closes the menu");
}

/// **A node card UNDER the popup does not steal the click.**
///
/// This is the bug (Enio, smoke: *"não consigo inserir nenhum nó ao clicar no menu"*). The
/// popup is DRAWN on top of the canvas but it was not HIT-TESTED on top of it: the hit index
/// still handed the dispatcher the card underneath, so the click arrived at the panel as a
/// *node* gesture — select-and-drag — and the menu never resolved. On an empty canvas it
/// worked, which is exactly why every test was green: the harness had no cards under the menu,
/// and Enio's graph is full of them.
#[test]
fn a_card_under_the_popup_does_not_steal_the_click() {
    catalog();
    one_node_graph();
    let mut host = MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    let _ = drain_intents();
    let graph = layout().motion_graph;
    let (ox, oy) = (graph.x + 60.0, graph.y + 60.0);

    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);
    host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Secondary,
        ox,
        oy,
        SEC,
    ));
    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);
    let row = first_menu_row(&state, graph).expect("the menu has a first row");
    let (cx, cy) = (row.x + row.w * 0.5, row.y + row.h * 0.5);

    // Put a node card exactly where the artist is about to click — which is what a real graph
    // looks like: the menu opens OVER the nodes.
    node_at_screen(&state, graph, cx, cy);
    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);

    host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Primary,
        cx,
        cy,
        3 * SEC,
    ));
    host.dispatch_pointer_event(pointer(
        PointerKind::Up,
        PointerButton::Primary,
        cx,
        cy,
        3 * SEC + SEC / 100,
    ));
    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);

    let intents = drain_intents();
    assert!(
        intents
            .iter()
            .any(|i| matches!(i, GraphIntent::AddNode { .. })),
        "the popup is on top, so the click belongs to the popup - got {intents:?}"
    );
}

/// **The same click, through the DISPATCHER.** The test above pushes the graph gestures by
/// hand, which is what every other test in this crate does — and it is precisely why the
/// regression was invisible: the dispatcher decides whether a click on the popup becomes a
/// graph gesture at all, by looking at what the PAINT registered in the hit index.
#[test]
fn the_dispatcher_turns_a_click_on_a_row_into_an_added_node() {
    catalog();
    one_node_graph();
    let mut host = MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    let _ = drain_intents();
    let graph = layout().motion_graph;
    let (ox, oy) = (graph.x + 60.0, graph.y + 60.0); // empty canvas, well inside the graph

    // Frame 1 — paint (so the hit index has the graph surface), then RIGHT-PRESS through the
    // dispatcher.
    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);
    host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Secondary,
        ox,
        oy,
        SEC,
    ));
    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);
    assert!(menu_is_open(&state), "the right-press opened the add-menu");

    // Frame 2 — click the first row, Down + Up, through the dispatcher.
    let row = first_menu_row(&state, graph).expect("the menu has a first row");
    let (cx, cy) = (row.x + row.w * 0.5, row.y + row.h * 0.5);
    host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Primary,
        cx,
        cy,
        3 * SEC,
    ));
    host.dispatch_pointer_event(pointer(
        PointerKind::Up,
        PointerButton::Primary,
        cx,
        cy,
        3 * SEC + SEC / 100,
    ));
    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);

    let intents = drain_intents();
    assert_eq!(
        intents.len(),
        1,
        "the click on the row must reach the panel as a graph gesture and add a node - \
         got {intents:?}"
    );
    assert!(matches!(intents[0], GraphIntent::AddNode { .. }));
}

/// Publish a graph whose one node is drawn at the given SCREEN point (so it lands under the
/// popup). The panel's view is identity-ish after the fit, so the shell's own coordinate map
/// is what decides — this reads it back from the state rather than assuming it.
fn node_at_screen(state: &MotionGraphPanelState, rect: Rect, sx: f32, sy: f32) {
    let (gx, gy) = ph2d_panel_motion_graph::graph_point(state, rect, sx, sy);
    set_current_motion_graph(Some(GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![GraphNodeView {
            id: 1,
            kind: NodeViewKind::Node,
            display_name: "under".into(),
            category: NodeUiCategory::Utility,
            silhouette: NodeSilhouette::Rect,
            x: gx - 40.0,
            y: gy - 10.0,
            inputs: Vec::new(),
            outputs: Vec::new(),
            readout: None,
            count: None,
            hot: false,
            is_sink: false,
            preview: None,
        }],
        edges: Vec::new(),
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    }));
}

/// **THE BUG** (Enio, smoke 2026-07-13: *"não consigo inserir nenhum nó ao clicar no menu"*).
///
/// A hand moves. The dispatcher calls a press-release with ANY movement between them an `End`
/// (a drag), not a `Click` — so one pixel of drift turned the row the artist pressed into a
/// drag, the menu dismissed itself, and nothing was added. **Every test in this crate sent
/// Down and Up at the same coordinate**, which is the one thing a real hand never does: the
/// gates were green and the feature was unusable.
///
/// While a menu is open the pointer belongs to the menu, so where the button comes UP is what
/// it means — over a row, that row.
#[test]
fn a_click_that_drifts_one_pixel_still_picks_the_row() {
    catalog();
    one_node_graph();
    let mut host = MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    let _ = drain_intents();
    let graph = layout().motion_graph;
    let (ox, oy) = (graph.x + 60.0, graph.y + 60.0);

    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);
    host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Secondary,
        ox,
        oy,
        SEC,
    ));
    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);
    let row = first_menu_row(&state, graph).expect("the menu has a first row");
    let (cx, cy) = (row.x + row.w * 0.5, row.y + row.h * 0.5);

    host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Primary,
        cx,
        cy,
        3 * SEC,
    ));
    // The hand drifts. This is what makes the gesture an `End` instead of a `Click`.
    host.dispatch_pointer_event(pointer(
        PointerKind::Move,
        PointerButton::Primary,
        cx + 1.0,
        cy,
        3 * SEC + 1_000,
    ));
    host.dispatch_pointer_event(pointer(
        PointerKind::Up,
        PointerButton::Primary,
        cx + 1.0,
        cy,
        3 * SEC + SEC / 100,
    ));
    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);

    let intents = drain_intents();
    assert!(
        intents
            .iter()
            .any(|i| matches!(i, GraphIntent::AddNode { .. })),
        "one pixel of drift is still a click on that row - got {intents:?}"
    );
    assert!(!menu_is_open(&state), "and the menu closed behind it");
}

/// The mirror: pressing on a row and dragging OUT of the popup dismisses it and picks nothing.
/// The release is what decides, so a drag that leaves has decided against.
#[test]
fn a_drag_out_of_the_popup_dismisses_it_without_picking() {
    catalog();
    one_node_graph();
    let mut host = MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    let _ = drain_intents();
    let graph = layout().motion_graph;
    let (ox, oy) = (graph.x + 60.0, graph.y + 60.0);

    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);
    host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Secondary,
        ox,
        oy,
        SEC,
    ));
    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);
    let row = first_menu_row(&state, graph).expect("the menu has a first row");
    let (cx, cy) = (row.x + row.w * 0.5, row.y + row.h * 0.5);
    let (fx, fy) = (graph.x + graph.w - 20.0, graph.y + graph.h - 20.0); // far corner

    host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Primary,
        cx,
        cy,
        3 * SEC,
    ));
    host.dispatch_pointer_event(pointer(
        PointerKind::Move,
        PointerButton::Primary,
        fx,
        fy,
        3 * SEC + 1_000,
    ));
    host.dispatch_pointer_event(pointer(
        PointerKind::Up,
        PointerButton::Primary,
        fx,
        fy,
        3 * SEC + SEC / 100,
    ));
    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);

    assert!(
        drain_intents().is_empty(),
        "a release outside the popup picks nothing"
    );
    assert!(!menu_is_open(&state), "and it dismisses the menu");
}

/// **The search field lets go of the keyboard when the menu closes.**
///
/// It takes focus on open, which is what makes *press A and type* work. If it kept focus
/// afterwards, every shortcut in the editor would type into a buffer nobody can see — `A`
/// would not reopen the menu, it would insert an "a".
#[test]
fn closing_the_menu_gives_the_keyboard_back() {
    catalog();
    one_node_graph();
    let mut host = MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    let _ = drain_intents();
    let graph = layout().motion_graph;
    let (ox, oy) = (graph.x + 60.0, graph.y + 60.0);

    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);
    host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Secondary,
        ox,
        oy,
        SEC,
    ));
    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);
    assert_eq!(
        host.store().focus_id(),
        Some(ph2d_panel_motion_graph::menu_search_widget()),
        "the open menu's field owns the keyboard: press A and type"
    );

    // Pick a row (a plain click).
    let row = first_menu_row(&state, graph).expect("first row");
    let (cx, cy) = (row.x + row.w * 0.5, row.y + row.h * 0.5);
    host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Primary,
        cx,
        cy,
        3 * SEC,
    ));
    host.dispatch_pointer_event(pointer(
        PointerKind::Up,
        PointerButton::Primary,
        cx,
        cy,
        3 * SEC + SEC / 100,
    ));
    let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout(), VIEWPORT);

    assert!(!menu_is_open(&state));
    assert_eq!(
        host.store().focus_id(),
        None,
        "the menu is gone, so the keyboard goes back to the editor"
    );
}
