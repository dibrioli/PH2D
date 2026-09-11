//! Additive lanes playing CONTAINER strips (Enio, 2026-07-23).
//!
//! The bug these gates pin: the additive delta of a container strip is measured
//! against "the interior at `src_in`" — and an interior whose first strip starts
//! later than that (every strip is born AT THE PLAYHEAD, so almost all of them)
//! answered nothing there. The reference then fell back to the live value, and
//! `v - v` is an exact zero: the whole additive strip was silently inert, while
//! the same clip on the same lane worked (a track CLAMPS before its first key).
//!
//! The fixtures deliberately start the interior's animation AWAY from the rest
//! pose (10, not 0): with a first value equal to rest, "clamp to the first
//! voice" and "fall back to rest" produce the same number and a broken clamp
//! stays green.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{
    ClipLane, ClipStrip, LaneMode, PropKind, StripSource, TimelineDoc, apply_from_doc,
};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn scene() -> (World, TimelineDoc, u64) {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();
    (world, TimelineDoc::new(), e)
}

/// Key `prop` in clip `clip` as a ramp from `a` at t=0 to `b` at t=2.
fn ramp(doc: &mut TimelineDoc, clip: usize, e: u64, prop: PropKind, a: f32, b: f32) {
    let was = doc.active_index();
    doc.set_active(clip);
    doc.insert_key(e, prop, s(0.0), AnimValue::Float(a), Interp::Linear);
    doc.insert_key(e, prop, s(2.0), AnimValue::Float(b), Interp::Linear);
    doc.set_active(was);
}

fn xy_of(world: &World, e: u64) -> (f32, f32) {
    let t = world.get::<Transform>(Entity::from_bits(e)).unwrap();
    (t.translation.x, t.translation.y)
}

/// The user's exact scene: lane 1 (Override) plays two instances of the
/// container, lane 2 (Additive) plays a third — and the container's interior
/// starts at 0.25, not 0, because its strip was dropped at the playhead.
///
/// At t=1.0: lane 1 reads the interior at 1.0 (clip 0.75 -> 13.75). Lane 2
/// reads it at 0.5 (clip 0.25 -> 11.25) and its reference is the interior's
/// FIRST VOICE (0.25 -> clip 0.0 -> 10.0), so the delta is 1.25 and the pose
/// is 15.0. An unclamped reference finds silence, falls back to rest (0), and
/// reports a delta of 11.25 (x = 25.0); the original bug fell back to the live
/// value and reported zero (x = 13.75). Both die here.
#[test]
fn the_additive_reference_of_a_container_clamps_to_its_interiors_first_voice() {
    let (mut world, mut doc, e) = scene();
    ramp(&mut doc, 0, e, PropKind::TranslationX, 10.0, 20.0);
    assert_eq!(doc.add_container("C".to_string()), 0);

    let mut inner = ClipLane::new("inner");
    inner.insert(ClipStrip::new(StripSource::Clip(0), 0.25, 2.0, 1.75));
    doc.container_stack_mut(0).unwrap().push(inner);

    let mut base = ClipLane::new("Lane 1");
    base.insert(ClipStrip::new(StripSource::Container(0), 0.0, 2.0, 2.0));
    base.insert(ClipStrip::new(StripSource::Container(0), 3.0, 5.0, 2.0));
    doc.stack_mut().push(base);

    let mut add = ClipLane::new("Lane 2");
    add.mode = LaneMode::Additive;
    add.insert(ClipStrip::new(StripSource::Container(0), 0.5, 2.5, 2.0));
    doc.stack_mut().push(add);

    apply_from_doc(&mut world, &mut doc, 1.0);
    let (x, _) = xy_of(&world, e);
    assert!(
        (x - 15.0).abs() < 1e-4,
        "x = {x}: the additive container strip must add its motion-since-first-voice \
         (13.75 from lane 1 + a delta of 1.25) — 13.75 means the strip is inert, \
         25.0 means the reference ignored the interior's first voice"
    );
}

/// A channel the clamped reference frame STILL cannot answer — a lane deeper in
/// the interior that starts later than the interior's first voice — measures
/// its delta against REST, which is what soloing the container shows there
/// (nothing written, the captured base standing).
///
/// Interior: lane A keys x from 0.25 on (the first voice), lane B keys y from
/// 1.0 on. At t=1.5 the additive strip reads y = 6.0; the reference frame sits
/// at 0.25, where lane B is silent, so y's base is rest (0) and the pose is
/// 6.0. Falling back to the live value — the bug — leaves y at exactly 0.
#[test]
fn a_channel_the_reference_frame_cannot_answer_measures_against_rest() {
    let (mut world, mut doc, e) = scene();
    doc.add_clip("B".to_string()); // clip 1
    ramp(&mut doc, 0, e, PropKind::TranslationX, 10.0, 20.0);
    ramp(&mut doc, 1, e, PropKind::TranslationY, 5.0, 9.0);
    assert_eq!(doc.add_container("C".to_string()), 0);

    let mut lane_a = ClipLane::new("A");
    lane_a.insert(ClipStrip::new(StripSource::Clip(0), 0.25, 2.0, 1.75));
    let mut lane_b = ClipLane::new("B");
    lane_b.insert(ClipStrip::new(StripSource::Clip(1), 1.0, 2.0, 1.0));
    doc.container_stack_mut(0).unwrap().push(lane_a);
    doc.container_stack_mut(0).unwrap().push(lane_b);

    let mut add = ClipLane::new("Add");
    add.mode = LaneMode::Additive;
    add.insert(ClipStrip::new(StripSource::Container(0), 0.0, 2.0, 2.0));
    doc.stack_mut().push(add);

    apply_from_doc(&mut world, &mut doc, 1.5);
    let (_, y) = xy_of(&world, e);
    assert!(
        (y - 6.0).abs() < 1e-4,
        "y = {y}: a late-starting interior lane must contribute v - rest (6.0); \
         0.0 means the reference fell back to the live value and went inert"
    );
}

/// Compatibility pin: an interior that DOES speak at `src_in` is untouched by
/// the clamp (`max(src_in, 0.0) == src_in`) — a container additive strip over a
/// full-length interior behaves exactly like the same clip on the same lane.
#[test]
fn a_container_and_its_clip_agree_when_the_interior_starts_at_zero() {
    let run = |top: StripSource| {
        let (mut world, mut doc, e) = scene();
        ramp(&mut doc, 0, e, PropKind::TranslationX, 10.0, 20.0);
        assert_eq!(doc.add_container("C".to_string()), 0);
        let mut inner = ClipLane::new("inner");
        inner.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 2.0));
        doc.container_stack_mut(0).unwrap().push(inner);

        let mut base = ClipLane::new("Lane 1");
        base.insert(ClipStrip::new(StripSource::Container(0), 0.0, 2.0, 2.0));
        doc.stack_mut().push(base);
        let mut add = ClipLane::new("Lane 2");
        add.mode = LaneMode::Additive;
        add.insert(ClipStrip::new(top, 0.5, 2.5, 2.0));
        doc.stack_mut().push(add);

        apply_from_doc(&mut world, &mut doc, 1.0);
        xy_of(&world, e).0
    };
    let clip = run(StripSource::Clip(0));
    let container = run(StripSource::Container(0));
    assert!(
        (clip - container).abs() < 1e-6,
        "clip-top {clip} != container-top {container}: the two sources must blend identically"
    );
    assert!(
        (clip - 17.5).abs() < 1e-4,
        "clip-top {clip}: expected 17.5 = 15.0 (lane 1 at clip time 1.0) + a delta of 2.5 \
         (12.5 at clip time 0.5, against 10.0 at src_in)"
    );
}
