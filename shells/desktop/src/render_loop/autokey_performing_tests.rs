//! Performing / record (W5) tests + the record-simplify (Schneider) cleanup.
//! Helpers live in `autokey_test_helpers.rs`.
use super::test_helpers::*;
use super::*;
use ph2d_timeline::{PropKind, TimelineIntent as I, apply_intent};

// ── Performing / record (W5) — the aviso do Enio, made irrefutable ──────────
// The rule: while PLAYING, only an active gizmo drag with Record armed writes a
// key. The passive pose the animation is driving must NEVER mint one — a plain
// Play (even with AutoKey armed) records nothing.

#[test]
fn a_plain_play_with_autokey_armed_records_nothing() {
    // THE bug Enio warned about: play with AutoKey on used to mint keys. Even
    // with the pose sitting OFF the curve (worst case), no drag = no key.
    let (mut st, mut ph) = state_with_tx_track();
    ph.seek(0.5);
    ph.play();
    let mut ak = AutokeyState::default();
    // armed = auto-key on; performing off; NO drag.
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 7.0)]))],
        false,
        true,
        &mut ak,
    );
    assert_eq!(
        track_len(&st),
        2,
        "a plain Play (AutoKey armed, no drag) must record nothing"
    );
}

#[test]
fn performing_without_a_drag_records_nothing() {
    // Record armed, playing, pose off the curve — but no gesture. The animation
    // playing on its own is not a performance: nothing is recorded.
    let (mut st, mut ph) = state_with_tx_track();
    ph.seek(0.5);
    ph.play();
    let mut ak = AutokeyState::default();
    frame_perf(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 7.0)]))],
        false,
        false,
        &mut ak,
    );
    assert_eq!(
        track_len(&st),
        2,
        "Record without a drag records nothing — it is the GESTURE that captures"
    );
}

#[test]
fn performing_with_a_drag_records_the_dragged_pose() {
    // The feature: playing + Record + a live drag pushing the pose off its curve
    // → the dragged value is captured at the playhead.
    let (mut st, mut ph) = state_with_tx_track();
    ph.seek(0.5);
    ph.play();
    let mut ak = AutokeyState::default();
    // drag_now = true; performing armed (frame_perf).
    frame_perf(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 7.0)]))],
        true,
        false,
        &mut ak,
    );
    assert_eq!(track_len(&st), 3, "the drag records a key at the playhead");
    assert_eq!(
        tx_at(&st, 0.5),
        Some(7.0),
        "and it carries the dragged value, not the curve's"
    );
}

#[test]
fn performing_is_inert_when_paused() {
    // Record is a play-only mode: paused, it changes nothing (paused authoring
    // is AutoKey / manual K). Pose ON its curve so the pin doesn't engage.
    let (mut st, mut ph) = state_with_tx_track();
    ph.seek(0.5);
    ph.pause();
    let on_curve = pose(&[(TX, tx_at(&st, 0.5).unwrap())]);
    let mut ak = AutokeyState::default();
    // performing on, auto-key OFF, a drag — but PAUSED.
    frame_perf(&mut st, &ph, &[(E, on_curve)], true, false, &mut ak);
    assert_eq!(track_len(&st), 2, "Record does nothing while paused");
}

#[test]
fn a_performing_session_is_one_undo_step() {
    // A record spans many played frames (one drag). The whole trajectory must
    // collapse into ONE undo step, like a gizmo drag — not one per frame.
    let (mut st, mut ph) = state_with_tx_track();
    ph.play();
    let mut ak = AutokeyState::default();
    // Drag held across three played frames, recording at three times.
    for (t, v) in [(0.25, 3.0f32), (0.5, 6.0), (0.75, 9.0)] {
        ph.seek(t);
        frame_perf(&mut st, &ph, &[(E, pose(&[(TX, v)]))], true, false, &mut ak);
    }
    // Release the drag — closes the one bracket + runs the record simplify.
    ph.seek(0.75);
    frame_perf(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 9.0)]))],
        false,
        false,
        &mut ak,
    );
    // The simplify collapsed the recorded run (here a straight ramp → its two
    // ends); the point of THIS test is that whatever it produced is ONE step.
    assert!(track_len(&st) > 2, "the record added keys");
    assert!(st.history.can_undo(), "the session banked an undo step");
    apply_intent(&mut st, &mut ph, I::Undo);
    assert_eq!(
        track_len(&st),
        2,
        "ONE undo removes the whole recording session, not just the last frame"
    );
}

#[test]
fn a_performing_session_simplifies_the_recorded_keys_on_release() {
    // Record ~60 frames of a sine bump by dragging TX, then release. The dense
    // per-frame keys must collapse to a clean handful — and the whole record +
    // cleanup is ONE undo step.
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    for (t, v) in [(0.0, 0.0f32), (2.0, 0.0)] {
        apply_intent(
            &mut st,
            &mut ph,
            I::AddKey {
                entity: E,
                prop: PropKind::TranslationX,
                t: RationalTime::from_seconds(t),
                value: AnimValue::Float(v),
                interp: ph2d_anim::Interp::Linear,
            },
        );
    }
    let target = st
        .doc
        .binding_for(E, PropKind::TranslationX)
        .unwrap()
        .target;
    ph.play();
    let mut ak = AutokeyState::default();
    let frames = 60;
    for i in 0..frames {
        let t = 2.0 * f64::from(i) / f64::from(frames - 1);
        ph.seek(t);
        let v = 30.0 * (t * std::f64::consts::PI / 2.0).sin();
        frame_perf(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, v as f32)]))],
            true,
            false,
            &mut ak,
        );
    }
    let dense = st.doc.active_clip().track(target).unwrap().len();
    assert!(dense > 20, "the record laid down many keys: {dense}");

    // Release the drag → the session ends and simplify runs.
    frame_perf(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 0.0)]))],
        false,
        false,
        &mut ak,
    );
    let simplified = st.doc.active_clip().track(target).unwrap().len();
    assert!(
        simplified <= dense / 3,
        "release simplified the recording: {dense} → {simplified} keys"
    );
    assert!(simplified >= 2, "and kept a real curve");

    // Fidelity: the cleaned curve still traces the recorded bump.
    use ph2d_anim::AttributeEvaluator;
    let tr = st.doc.active_clip().track(target).unwrap();
    for i in 0..frames {
        let t = 2.0 * f64::from(i) / f64::from(frames - 1);
        let want = 30.0 * (t * std::f64::consts::PI / 2.0).sin();
        let got = match tr.sample(t) {
            AnimValue::Float(v) => f64::from(v),
            _ => unreachable!(),
        };
        // 0.5% of the ~30-unit range ≈ 0.15, plus f32 slack.
        assert!(
            (got - want).abs() < 0.3,
            "fidelity at t={t}: {got} vs {want}"
        );
    }

    // ONE undo reverts the whole session — back to the two-key baseline.
    apply_intent(&mut st, &mut ph, I::Undo);
    assert_eq!(
        st.doc.active_clip().track(target).map(|t| t.len()),
        Some(2),
        "one undo reverts record + simplify together"
    );
}

#[test]
fn simplify_only_fires_for_a_performing_session_not_a_paused_drag() {
    // A paused gizmo drag (ordinary auto-key) records ONE key and must NOT run
    // the record simplify — `ak.record` only fills while playing.
    let (mut st, ph) = state_with_tx_track(); // paused
    let mut ak = AutokeyState::default();
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 7.0)]))],
        true,
        true,
        &mut ak,
    );
    assert!(
        ak.record.is_empty(),
        "a paused drag records no performing span"
    );
}
