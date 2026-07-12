//! Shared harness for the auto-key test modules (`autokey_pass_tests.rs`,
//! `autokey_performing_tests.rs`) — helpers split out so each stays under the
//! HR-18 shell LOC cap. Pub(super) so both sibling test modules see them.
use super::*;
use ph2d_timeline::{TimelineIntent as I, apply_intent};

pub(super) const E: u64 = 1;
pub(super) const TX: usize = 0;
pub(super) const ROT: usize = 2;

pub(super) fn state_with_tx_track() -> (TimelineState, Playhead) {
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

pub(super) fn pose(vals: &[(usize, f32)]) -> PoseSample {
    let mut p: PoseSample = [None; 6];
    for &(i, v) in vals {
        p[i] = Some(v);
    }
    p
}

pub(super) fn tx_at(st: &TimelineState, t: f64) -> Option<f32> {
    use ph2d_anim::AttributeEvaluator;
    let target = st.doc.binding_for(E, PropKind::TranslationX)?.target;
    match st.doc.active_clip().track(target)?.sample(t) {
        AnimValue::Float(v) => Some(v),
        _ => None,
    }
}

/// One frame of the pass with the given selection samples (performing off —
/// these tests exercise ordinary auto-key; performing has its own block below).
pub(super) fn frame(
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
pub(super) fn frame_perf(
    st: &mut TimelineState,
    ph: &Playhead,
    samples: &[(u64, PoseSample)],
    drag_now: bool,
    armed: bool,
    ak: &mut AutokeyState,
) {
    apply_samples(st, ph, samples, drag_now, armed, true, ak);
}

pub(super) fn track_len(st: &TimelineState) -> usize {
    let target = st
        .doc
        .binding_for(E, PropKind::TranslationX)
        .unwrap()
        .target;
    st.doc.active_clip().track(target).unwrap().len()
}
