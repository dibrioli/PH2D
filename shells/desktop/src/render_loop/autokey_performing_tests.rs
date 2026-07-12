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
        "release simplified the recording: {dense} -> {simplified} keys"
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
        // Tracks the bump within a few percent of the ~30-unit range
        // (approximate by design — a key per turn, not per frame).
        assert!(
            (got - want).abs() < 1.0,
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

#[test]
fn a_session_aligns_every_track_of_an_object_on_shared_key_times() {
    // Enio (2026-07-11): "keys for x and y of translate and scale created at the
    // same point in time". Record TWO channels of one object with DIFFERENT
    // shapes (so their own extrema fall at different times) and assert the
    // cleanup lands them on ONE shared set of columns.
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    for prop in [PropKind::TranslationX, PropKind::TranslationY] {
        for (t, v) in [(0.0, 0.0f32), (4.0, 0.0)] {
            apply_intent(
                &mut st,
                &mut ph,
                I::AddKey {
                    entity: E,
                    prop,
                    t: RationalTime::from_seconds(t),
                    value: AnimValue::Float(v),
                    interp: ph2d_anim::Interp::Linear,
                },
            );
        }
    }
    ph.play();
    let mut ak = AutokeyState::default();
    // X is a 2-cycle wave, Y a 3-cycle one — their turns do NOT coincide.
    let frames = 120;
    for i in 0..frames {
        let t = 4.0 * f64::from(i) / f64::from(frames - 1);
        ph.seek(t);
        let x = 40.0 * (t * 3.0).sin();
        let y = 25.0 * (t * 4.4).sin();
        let mut p: PoseSample = [None; 6];
        p[0] = Some(x as f32);
        p[1] = Some(y as f32);
        frame_perf(&mut st, &ph, &[(E, p)], true, false, &mut ak);
    }
    // Release → the session ends, tracks are fitted and column-aligned.
    frame_perf(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 0.0)]))],
        false,
        false,
        &mut ak,
    );

    let times = |prop| -> Vec<f64> {
        let target = st.doc.binding_for(E, prop).unwrap().target;
        st.doc
            .active_clip()
            .track(target)
            .unwrap()
            .keys()
            .iter()
            .map(|k| k.t.to_seconds())
            .collect()
    };
    let tx = times(PropKind::TranslationX);
    let ty = times(PropKind::TranslationY);
    assert!(tx.len() > 2 && ty.len() > 2, "both tracks were simplified");
    assert_eq!(
        tx.len(),
        ty.len(),
        "both channels key at the same COUNT of columns: {tx:?} vs {ty:?}"
    );
    for (a, b) in tx.iter().zip(&ty) {
        assert!(
            (a - b).abs() < 1e-6,
            "every key shares a column: {tx:?} vs {ty:?}"
        );
    }
}

// ── Channel semantics of the record cleanup (rotation unwrap · opacity bounds) ─

#[test]
fn a_recorded_two_turn_spin_survives_the_cleanup() {
    // THE product-level assertion for the rotation unwrap. The rotate gizmo writes
    // `Transform.rotation` as `start + (atan2(now) - atan2(start))`, and both
    // `atan2`s live in `(-PI, PI]` — so the pose this pass samples during a spin is
    // a +-2PI SAWTOOTH. Fitted as a plain scalar it used to reconstruct a 2-turn
    // spin as a net rotation of ~zero: the spin was simply gone.
    //
    // Driven through the real `simplify_recorded` seam, not the fit's own tests —
    // the damage was never in either half, it was in the channel semantics between
    // them ([[feedback_tool_unit_green_integration_dead]]).
    use ph2d_anim::AttributeEvaluator;
    use std::f64::consts::{PI, TAU};

    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    for (t, v) in [(0.0, 0.0f32), (2.0, 0.0)] {
        apply_intent(
            &mut st,
            &mut ph,
            I::AddKey {
                entity: E,
                prop: PropKind::Rotation,
                t: RationalTime::from_seconds(t),
                value: AnimValue::Float(v),
                interp: ph2d_anim::Interp::Linear,
            },
        );
    }
    let target = st.doc.binding_for(E, PropKind::Rotation).unwrap().target;

    // Perform a continuous two-turn spin (plus a radian, so the gesture ENDS on a
    // pose that differs from the seed key — otherwise the last frame diffs to zero,
    // no key is recorded there, and the range the cleanup owns stops short of it).
    let total = 2.0 * TAU + 1.0;
    let truth = |t: f64| total * (t / 2.0);
    let wrapped = |t: f64| {
        let mut a = truth(t) % TAU;
        if a > PI {
            a -= TAU;
        }
        a
    };
    ph.play();
    let mut ak = AutokeyState::default();
    let frames = 120;
    for i in 0..frames {
        let t = 2.0 * f64::from(i) / f64::from(frames - 1);
        ph.seek(t);
        frame_perf(
            &mut st,
            &ph,
            &[(E, pose(&[(ROT, wrapped(t) as f32)]))],
            true,
            false,
            &mut ak,
        );
    }
    // Release: the session ends, the cleanup runs.
    frame_perf(
        &mut st,
        &ph,
        &[(E, pose(&[(ROT, wrapped(2.0) as f32)]))],
        false,
        false,
        &mut ak,
    );

    let tr = st.doc.active_clip().track(target).unwrap();
    let at = |t: f64| match tr.sample(t) {
        AnimValue::Float(v) => f64::from(v),
        _ => unreachable!(),
    };
    // It really turned twice, and it never un-spins on the way.
    let span = at(2.0) - at(0.0);
    assert!(
        (span - total).abs() < 0.35,
        "the spin really turned {total:.2} rad; the cleaned track replays {span:.2} \
         (before the unwrap: 0.00 — the whole spin was gone)"
    );
    let mut prev = at(0.0);
    for i in 1..=200 {
        let now = at(2.0 * f64::from(i) / 200.0);
        assert!(
            now >= prev - 0.02,
            "a forward spin never snaps backward: {prev:.3} -> {now:.3}"
        );
        prev = now;
    }
}

#[test]
fn a_recorded_fade_is_cleaned_up_inside_the_opacity_bounds() {
    // Opacity is `[0, 1]`. A least-squares cubic through a fade that settles ON the
    // bound overshoots past it; the runtime clamps the display, but the graph
    // editor draws the CURVE.
    use ph2d_anim::AttributeEvaluator;
    const OPACITY: usize = 5;

    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    for (t, v) in [(0.0, 0.0f32), (1.5, 1.0)] {
        apply_intent(
            &mut st,
            &mut ph,
            I::AddKey {
                entity: E,
                prop: PropKind::Opacity,
                t: RationalTime::from_seconds(t),
                value: AnimValue::Float(v),
                interp: ph2d_anim::Interp::Linear,
            },
        );
    }
    let target = st.doc.binding_for(E, PropKind::Opacity).unwrap().target;

    ph.play();
    let mut ak = AutokeyState::default();
    let frames = 90;
    for i in 0..frames {
        let t = 1.5 * f64::from(i) / f64::from(frames - 1);
        ph.seek(t);
        // Fade in fast, then rest exactly on the bound.
        let v = if t < 0.5 { 2.0 * t } else { 1.0 };
        frame_perf(
            &mut st,
            &ph,
            &[(E, pose(&[(OPACITY, v as f32)]))],
            true,
            false,
            &mut ak,
        );
    }
    frame_perf(
        &mut st,
        &ph,
        &[(E, pose(&[(OPACITY, 1.0)]))],
        false,
        false,
        &mut ak,
    );

    let tr = st.doc.active_clip().track(target).unwrap();
    for i in 0..=300 {
        let t = 1.5 * f64::from(i) / 300.0;
        let v = match tr.sample(t) {
            AnimValue::Float(v) => f64::from(v),
            _ => unreachable!(),
        };
        assert!(
            (-1e-5..=1.0 + 1e-5).contains(&v),
            "the cleaned fade stays inside [0, 1]: {v} at t={t}"
        );
    }
}
