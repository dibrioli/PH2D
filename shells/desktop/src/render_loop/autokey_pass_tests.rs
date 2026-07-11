//! Tests for [`super`] (`autokey_pass.rs`) — the per-frame diff, the undo
//! brackets and the displaced-pose pin. Extracted to a sibling module
//! (`#[path]`) under the HR-18 shell LOC cap. Pure relocation.
use super::*;
use ph2d_timeline::{PropKind, TimelineIntent as I, apply_intent};

const E: u64 = 1;
const TX: usize = 0;
const ROT: usize = 2;

fn state_with_tx_track() -> (TimelineState, Playhead) {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    for (t, v) in [(0.0, 0.0), (1.0, 10.0)] {
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
    ph.seek(0.5); // curve says x = 5 here
    ph.pause(); // the tests edit while PAUSED; play-suppression is its own test
    (st, ph)
}

fn pose(vals: &[(usize, f32)]) -> PoseSample {
    let mut p: PoseSample = [None; 6];
    for &(i, v) in vals {
        p[i] = Some(v);
    }
    p
}

fn tx_at(st: &TimelineState, t: f64) -> Option<f32> {
    use ph2d_anim::AttributeEvaluator;
    let target = st.doc.binding_for(E, PropKind::TranslationX)?.target;
    match st.doc.active_clip().track(target)?.sample(t) {
        AnimValue::Float(v) => Some(v),
        _ => None,
    }
}

/// One frame of the pass with the given selection samples (performing off —
/// these tests exercise ordinary auto-key; performing has its own block below).
fn frame(
    st: &mut TimelineState,
    ph: &Playhead,
    samples: &[(u64, PoseSample)],
    drag_now: bool,
    armed: bool,
    ak: &mut AutokeyState,
) {
    apply_samples(st, ph, samples, drag_now, armed, false, ak);
}

/// One frame with performing (record) armed — for the record-during-play tests.
fn frame_perf(
    st: &mut TimelineState,
    ph: &Playhead,
    samples: &[(u64, PoseSample)],
    drag_now: bool,
    armed: bool,
    ak: &mut AutokeyState,
) {
    apply_samples(st, ph, samples, drag_now, armed, true, ak);
}

fn track_len(st: &TimelineState) -> usize {
    let target = st
        .doc
        .binding_for(E, PropKind::TranslationX)
        .unwrap()
        .target;
    st.doc.active_clip().track(target).unwrap().len()
}

// Pausing mid-play lands the playhead at a RAW time (multiples of the sim
// dt, 1/60), while frame snap quantizes authored times to display frames
// (1/24) — two different clocks. The apply wrote `world = curve(t_raw)`;
// the diff and the displaced-pose pin must compare against THAT clock, not
// `curve(t_snap)`, or an untouched pose reads as "dragged" the moment the
// playhead rests off-frame: armed mints a key out of thin air ("autoplay
// creates keyframes", Enio 2026-07-11), disarmed pins the entity so the
// apply stops writing it and undo looks like it leaves residues.
#[test]
fn a_pose_on_its_curve_at_an_off_frame_pause_keys_nothing() {
    let (mut st, mut ph) = state_with_tx_track();
    ph.seek(0.51); // 0.51 * 24 fps = 12.24 → snaps to 12/24 = 0.5
    ph.pause();
    let on_curve = pose(&[(TX, tx_at(&st, 0.51).unwrap())]); // what the apply wrote
    let mut ak = AutokeyState::default();
    frame(&mut st, &ph, &[(E, on_curve)], false, true, &mut ak);
    let target = st
        .doc
        .binding_for(E, PropKind::TranslationX)
        .unwrap()
        .target;
    assert_eq!(
        st.doc.active_clip().track(target).unwrap().len(),
        2,
        "an untouched pose at an off-frame pause must not mint a key"
    );
}

#[test]
fn an_untouched_pose_at_an_off_frame_pause_is_not_pinned() {
    let (mut st, mut ph) = state_with_tx_track();
    ph.seek(0.51);
    ph.pause();
    let on_curve = pose(&[(TX, tx_at(&st, 0.51).unwrap())]);
    let mut ak = AutokeyState::default();
    frame(&mut st, &ph, &[(E, on_curve)], false, false, &mut ak);
    assert!(
        ak.displaced.is_empty(),
        "an untouched pose must not be pinned: the apply would stop \
         writing the entity and every timeline edit (undo included) would \
         stop reaching the scene"
    );
}

#[test]
fn a_bound_prop_dragged_off_its_curve_gets_keyed_in_one_undo_step() {
    let (mut st, ph) = state_with_tx_track();
    let mut ak = AutokeyState::default();
    let before = st.doc.clone();
    // A discrete edit (no gizmo drag): world x = 7 at t = 0.5, curve says 5.
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 7.0)]))],
        false,
        true,
        &mut ak,
    );
    assert_eq!(
        tx_at(&st, 0.5),
        Some(7.0),
        "the pose was keyed at the playhead"
    );
    // Exactly one undo step, and it reverts the whole thing.
    apply_intent(&mut st, &mut Playhead::new(1.0 / 60.0), I::Undo);
    assert_eq!(st.doc, before, "the discrete auto-key is one undo step");
}

#[test]
fn a_pose_on_its_curve_keys_nothing() {
    // THE anti-feedback case in the shell glue: world == curve → no key, so an
    // undo/paste/scrub (which the apply writes back to the world) is not undone.
    let (mut st, ph) = state_with_tx_track();
    let before = st.doc.clone();
    let mut ak = AutokeyState::default();
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 5.0)]))],
        false,
        true,
        &mut ak,
    );
    assert_eq!(st.doc, before, "on-curve pose left the document untouched");
}

#[test]
fn a_gizmo_drag_is_a_single_undo_step_across_many_frames() {
    let (mut st, ph) = state_with_tx_track();
    let before = st.doc.clone();
    let mut ak = AutokeyState::default();
    // Three frames of a drag (drag_now = true), the pose creeping 6 → 7 → 8.
    for x in [6.0, 7.0, 8.0] {
        frame(&mut st, &ph, &[(E, pose(&[(TX, x)]))], true, true, &mut ak);
    }
    assert_eq!(tx_at(&st, 0.5), Some(8.0), "the last dragged pose stuck");
    // Drag ends (drag_now = false) → the step commits.
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 8.0)]))],
        false,
        true,
        &mut ak,
    );
    // ONE undo reverts the whole drag, not one frame of it.
    apply_intent(&mut st, &mut Playhead::new(1.0 / 60.0), I::Undo);
    assert_eq!(st.doc, before, "the whole drag undoes in one step");
}

#[test]
fn an_unbound_prop_that_moves_auto_creates_a_track() {
    let (mut st, ph) = state_with_tx_track();
    let mut ak = AutokeyState::default();
    assert!(st.doc.binding_for(E, PropKind::Rotation).is_none());
    // Frame 1 establishes the baseline (rotation = 0), keys nothing new.
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(ROT, 0.0)]))],
        false,
        true,
        &mut ak,
    );
    assert!(
        st.doc.binding_for(E, PropKind::Rotation).is_none(),
        "no track from mere baseline"
    );
    // Frame 2 the rotation moved → auto-create the track + key.
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(ROT, 1.5)]))],
        false,
        true,
        &mut ak,
    );
    assert!(
        st.doc.binding_for(E, PropKind::Rotation).is_some(),
        "track auto-created"
    );
}

#[test]
fn disarmed_keys_nothing_but_still_tracks_the_baseline() {
    // With auto-key off, a move records no key — but the baseline must still
    // advance, or arming mid-drag would treat a stale pose as a jump.
    let (mut st, ph) = state_with_tx_track();
    let before = st.doc.clone();
    let mut ak = AutokeyState::default();
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 7.0)]))],
        false,
        false,
        &mut ak,
    );
    assert_eq!(st.doc, before, "disarmed: no key");
    assert_eq!(
        ak.baseline.get(&E).and_then(|p| p[TX]),
        Some(7.0),
        "baseline advanced anyway"
    );
}

#[test]
fn deselecting_an_entity_drops_it_from_the_baseline() {
    let (mut st, ph) = state_with_tx_track();
    let mut ak = AutokeyState::default();
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 7.0)]))],
        false,
        true,
        &mut ak,
    );
    assert!(ak.baseline.contains_key(&E));
    // Next frame nothing is selected → the baseline empties.
    frame(&mut st, &ph, &[], false, true, &mut ak);
    assert!(
        ak.baseline.is_empty(),
        "an unselected entity leaves the baseline"
    );
}

#[test]
fn a_disarmed_displaced_pose_pins_the_entity_for_a_manual_k() {
    // THE pose-to-pose bug: without auto-key, the apply pass snapped a
    // posed object back to its curve before K could record it. Disarmed,
    // an off-curve bound pose must join the displaced set (the apply skips
    // it); once the pose returns to the curve (K keyed it / an undo), it
    // heals out.
    let (mut st, ph) = state_with_tx_track(); // paused; curve says x = 5
    let mut ak = AutokeyState::default();
    // Disarmed, pose off-curve (7 vs 5) → pinned, and nothing keyed.
    let before = st.doc.clone();
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 7.0)]))],
        false,
        false,
        &mut ak,
    );
    assert!(ak.displaced.contains(&E), "off-curve disarmed pose pins");
    assert_eq!(st.doc, before, "disarmed: still no key");
    // The pose returns to the curve (a K keyed it, or an undo) → heals out.
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 5.0)]))],
        false,
        false,
        &mut ak,
    );
    assert!(ak.displaced.is_empty(), "on-curve pose heals the pin out");
    // Armed, the same off-curve pose is KEYED, never pinned.
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 7.0)]))],
        false,
        true,
        &mut ak,
    );
    assert!(ak.displaced.is_empty(), "armed keys instead of pinning");
    assert_eq!(tx_at(&st, 0.5), Some(7.0), "armed: the pose was keyed");
}

#[test]
fn a_remapped_entity_diffs_and_keys_at_its_source_time_not_the_playhead() {
    // The "Time bugado" shell half: the apply samples a remapped entity's
    // tracks at its SOURCE time, so the auto-key diff must compare — and a
    // new key must land — at that same time. Comparing at the raw playhead
    // reads the applied pose as "off-curve" every frame (spurious keys /
    // spurious pins), and a key inserted at the playhead is invisible at
    // the current pose (snap-back).
    let (mut st, ph) = state_with_tx_track(); // paused at 0.5; TX 0→0, 1→10
    let mut ak = AutokeyState::default();
    // 2x remap (0 → 0, 1 → 2): at playhead 0.5 the source time is 1.0,
    // where the TX curve says 10 (NOT the 5 it says at raw 0.5).
    let mut ph2 = Playhead::new(1.0 / 60.0);
    for (t, v) in [(0.0, 0.0f32), (1.0, 2.0)] {
        apply_intent(
            &mut st,
            &mut ph2,
            I::AddKey {
                entity: E,
                prop: PropKind::TimeRemap,
                t: RationalTime::from_seconds(t),
                value: AnimValue::Float(v),
                interp: ph2d_anim::Interp::Linear,
            },
        );
    }
    // The pose the apply just wrote (curve at SOURCE 1.0 = 10): on-curve →
    // keys nothing, pins nothing. (Diffing at raw 0.5 would scream 10 ≠ 5.)
    let before = st.doc.clone();
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 10.0)]))],
        false,
        true,
        &mut ak,
    );
    assert_eq!(st.doc, before, "the applied remapped pose is on-curve");
    // Disarmed, the same pose must not pin either.
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 10.0)]))],
        false,
        false,
        &mut ak,
    );
    assert!(ak.displaced.is_empty(), "on-curve at source time: no pin");
    // A pose that happens to sit on the curve's RAW-playhead value (5) is
    // genuinely displaced from the SOURCE pose (10) — it must be keyed. A
    // diff at the raw playhead reads it as on-curve and drops the edit.
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 5.0)]))],
        false,
        true,
        &mut ak,
    );
    assert_eq!(
        tx_at(&st, 1.0),
        Some(5.0),
        "the displaced pose keyed at SOURCE 1.0, where the apply samples it"
    );
    // And again with an arbitrary pose: the key lands at the source time.
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 12.0)]))],
        false,
        true,
        &mut ak,
    );
    assert_eq!(
        tx_at(&st, 1.0),
        Some(12.0),
        "the key landed at source 1.0, not playhead 0.5"
    );
}

#[test]
fn playing_does_not_auto_key_even_when_the_pose_looks_off_its_curve() {
    // The Play + AutoKey regression: during playback the apply pass rewrites
    // the pose every frame, and the frame-snap/raw-time mismatch made the
    // off-curve diff fire, minting a key per frame. While playing, the pass
    // must be inert regardless of the pose.
    let (mut st, mut ph) = state_with_tx_track(); // paused; curve x = 5 at 0.5
    let before = st.doc.clone();
    let mut ak = AutokeyState::default();
    // Armed (panel open + auto_key on) and the pose looks off-curve (7 vs 5),
    // but the transport is PLAYING → nothing is keyed.
    ph.play();
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 7.0)]))],
        false,
        true,
        &mut ak,
    );
    assert_eq!(st.doc, before, "playing suppresses auto-key");
    assert_eq!(
        ak.baseline.get(&E).and_then(|p| p[TX]),
        Some(7.0),
        "the baseline still advances while playing (so pausing is not a jump)"
    );
    // Control: the SAME off-curve pose DOES key once PAUSED, proving the play
    // gate — not something else — is what suppressed it.
    ph.pause();
    frame(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 7.0)]))],
        false,
        true,
        &mut ak,
    );
    assert_eq!(
        tx_at(&st, 0.5),
        Some(7.0),
        "paused: the off-curve pose keys as before"
    );
}

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
    // Release the drag — closes the one bracket.
    ph.seek(0.75);
    frame_perf(
        &mut st,
        &ph,
        &[(E, pose(&[(TX, 9.0)]))],
        false,
        false,
        &mut ak,
    );
    assert_eq!(track_len(&st), 5, "three recorded keys + the two originals");
    assert!(st.history.can_undo(), "the session banked an undo step");
    apply_intent(&mut st, &mut ph, I::Undo);
    assert_eq!(
        track_len(&st),
        2,
        "ONE undo removes the whole recording session, not just the last frame"
    );
}
