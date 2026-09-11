//! Per-track EXTRAPOLATION at the DOCUMENT level (crown-jewels plan §6) — the two
//! gates that need the real apply: a track's `post = Loop` composes with a strip's
//! Loop (they are different layers and both take effect), and it changes the
//! sampled pose only when opted in.
//!
//! Sampled through `apply_from_doc` into a real world (the fade fingerprint's
//! discipline), never on a field being present.

use ph2d_anim::{AnimValue, Extrap, Interp, RationalTime};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{PropKind, StackHost, StripSource, TimelineDoc, apply_from_doc};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

/// A scene whose clip 0 keys `TranslationX` as a ramp `0 → 10` over `[0, 1]s`, with
/// the track's `post` set to `mode`. A scene lane plays clip 0 through a strip that
/// **loops** a 2 s slice over a 6 s timeline. Returns `(world, doc, entity)`.
///
/// Two loop layers, on purpose: the STRIP loop wraps the timeline time into the
/// clip's `[0, 2)` slice; the TRACK loop extrapolates the `[0, 1]` keys across the
/// second half of that slice. They compose.
fn scene(mode: Extrap) -> (World, TimelineDoc, u64) {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();
    let mut doc = TimelineDoc::new();

    doc.set_active(0);
    doc.insert_key(
        e,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    doc.insert_key(
        e,
        PropKind::TranslationX,
        s(1.0),
        AnimValue::Float(10.0),
        Interp::Linear,
    );

    // Set the track's post-range extrapolation.
    let target = doc
        .bindings()
        .iter()
        .find(|b| b.entity == e && b.prop == PropKind::TranslationX)
        .expect("translation binding")
        .target;
    doc.active_clip_mut()
        .track_mut(target)
        .expect("translation track")
        .set_post(mode);

    // Scene lane: one strip playing clip 0, a 2 s slice looped over 6 s.
    let la = doc.add_lane("A".into()).expect("scene lane");
    let id = doc
        .add_strip_to(StackHost::Document, la, StripSource::Clip(0), 0.0, 6.0)
        .expect("strip");
    {
        let strip = doc.strip_mut(la, id).expect("strip mut");
        strip.src_in = 0.0;
        strip.speed = 1.0; // real time: add_strip_to stretches to fill the span; we want the loop
        strip.src_out = 2.0; // a 2 s slice (the track is keyed only in its first second)
        strip.loop_mode = ph2d_timeline::StripLoop::Loop;
    }
    (world, doc, e)
}

fn sample(mode: Extrap, t: f64) -> f32 {
    let (mut world, mut doc, e) = scene(mode);
    apply_from_doc(&mut world, &mut doc, t);
    world
        .get::<Transform>(Entity::from_bits(e))
        .expect("transform")
        .translation
        .x
}

#[test]
fn a_strip_loop_and_a_track_loop_out_compose() {
    // t = 3.3: the STRIP loop wraps 3.3 into source 1.3 (3.3 mod 2); the TRACK is
    // beyond its last key (t = 1), so post = Loop folds 1.3 to 0.3 -> value 3.0.
    // Both layers fired: strip wrap AND track loop.
    let looped = sample(Extrap::Loop, 3.3);
    assert!(
        (looped - 3.0).abs() < 1e-4,
        "strip loop (3.3 -> 1.3) then track loop (1.3 -> 0.3) = 3.0; got {looped}"
    );
}

#[test]
fn the_track_loop_out_only_changes_the_value_when_opted_in() {
    // Same instant, Hold (the default): the track holds its last value (10) across
    // the strip's second-half source time. Loop reads 3.0. The layer is opt-in.
    let held = sample(Extrap::Hold, 3.3);
    let looped = sample(Extrap::Loop, 3.3);
    assert!(
        (held - 10.0).abs() < 1e-4,
        "with Hold the track flat-clamps at 10.0; got {held}"
    );
    assert!(
        (held - looped).abs() > 1.0,
        "Loop must differ from Hold (opt-in changes the pose): held {held}, looped {looped}"
    );
}

#[test]
fn the_strip_loop_still_wraps_where_the_track_is_in_range() {
    // t = 2.7 -> strip source 0.7 (2.7 mod 2), which is INSIDE the track's keyed
    // range -> value 7.0, regardless of post. The strip loop is untouched by the
    // track extrapolation.
    for mode in [Extrap::Hold, Extrap::Loop] {
        let v = sample(mode, 2.7);
        assert!(
            (v - 7.0).abs() < 1e-4,
            "strip wraps 2.7 -> 0.7, track in-range -> 7.0 ({mode:?}); got {v}"
        );
    }
}
