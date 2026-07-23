//! Unit tests for [`super`] (`duration_drag.rs`) — the veil's duration handle
//! drag. Sibling `#[path]` module so the gesture source stays under the panel cap.

use super::*;
use crate::tab::Tab;
use ph2d_editor_core::interaction::{GestureMods, TimelineHitKind};
use ph2d_host::PointerButton;

/// 100 px/s from x = 0, 60 fps, frame-snap on, an authored 4 s duration.
fn snap() -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 60.0,
        frame_snap: true,
        view_length_seconds: 4.0,
        view_length_explicit: true,
        ..TimelineViewSnapshot::default()
    }
}

/// The edge (veil left border) x for `snap()`: 4 s x 100 px/s from view_start 0.
const EDGE_X: f32 = 400.0;

fn gesture(phase: GesturePhase, x: f32) -> TimelineGesture {
    TimelineGesture {
        surface: ph2d_a11y::NodeId(0),
        kind: TimelineHitKind::DurationHandle,
        phase,
        x,
        y: 0.0,
        button: PointerButton::Primary,
        mods: GestureMods::default(),
    }
}

/// GRAB at `grab_x`, drag to `to_x`, release. Return the drained intents.
fn drag_from(
    state: &mut TimelinePanelState,
    s: &TimelineViewSnapshot,
    grab_x: f32,
    to_x: f32,
) -> Vec<TimelineIntent> {
    apply(state, 0.0, 100.0, s, gesture(GesturePhase::Begin, grab_x));
    apply(state, 0.0, 100.0, s, gesture(GesturePhase::Update, to_x));
    apply(state, 0.0, 100.0, s, gesture(GesturePhase::End, to_x));
    state::drain_intents()
}

/// Grab the EDGE and drag to `to_x` (the edge follows the pointer straight).
fn drag(
    state: &mut TimelinePanelState,
    s: &TimelineViewSnapshot,
    to_x: f32,
) -> Vec<TimelineIntent> {
    drag_from(state, s, EDGE_X, to_x)
}

#[test]
fn a_drag_on_keys_authors_the_clip_duration_snapped() {
    // Grab the edge (x=400, t=4), drag to x=200 (t=2.0, on a frame). Keys tab ->
    // the CLIP scope.
    let mut st = TimelinePanelState::default(); // tab: Keys
    let got = drag(&mut st, &snap(), 200.0);
    assert!(
        got.contains(&TimelineIntent::SetClipLength { len: Some(2.0) }),
        "dragging the veil edge to t=2 authors the clip duration, got {got:?}"
    );
}

#[test]
fn grabbing_the_arrow_off_the_edge_does_not_jump_the_duration() {
    // The double-arrow sits 20 px right of the edge. GRAB there (x=420) and DON'T
    // move: the duration must stay 4.0, not jump to t(420)=4.2. This is the
    // grab-relative offset — an absolute map would jump on grab.
    let mut st = TimelinePanelState::default();
    let got = drag_from(&mut st, &snap(), 420.0, 420.0);
    assert!(
        got.contains(&TimelineIntent::SetClipLength { len: Some(4.0) }),
        "grabbing the arrow and not moving keeps the duration at 4.0, got {got:?}"
    );
    // And dragging the arrow LEFT by 200 px shrinks the edge by 2 s (4 -> 2).
    let mut st = TimelinePanelState::default();
    let got = drag_from(&mut st, &snap(), 420.0, 220.0);
    assert!(
        got.contains(&TimelineIntent::SetClipLength { len: Some(2.0) }),
        "dragging the arrow 200 px left moves the edge 2 s, got {got:?}"
    );
}

#[test]
fn the_whole_drag_is_one_undo_bracket() {
    // A length is a DOCUMENT edit: the drag must open exactly one BeginEdit at the
    // start and close one EndEdit at the end, or a slow drag leaves a Ctrl+Z trail.
    let mut st = TimelinePanelState::default();
    let got = drag(&mut st, &snap(), 200.0);
    assert_eq!(
        got.first(),
        Some(&TimelineIntent::BeginEdit),
        "opens a bracket"
    );
    assert_eq!(got.last(), Some(&TimelineIntent::EndEdit), "closes it");
    assert_eq!(
        got.iter()
            .filter(|i| matches!(i, TimelineIntent::BeginEdit | TimelineIntent::EndEdit))
            .count(),
        2,
        "exactly one bracket, got {got:?}"
    );
    assert!(st.dur_drag.is_none(), "the drag state cleared at End");
}

#[test]
fn the_arrange_tab_authors_the_scene_duration() {
    let mut st = TimelinePanelState {
        tab: Tab::Arrange,
        ..TimelinePanelState::default()
    };
    let got = drag(&mut st, &snap(), 200.0);
    assert!(
        got.contains(&TimelineIntent::SetSceneLength { len: Some(2.0) }),
        "Arrange -> the SCENE scope, got {got:?}"
    );
}

#[test]
fn inside_a_container_it_authors_the_container_duration() {
    let mut st = TimelinePanelState {
        tab: Tab::Arrange,
        ..TimelinePanelState::default()
    };
    let s = TimelineViewSnapshot {
        container_open: Some(3),
        ..snap()
    };
    let got = drag(&mut st, &s, 200.0);
    assert!(
        got.contains(&TimelineIntent::SetContainerLength {
            container: 3,
            len: Some(2.0),
        }),
        "an open container -> the CONTAINER scope, got {got:?}"
    );
}

#[test]
fn the_drag_frame_snaps_the_result_like_the_playhead() {
    // Grab the edge, drag to x=157 px (t=1.57 s) -> nearest frame 94/60 = 1.5666…
    let mut st = TimelinePanelState::default();
    let got = drag(&mut st, &snap(), 157.0);
    let frame = (1.57 * 60.0_f64).round() / 60.0;
    let len = got.iter().find_map(|i| match i {
        TimelineIntent::SetClipLength { len: Some(t) } => Some(*t),
        _ => None,
    });
    assert!(
        len.is_some_and(|t| (t - frame).abs() < 1e-9),
        "snapped to a frame ({frame}), got {len:?}"
    );
}

#[test]
fn snap_off_keeps_the_exact_time() {
    let mut st = TimelinePanelState::default();
    let s = TimelineViewSnapshot {
        frame_snap: false,
        ..snap()
    };
    let got = drag(&mut st, &s, 157.0);
    assert!(
        got.contains(&TimelineIntent::SetClipLength { len: Some(1.57) }),
        "snap off -> the raw time, got {got:?}"
    );
}

#[test]
fn a_drag_can_never_zero_the_duration() {
    // Grab the edge, drag far left (x=-500 -> t=-5): it clamps to one frame, never
    // 0, so the veil (and this handle) never vanish out from under the pointer.
    let mut st = TimelinePanelState::default();
    let got = drag(&mut st, &snap(), -500.0);
    let one_frame = 1.0 / 60.0;
    assert!(
        got.contains(&TimelineIntent::SetClipLength {
            len: Some(one_frame)
        }),
        "clamps to one frame, not 0, got {got:?}"
    );
}

#[test]
fn a_bare_click_closes_the_bracket_and_authors_nothing() {
    // A grab that did not drag: the bracket the Begin opened must still close (no
    // stray open bracket), and no length is authored — the handle is a grip, not a
    // seek.
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        gesture(GesturePhase::Begin, EDGE_X),
    );
    apply(
        &mut st,
        0.0,
        100.0,
        &s,
        gesture(GesturePhase::Click, EDGE_X),
    );
    let got = state::drain_intents();
    assert_eq!(
        got,
        vec![TimelineIntent::BeginEdit, TimelineIntent::EndEdit],
        "a click opens then closes the bracket, authoring nothing — got {got:?}"
    );
    assert!(
        !got.iter().any(|i| matches!(i, TimelineIntent::Scrub(_))),
        "the handle is a grip; a click must NOT scrub"
    );
    assert!(st.dur_drag.is_none(), "the drag state cleared on click");
}
