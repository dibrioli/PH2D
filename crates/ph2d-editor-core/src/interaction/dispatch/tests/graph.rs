//! Motion Nodes M0.T3 — graph-surface dispatch tests.
//!
//! Editor-core owns no graph semantics: it captures an
//! [`InteractiveState::GraphSurface`] hit and streams [`GraphGesture`]s /
//! [`GraphKey`]s / anchored zoom into the store for the panel to drain. These
//! tests exercise that plumbing end-to-end through the public dispatch entries
//! (the panel that interprets the gestures lands in M1).

use super::*;
use crate::interaction::dispatch::{
    KEY_DELETE, KEY_ESCAPE, KEY_KEY_A, KEY_KEY_D, KEY_KEY_F, KEY_KEY_K, KEY_KEY_P,
};
use crate::interaction::{GesturePhase, GraphHitKind, GraphKey};
use ph2d_host::{PointerButton, WheelEvent};

const SURFACE: NodeId = NodeId(500);
const TARGET: NodeId = NodeId(501);
/// The registered hit rect for `TARGET` (x 50..90, y 50..90).
const HIT: Rect = Rect::new(50.0, 50.0, 40.0, 40.0);
const CANVAS: Rect = Rect::new(0.0, 0.0, 400.0, 300.0);

/// A store + hit index with one graph surface target of `kind`.
fn graph_setup(kind: GraphHitKind) -> (WidgetStore, HitIndex) {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        TARGET,
        InteractiveState::GraphSurface {
            parent: SURFACE,
            kind,
            canvas: CANVAS,
        },
    );
    let mut hits = HitIndex::new();
    hits.register(TARGET, HIT);
    (store, hits)
}

fn pointer_btn(kind: PointerKind, x: f32, y: f32, button: PointerButton) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button,
        timestamp_ns: 0,
    }
}

fn wheel(x: f32, y: f32, delta_y: f32) -> WheelEvent {
    WheelEvent {
        x,
        y,
        delta_x: 0.0,
        delta_y,
        modifiers: Modifiers {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        },
        timestamp_ns: 0,
    }
}

fn key_cmd(kc: u32, cmd: bool) -> KeyEvent {
    KeyEvent {
        keycode: kc,
        modifiers: Modifiers {
            shift: false,
            ctrl: cmd,
            alt: false,
            meta: false,
        },
        kind: KeyKind::Down,
        timestamp_ns: 0,
    }
}

// ── Pointer gesture lifecycle ────────────────────────────────────────────

#[test]
fn node_down_move_up_streams_begin_update_end() {
    let (mut store, hits) = graph_setup(GraphHitKind::Node { node: 7 });
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 60.0, 60.0),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, 120.0, 90.0),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, 120.0, 90.0),
        &arena,
    );

    let g: Vec<_> = store.drain_graph_gestures().collect();
    assert_eq!(g.len(), 3);
    assert_eq!(g[0].phase, GesturePhase::Begin);
    assert_eq!(g[1].phase, GesturePhase::Update);
    assert_eq!(g[2].phase, GesturePhase::End);
    // The surface + hit kind ride along unchanged on every phase.
    for gg in &g {
        assert_eq!(gg.surface, SURFACE);
        assert_eq!(gg.kind, GraphHitKind::Node { node: 7 });
    }
    // Up released the capture.
    assert_eq!(store.active_id(), None);
    assert!(store.active_rect().is_none());
}

#[test]
fn down_up_without_movement_is_a_click() {
    let (mut store, hits) = graph_setup(GraphHitKind::Background);
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 60.0, 60.0),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, 60.0, 60.0),
        &arena,
    );

    let g: Vec<_> = store.drain_graph_gestures().collect();
    assert_eq!(g.len(), 2);
    assert_eq!(g[0].phase, GesturePhase::Begin);
    assert_eq!(g[1].phase, GesturePhase::Click);
}

#[test]
fn update_fires_even_after_the_pointer_leaves_the_rect() {
    // A node drag must keep updating past the panel edge — the Move arm is
    // gated on the capture being active, NOT on rect containment.
    let (mut store, hits) = graph_setup(GraphHitKind::Node { node: 1 });
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 60.0, 60.0),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, 999.0, 999.0),
        &arena,
    );

    let g: Vec<_> = store.drain_graph_gestures().collect();
    assert_eq!(g.len(), 2);
    assert_eq!(g[1].phase, GesturePhase::Update);
    assert_eq!((g[1].x, g[1].y), (999.0, 999.0));
}

#[test]
fn right_click_on_graph_is_captured_not_a_context_menu() {
    // The graph arm runs BEFORE the context-menu delegation, so a right-click
    // opens the graph's own add-menu (panel-side) instead of the global note
    // menu — the plan's "R-click antes dos ContextMenuKind".
    let (mut store, hits) = graph_setup(GraphHitKind::Background);
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer_btn(PointerKind::Down, 60.0, 60.0, PointerButton::Secondary),
        &arena,
    );

    let g: Vec<_> = store.drain_graph_gestures().collect();
    assert_eq!(g.len(), 1);
    assert_eq!(g[0].phase, GesturePhase::Begin);
    assert_eq!(g[0].button, PointerButton::Secondary);
    assert!(
        store.context_menu().is_none(),
        "the graph intercepts the right-click; no global context menu"
    );
}

#[test]
fn middle_button_identity_is_carried_through() {
    // M0.T1 plumbing: a middle-drag reaches the graph channel with its identity
    // (the panel routes it to pan).
    let (mut store, hits) = graph_setup(GraphHitKind::Background);
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer_btn(PointerKind::Down, 60.0, 60.0, PointerButton::Middle),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer_btn(PointerKind::Move, 70.0, 70.0, PointerButton::Middle),
        &arena,
    );

    let g: Vec<_> = store.drain_graph_gestures().collect();
    assert_eq!(g.len(), 2);
    assert!(g.iter().all(|gg| gg.button == PointerButton::Middle));
}

#[test]
fn gesture_carries_the_cached_modifiers() {
    let (mut store, hits) = graph_setup(GraphHitKind::Background);
    store.set_shift_held(true);
    store.set_alt_held(true);
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 60.0, 60.0),
        &arena,
    );

    let g: Vec<_> = store.drain_graph_gestures().collect();
    assert!(g[0].mods.shift && g[0].mods.alt && !g[0].mods.cmd);
}

// ── Anchored zoom ────────────────────────────────────────────────────────

#[test]
fn wheel_over_canvas_accumulates_anchored_zoom_and_consumes() {
    let (mut store, _hits) = graph_setup(GraphHitKind::Background);
    store.set_graph_canvas(SURFACE, CANVAS);
    let arena = Bump::new();

    let ev = dispatch_wheel(&mut store, wheel(100.0, 120.0, 3.0), &arena);
    assert!(ev.is_empty(), "wheel over the graph is consumed as zoom");
    let _ = dispatch_wheel(&mut store, wheel(110.0, 130.0, 2.0), &arena);

    let z = store.take_graph_zoom(SURFACE).expect("zoom accumulated");
    assert_eq!(z.delta, 5.0, "notches sum (3 + 2)");
    assert_eq!(
        (z.anchor_x, z.anchor_y),
        (110.0, 130.0),
        "anchor follows the latest cursor"
    );
    // Draining removed the entry.
    assert!(store.take_graph_zoom(SURFACE).is_none());
}

#[test]
fn wheel_outside_the_canvas_is_not_a_graph_zoom() {
    let (mut store, _hits) = graph_setup(GraphHitKind::Background);
    store.set_graph_canvas(SURFACE, Rect::new(0.0, 0.0, 100.0, 100.0));
    let arena = Bump::new();
    let _ = dispatch_wheel(&mut store, wheel(500.0, 500.0, 3.0), &arena);
    assert!(store.take_graph_zoom(SURFACE).is_none());
}

// ── Keyboard commands ────────────────────────────────────────────────────

#[test]
fn graph_shortcuts_route_when_the_surface_is_focused() {
    let (mut store, _hits) = graph_setup(GraphHitKind::Background);
    store.set_graph_focused(Some(SURFACE));
    let arena = Bump::new();

    let cases = [
        (KEY_DELETE, false, GraphKey::Delete),
        (KEY_KEY_F, false, GraphKey::Fit),
        (KEY_KEY_A, false, GraphKey::Add),
        (KEY_ESCAPE, false, GraphKey::Escape),
        (KEY_KEY_K, false, GraphKey::Knife),
        (KEY_KEY_P, false, GraphKey::Probe),
        (KEY_KEY_D, true, GraphKey::Duplicate),
    ];
    for (kc, cmd, expected) in cases {
        let _ = dispatch_key(&mut store, key_cmd(kc, cmd), &arena);
        let keys: Vec<_> = store.drain_graph_keys().collect();
        assert_eq!(keys, vec![expected], "keycode {kc:#06x} (cmd={cmd})");
    }
}

#[test]
fn cmd_a_stays_select_all_not_graph_add() {
    let (mut store, _hits) = graph_setup(GraphHitKind::Background);
    store.set_graph_focused(Some(SURFACE));
    let arena = Bump::new();
    let _ = dispatch_key(&mut store, key_cmd(KEY_KEY_A, true), &arena);
    assert!(
        store.drain_graph_keys().next().is_none(),
        "Cmd/Ctrl+A must fall through to select-all, not become graph Add"
    );
}

#[test]
fn shortcuts_ignored_when_the_graph_is_not_focused() {
    let (mut store, _hits) = graph_setup(GraphHitKind::Background);
    // graph_focused defaults to None.
    let arena = Bump::new();
    let _ = dispatch_key(&mut store, key_cmd(KEY_DELETE, false), &arena);
    assert!(store.drain_graph_keys().next().is_none());
}

#[test]
fn shortcuts_yield_to_a_focused_text_widget() {
    // Editing a text field (focus_id set) beats graph shortcuts, so typing 'F'
    // into a field doesn't fit the graph.
    let (mut store, _hits) = graph_setup(GraphHitKind::Background);
    store.set_graph_focused(Some(SURFACE));
    store.register(NodeId(600), text_input("hi"));
    store.set_focus(Some(NodeId(600)));
    let arena = Bump::new();
    let _ = dispatch_key(&mut store, key_cmd(KEY_KEY_F, false), &arena);
    assert!(
        store.drain_graph_keys().next().is_none(),
        "a focused text widget swallows the key before the graph arm"
    );
}

/// **A graph surface really sees a DOUBLE-CLICK** — the producer half of the Motion editor's
/// waypoint gestures (doc 44).
///
/// The graph's Down CAPTURES the pointer and returns early, past the general double-click
/// detection at the bottom of `pointer_down`. So the flag has to be recorded on the graph's
/// own capture path — and until it was, a graph surface could never see a double-click *at
/// all*, whatever a panel asked for: the Motion panel's "double-click a wire to route it"
/// was wired at the panel end and silently dead.
///
/// FALSIFIED by removing `set_graph_double`/`take_graph_double`: the second tap comes back
/// as a plain `Click` and the gesture the panel is waiting for never arrives.
#[test]
fn a_second_tap_on_a_graph_surface_is_a_double_click() {
    let (mut store, hits) = graph_setup(GraphHitKind::Background);
    let arena = Bump::new();
    let tap = |store: &mut WidgetStore, at_ns: u128| {
        for kind in [PointerKind::Down, PointerKind::Up] {
            let mut e = pointer(kind, 60.0, 60.0);
            e.timestamp_ns = at_ns;
            let _ = dispatch_pointer(store, &hits, e, &arena);
        }
    };

    tap(&mut store, 0);
    tap(&mut store, 100_000_000); // 100 ms later — inside the 350 ms window

    let g: Vec<_> = store.drain_graph_gestures().collect();
    assert_eq!(g.len(), 4, "Begin, Click, Begin, DoubleClick");
    assert_eq!(g[1].phase, GesturePhase::Click, "the first tap is a click");
    assert_eq!(
        g[3].phase,
        GesturePhase::DoubleClick,
        "the second tap upgrades to DoubleClick"
    );

    // A slow second tap stays a plain Click — and the flag never leaks into it.
    tap(&mut store, 1_000_000_000); // a second later
    let g: Vec<_> = store.drain_graph_gestures().collect();
    assert_eq!(g[1].phase, GesturePhase::Click, "too slow to be a double");
}

/// A double-click that DRAGGED is an `End`, not a `DoubleClick` — a gesture that moved is a
/// drag, whatever the tap count, and the flag must not survive to poison the next tap.
#[test]
fn a_dragged_second_tap_is_an_end_not_a_double_click() {
    let (mut store, hits) = graph_setup(GraphHitKind::Background);
    let arena = Bump::new();
    for kind in [PointerKind::Down, PointerKind::Up] {
        let _ = dispatch_pointer(&mut store, &hits, pointer(kind, 60.0, 60.0), &arena);
    }
    let mut down = pointer(PointerKind::Down, 60.0, 60.0);
    down.timestamp_ns = 100_000_000;
    let _ = dispatch_pointer(&mut store, &hits, down, &arena);
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, 200.0, 200.0),
        &arena,
    );
    let mut up = pointer(PointerKind::Up, 200.0, 200.0);
    up.timestamp_ns = 120_000_000;
    let _ = dispatch_pointer(&mut store, &hits, up, &arena);

    let g: Vec<_> = store.drain_graph_gestures().collect();
    assert_eq!(g.last().unwrap().phase, GesturePhase::End, "it dragged");
}
