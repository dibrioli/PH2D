//! Unit tests for [`super`] (`loop_drag.rs`) — extracted to a sibling module
//! (`#[path]`) so the gesture source stays under the 600-LOC panel cap.

use super::*;
use ph2d_editor_core::interaction::{GestureMods, TimelineHitKind};
use ph2d_host::PointerButton;

/// 100 px/s from x = 0, 60 fps, frame-snap on, loop range 1..3 s.
fn snap() -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 60.0,
        frame_snap: true,
        loop_range: Some((1.0, 3.0)),
        ..TimelineViewSnapshot::default()
    }
}

fn gesture(edge: u8, phase: GesturePhase, x: f32) -> TimelineGesture {
    TimelineGesture {
        surface: ph2d_a11y::NodeId(0),
        kind: TimelineHitKind::LoopBrace { edge },
        phase,
        x,
        y: 0.0,
        button: PointerButton::Primary,
        mods: GestureMods::default(),
    }
}

/// Drive Begin→Update to `x` for `edge` and return the range the SetLoop carried.
fn drag_to(edge: u8, x: f32) -> (f64, f64) {
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        edge,
        gesture(edge, GesturePhase::Begin, 100.0),
    );
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        edge,
        gesture(edge, GesturePhase::Update, x),
    );
    let intents = state::drain_intents();
    match intents.last() {
        Some(TimelineIntent::SetLoop(Some(r))) => *r,
        other => panic!("expected SetLoop, got {other:?}"),
    }
}

#[test]
fn dragging_the_start_handle_moves_the_range_start() {
    // x = 50 px is t = 0.5 s; the end stays at 3.
    assert_eq!(drag_to(0, 50.0), (0.5, 3.0));
}

#[test]
fn dragging_the_end_handle_moves_the_range_end() {
    // x = 250 px is t = 2.5 s; the start stays at 1.
    assert_eq!(drag_to(1, 250.0), (1.0, 2.5));
}

#[test]
fn the_start_cannot_cross_the_end() {
    // Drag the start way past the end: it stops one frame short of it.
    let (a, b) = drag_to(0, 900.0);
    assert_eq!(b, 3.0);
    assert!(
        (a - (3.0 - 1.0 / 60.0)).abs() < 1e-9,
        "start clamps to end - 1 frame: {a}"
    );
}

#[test]
fn the_end_cannot_cross_the_start() {
    let (a, b) = drag_to(1, -100.0);
    assert_eq!(a, 1.0);
    assert!(
        (b - (1.0 + 1.0 / 60.0)).abs() < 1e-9,
        "end clamps to start + 1 frame: {b}"
    );
}

#[test]
fn the_start_cannot_go_negative() {
    // x = -200 px would be t = -2; clamps to 0.
    assert_eq!(drag_to(0, -200.0), (0.0, 3.0));
}

#[test]
fn dragging_the_start_handle_never_panics_on_a_sub_frame_loop() {
    // A loop shorter than one frame makes `end − one_frame` negative, so a raw
    // `.clamp(0.0, b0 - min_len)` would panic (min > max). The guarded upper
    // bound collapses the start to 0 instead of crashing.
    let mut st = TimelinePanelState::default();
    let s = TimelineViewSnapshot {
        fps: 60.0,          // one frame = 16.7 ms
        frame_snap: false,
        loop_range: Some((0.0, 0.01)), // 10 ms < one frame
        ..TimelineViewSnapshot::default()
    };
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(0, GesturePhase::Begin, 100.0),
    );
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(0, GesturePhase::Update, 50.0),
    );
    match state::drain_intents().last() {
        Some(TimelineIntent::SetLoop(Some((a, b)))) => {
            assert_eq!(*a, 0.0, "start collapses to 0, not a panic");
            assert!((*b - 0.01).abs() < 1e-9, "end unchanged");
        }
        o => panic!("expected SetLoop, got {o:?}"),
    }
}

#[test]
fn dragging_the_body_moves_both_ends_and_keeps_the_length() {
    // Body grabbed at x = 100 (t = 1); dragged to x = 150 (t = 1.5) → +0.5 s.
    // Both ends shift; the 2 s length is preserved.
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        2,
        gesture(2, GesturePhase::Begin, 100.0),
    );
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        2,
        gesture(2, GesturePhase::Update, 150.0),
    );
    let got = match state::drain_intents().last() {
        Some(TimelineIntent::SetLoop(Some(r))) => *r,
        o => panic!("{o:?}"),
    };
    assert_eq!(got, (1.5, 3.5));
}

#[test]
fn dragging_the_body_left_clamps_the_start_at_zero_and_keeps_the_length() {
    // Grab at t = 1, drag to t = -1 (raw −2). The start clamps to 0, and the end
    // follows so the 2 s length survives.
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        2,
        gesture(2, GesturePhase::Begin, 100.0),
    );
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        2,
        gesture(2, GesturePhase::Update, -100.0),
    );
    let got = match state::drain_intents().last() {
        Some(TimelineIntent::SetLoop(Some(r))) => *r,
        o => panic!("{o:?}"),
    };
    assert_eq!(got, (0.0, 2.0));
}

#[test]
fn a_drag_frame_snaps_the_edited_edge() {
    // 100 px/s, 60 fps: x = 57 px is t = 0.57 s → nearest frame is 34/60 =
    // 0.5666… The end handle lands exactly on a frame.
    let (_, b) = drag_to(1, 157.0);
    let frame = (1.57 * 60.0_f64).round() / 60.0;
    assert!((b - frame).abs() < 1e-9, "{b} vs {frame}");
}

#[test]
fn end_clears_the_drag_state() {
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(0, GesturePhase::Begin, 100.0),
    );
    assert!(st.loop_drag.is_some());
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(0, GesturePhase::End, 60.0),
    );
    assert!(st.loop_drag.is_none(), "the drag ended");
    let _ = state::drain_intents();
}

#[test]
fn clicking_inside_the_loop_band_scrubs_to_that_time() {
    // A plain click on the body (edge 2) seeks the playhead there instead of
    // being swallowed — the range only slides on a drag.
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        2,
        gesture(2, GesturePhase::Begin, 200.0),
    );
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        2,
        gesture(2, GesturePhase::Click, 200.0),
    );
    assert_eq!(
        state::drain_intents().last(),
        Some(&TimelineIntent::Scrub(2.0)),
        "x = 200 px is t = 2 s"
    );
    assert!(st.loop_drag.is_none());
}

#[test]
fn clicking_a_brace_handle_does_not_scrub() {
    // A click on an edge handle is a grab that changed nothing — it must not
    // also seek.
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(0, GesturePhase::Begin, 100.0),
    );
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(0, GesturePhase::Click, 100.0),
    );
    assert!(
        !state::drain_intents()
            .iter()
            .any(|i| matches!(i, TimelineIntent::Scrub(_))),
        "an edge-handle click must not scrub"
    );
}
