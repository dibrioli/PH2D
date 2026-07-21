//! **The view-side of nesting** — what the snapshot publishes when a container is open.
//!
//! The eval side lives in `nesting_clock.rs`/`nesting_leads.rs`; these gates pin the
//! SNAPSHOT, because the panel paints only what it is handed and a wrong (or panicking)
//! publish is invisible to every eval gate.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Playhead;
use ph2d_timeline::{
    EnterStep, PropKind, StackHost, StripSource, TimelineDoc, TimelineState,
    TimelineViewSnapshot,
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
            strip: first,
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

    assert_eq!(snap.host_time, None, "the Keys ruler is the clip's, not the host's");
    assert_eq!(snap.host_map, None, "no Arrange map is in view on the Keys tab");
    assert!(!snap.crumbs.is_empty(), "the trail still says where you are");
}

/// **The Arrange snapshot's map survives every playhead position** — including the gaps
/// between instances, where the scratch-derived map used to vanish and the ruler with it
/// (Enio, 2026-07-20: *"não consigo controlar/arrastar a playhead"*). The MARKER stays
/// honest to the scene clock: present only while the scene playhead is inside the entered
/// instance's window, mapped through the same relation the scrub writes with.
#[test]
fn the_arrange_map_holds_in_the_gaps_and_the_marker_stays_honest() {
    let (mut st, step) = nested_state();
    st.edit_path.push(step);
    let mut playhead = Playhead::new(1.0 / 60.0);
    let mut snap = TimelineViewSnapshot::default();

    // Inside the entered instance: map AND marker, in the interior's clock.
    playhead.seek(1.5);
    snap.rebuild(&mut st, &playhead, false);
    let m = snap.host_map.expect("inside the window the map is there");
    assert!((m.t0 - 0.0).abs() < 1e-9 && (m.t1 - 2.0).abs() < 1e-9);
    assert!(
        snap.host_time.is_some_and(|u| (u - 1.5).abs() < 1e-9),
        "marker at the interior second the map names, got {:?}",
        snap.host_time
    );

    // In the GAP between instances: the map — and with it the scrub — is still offered
    // (this is the fix); the marker is not (the interior is not being played HERE).
    playhead.seek(3.0);
    snap.rebuild(&mut st, &playhead, false);
    assert!(
        snap.host_map.is_some(),
        "the entry map must not flicker away in the gap — that froze the ruler"
    );
    assert_eq!(
        snap.host_time, None,
        "no marker: the scene playhead is outside the entered instance"
    );

    // Inside the OTHER instance: same story — the map is the ENTERED one's, the marker
    // absent (this instance is not the one the animator walked into).
    playhead.seek(5.0);
    snap.rebuild(&mut st, &playhead, false);
    let m = snap.host_map.expect("still the entered instance's map");
    assert!((m.t1 - 2.0).abs() < 1e-9, "still [0,2), not [4,6)");
    assert_eq!(snap.host_time, None);
}
