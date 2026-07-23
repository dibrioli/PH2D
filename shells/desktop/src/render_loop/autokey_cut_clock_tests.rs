//! **The authored-duration cut clock** (Enio, 2026-07-23) — the auto-key half.
//! Sibling of `autokey_pass_tests.rs` (HR-18 split); helpers in
//! `autokey_test_helpers.rs`.
use super::test_helpers::*;
use super::*;
use ph2d_timeline::PropKind;

// ── The authored-duration scrub regression (Enio, 2026-07-23) ───────────────
//
// "Quando coloco um valor baixo na duração e o keyhead está fora da área de
// duração, o autokey produz um bug que já havia sido resolvido: cria keyframes
// se arrasto em toda timeline por onde arrasto o playhead, mesmo o objeto parado."
//
// The cut (`length_override`) entered the APPLY's clock only: past the authored
// end the pose freezes at `curve(cut)` while the diff still sampled `curve(raw)`
// — a phantom delta on every scrubbed frame, keyed at a time the apply never
// samples. The playhead clamp upstream cannot be the defense: the Arrange clamp
// arm reads `scene_length`, which can be unauthored while the clip's own cut is
// live — the exact reported state, and the fixture below.

/// A world animated over `0..4 s` (`x = 10·t`) — long enough that an authored
/// duration of 2 s cuts MID-RAMP. The fixture must contain the phenomenon: at
/// the cut the curve is still CLIMBING, so freezing there leaves a fat phantom
/// delta against the raw clock (a flat tail would hide the bug entirely).
fn animated_world_4s() -> (ph2d_ecs::World, u64, TimelineState, Playhead) {
    use ph2d_ecs::{Transform, World};
    let mut w = World::new();
    let e = w
        .spawn(Transform::from_translation(ph2d_core::Vec2::ZERO))
        .id();
    let bits = e.to_bits();
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    for (t, v) in [(0.0, 0.0f32), (4.0, 40.0)] {
        ph2d_timeline::apply_intent(
            &mut st,
            &mut ph,
            ph2d_timeline::TimelineIntent::AddKey {
                entity: bits,
                prop: PropKind::TranslationX,
                t: RationalTime::from_seconds(t),
                value: AnimValue::Float(v),
                interp: ph2d_anim::Interp::Linear,
            },
        );
    }
    ph.pause();
    (w, bits, st, ph)
}

#[test]
fn scrubbing_beyond_an_authored_duration_mints_no_keys() {
    // Arrange view (keys_mode = false), EMPTY stack, `scene_length` unauthored:
    // the clamp arm has nothing to pin with, so the playhead genuinely sits
    // beyond the clip's cut — Enio's repro, both symptoms live.
    let (mut w, e, mut st, mut ph) = animated_world_4s();
    st.doc.set_clip_length_override(0, Some(2.0));
    let target = st
        .doc
        .binding_for(e, PropKind::TranslationX)
        .unwrap()
        .target;
    let before = st.doc.active_clip().track(target).unwrap().len();
    let mut ak = AutokeyState::default();

    // Scrub the dead region, off the rational grid (the 2026-07-12 lesson: a
    // clean 0.5 round-trips exactly and shows nothing).
    for i in 0..=60 {
        let t = 2.0 + 2.0 * f64::from(i) / 60.0 + 1.0 / 3.0 * 1e-6;
        ph.seek(t);
        ph2d_timeline::apply_from_doc(&mut w, &mut st.doc, ph.time());
        let pose = super::sample_pose(&w, e);
        apply_samples(
            &mut st,
            &ph,
            &[(e, pose)],
            false,
            true,
            false,
            &mut ak,
            &mut ph2d_editor::ToastQueue::new(),
        );
        assert_eq!(
            st.doc.active_clip().track(target).unwrap().len(),
            before,
            "scrubbing to t={t} minted a key: the apply froze the pose at \
             curve(cut) but the diff read curve(raw)"
        );
    }
}

#[test]
fn the_keys_solo_diff_rides_the_cut_clock_too() {
    // The same clock, solo half: Keys view (`keys_mode`) drives
    // `apply_active_clip`, which cuts by the clip — the diff must too. (In the
    // product the Keys clamp arm masks a plain scrub; the pass cannot lean on
    // that — scrub routes that skip the bridge exist, and the clamp is UX.)
    let (mut w, e, mut st, mut ph) = animated_world_4s();
    st.keys_mode = true;
    st.doc.set_clip_length_override(0, Some(2.0));
    let target = st
        .doc
        .binding_for(e, PropKind::TranslationX)
        .unwrap()
        .target;
    let before = st.doc.active_clip().track(target).unwrap().len();
    let mut ak = AutokeyState::default();

    for i in 0..=60 {
        let t = 2.0 + 2.0 * f64::from(i) / 60.0 + 1.0 / 3.0 * 1e-6;
        ph.seek(t);
        ph2d_timeline::apply_active_clip(&mut w, &mut st.doc, ph.time(), |_| false);
        let pose = super::sample_pose(&w, e);
        apply_samples(
            &mut st,
            &ph,
            &[(e, pose)],
            false,
            true,
            false,
            &mut ak,
            &mut ph2d_editor::ToastQueue::new(),
        );
        assert_eq!(
            st.doc.active_clip().track(target).unwrap().len(),
            before,
            "Keys solo: scrubbing to t={t} minted a key beyond the cut"
        );
    }
}

#[test]
fn a_deliberate_pose_edit_beyond_the_cut_keys_at_the_boundary() {
    // The fix tightens the CLOCK, not the ear (the rejected alternative was
    // suppressing auto-key past the end, which would deafen deliberate edits).
    // Park beyond the cut, MOVE the object: it still keys — and the key lands
    // AT the cut, the frame the animator is looking at, never at the raw
    // playhead the apply cannot reach.
    let (mut w, e, mut st, mut ph) = animated_world_4s();
    st.doc.set_clip_length_override(0, Some(2.0));
    let target = st
        .doc
        .binding_for(e, PropKind::TranslationX)
        .unwrap()
        .target;
    let before = st.doc.active_clip().track(target).unwrap().len();
    let mut ak = AutokeyState::default();

    ph.seek(3.0);
    ph2d_timeline::apply_from_doc(&mut w, &mut st.doc, ph.time());
    // Settle the baseline (this frame the pose IS the frozen curve).
    let frozen = super::sample_pose(&w, e);
    apply_samples(
        &mut st,
        &ph,
        &[(e, frozen)],
        false,
        true,
        false,
        &mut ak,
        &mut ph2d_editor::ToastQueue::new(),
    );
    assert_eq!(st.doc.active_clip().track(target).unwrap().len(), before);

    // The user drags the object off the frozen pose.
    apply_samples(
        &mut st,
        &ph,
        &[(e, pose(&[(TX, 99.0)]))],
        false,
        true,
        false,
        &mut ak,
        &mut ph2d_editor::ToastQueue::new(),
    );
    let keys = st.doc.active_clip().track(target).unwrap().keys().to_vec();
    assert_eq!(
        keys.len(),
        before + 1,
        "an ACTUAL pose edit beyond the cut must still key"
    );
    let new_key = keys
        .iter()
        .find(|k| (k.t.to_seconds() - 2.0).abs() < 1e-6)
        .unwrap_or_else(|| panic!("the new key must land ON the cut (2.0), keys: {keys:?}"));
    assert_eq!(
        new_key.value,
        AnimValue::Float(99.0),
        "and carry the dragged pose"
    );
}
