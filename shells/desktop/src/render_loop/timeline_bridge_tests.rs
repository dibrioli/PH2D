//! Unit tests for [`super`] (`timeline_bridge.rs`) — extracted to a sibling module
//! (`#[path]`, the idiom `autokey_pass` already uses) so the bridge itself stays
//! under the HR-18 shell LOC cap. The bridge is 240 lines of code; these were 370
//! lines of tests sharing its file, and the cap was measuring the sum.

use super::*;
use ph2d_editor::ids;

#[test]
fn transport_ids_map_to_intents() {
    let st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.seek_frame(10, st.doc.fps_display);

    assert_eq!(
        intent_for_transport(&PanelEvent::Click(ids::TIMELINE_PLAY), &st, &ph),
        Some(TimelineIntent::TogglePlay)
    );
    assert_eq!(
        intent_for_transport(&PanelEvent::Click(ids::TIMELINE_NEXT_FRAME), &st, &ph),
        Some(TimelineIntent::SeekFrame(11))
    );
    assert_eq!(
        intent_for_transport(&PanelEvent::Click(ids::TIMELINE_PREV_FRAME), &st, &ph),
        Some(TimelineIntent::SeekFrame(9))
    );
    assert_eq!(
        intent_for_transport(&PanelEvent::SetValue(ids::TIMELINE_TIME_NUM, 1.5), &st, &ph),
        Some(TimelineIntent::Scrub(1.5))
    );
    assert_eq!(
        intent_for_transport(&PanelEvent::Toggle(ids::TIMELINE_AUTOKEY, true), &st, &ph),
        Some(TimelineIntent::SetAutoKey(true))
    );
    assert_eq!(
        intent_for_transport(&PanelEvent::Toggle(ids::TIMELINE_RECORD, true), &st, &ph),
        Some(TimelineIntent::SetPerforming(true))
    );
    assert_eq!(
        intent_for_transport(&PanelEvent::Toggle(ids::TIMELINE_SNAP, false), &st, &ph),
        Some(TimelineIntent::SetFrameSnap(false))
    );
    // Physics (ADR-0131): one transport, two consumers.
    assert_eq!(
        intent_for_transport(&PanelEvent::Toggle(ids::TIMELINE_PHYSICS, true), &st, &ph),
        Some(TimelineIntent::SetSimulatePhysics(true))
    );
    // A non-transport id (Close is handled in the panel, not translated).
    assert_eq!(
        intent_for_transport(&PanelEvent::Click(ids::TIMELINE_CLOSE), &st, &ph),
        None
    );
}

#[test]
fn go_start_and_go_end_scrub_to_the_clip_bounds() {
    use ph2d_timeline::PropKind;
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);

    // Empty doc: both ends are t = 0.
    assert_eq!(
        intent_for_transport(&PanelEvent::Click(ids::TIMELINE_GO_START), &st, &ph),
        Some(TimelineIntent::Scrub(0.0))
    );
    assert_eq!(
        intent_for_transport(&PanelEvent::Click(ids::TIMELINE_GO_END), &st, &ph),
        Some(TimelineIntent::Scrub(0.0))
    );

    // Key at 2.5 s: go-to-end must follow the LAST KEY, not the clip's
    // authored duration (0 on a fresh clip — a dead button otherwise).
    ph2d_timeline::apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::AddKey {
            entity: 1,
            prop: PropKind::TranslationX,
            t: ph2d_anim::RationalTime::from_seconds(2.5),
            value: ph2d_anim::AnimValue::Float(3.0),
            interp: default_interp(),
        },
    );
    assert_eq!(
        intent_for_transport(&PanelEvent::Click(ids::TIMELINE_GO_END), &st, &ph),
        Some(TimelineIntent::Scrub(2.5))
    );
    assert_eq!(
        intent_for_transport(&PanelEvent::Click(ids::TIMELINE_GO_START), &st, &ph),
        Some(TimelineIntent::Scrub(0.0))
    );
}

#[test]
fn the_default_loop_range_spans_to_the_last_key() {
    use ph2d_timeline::PropKind;
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    ph2d_timeline::apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::AddKey {
            entity: 1,
            prop: PropKind::TranslationX,
            t: ph2d_anim::RationalTime::from_seconds(4.0),
            value: ph2d_anim::AnimValue::Float(1.0),
            interp: default_interp(),
        },
    );
    assert_eq!(
        intent_for_transport(&PanelEvent::Toggle(ids::TIMELINE_LOOP, true), &st, &ph),
        Some(TimelineIntent::SetLoop {
            range: Some((0.0, 4.0)),
            ping_pong: false,
        }),
        "looping a hand-keyed clip must not collapse to a zero-length range"
    );
}

/// **Arming the loop in Arrange brackets the WHOLE stack, not the first strip**
/// (Enio, 2026-07-16).
///
/// The fixture is the trap: the active clip's last key is at 3 s and the first strip
/// plays it 1:1 from the top, so bracketing the CLIP produces `(0, 3)` — a range that
/// looks deliberate and leaves the strip ending at 9 s outside the loop.
///
/// Go-to-end is asserted beside it because it is the SAME question through the same
/// door: "where does the content end". Fixing one and not the other would leave the
/// transport disagreeing with itself about the end of the same timeline.
#[test]
fn in_arrange_the_end_is_the_last_strips_end_not_the_active_clips() {
    use ph2d_timeline::PropKind;
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    ph2d_timeline::apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::AddKey {
            entity: 1,
            prop: PropKind::TranslationX,
            t: ph2d_anim::RationalTime::from_seconds(3.0),
            value: ph2d_anim::AnimValue::Float(1.0),
            interp: default_interp(),
        },
    );
    let lane = st.doc.add_lane("L".into()).expect("lane");
    st.doc.add_strip(lane, 0, 0.0, 3.0).expect("first strip");
    st.doc.add_strip(lane, 0, 6.0, 9.0).expect("last strip");

    // Arrange: the timeline's clock, so the stack is the content.
    st.keys_mode = false;
    assert_eq!(
        intent_for_transport(&PanelEvent::Toggle(ids::TIMELINE_LOOP, true), &st, &ph),
        Some(TimelineIntent::SetLoop {
            range: Some((0.0, 9.0)),
            ping_pong: false,
        }),
        "the loop must bracket every strip, not just the first"
    );
    assert_eq!(
        intent_for_transport(&PanelEvent::Click(ids::TIMELINE_GO_END), &st, &ph),
        Some(TimelineIntent::Scrub(9.0)),
        "and go-to-end lands on the same end"
    );

    // Keys: the clip's own clock, so the clip is the content — the mirror, without
    // which "always ask the stack" would pass both asserts above.
    st.keys_mode = true;
    assert_eq!(
        intent_for_transport(&PanelEvent::Toggle(ids::TIMELINE_LOOP, true), &st, &ph),
        Some(TimelineIntent::SetLoop {
            range: Some((0.0, 3.0)),
            ping_pong: false,
        }),
        "the Keys view still brackets the clip it is editing"
    );
}

/// A document nobody has arranged has no stack to bracket, so the clip IS the
/// timeline — the fallback must not collapse the range to zero.
#[test]
fn with_no_strips_the_timeline_end_is_still_the_clips_end() {
    use ph2d_timeline::PropKind;
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    ph2d_timeline::apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::AddKey {
            entity: 1,
            prop: PropKind::TranslationX,
            t: ph2d_anim::RationalTime::from_seconds(4.0),
            value: ph2d_anim::AnimValue::Float(1.0),
            interp: default_interp(),
        },
    );
    st.keys_mode = false; // Arrange, but nothing arranged
    assert_eq!(
        intent_for_transport(&PanelEvent::Toggle(ids::TIMELINE_LOOP, true), &st, &ph),
        Some(TimelineIntent::SetLoop {
            range: Some((0.0, 4.0)),
            ping_pong: false,
        })
    );
}

#[test]
fn only_absolute_jumps_ask_the_panel_to_pan() {
    for ev in [
        PanelEvent::Click(ids::TIMELINE_GO_START),
        PanelEvent::Click(ids::TIMELINE_GO_END),
        PanelEvent::Click(ids::TIMELINE_PREV_FRAME),
        PanelEvent::Click(ids::TIMELINE_NEXT_FRAME),
        PanelEvent::SetValue(ids::TIMELINE_TIME_NUM, 9.0),
        PanelEvent::SetValue(ids::TIMELINE_FRAME_NUM, 200.0),
    ] {
        assert!(jumps_the_playhead(&ev), "{ev:?} lands at an absolute time");
    }
    // The ruler scrub maps a fraction of the VISIBLE span, so it can never
    // land off-screen — panning after it would fight the drag.
    assert!(!jumps_the_playhead(&PanelEvent::SetValue(
        ids::TIMELINE_RULER,
        0.5
    )));
    assert!(!jumps_the_playhead(&PanelEvent::Click(ids::TIMELINE_PLAY)));
    assert!(!jumps_the_playhead(&PanelEvent::Toggle(
        ids::TIMELINE_LOOP,
        true
    )));
}

#[test]
fn sample_reads_transform_and_opacity() {
    use ph2d_anim::AnimValue;
    use ph2d_core::Vec2;
    use ph2d_ecs::{Transform, World};
    let mut w = World::new();
    let e = w
        .spawn(Transform::from_translation(Vec2::new(7.0, -3.0)))
        .id();
    assert_eq!(
        sample_prop_value(&w, e.to_bits(), PropKind::TranslationX),
        Some(AnimValue::Float(7.0))
    );
    assert_eq!(
        sample_prop_value(&w, e.to_bits(), PropKind::TranslationY),
        Some(AnimValue::Float(-3.0))
    );
    // No Sprite component → opacity sample is None (skipped, not a panic).
    assert_eq!(sample_prop_value(&w, e.to_bits(), PropKind::Opacity), None);
}

#[test]
fn addprop_ids_map_to_prop_kinds() {
    use ph2d_timeline::PropKind;
    assert_eq!(
        prop_for_addprop_id(ids::TIMELINE_ADDPROP_TX),
        Some(PropKind::TranslationX)
    );
    assert_eq!(
        prop_for_addprop_id(ids::TIMELINE_ADDPROP_OPACITY),
        Some(PropKind::Opacity)
    );
    assert_eq!(
        prop_for_addprop_id(ids::TIMELINE_ADDPROP_TIME),
        Some(PropKind::TimeRemap)
    );
    assert_eq!(prop_for_addprop_id(ids::TIMELINE_PLAY), None);
}
