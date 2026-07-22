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

/// The scene of the container-navigation gates: a 2 s container instanced at `[4, 12)`
/// (stretched 2× → speed 0.5) with a 1 s lead-in and 0.5 s lead-out, and a DOCUMENT loop
/// over `[0, 20)`. Returns the state (path stamped empty) and the entry step.
fn nav_scene() -> (TimelineState, ph2d_timeline::EnterStep) {
    use ph2d_timeline::{StackHost, StripSource};
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    let c = doc.add_container("C".into());
    doc.add_lane_in(StackHost::Container(c), "l".into())
        .unwrap();
    doc.add_strip_to(StackHost::Container(c), 0, StripSource::Clip(0), 0.0, 2.0)
        .unwrap();
    let lane = doc.add_lane("doc".into()).unwrap();
    let strip = doc
        .add_strip_to(
            StackHost::Document,
            lane,
            StripSource::Container(u16::try_from(c).unwrap()),
            4.0,
            12.0,
        )
        .unwrap();
    {
        let s = doc.strip_in_mut(StackHost::Document, lane, strip).unwrap();
        s.lead_in = 1.0;
        s.lead_out = 0.5;
    }
    doc.set_active_loop_for(false, Some((0.0, 20.0)));
    (
        st,
        ph2d_timeline::EnterStep {
            container: c,
            lane,
            strip: Some(strip),
        },
    )
}

/// **Entering a container brackets the transport loop around the entered instance —
/// leads included — and leaving re-installs the document's own loop** (Enio, 2026-07-20:
/// *"dentro do container o loop não se ajustou automaticamente"*). Navigation never
/// touches the DOCUMENT's loop: it is the artist's authored range, merely stepped aside
/// from while inside.
#[test]
fn entering_brackets_the_loop_around_the_instance_and_leaving_restores() {
    let (st, step) = nav_scene();
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.seek(5.0);

    on_nav_change(&st.doc, &[step], &mut ph);
    assert_eq!(
        ph.loop_range(),
        Some((3.0, 12.5)),
        "the instance's window plus its leads — bracketing only [4,12) cuts the fades"
    );
    assert!((ph.time() - 5.0).abs() < 1e-9, "already inside: no seek");
    assert_eq!(
        st.doc.active_loop_for(false),
        Some((0.0, 20.0)),
        "navigation is not an edit — the document loop is untouched"
    );

    on_nav_change(&st.doc, &[], &mut ph);
    assert_eq!(
        ph.loop_range(),
        Some((0.0, 20.0)),
        "leaving hands the transport back to the document's own loop"
    );
}

/// **A playhead standing outside the entered instance is moved to its start** — the
/// alternative is a marker-less ruler that reads as broken (the pre-fix symptom).
#[test]
fn entering_from_outside_the_window_seeks_to_its_start() {
    let (st, step) = nav_scene();
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.seek(17.0); // beyond the instance's reach [3, 12.5]
    on_nav_change(&st.doc, &[step], &mut ph);
    assert!(
        (ph.time() - 3.0).abs() < 1e-9,
        "outside the reach the transport lands at its start, got {}",
        ph.time()
    );
}

/// **A stale walk leaves the transport alone** — no window, no guess.
#[test]
fn a_stale_walk_leaves_the_transport_as_it_stands() {
    let (mut st, step) = nav_scene();
    st.doc.remove_strip_in(
        ph2d_timeline::StackHost::Document,
        step.lane,
        step.strip.unwrap(),
    );
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.set_loop(0.0, 20.0);
    ph.seek(5.0);
    on_nav_change(&st.doc, &[step], &mut ph);
    assert_eq!(ph.loop_range(), Some((0.0, 20.0)), "loop untouched");
    assert!((ph.time() - 5.0).abs() < 1e-9, "playhead untouched");
}

/// **Dentro de um container, Loop/PingPong/ir-ao-início/ir-ao-fim falam da INSTÂNCIA** —
/// transporte-apenas, o documento fica de fora (Enio, 2026-07-20: *"o loop dentro do
/// container deve se ajustar automaticamente quando ligado às strips"*). Escrever
/// `SetLoop` dali reescreveria o loop autorado da CENA com a janela de uma instância, e
/// sair do container revelaria o estrago.
#[test]
fn inside_a_container_the_loop_toggles_bracket_the_instance_on_the_transport() {
    let (mut st, step) = nav_scene();
    st.edit_path = vec![step];
    let ph = Playhead::new(1.0 / 60.0);
    let ev = |on| PanelEvent::Toggle(ph2d_editor::ids::TIMELINE_LOOP, on);

    assert_eq!(
        intent_for_transport(&ev(true), &st, &ph),
        Some(TimelineIntent::SetTransportLoop {
            range: Some((3.0, 12.5)), // o alcance da instância, leads incluídos
            ping_pong: false,
        }),
    );
    assert_eq!(
        intent_for_transport(&ev(false), &st, &ph),
        Some(TimelineIntent::SetTransportLoop {
            range: None,
            ping_pong: false,
        }),
    );
    assert_eq!(
        intent_for_transport(
            &PanelEvent::Toggle(ph2d_editor::ids::TIMELINE_PINGPONG, true),
            &st,
            &ph
        ),
        Some(TimelineIntent::SetTransportLoop {
            range: Some((3.0, 12.5)),
            ping_pong: true,
        }),
    );
    assert_eq!(
        intent_for_transport(
            &PanelEvent::Click(ph2d_editor::ids::TIMELINE_GO_START),
            &st,
            &ph
        ),
        Some(TimelineIntent::Scrub(3.0)),
        "ir ao início vai ao início da instância"
    );
    assert_eq!(
        intent_for_transport(
            &PanelEvent::Click(ph2d_editor::ids::TIMELINE_GO_END),
            &st,
            &ph
        ),
        Some(TimelineIntent::Scrub(12.5)),
        "ir ao fim vai ao fim do alcance dela"
    );

    // Na RAIZ nada muda: o toggle segue escrevendo o loop do DOCUMENTO.
    st.edit_path.clear();
    assert!(
        matches!(
            intent_for_transport(&ev(true), &st, &ph),
            Some(TimelineIntent::SetLoop {
                range: Some(_),
                ping_pong: false
            })
        ),
        "na cena o Loop é do documento, como sempre"
    );
}

/// **`SetTransportLoop` arma o RELÓGIO e não toca o documento** — a metade que protege o
/// loop autorado da cena.
#[test]
fn set_transport_loop_arms_the_clock_and_leaves_the_document_alone() {
    let (mut st, _step) = nav_scene();
    let mut ph = Playhead::new(1.0 / 60.0);
    let before = st.doc.active_loop_for(false);
    ph2d_timeline::apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::SetTransportLoop {
            range: Some((2.0, 5.0)),
            ping_pong: true,
        },
    );
    assert_eq!(ph.loop_range(), Some((2.0, 5.0)));
    assert!(ph.is_ping_pong());
    assert_eq!(
        st.doc.active_loop_for(false),
        before,
        "o documento fica de fora"
    );
    ph2d_timeline::apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::SetTransportLoop {
            range: None,
            ping_pong: false,
        },
    );
    assert_eq!(ph.loop_range(), None, "off limpa o relógio");
}
