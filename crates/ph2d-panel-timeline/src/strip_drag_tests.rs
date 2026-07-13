//! Unit tests for [`super`] (`strip_drag.rs`) — extracted to a sibling module
//! (`#[path]`) so the gesture source stays under the 600-LOC panel cap.

use super::*;
use ph2d_editor_core::interaction::{GestureMods, TimelineHitKind};
use ph2d_host::PointerButton;
use ph2d_timeline::{LaneMode, LaneView, StripLoop, StripView};

/// One lane, one strip on `[1, 3)`. 100 px/s, 60 fps, frame-snap ON.
fn snap() -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 60.0,
        frame_snap: true,
        lanes: vec![LaneView {
            name: "L".into(),
            muted: false,
            weight: 1.0,
            mode: LaneMode::Override,
            strips: vec![StripView {
                id: StripId(7),
                clip_name: "Main".into(),
                t_start: 1.0,
                t_end: 3.0,
                blend_in: 0.0,
                blend_out: 0.0,
                ease_locked_in: false,
                ease_locked_out: false,
                loop_mode: StripLoop::Once,
                speed: 1.0,
            }],
        }],
        ..TimelineViewSnapshot::default()
    }
}

fn gesture(edge: u8, phase: GesturePhase, x: f32) -> TimelineGesture {
    with_mods(edge, phase, x, GestureMods::default())
}

fn with_mods(edge: u8, phase: GesturePhase, x: f32, mods: GestureMods) -> TimelineGesture {
    TimelineGesture {
        surface: ph2d_a11y::NodeId(0),
        kind: TimelineHitKind::Strip {
            lane: 0,
            strip: 7,
            edge,
        },
        phase,
        x,
        y: 0.0,
        button: PointerButton::Primary,
        mods,
    }
}

/// Drive Begin at x=100 then End at `x`, and return every intent it raised.
fn drag(edge: u8, x: f32) -> Vec<TimelineIntent> {
    let _ = state::drain_intents(); // a previous test's residue
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        100.0,
        &s,
        0,
        7,
        edge,
        gesture(edge, GesturePhase::Begin, 100.0),
    );
    apply(
        &mut st,
        100.0,
        &s,
        0,
        7,
        edge,
        gesture(edge, GesturePhase::End, x),
    );
    state::drain_intents()
}

/// Dragging the body SLIDES the strip: 100 px right at 100 px/s is one second,
/// and the span rides along (the intent carries only the start; the document
/// preserves the length).
#[test]
fn dragging_the_body_slides_the_strip() {
    let out = drag(2, 200.0);
    assert!(
        matches!(out.first(), Some(TimelineIntent::BeginEdit)),
        "the bracket opens FIRST, or each frame of the drag is its own undo step"
    );
    assert!(
        out.iter().any(|i| matches!(
            i,
            TimelineIntent::MoveStrip { lane: 0, t_start, .. } if (t_start - 2.0).abs() < 1e-9
        )),
        "1 s to the right: {out:?}"
    );
    assert!(
        matches!(out.last(), Some(TimelineIntent::EndEdit)),
        "and it closes, folding the whole gesture into ONE Ctrl+Z"
    );
}

/// Dragging an edge TRIMS. It must not be a `MoveStrip` — a trim reveals or hides
/// content, and confusing the two is how a trim silently retimes an animation.
#[test]
fn dragging_an_edge_trims_and_never_moves() {
    for (edge, want) in [(0_u8, 2.0_f64), (1, 4.0)] {
        let out = drag(edge, 200.0);
        assert!(
            out.iter().any(|i| matches!(
                i,
                TimelineIntent::TrimStrip { edge: e, t, .. } if *e == edge && (t - want).abs() < 1e-9
            )),
            "edge {edge} should trim to {want}: {out:?}"
        );
        assert!(
            !out.iter()
                .any(|i| matches!(i, TimelineIntent::MoveStrip { .. })),
            "an edge drag is not a move"
        );
    }
}

/// The SAME edge drag, with Cmd held, is a stretch and not a trim — and the
/// modifier is latched at Begin: releasing it mid-drag must not turn a retime into
/// a trim halfway through the gesture (the strip would be half one edit, half the
/// other, and a single Ctrl+Z would undo an edit nobody made).
#[test]
fn cmd_turns_an_edge_drag_into_a_stretch_and_the_modifier_latches() {
    let _ = state::drain_intents();
    let mut st = TimelinePanelState::default();
    let s = snap();
    let cmd = GestureMods {
        cmd: true,
        ..GestureMods::default()
    };
    apply(
        &mut st,
        100.0,
        &s,
        0,
        7,
        1,
        with_mods(1, GesturePhase::Begin, 100.0, cmd),
    );
    // The modifier is RELEASED for the rest of the drag — and the gesture holds.
    apply(
        &mut st,
        100.0,
        &s,
        0,
        7,
        1,
        gesture(1, GesturePhase::End, 200.0),
    );
    let out = state::drain_intents();
    assert!(
        out.iter().any(|i| matches!(
            i,
            TimelineIntent::StretchStrip { edge: 1, t, .. } if (t - 4.0).abs() < 1e-9
        )),
        "Cmd + end edge must stretch to 4 s: {out:?}"
    );
    assert!(
        !out.iter()
            .any(|i| matches!(i, TimelineIntent::TrimStrip { .. })),
        "and it is NOT a trim: a released modifier cannot change what the drag means"
    );
}

/// The delta applies to the span captured at Begin, never to the live one — a
/// drag that reads back its own output drifts, and the arch gate
/// `arch_no_absolute_drag_pattern` exists because this project already paid for
/// it. Two Updates to the same x must therefore land on the same time.
#[test]
fn a_slow_drag_does_not_drift() {
    let _ = state::drain_intents();
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        100.0,
        &s,
        0,
        7,
        2,
        gesture(2, GesturePhase::Begin, 100.0),
    );
    let mut seen = Vec::new();
    for _ in 0..5 {
        apply(
            &mut st,
            100.0,
            &s,
            0,
            7,
            2,
            gesture(2, GesturePhase::Update, 150.0),
        );
        for i in state::drain_intents() {
            if let TimelineIntent::MoveStrip { t_start, .. } = i {
                seen.push(t_start);
            }
        }
    }
    assert!(
        seen.windows(2).all(|w| (w[0] - w[1]).abs() < 1e-12),
        "five Updates at one x must name one time, got {seen:?}"
    );
}

/// A press that never moved still closes the bracket. Left open, the NEXT atomic
/// edit would be swallowed into it and the undo step would cover two gestures.
#[test]
fn a_click_without_a_drag_closes_the_bracket_it_opened() {
    let _ = state::drain_intents();
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        100.0,
        &s,
        0,
        7,
        2,
        gesture(2, GesturePhase::Begin, 100.0),
    );
    apply(
        &mut st,
        100.0,
        &s,
        0,
        7,
        2,
        gesture(2, GesturePhase::Click, 100.0),
    );
    let out = state::drain_intents();
    assert!(matches!(out.first(), Some(TimelineIntent::BeginEdit)));
    assert!(matches!(out.last(), Some(TimelineIntent::EndEdit)));
    assert!(st.strip_drag.is_none(), "and the drag state is cleared");
}

/// A strip the snapshot no longer has (deleted since the paint that registered
/// its hit) arms nothing at all — not even a bracket. The action expires with its
/// target, as Delete Track's does.
#[test]
fn a_gesture_on_a_vanished_strip_arms_nothing() {
    let _ = state::drain_intents();
    let mut st = TimelinePanelState::default();
    let s = TimelineViewSnapshot::default(); // no lanes
    apply(
        &mut st,
        100.0,
        &s,
        0,
        7,
        2,
        gesture(2, GesturePhase::Begin, 100.0),
    );
    assert!(st.strip_drag.is_none());
    assert!(
        state::drain_intents().is_empty(),
        "an unopened bracket cannot leak"
    );
}

/// **B4: the corner grip authors the strip's OWN fade** — the thing a lone strip could
/// not do at all before (`ease_in`/`ease_out` existed, the evaluator honoured them, and
/// nothing wrote them: a strip alone on a lane entered and left hard).
///
/// The strip runs [1, 3) and the view is 100 px/s, so dragging the fade-in grip 50 px to
/// the right is half a second of fade.
#[test]
fn dragging_the_fade_in_grip_authors_the_strips_own_fade() {
    let out = drag(3, 150.0);
    assert!(matches!(out.first(), Some(TimelineIntent::BeginEdit)));
    assert!(matches!(out.last(), Some(TimelineIntent::EndEdit)));
    let Some(TimelineIntent::SetStripEase {
        lane,
        id,
        edge,
        seconds,
    }) = out
        .iter()
        .find(|i| matches!(i, TimelineIntent::SetStripEase { .. }))
    else {
        panic!("the fade grip must raise SetStripEase: {out:?}");
    };
    assert_eq!((*lane, *id), (0, StripId(7)));
    assert_eq!(
        *edge, 0,
        "the panel's grip 3 is the document's edge 0 (the start)"
    );
    assert!(
        (*seconds - 0.5).abs() < 1e-9,
        "50 px at 100 px/s is half a second of fade: {seconds}"
    );
}

/// **The fade-out grip grows the fade by dragging LEFT** — it rides the tip of the wedge,
/// which travels INTO the strip. Getting this sign backwards is the whole bug this gate
/// exists for: the handle would shrink when pulled and the artist would fight it.
///
/// And it never goes negative: dragging the tip back PAST the corner is zero fade, not a
/// fade of minus half a second (the apply clamps too — but a UI that emits nonsense and
/// leans on the document to sanitise it is a UI that will emit nonsense somewhere the
/// document does not).
#[test]
fn the_fade_out_grip_grows_the_fade_by_dragging_left_and_never_goes_negative() {
    let ease_of = |out: &[TimelineIntent]| -> f64 {
        out.iter()
            .find_map(|i| match i {
                TimelineIntent::SetStripEase { edge, seconds, .. } => {
                    assert_eq!(*edge, 1, "grip 4 is the document's edge 1 (the end)");
                    Some(*seconds)
                }
                _ => None,
            })
            .expect("the fade grip must raise SetStripEase")
    };
    // 40 px LEFT of where it began, at 100 px/s: 0.4 s of fade-out.
    assert!(
        (ease_of(&drag(4, 60.0)) - 0.4).abs() < 1e-9,
        "dragging the end grip INWARD (left) must GROW the fade"
    );
    // …and dragging it outward, past the corner, is no fade — never a negative one.
    assert_eq!(
        ease_of(&drag(4, 140.0)),
        0.0,
        "a fade cannot be negative: the tip stops at the corner"
    );
}
