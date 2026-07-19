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
                container: None,
                t_start: 1.0,
                t_end: 3.0,
                // Uma fade JA autorada: 0,25 s de cada lado. Com a cunha em zero, `start_ease`
                // seria sempre 0 e o gate nao veria a diferenca entre "delta sobre o que ja
                // existe" e "valor absoluto" -- que e exatamente o bug que o
                // `arch_no_absolute_drag_pattern` existe para pegar.
                blend_in: 0.25,
                blend_out: 0.25,
                lead_in: 0.0,
                lead_out: 0.0,
                marks: [0.0; 4],
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

/// **The GREEN top corner STRETCHES, not trims** (Enio, 2026-07-16). Trim and stretch used
/// to share an edge behind a Cmd modifier; now each is its own corner — grip code `6` is
/// stretch-end (top-right), and it retimes rather than cutting. No modifier to latch, no
/// modifier to release mid-drag.
#[test]
fn the_green_top_corner_stretches_and_never_trims() {
    // Code 6 = stretch-end. The strip runs [1, 3); dragging the pointer 100 px right at
    // 100 px/s stretches the end to 4 s (StretchStrip edge 1).
    let out = drag(6, 200.0);
    assert!(
        out.iter().any(|i| matches!(
            i,
            TimelineIntent::StretchStrip { edge: 1, t, .. } if (t - 4.0).abs() < 1e-9
        )),
        "the green end corner stretches to 4 s: {out:?}"
    );
    assert!(
        !out.iter()
            .any(|i| matches!(i, TimelineIntent::TrimStrip { .. })),
        "and it is NOT a trim: stretch and trim are different corners now"
    );
}

/// **Every edge intent carries the ANCHOR its change bar is measured against** — where
/// the edge sat when the gesture began, not where the previous frame left it. Only the
/// panel knows that; the document sees a stream of absolute positions and cannot tell
/// the first frame from the fortieth. Send a per-frame delta instead and a slow drag
/// accumulates a bar many times the change it made.
///
/// The strip runs `[1, 3)`, so the trim-end anchor is `3.0` and the stretch-start
/// anchor is `1.0` — the two ends of the CAPTURED span, which is what makes the anchor
/// hold still while the pointer moves.
#[test]
fn an_edge_drag_reports_where_the_gesture_started_not_where_the_last_frame_did() {
    for (code, want_from) in [(1u8, 3.0), (5, 1.0)] {
        let out = drag(code, 250.0);
        let from = out.iter().find_map(|i| match i {
            TimelineIntent::TrimStrip { from, .. } | TimelineIntent::StretchStrip { from, .. } => {
                Some(*from)
            }
            _ => None,
        });
        assert_eq!(
            from,
            Some(want_from),
            "grip {code} must anchor on the captured span: {out:?}"
        );
    }
}

/// …and it stays put while the pointer walks. Two Updates at different x must report the
/// SAME anchor: the moment it starts following the pointer, the bar stops describing the
/// gesture and starts describing one frame of it.
#[test]
fn the_anchor_does_not_move_with_the_pointer() {
    let _ = state::drain_intents();
    let mut st = TimelinePanelState::default();
    let s = snap();
    apply(
        &mut st,
        100.0,
        &s,
        0,
        7,
        1,
        gesture(1, GesturePhase::Begin, 200.0),
    );
    let anchors: Vec<f64> = [250.0_f32, 300.0, 400.0]
        .iter()
        .flat_map(|x| {
            apply(
                &mut st,
                100.0,
                &s,
                0,
                7,
                1,
                gesture(1, GesturePhase::Update, *x),
            );
            state::drain_intents().into_iter().filter_map(|i| match i {
                TimelineIntent::TrimStrip { from, .. } => Some(from),
                _ => None,
            })
        })
        .collect();
    assert_eq!(anchors.len(), 3, "three frames, three intents");
    assert!(
        anchors.iter().all(|a| (a - 3.0).abs() < 1e-9),
        "the anchor is the gesture's, not the frame's: {anchors:?}"
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
    // A fixture ja tinha 0,25 s de fade-in. 50 px a 100 px/s sao mais meio segundo -> 0,75.
    // Se o drag ignorasse o `start_ease` (valor absoluto em vez de delta), daria 0,5 e a alca
    // SALTARIA pra tras no primeiro pixel do arrasto.
    assert!(
        (*seconds - 0.75).abs() < 1e-9,
        "o arrasto e um DELTA sobre a fade que ja existe (0,25 + 0,5): {seconds}"
    );
}

/// **Dragging the fade-in grip PAST the start edge, into the gap, authors the OUTWARD
/// fade (`SetStripLead`)** — the travel across the gap (Enio, 2026-07-16). Right of the
/// edge it is the inward `SetStripEase` (the gate above); the edge is the pivot, one
/// handle two intents.
#[test]
fn dragging_the_fade_in_grip_into_the_gap_authors_the_outward_lead() {
    // The fixture's fade-in tip sits at t = 1.25 (blend_in 0.25). Drag the pointer left
    // to x = 50 (−0.5 s): the tip lands at t = 0.75, which is 0.25 s BEFORE the strip's
    // start — an outward lead of 0.25 s, not a negative inward fade.
    let out = drag(3, 50.0);
    let lead = out.iter().find_map(|i| match i {
        TimelineIntent::SetStripLead {
            lane,
            id,
            edge,
            seconds,
        } => {
            assert_eq!((*lane, *id), (0, StripId(7)));
            assert_eq!(*edge, 0, "the START edge — this is lead_in");
            Some(*seconds)
        }
        _ => None,
    });
    assert!(
        lead.is_some_and(|s| (s - 0.25).abs() < 1e-9),
        "into the gap -> SetStripLead(0.25): {out:?}"
    );
    assert!(
        !out.iter()
            .any(|i| matches!(i, TimelineIntent::SetStripEase { .. })),
        "and NOT an inward SetStripEase — the tip crossed the start edge"
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
    // 40 px a ESQUERDA, a 100 px/s: +0,4 s sobre os 0,25 s que a fixture ja tinha.
    assert!(
        (ease_of(&drag(4, 60.0)) - 0.65).abs() < 1e-9,
        "arrastar a alca do fim pra DENTRO (esquerda) tem de CRESCER a fade, a partir da que ja \
         existe: {}",
        ease_of(&drag(4, 60.0))
    );
}

/// **Dragging the fade-OUT grip PAST the end edge, into the gap, authors the OUTWARD
/// fade-out (`SetStripLead { edge: 1 }`)** — the mirror of the lead-in gate above (Enio,
/// 2026-07-19). LEFT of the edge it is the inward `SetStripEase`; the edge is the pivot,
/// one handle two intents. This used to be a hard "zero fade" clamp; now it is `lead_out`.
#[test]
fn dragging_the_fade_out_grip_into_the_gap_authors_the_outward_lead_out() {
    // The fixture's fade-out is 0.25 s; drag the tip RIGHT by 1.0 s (x = 200 at 100 px/s),
    // which is 0.75 s PAST the end edge — an outward lead-out of 0.75 s.
    let out = drag(4, 200.0);
    let lead = out.iter().find_map(|i| match i {
        TimelineIntent::SetStripLead {
            lane,
            id,
            edge,
            seconds,
        } => {
            assert_eq!((*lane, *id), (0, StripId(7)));
            assert_eq!(*edge, 1, "the END edge — this is lead_out");
            Some(*seconds)
        }
        _ => None,
    });
    assert!(
        lead.is_some_and(|s| (s - 0.75).abs() < 1e-9),
        "into the gap after -> SetStripLead(edge 1, 0.75): {out:?}"
    );
    assert!(
        !out.iter()
            .any(|i| matches!(i, TimelineIntent::SetStripEase { .. })),
        "and NOT an inward SetStripEase — the tip crossed the end edge"
    );
}
