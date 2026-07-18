//! **The outward fade-in ("lead-in") — the travel across a gap that leaves the next
//! clip's opening untouched** (Enio, 2026-07-16).
//!
//! Where an INWARD fade-in blends against the clip while it PLAYS (spending its
//! opening in the crossfade), the lead-in blends against the clip's FROZEN first
//! frame: the object travels from the previous strip's held pose to this clip's start
//! pose during the gap, and then the clip plays from frame 0 untouched. It composes
//! with the gap-hold (`ClipLane::hold_at`) — the held strip is the pose it travels
//! FROM — without touching it.
//!
//! The oracle is the POSE (where the sprite is), never the ramp that produces it.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Playhead;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_timeline::{
    ClipStrip, PropKind, StripLoop, TimelineIntent, TimelineState, apply_from_doc, apply_intent,
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

/// One lane, one entity. `A` holds x at −3 over `[0, 2)`; then a GAP `[2, 3)`; then `B`
/// over `[3, 6)` ramps x from +1 (its first frame) to +5. Returns the state, the entity
/// bits, the lane index, and `B`'s strip id.
fn scene() -> (SimWorld, TimelineState, u64, usize, ph2d_timeline::StripId) {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Lead")))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.rename_clip(0, "A".into());
    key(doc, bits, PropKind::TranslationX, 0.0, -3.0);
    key(doc, bits, PropKind::TranslationX, 3.0, -3.0); // A holds −3 everywhere
    let b = doc.add_clip("B".into());
    doc.set_active(b);
    key(doc, bits, PropKind::TranslationX, 0.0, 1.0);
    key(doc, bits, PropKind::TranslationX, 3.0, 5.0); // B ramps 1 → 5 over its 3 s
    doc.set_active(0);
    let lane = doc.add_lane("L".into()).unwrap();
    doc.add_strip(lane, 0, 0.0, 2.0); // A on [0, 2)
    let b_strip = doc.add_strip(lane, b, 3.0, 6.0).unwrap(); // B on [3, 6), gap [2, 3)
    (sim, st, bits, lane, b_strip)
}

fn x_at(sim: &mut SimWorld, st: &mut TimelineState, bits: u64, t: f64) -> f64 {
    apply_from_doc(sim.world_mut(), &mut st.doc, t);
    let e = ph2d_ecs::Entity::from_bits(bits);
    f64::from(sim.world().get::<Transform>(e).unwrap().translation.x)
}

fn set_lead(st: &mut TimelineState, lane: usize, id: ph2d_timeline::StripId, secs: f64) {
    st.doc.strip_mut(lane, id).unwrap().lead_in = secs;
}

/// **With a lead-in, the object TRAVELS across the gap; without it, it HOLDS.** The
/// whole feature in one contrast: at the middle of the gap the sprite is on its way to
/// B's start pose (a lead of 1 s fills the gap `[2, 3)`), where before it simply sat at
/// A's held −3.
#[test]
fn the_lead_in_travels_across_the_gap_where_the_hold_would_sit_still() {
    let (mut sim, mut st, bits, lane, b) = scene();

    // No lead yet: the gap holds A's last pose (the 2026-07-16 gap-hold).
    assert_eq!(
        x_at(&mut sim, &mut st, bits, 2.5),
        -3.0,
        "no lead → A holds −3"
    );

    // A 1 s lead fills the gap. At its middle the sprite is between the two poses —
    // strictly moving, neither −3 (held) nor +1 (arrived).
    set_lead(&mut st, lane, b, 1.0);
    let mid = x_at(&mut sim, &mut st, bits, 2.5);
    assert!(
        -3.0 < mid && mid < 1.0,
        "mid-gap the sprite travels between A(−3) and B(+1), got {mid}"
    );
    assert!(mid > -2.0, "it has clearly LEFT the held pose, got {mid}");
}

/// **It arrives at B's FIRST frame exactly when B starts — and then B plays clean.**
/// The travel target is the frozen first frame (+1), reached at `t = 3`; from there the
/// clip plays LIVE and uncompromised (at `t = 3.5` the pose is B's own value half a
/// second in, not a blend).
#[test]
fn the_opening_of_the_next_clip_is_left_untouched() {
    let (mut sim, mut st, bits, lane, b) = scene();
    set_lead(&mut st, lane, b, 1.0);

    // Arrival: at B's start the weight is 1, so the pose is B's first frame, no blend.
    assert!(
        (x_at(&mut sim, &mut st, bits, 3.0) - 1.0).abs() < 1e-9,
        "at B.start the pose is B's first frame (+1), reached by the travel"
    );

    // B's opening is UNCOMPROMISED: half a second in, the pose is B's own ramp value
    // there (1 + 4·0.5/3 = 1.6667), the clip playing clean from frame 0.
    let pure = 1.0 + 4.0 * 0.5 / 3.0;
    assert!(
        (x_at(&mut sim, &mut st, bits, 3.5) - pure).abs() < 1e-6,
        "B plays clean from its start — {pure} expected"
    );
    // And it reaches its own interior normally.
    assert!(
        (x_at(&mut sim, &mut st, bits, 4.5) - 3.0).abs() < 1e-9,
        "B at clip 1.5 ramps to 3"
    );
}

/// **This is what the INWARD fade cannot do.** With an inward `ease_in` of the same
/// length the SAME instant (`t = 3.5`) is a blend of A's hold and B's live opening —
/// B's opening is spent in the crossfade. The lead-in leaves it pure. Same length, two
/// different pictures: that is why the outward fade is not redundant.
#[test]
fn the_lead_in_is_not_the_inward_fade() {
    let (mut sim, mut st, bits, lane, b) = scene();

    // Inward ease of 1 s: at t = 3.5, B's opening (clip 0.5 → 1.6667) is blended with
    // A's held −3 at weight smoothstep(0.5) = 0.5 → −0.6667.
    st.doc.strip_mut(lane, b).unwrap().ease_in = 1.0;
    let blended = x_at(&mut sim, &mut st, bits, 3.5);
    assert!(
        blended < 0.0,
        "inward ease blends B's opening away, got {blended}"
    );

    // Outward lead of 1 s: at the SAME instant B is pure (1.6667).
    st.doc.strip_mut(lane, b).unwrap().ease_in = 0.0;
    set_lead(&mut st, lane, b, 1.0);
    let pure = x_at(&mut sim, &mut st, bits, 3.5);
    assert!(
        pure > 1.0,
        "outward lead leaves B's opening pure, got {pure}"
    );
    assert!(
        (pure - blended).abs() > 2.0,
        "the two fades paint different pictures ({pure} vs {blended})"
    );
}

/// **The frozen first frame is `src_in`, whatever the loop mode.** The lead-in window
/// samples the clip's first frame (the travel target), not a wrapped or extrapolated
/// time — a `Loop` strip's lead-in still points at frame 0, not at wherever a negative
/// time would fold to.
#[test]
fn the_lead_in_window_reads_the_frozen_first_frame() {
    let mut s = ClipStrip::new(0, 3.0, 6.0, 3.0); // clip on [3, 6), slice 0..3
    s.src_in = 0.0;
    s.lead_in = 1.0; // reaches back to t = 2
    for mode in [StripLoop::Once, StripLoop::Loop, StripLoop::PingPong] {
        s.loop_mode = mode;
        // In the lead window [2, 3): frozen at src_in, whatever the loop mode.
        assert_eq!(
            s.source_time_with_lead(2.0),
            Some(0.0),
            "{mode:?} lead start"
        );
        assert_eq!(s.source_time_with_lead(2.5), Some(0.0), "{mode:?} mid-lead");
        // Inside the span it reads normally; before the lead and after the end, None.
        assert_eq!(s.source_time_with_lead(3.0), Some(0.0), "{mode:?} at start");
        assert_eq!(
            s.source_time_with_lead(1.9),
            None,
            "{mode:?} before the lead"
        );
        assert_eq!(s.source_time_with_lead(6.0), None, "{mode:?} past the end");
    }
}

/// **The lead-in is clamped to the GAP, and it clears the inward fade.** A lead longer
/// than the empty time before the strip is trimmed to the gap (it cannot overrun the
/// previous strip's live span), and authoring it zeroes `ease_in` — the fade-in handle
/// is on one side of the edge or the other, never both.
#[test]
fn set_strip_lead_clamps_to_the_gap_and_clears_the_inward_fade() {
    let (_sim, mut st, _bits, lane, b) = scene();
    let mut ph = Playhead::new(1.0 / 60.0);
    // Give the strip an inward fade first, to prove the outward one clears it.
    st.doc.strip_mut(lane, b).unwrap().ease_in = 0.5;

    // The gap before B is [2, 3) = 1 s. Ask for 10 s of lead.
    apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::SetStripLead {
            lane,
            id: b,
            seconds: 10.0,
        },
    );
    let s = st.doc.strip(lane, b).unwrap();
    assert!(
        (s.lead_in - 1.0).abs() < 1e-9,
        "lead clamped to the 1 s gap, got {}",
        s.lead_in
    );
    assert_eq!(s.ease_in, 0.0, "the inward fade was cleared");
}
