//! W2.E5b — timeline dope-sheet dispatch tests.
//!
//! Editor-core owns no timeline semantics: it captures an
//! [`InteractiveState::TimelineSurface`] hit and streams [`TimelineGesture`]s
//! into the store for the panel to drain. These exercise that plumbing through
//! the public dispatch entries (the panel that interprets them is tested in
//! `ph2d-panel-timeline`). Lean mirror of the graph-surface tests.

use super::*;
use crate::interaction::{GesturePhase, TimelineHitKind};

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
