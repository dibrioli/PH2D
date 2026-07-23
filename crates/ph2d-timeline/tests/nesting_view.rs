//! **The view-side of nesting** — what the snapshot publishes when a container is open.
//!
//! The eval side lives in `nesting_clock.rs`/`nesting_leads.rs`; these gates pin the
//! SNAPSHOT, because the panel paints only what it is handed and a wrong (or panicking)
//! publish is invisible to every eval gate.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Playhead;
use ph2d_timeline::{
    EnterStep, PropKind, StackHost, StripSource, TimelineDoc, TimelineState, TimelineViewSnapshot,
};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn key(doc: &mut TimelineDoc, e: u64, prop: PropKind, t: f64, v: f32) {
    doc.upsert_key(e, prop, s(t), AnimValue::Float(v), Interp::Linear);
}

/// One container ("Walk", 2 s of interior) instanced twice on the scene: `[0,2)` and
/// `[4,6)` — entered through the FIRST instance.
fn nested_state() -> (TimelineState, EnterStep) {
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    let e = 7u64;
    doc.rename_clip(0, "Step".to_string());
    key(doc, e, PropKind::TranslationX, 0.0, -2.0);
    key(doc, e, PropKind::TranslationX, 2.0, 0.0);
    let walk = doc.add_container("Walk".to_string());
    let host = StackHost::Container(walk);
    let inner = doc.add_lane_in(host, "Steps".to_string()).unwrap();
    doc.add_strip_to(host, inner, StripSource::Clip(0), 0.0, 2.0)
        .unwrap();
    let lane = doc.add_lane("Timeline".to_string()).unwrap();
    let src = StripSource::Container(u16::try_from(walk).unwrap());
    let first = doc
        .add_strip_to(StackHost::Document, lane, src, 0.0, 2.0)
        .unwrap();
    doc.add_strip_to(StackHost::Document, lane, src, 4.0, 6.0)
        .unwrap();
    (
        st,
        EnterStep {
            container: walk,
            lane,
            strip: Some(first),
        },
    )
}

/// **Clicking the Keys tab inside a container must not crash the app** (Enio, 2026-07-20:
/// *"panic quando eu estava dentro do container Jump e tentei apertar na aba keys"*).
///
/// In Keys mode the rebuild skips `prime_stack` (there is no stack in view), but the
/// open-container branch still asked `container_playhead`/`container_map` — whose scratch
/// precondition (`debug_assert_scratch_at`) the skipped prime no longer satisfies. The
/// publish must not ride a clock it did not prime: in Keys mode the host readouts are
/// simply not in view, so they publish as `None`.
#[test]
fn the_keys_tab_inside_a_container_does_not_panic() {
    let (mut st, step) = nested_state();
    st.edit_path.push(step);
    let mut playhead = Playhead::new(1.0 / 60.0);
    playhead.seek(0.5);

    // Prime an ARRANGE frame first, at a DIFFERENT instant — the real sequence: the artist
    // stood in Arrange (primed at its clock), then clicked Keys (another clock, no prime).
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &playhead, false);
    playhead.seek(1.25);
    snap.rebuild(&mut st, &playhead, true); // keys_mode — this line panicked

    assert_eq!(
        snap.host_time, None,
        "the Keys ruler is the clip's, not the host's"
    );
    assert_eq!(
        snap.host_map, None,
        "no Arrange map is in view on the Keys tab"
    );
    assert!(
        !snap.crumbs.is_empty(),
        "the trail still says where you are"
    );
}

/// **Inside a container the ruler is the container's OWN clock — identity, not the scene
/// mapped** (Enio, 2026-07-22: *"o playback deve ser relativo ao container aberto"*). The
/// shell hands `rebuild` the CONTAINER playhead, so `time_seconds`/`host_time` ARE the
/// interior-local time, `container_open` marks the view, and the `host_map` readout — a
/// SCENE fact (where the instance plays) — survives untouched because it is a pure function
/// of the doc+path, independent of which clock the transport runs.
#[test]
fn inside_a_container_the_ruler_is_its_own_clock_and_the_scene_readout_survives() {
    let (mut st, step) = nested_state();
    st.edit_path.push(step);
    let mut playhead = Playhead::new(1.0 / 60.0);
    let mut snap = TimelineViewSnapshot::default();

    // The playhead handed in is the CONTAINER clock: whatever local second it stands at,
    // that is the ruler's now, and it is offered as such.
    for local in [0.25, 1.5] {
        playhead.seek(local);
        snap.rebuild(&mut st, &playhead, false);
        assert_eq!(
            snap.container_open,
            Some(step.container),
            "the view is marked as the container's — the door the ruler reads"
        );
        assert!(
            snap.host_time.is_some_and(|u| (u - local).abs() < 1e-9),
            "the ruler marks the interior-local playhead ({local}), got {:?}",
            snap.host_time
        );
        assert!(
            (snap.time_seconds - local).abs() < 1e-9,
            "time_seconds IS the container clock now"
        );
    }

    // The breadcrumb readout is the SCENE relation — where the entered instance plays on
    // the timeline — and it is there regardless of where the container clock stands.
    let m = snap
        .host_map
        .expect("the scene relation survives the clock change");
    assert!(
        (m.t0 - 0.0).abs() < 1e-9 && (m.t1 - 2.0).abs() < 1e-9,
        "the [0,2) instance"
    );
}

/// **The loop braces inside a container are the CONTAINER's OWN loop** — read from the
/// document (`container_loop`), in the interior's own seconds, NEVER the scene's loop
/// mapped in (Enio, 2026-07-22: *"o loop deve ser independente em cada modo"*). The SCENE
/// loop is a DECOY on purpose: a display that reads it bleeds the Arrange's cycle into the
/// container.
#[test]
fn the_loop_braces_inside_are_the_containers_own_loop() {
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    let e = 7u64;
    doc.rename_clip(0, "Step".to_string());
    key(doc, e, PropKind::TranslationX, 0.0, -2.0);
    key(doc, e, PropKind::TranslationX, 2.0, 0.0);
    let walk = doc.add_container("Walk".to_string());
    let host = StackHost::Container(walk);
    let inner = doc.add_lane_in(host, "Steps".to_string()).unwrap();
    doc.add_strip_to(host, inner, StripSource::Clip(0), 0.0, 2.0)
        .unwrap();
    let lane = doc.add_lane("Timeline".to_string()).unwrap();
    let strip = doc
        .add_strip_to(
            StackHost::Document,
            lane,
            StripSource::Container(u16::try_from(walk).unwrap()),
            4.0,
            6.0,
        )
        .unwrap();
    doc.set_active_loop_for(false, Some((0.0, 20.0))); // o decoy: o loop da CENA
    // The container's OWN loop — a proper subset of its own [0,2) interior, distinct from
    // the scene decoy so a display that read the wrong one cannot match by accident.
    doc.set_container_loop(walk, Some((0.5, 1.5)), false);
    st.edit_path.push(EnterStep {
        container: walk,
        lane,
        strip: Some(strip),
    });

    let mut playhead = Playhead::new(1.0 / 60.0);
    playhead.seek(1.0);
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &playhead, false);
    assert_eq!(
        snap.loop_range,
        Some((0.5, 1.5)),
        "as chaves são o loop DO CONTAINER (0.5,1.5) — não o (0,20) da cena"
    );
    assert!(!snap.loop_ping_pong);

    // Ping-pong is the container's own too.
    st.doc.set_container_loop(walk, Some((0.5, 1.5)), true);
    snap.rebuild(&mut st, &playhead, false);
    assert!(snap.loop_ping_pong, "ping-pong do container, não da cena");

    // E na aba KEYS o loop volta a ser o do clip (o relógio dali é o do clip).
    snap.rebuild(&mut st, &playhead, true);
    assert_eq!(
        snap.loop_range,
        st.doc.active_loop_for(true),
        "a Keys mostra o loop do clip, como sempre"
    );
}
