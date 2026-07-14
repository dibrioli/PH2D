//! **R-click a backdrop's header: the palette** (doc 62) — through the real paint and the real
//! dispatcher.
//!
//! What a right-press MEANS now depends on what is under it. Before this, the Secondary button was
//! short-circuited at the top of `apply` and opened the **node library** over *anything* — including
//! a backdrop's header, where a list of 88 node types is not the question anybody is asking.
//!
//! It also pins a bug that has nothing to do with colour: the popup painted a **search field on
//! every menu**, including ones with nothing to search — and that field *took the keyboard*. It had
//! been doing that on the card-ports menu since the search landed. Eight tints do not need a filter.
//!
//! And the click itself is dispatched the way a hand makes it: **press, drift a pixel, release**
//! ([[feedback_a_click_is_a_press_that_drifted]]).

use ph2d_editor_core::interaction::{
    GestureMods, GesturePhase, GraphGesture, GraphHitKind, InteractiveState,
};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::layout::{CenterSplit, HeroLayout};
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerButton;
use ph2d_panel_motion_graph::{
    GraphBackdropView, GraphIntent, GraphViewSnapshot, MotionGraphPanel, MotionGraphPanelState,
    drain_intents, set_current_motion_graph,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

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

/// One backdrop, tinted 2 ("Lime").
fn scene() {
    set_current_motion_graph(Some(GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: Vec::new(),
        edges: Vec::new(),
        backdrops: vec![GraphBackdropView {
            id: 7,
            x: 0.0,
            y: 0.0,
            w: 400.0,
            h: 200.0,
            color: 2,
            title: "The Snow".into(),
        }],
        probe: None,
        now: 0.0,
    }));
}

fn frame(host: &mut MockPanelHost, state: &mut MotionGraphPanelState) {
    let _ = host.paint_with_layout::<MotionGraphPanel>(state, layout(), VIEWPORT);
}

fn press_header(host: &mut MockPanelHost, button: PointerButton, x: f32, y: f32) {
    host.store_mut().push_graph_gesture(GraphGesture {
        surface: ph2d_editor_core::ids::MOTION_GRAPH_PANEL,
        kind: GraphHitKind::Backdrop { id: 7 },
        phase: GesturePhase::Begin,
        x,
        y,
        button,
        mods: GestureMods::default(),
    });
}

/// A background gesture — how the popup's rows are reached (the menu registers a full-canvas
/// shield, so every click over an open menu arrives as `Background`).
fn bg(host: &mut MockPanelHost, phase: GesturePhase, x: f32, y: f32) {
    host.store_mut().push_graph_gesture(GraphGesture {
        surface: ph2d_editor_core::ids::MOTION_GRAPH_PANEL,
        kind: GraphHitKind::Background,
        phase,
        x,
        y,
        button: PointerButton::Primary,
        mods: GestureMods::default(),
    });
}

/// **The gesture, end to end.** Right-click the header, and the row you release over is the tint
/// the backdrop takes — even though the hand drifted a pixel between the press and the release,
/// which is what makes the dispatcher call it a drag rather than a click.
#[test]
fn right_clicking_the_header_opens_the_palette_and_a_row_tints_it() {
    scene();
    let mut host = MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    let _ = drain_intents();
    frame(&mut host, &mut state);

    press_header(&mut host, PointerButton::Secondary, 300.0, 600.0);
    frame(&mut host, &mut state);

    // The palette is open — where the node library used to be, which over a backdrop was an
    // answer to a question nobody asked. And opening it edits nothing.
    let row = ph2d_panel_motion_graph::first_menu_row(&state, layout().motion_graph)
        .expect("the right-press must open the palette");
    assert!(
        drain_intents().is_empty(),
        "opening the palette is not an edit"
    );

    // Row 0 is "Red" (tint 0). Press it, drift one pixel — a hand always does — and release.
    let (cx, cy) = (row.x + row.w * 0.5, row.y + row.h * 0.5);
    bg(&mut host, GesturePhase::Begin, cx, cy);
    bg(&mut host, GesturePhase::Update, cx + 1.0, cy);
    bg(&mut host, GesturePhase::End, cx + 1.0, cy);
    frame(&mut host, &mut state);

    let intents = drain_intents();
    assert_eq!(
        intents.len(),
        1,
        "picking a tint must set the tint - got {intents:?}"
    );
    assert!(
        matches!(
            intents[0],
            GraphIntent::SetBackdropColor { id: 7, color: 0 }
        ),
        "the row that was DRAWN first is the tint that lands: {:?}",
        intents[0]
    );
    assert!(
        ph2d_panel_motion_graph::first_menu_row(&state, layout().motion_graph).is_none(),
        "picking a row closes the palette"
    );
}

/// **A palette has nothing to search, and it does not take the keyboard to prove it.**
///
/// The search field belonged to every popup, not just the library — so the card-ports menu had been
/// showing an inert box that ate the keyboard since the search landed. A left-press on the header
/// (which selects and drags, and opens nothing) is the control: no menu, no field either way.
#[test]
fn the_palette_has_no_search_field() {
    scene();
    let mut host = MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    frame(&mut host, &mut state);

    press_header(&mut host, PointerButton::Secondary, 300.0, 600.0);
    frame(&mut host, &mut state);
    assert!(
        ph2d_panel_motion_graph::first_menu_row(&state, layout().motion_graph).is_some(),
        "the palette is open"
    );
    assert_eq!(
        host.store().focus_id(),
        None,
        "the palette grabbed the keyboard for a field that filters nothing"
    );

    // …and the library, which DOES have something to search, still takes it.
    bg(&mut host, GesturePhase::Begin, 40.0, 600.0);
    bg(&mut host, GesturePhase::Click, 40.0, 600.0);
    frame(&mut host, &mut state);
    host.store_mut()
        .push_graph_key(ph2d_editor_core::interaction::GraphKey::Add);
    frame(&mut host, &mut state);
    match host.store().focus_id().and_then(|id| host.store().get(id)) {
        Some(InteractiveState::TextInput { .. }) => {}
        other => panic!("the ADD menu must still own the keyboard for its search: {other:?}"),
    }
}

/// A left-press still selects and drags the backdrop — the palette took the right button, which
/// over a backdrop was opening the node library, and left the ordinary gesture alone.
#[test]
fn a_left_press_on_the_header_still_drags_it() {
    scene();
    let mut host = MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    let _ = drain_intents();
    frame(&mut host, &mut state);

    press_header(&mut host, PointerButton::Primary, 300.0, 600.0);
    frame(&mut host, &mut state);
    assert!(
        ph2d_panel_motion_graph::first_menu_row(&state, layout().motion_graph).is_none(),
        "a LEFT press selects and drags; it does not open the palette"
    );
    assert_eq!(
        ph2d_panel_motion_graph::current_graph_backdrop_selection(),
        Some(7),
        "and it selects the backdrop"
    );
}
