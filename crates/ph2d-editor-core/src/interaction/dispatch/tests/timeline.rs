//! W2.E5b — timeline dope-sheet dispatch tests.
//!
//! Editor-core owns no timeline semantics: it captures an
//! [`InteractiveState::TimelineSurface`] hit and streams [`TimelineGesture`]s
//! into the store for the panel to drain. These exercise that plumbing through
//! the public dispatch entries (the panel that interprets them is tested in
//! `ph2d-panel-timeline`). Lean mirror of the graph-surface tests.

use super::*;
use crate::interaction::{GesturePhase, TimelineHitKind};
use ph2d_host::WheelEvent;

const SURFACE: NodeId = NodeId(700);
const KEY_TARGET: NodeId = NodeId(701);
/// The registered hit rect for `KEY_TARGET` (x 50..90, y 50..70).
const HIT: Rect = Rect::new(50.0, 50.0, 40.0, 20.0);
const CANVAS: Rect = Rect::new(0.0, 0.0, 400.0, 200.0);

/// A store + hit index with one timeline surface target of `kind`.
fn timeline_setup(kind: TimelineHitKind) -> (WidgetStore, HitIndex) {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        KEY_TARGET,
        InteractiveState::TimelineSurface {
            parent: SURFACE,
            kind,
            canvas: CANVAS,
        },
    );
    let mut hits = HitIndex::new();
    hits.register(KEY_TARGET, HIT);
    (store, hits)
}

#[test]
fn key_down_move_up_streams_begin_update_end_with_identity() {
    let kind = TimelineHitKind::Key { target: 42, key: 3 };
    let (mut store, hits) = timeline_setup(kind);
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
        pointer(PointerKind::Move, 140.0, 60.0),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, 140.0, 60.0),
        &arena,
    );

    let g: Vec<_> = store.drain_timeline_gestures().collect();
    assert_eq!(g.len(), 3);
    assert_eq!(g[0].phase, GesturePhase::Begin);
    assert_eq!(g[1].phase, GesturePhase::Update);
    assert_eq!(g[2].phase, GesturePhase::End);
    // The surface + key identity ride along unchanged on every phase.
    for gg in &g {
        assert_eq!(gg.surface, SURFACE);
        assert_eq!(gg.kind, kind);
    }
    assert_eq!(store.active_id(), None, "Up released the capture");
}

#[test]
fn tap_on_a_key_is_a_click_not_an_end() {
    let (mut store, hits) = timeline_setup(TimelineHitKind::Key { target: 1, key: 0 });
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

    let g: Vec<_> = store.drain_timeline_gestures().collect();
    assert_eq!(g.len(), 2);
    assert_eq!(g[0].phase, GesturePhase::Begin);
    assert_eq!(g[1].phase, GesturePhase::Click);
}

#[test]
fn update_streams_past_the_rect_edge() {
    // A key drag must keep updating past the panel edge — the Move arm keys off
    // the active capture, not rect containment.
    let (mut store, hits) = timeline_setup(TimelineHitKind::Key { target: 1, key: 1 });
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

    let g: Vec<_> = store.drain_timeline_gestures().collect();
    assert_eq!(g.len(), 2);
    assert_eq!(g[1].phase, GesturePhase::Update);
    assert_eq!((g[1].x, g[1].y), (999.0, 999.0));
}

fn wheel(x: f32, y: f32, delta_x: f32, delta_y: f32, shift: bool) -> WheelEvent {
    WheelEvent {
        x,
        y,
        delta_x,
        delta_y,
        modifiers: Modifiers {
            shift,
            ctrl: false,
            alt: false,
            meta: false,
        },
        timestamp_ns: 0,
    }
}

// ── Wheel → anchored zoom / pan (W2.E6) ──────────────────────────────────

#[test]
fn wheel_over_the_time_axis_accumulates_zoom_and_consumes() {
    let (mut store, _hits) = timeline_setup(TimelineHitKind::Lane);
    store.set_timeline_canvas(SURFACE, CANVAS);
    let arena = Bump::new();

    let ev = dispatch_wheel(&mut store, wheel(100.0, 120.0, 0.0, 3.0, false), &arena);
    assert!(ev.is_empty(), "wheel over the timeline is consumed as zoom");
    let _ = dispatch_wheel(&mut store, wheel(110.0, 130.0, 0.0, 2.0, false), &arena);

    let z = store.take_timeline_zoom(SURFACE).expect("zoom accumulated");
    assert_eq!(z.zoom_delta, 5.0, "notches sum (3 + 2)");
    assert_eq!(z.pan_delta, 0.0);
    assert_eq!(z.anchor_x, 110.0, "anchor follows the latest cursor");
    assert!(
        store.take_timeline_zoom(SURFACE).is_none(),
        "draining removes it"
    );
}

#[test]
fn shift_wheel_pans_instead_of_zooming() {
    let (mut store, _hits) = timeline_setup(TimelineHitKind::Lane);
    store.set_timeline_canvas(SURFACE, CANVAS);
    let arena = Bump::new();
    let _ = dispatch_wheel(&mut store, wheel(100.0, 120.0, 0.0, 4.0, true), &arena);

    let z = store
        .take_timeline_zoom(SURFACE)
        .expect("wheel accumulated");
    assert_eq!(z.zoom_delta, 0.0, "Shift+wheel must not zoom");
    assert_eq!(z.pan_delta, 4.0);
}

#[test]
fn horizontal_wheel_pans() {
    let (mut store, _hits) = timeline_setup(TimelineHitKind::Lane);
    store.set_timeline_canvas(SURFACE, CANVAS);
    let arena = Bump::new();
    let _ = dispatch_wheel(&mut store, wheel(100.0, 120.0, 6.0, 0.0, false), &arena);

    let z = store
        .take_timeline_zoom(SURFACE)
        .expect("wheel accumulated");
    assert_eq!((z.zoom_delta, z.pan_delta), (0.0, 6.0));
}

#[test]
fn wheel_outside_the_time_axis_is_not_a_timeline_zoom() {
    let (mut store, _hits) = timeline_setup(TimelineHitKind::Lane);
    store.set_timeline_canvas(SURFACE, Rect::new(0.0, 0.0, 100.0, 100.0));
    let arena = Bump::new();
    let _ = dispatch_wheel(&mut store, wheel(500.0, 500.0, 0.0, 3.0, false), &arena);
    assert!(store.take_timeline_zoom(SURFACE).is_none());
}

#[test]
fn a_hidden_timeline_leaves_no_stale_zoom_rect() {
    let (mut store, _hits) = timeline_setup(TimelineHitKind::Lane);
    store.set_timeline_canvas(SURFACE, CANVAS);
    store.clear_timeline_canvas(); // what `paint` does while hidden
    let arena = Bump::new();
    let _ = dispatch_wheel(&mut store, wheel(100.0, 120.0, 0.0, 3.0, false), &arena);
    assert!(store.take_timeline_zoom(SURFACE).is_none());
}

#[test]
fn lane_click_carries_the_lane_kind_and_shift_modifier() {
    let (mut store, hits) = timeline_setup(TimelineHitKind::Lane);
    store.set_shift_held(true);
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

    let g: Vec<_> = store.drain_timeline_gestures().collect();
    assert_eq!(g.len(), 2);
    assert_eq!(g[1].phase, GesturePhase::Click);
    assert_eq!(g[0].kind, TimelineHitKind::Lane);
    assert!(g[0].mods.shift, "cached shift rides the gesture");
}
