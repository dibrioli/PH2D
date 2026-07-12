//! **The refusal has to be audible** (ADR-0115 R9 + B7).
//!
//! Under a clip stack, "key it here" can have no answer, and the document refuses
//! rather than write a key that moves the object behind the animator's back. That
//! refusal is correct — and, unsaid, it is indistinguishable from a bug: you drag,
//! the object snaps back, nothing keys, nothing explains.
//!
//! These tests assert the two things that make it a feature instead: that it
//! **speaks**, with the right reason, and that it speaks **once** — a drag refuses
//! on every frame it lives, and sixty identical toasts a second is not information.

use super::test_helpers::*;
use super::*;
use ph2d_editor::ToastQueue;
use ph2d_timeline::{TimelineIntent as I, apply_intent};

/// The pose the animator is dragging to, frame by frame — a live drag moves every
/// frame, which is exactly the condition that would spam a per-frame toast.
fn drag_frames(
    st: &mut TimelineState,
    ph: &Playhead,
    ak: &mut AutokeyState,
    toasts: &mut ToastQueue,
    xs: &[f32],
) {
    for &x in xs {
        frame_toasts(st, ph, &[(E, pose(&[(TX, x)]))], true, true, ak, toasts);
    }
}

/// A lane holding one strip of `clip` over `[t0, t1)`.
fn lane_with_strip(st: &mut TimelineState, ph: &mut Playhead, clip: usize, t0: f64, t1: f64) {
    apply_intent(st, ph, I::AddLane);
    let lane = st.doc.stack().len() - 1;
    apply_intent(
        st,
        ph,
        I::AddStrip {
            lane,
            clip,
            t_start: t0,
            t_end: t1,
        },
    );
}

/// **The clip you are editing is not on screen right now.** Its strip sits at
/// `[2, 4)` and the playhead is at `0.5`: "here" is nowhere in it, so no key can
/// land. Say so — and say it once, however long the drag lasts.
#[test]
fn a_pose_keyed_where_the_clip_does_not_play_is_refused_out_loud_exactly_once() {
    let (mut st, mut ph) = state_with_tx_track();
    lane_with_strip(&mut st, &mut ph, 0, 2.0, 4.0); // the active clip, elsewhere
    let before = track_len(&st);

    let mut ak = AutokeyState::default();
    let mut toasts = ToastQueue::new();
    // Frame 1 seeds the baseline; the next three are the drag.
    drag_frames(
        &mut st,
        &ph,
        &mut ak,
        &mut toasts,
        &[5.0, 99.0, 100.0, 101.0],
    );

    assert_eq!(
        track_len(&st),
        before,
        "a refused key must not be written — that is the whole point of refusing"
    );
    let msgs: Vec<String> = toasts.iter().map(|t| t.message.clone()).collect();
    assert_eq!(
        msgs.len(),
        1,
        "three refusing frames, ONE toast: {msgs:?} (the latch is the feature)"
    );
    assert_eq!(msgs[0], ph2d_timeline::KeyRefusal::NotPlaying.message());
    assert_eq!(
        ak.refusal,
        Some(ph2d_timeline::KeyRefusal::NotPlaying),
        "and the latch holds the reason, so a CHANGE of reason can speak again"
    );
}

/// **A lane above owns the channel.** The active clip plays, so there is a place
/// to put the key — but an `Override` lane at full weight above it means nothing
/// the clip stores changes the pose. The solve is degenerate, and the honest
/// answers are "refuse" or "move the object silently".
#[test]
fn a_pose_a_lane_above_overrides_is_refused_with_the_reason_that_names_it() {
    let (mut st, mut ph) = state_with_tx_track();

    // A second clip, keyed on the SAME binding (bindings are the document's, not
    // the clip's — the AE precomp model), so it can own the channel.
    apply_intent(&mut st, &mut ph, I::AddClip);
    apply_intent(&mut st, &mut ph, I::SetActiveClip { index: 1 });
    apply_intent(
        &mut st,
        &mut ph,
        I::AddKey {
            entity: E,
            prop: PropKind::TranslationX,
            t: RationalTime::from_seconds(0.0),
            value: AnimValue::Float(42.0),
            interp: ph2d_anim::Interp::Linear,
        },
    );
    apply_intent(&mut st, &mut ph, I::SetActiveClip { index: 0 });

    // Lane 0: the clip being edited. Lane 1, ABOVE it: the other clip, Override at
    // full weight, covering the same span.
    lane_with_strip(&mut st, &mut ph, 0, 0.0, 4.0);
    lane_with_strip(&mut st, &mut ph, 1, 0.0, 4.0);
    let before = track_len(&st);

    let mut ak = AutokeyState::default();
    let mut toasts = ToastQueue::new();
    drag_frames(&mut st, &ph, &mut ak, &mut toasts, &[42.0, 99.0, 100.0]);

    assert_eq!(track_len(&st), before, "nothing the clip stores would land");
    let msgs: Vec<String> = toasts.iter().map(|t| t.message.clone()).collect();
    assert_eq!(msgs.len(), 1, "one reason, said once: {msgs:?}");
    assert_eq!(
        msgs[0],
        ph2d_timeline::KeyRefusal::Overridden.message(),
        "the clip DOES play here — the reason is the lane above it, and saying \
         `does not play here` would send the animator hunting the wrong thing"
    );
}

/// With no stack at all — the overwhelmingly common case — a pose edit keys, and
/// the animator is told nothing. A toast on the happy path is not a feature.
#[test]
fn an_ordinary_pose_edit_keys_and_says_nothing() {
    let (mut st, ph) = state_with_tx_track();
    let before = track_len(&st);

    let mut ak = AutokeyState::default();
    let mut toasts = ToastQueue::new();
    drag_frames(&mut st, &ph, &mut ak, &mut toasts, &[5.0, 99.0]);

    assert_eq!(track_len(&st), before + 1, "the key lands");
    assert_eq!(toasts.len(), 0, "and nothing is said about it");
    assert_eq!(ak.refusal, None);
}
