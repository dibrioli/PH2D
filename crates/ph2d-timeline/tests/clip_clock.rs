//! **One ruler cannot serve two clocks.**
//!
//! A key is stamped in its CLIP's time; a strip sits in the TIMELINE's. The panel
//! draws both against the same pixel column, so under a stack the same column
//! means two different instants — and nothing on screen says so. These gates pin
//! the clock the keys are ruled by ([`ph2d_timeline::clip_playhead`]) and hold it
//! to the SAME answer `K` gets, because a ruler that draws a playhead where the
//! key is refused is a ruler that lies.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Playhead;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_timeline::{
    KeyRefusal, PropKind, TimelineState, TimelineViewSnapshot, apply_from_doc, clip_playhead,
    key_home,
};

fn key(doc: &mut ph2d_timeline::TimelineDoc, bits: u64, p: PropKind, t: f64, v: f32) {
    doc.upsert_key(
        bits,
        p,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

/// The `PH2D_STACK_SMOKE` scene, headless: one object, clips `Left` (index 0) and
/// `Right`, one lane, `Left`[0,3) and `Right`[2,5) — a 1 s overlap.
///
/// **Both clips' keys live at clip time 0..3.** That is the whole point: `Right`'s
/// keys are authored at 0..3 and its strip plays at 2..5, so "3.0" names two
/// different instants depending on which half of the panel you read it in.
fn smoke_scene() -> (SimWorld, TimelineState, u64, usize) {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("StackDemo")))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.rename_clip(0, "Left".into());
    key(doc, bits, PropKind::TranslationX, 0.0, -3.0);
    key(doc, bits, PropKind::TranslationX, 3.0, -3.0);
    let right = doc.add_clip("Right".into());
    doc.set_active(right);
    key(doc, bits, PropKind::TranslationX, 0.0, 3.0);
    key(doc, bits, PropKind::TranslationX, 3.0, 3.0);
    doc.set_active(0);
    let lane = doc.add_lane("L".into()).unwrap();
    doc.add_strip(lane, 0, 0.0, 3.0);
    doc.add_strip(lane, right, 2.0, 5.0);
    (sim, st, bits, right)
}

/// **The bug, stated as a number.** Select `Right` and put the playhead at 4.0:
/// its keys draw at 0..3 while its strip sits at 2..5. The clip's clock reads
/// 2.0 there — so a ruler that drew the playhead at 4.0 over `Right`'s keys would
/// be pointing a full second past the end of a 3-second clip.
#[test]
fn the_same_ruler_column_means_two_different_instants_under_a_stack() {
    let (_sim, mut st, _bits, right) = smoke_scene();
    st.doc.set_active(right);
    st.doc.prime_stack(4.0);

    let clip_now = clip_playhead(&st.doc, 4.0).expect("Right plays exactly once at t=4");
    assert!(
        (clip_now - 2.0).abs() < 1e-9,
        "the clip's clock reads 2.0 at timeline 4.0, not {clip_now}"
    );
    assert!(
        clip_now < 3.0,
        "and it lands INSIDE the 3 s clip — the timeline's 4.0 does not: {clip_now}"
    );
}

/// Without a stack the clip IS the timeline, so the two clocks are one — which is
/// why a document that never touches the feature sees the ruler it always had.
#[test]
fn without_a_stack_the_clip_clock_is_the_timeline_clock() {
    let mut st = TimelineState::new();
    for t in [0.0, 0.5, 2.0, 7.25] {
        assert_eq!(clip_playhead(&st.doc, t), Ok(t));
    }
    // Still true once there are keys but no lanes — a stack is lanes, not keys.
    let mut sim = SimWorld::new();
    let bits = sim.world_mut().spawn(Transform::default()).id().to_bits();
    key(&mut st.doc, bits, PropKind::TranslationX, 1.0, 5.0);
    assert_eq!(clip_playhead(&st.doc, 1.5), Ok(1.5));
}

/// The clip is not under the playhead at all: there is no instant in it to point
/// at, and saying "0" would be a confident lie.
#[test]
fn a_clip_that_does_not_play_here_has_no_clock() {
    let (_sim, mut st, _bits, right) = smoke_scene();
    st.doc.set_active(right); // Right plays [2, 5)
    st.doc.prime_stack(0.5);
    assert_eq!(clip_playhead(&st.doc, 0.5), Err(KeyRefusal::NotPlaying));
}

/// It plays TWICE: "here" names two places in it. Picking one silently is exactly
/// what `sole_strip_of` exists to refuse — and the ruler inherits the refusal.
#[test]
fn a_clip_playing_twice_has_no_single_clock() {
    let (_sim, mut st, _bits, _right) = smoke_scene();
    // A second strip of `Left`, overlapping the first: at t=1 the clip plays twice.
    let lane = 0;
    st.doc.add_strip(lane, 0, 0.5, 3.5);
    st.doc.prime_stack(1.0);
    assert_eq!(st.doc.active_index(), 0, "Left is the clip being edited");
    assert_eq!(clip_playhead(&st.doc, 1.0), Err(KeyRefusal::PlaysTwice));
}

/// **One door.** The ruler and `K` must agree on *whether* the clip is playing
/// exactly once — a playhead drawn at an instant K refuses to key is a ruler that
/// lies. (They differ in what they return where it IS playing: `key_home` composes
/// the entity's Time Remap on top; the ruler has no entity to ask.)
#[test]
fn the_ruler_and_the_key_agree_on_whether_the_clip_plays_here() {
    let (_sim, mut st, bits, right) = smoke_scene();
    st.doc.set_active(right);
    for t in [0.0, 0.5, 1.9, 2.0, 2.5, 3.0, 4.0, 4.99, 5.0, 6.0] {
        st.doc.prime_stack(t);
        let ruler = clip_playhead(&st.doc, t);
        let key = key_home(&st.doc, bits, t);
        assert_eq!(
            ruler.is_ok(),
            key.is_ok(),
            "t={t}: the ruler says {ruler:?} and K says {key:?}"
        );
        // No Time Remap on this entity, so where both answer they answer the same.
        if let (Ok(a), Ok(b)) = (ruler, key) {
            assert!((a - b).abs() < 1e-9, "t={t}: ruler {a} vs key {b}");
        }
    }
}

/// The snapshot the panel paints from carries the clip's clock, so the panel never
/// has to work it out from the strips it can see (which would be a second door).
#[test]
fn the_snapshot_publishes_the_clip_clock_the_keys_are_ruled_by() {
    let (mut sim, mut st, _bits, right) = smoke_scene();
    st.doc.set_active(right);
    let mut playhead = Playhead::default();
    playhead.seek(4.0);
    // The apply is what primes the scratch in the real frame; do what it does.
    apply_from_doc(sim.world_mut(), &mut st.doc, playhead.time());

    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &playhead);
    assert_eq!(snap.time_seconds, 4.0, "the TIMELINE's clock");
    let clip_time = snap.clip_time.expect("Right plays once at t=4");
    assert!(
        (clip_time - 2.0).abs() < 1e-9,
        "the CLIP's clock: {clip_time}"
    );
    // And the two halves it rules really are in those two frames.
    assert!(
        snap.tracks
            .iter()
            .flat_map(|t| &t.keys)
            .all(|k| k.t_seconds <= 3.0),
        "Right's keys are authored in clip time (0..3)"
    );
    let strip = &snap.lanes[0]
        .strips
        .iter()
        .find(|s| s.clip_name == "Right")
        .expect("Right's strip");
    assert_eq!(
        (strip.t_start, strip.t_end),
        (2.0, 5.0),
        "and its strip sits in timeline time"
    );
}

/// A document with no stack publishes the timeline's clock as the clip's — the
/// panel reads ONE field either way, and it is right in both worlds.
#[test]
fn without_a_stack_the_snapshot_publishes_the_playhead_as_the_clip_clock() {
    let mut st = TimelineState::new();
    let mut playhead = Playhead::default();
    playhead.seek(1.25);
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &playhead);
    assert_eq!(snap.clip_time, Some(1.25));
    assert_eq!(snap.time_seconds, 1.25);
}
