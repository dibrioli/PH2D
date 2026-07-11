//! Unit tests for [`super`] (`marker_drag.rs`) — extracted to a sibling module
//! (`#[path]`) so the gesture source stays under the 600-LOC panel cap.

use super::*;
use ph2d_editor_core::interaction::{GestureMods, GesturePhase, TimelineHitKind};
use ph2d_host::PointerButton;

/// 100 px/s from x = 0, one marker "M1" at t = 1 s (storage index 0).
fn snap() -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 60.0,
        frame_snap: true,
        markers: vec![(1.0, "M1".into())],
        ..TimelineViewSnapshot::default()
    }
}

fn gesture(phase: GesturePhase, x: f32, alt: bool) -> TimelineGesture {
    TimelineGesture {
        surface: ph2d_a11y::NodeId(0),
        kind: TimelineHitKind::Marker { index: 0 },
        phase,
        x,
        y: 0.0,
        button: PointerButton::Primary,
        mods: GestureMods {
            shift: false,
            cmd: false,
            alt,
        },
    }
}

#[test]
fn a_plain_click_seeks_the_playhead_to_the_marker() {
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(GesturePhase::Begin, 100.0, false),
    );
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(GesturePhase::Click, 100.0, false),
    );
    assert_eq!(
        state::drain_intents(),
        vec![
            TimelineIntent::BeginEdit,
            TimelineIntent::EndEdit,
            TimelineIntent::Scrub(1.0),
        ],
        "a click seeks to the marker's time; the empty bracket commits no step"
    );
}

#[test]
fn an_alt_click_removes_the_marker_in_one_step() {
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(GesturePhase::Begin, 100.0, true),
    );
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(GesturePhase::Click, 100.0, true),
    );
    assert_eq!(
        state::drain_intents(),
        vec![
            TimelineIntent::BeginEdit,
            TimelineIntent::RemoveMarker { index: 0 },
            TimelineIntent::EndEdit,
        ],
        "Alt+click removes, bracketed as one undo step — and does NOT also seek"
    );
}

#[test]
fn dragging_streams_absolute_moves_inside_one_bracket() {
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(GesturePhase::Begin, 100.0, false),
    );
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(GesturePhase::Update, 250.0, false),
    );
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(GesturePhase::End, 250.0, false),
    );
    let got = state::drain_intents();
    assert_eq!(got.first(), Some(&TimelineIntent::BeginEdit));
    assert_eq!(got.last(), Some(&TimelineIntent::EndEdit));
    // x = 250 px is t = 2.5 s — an ABSOLUTE move, not a delta.
    let moves: Vec<_> = got
        .iter()
        .filter_map(|i| match i {
            TimelineIntent::MoveMarker { index, t_seconds } => Some((*index, *t_seconds)),
            _ => None,
        })
        .collect();
    assert!(moves.iter().all(|&(i, _)| i == 0));
    assert!(
        moves.last().is_some_and(|&(_, t)| (t - 2.5).abs() < 1e-9),
        "the last streamed move is the absolute release time: {moves:?}"
    );
    assert!(st.marker_drag.is_none(), "the drag ended");
}

#[test]
fn a_double_click_arms_the_rename_and_raises_no_edit() {
    let mut st = TimelinePanelState::default();
    let s = snap();
    // The pair's second press: Begin opens an empty bracket, DoubleClick closes
    // it and arms the rename for this marker (paint seeds + focuses next frame).
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(GesturePhase::Begin, 100.0, false),
    );
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(GesturePhase::DoubleClick, 100.0, false),
    );
    assert_eq!(
        state::drain_intents(),
        vec![TimelineIntent::BeginEdit, TimelineIntent::EndEdit],
        "the double-click closes its empty bracket and raises no document edit"
    );
    assert_eq!(
        st.marker_rename.map(|m| (m.index, m.opened)),
        Some((0, false)),
        "the rename is armed for marker 0, not yet seeded"
    );
    assert!(st.marker_drag.is_none());
}

#[test]
fn an_alt_double_click_deletes_and_never_opens_a_rename() {
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(GesturePhase::Begin, 100.0, true),
    );
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(GesturePhase::DoubleClick, 100.0, true),
    );
    assert!(
        st.marker_rename.is_none(),
        "Alt is the delete modifier — a double-click with it never renames"
    );
    let _ = state::drain_intents();
}

#[test]
fn a_move_never_goes_negative() {
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(GesturePhase::Begin, 100.0, false),
    );
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        0,
        gesture(GesturePhase::Update, -500.0, false),
    );
    let last_move = state::drain_intents()
        .into_iter()
        .filter_map(|i| match i {
            TimelineIntent::MoveMarker { t_seconds, .. } => Some(t_seconds),
            _ => None,
        })
        .next_back();
    assert_eq!(last_move, Some(0.0), "dragged past zero clamps to zero");
}
