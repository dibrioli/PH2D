//! ADR-0115 A4-A7: the stack evaluated through the REAL apply, into a REAL world.
//!
//! Not through a hand-rolled harness. A harness reproduces the mechanism, not the
//! context — every one of the Time Remap bugs this module has shipped was green in
//! a harness and red in the product. So: spawn entities, build a document, call
//! `apply_from_doc`, read the `Transform` back.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{
    ClipLane, ClipStrip, LaneMode, PropKind, StripSource, TimelineDoc, apply_from_doc,
};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

/// One sprite at `x0`, and a document. Nothing is animated yet.
fn scene(x0: f32) -> (World, TimelineDoc, u64) {
    let mut world = World::new();
    let e = world
        .spawn(Transform {
            translation: ph2d_core::Vec2::new(x0, 0.0),
            ..Default::default()
        })
        .id()
        .to_bits();
    (world, TimelineDoc::new(), e)
}

/// Key `prop` of `entity` in clip `clip` as a ramp from `a` at t=0 to `b` at t=2.
fn ramp(doc: &mut TimelineDoc, clip: usize, e: u64, prop: PropKind, a: f32, b: f32) {
    let was = doc.active_index();
    doc.set_active(clip);
    doc.insert_key(e, prop, s(0.0), AnimValue::Float(a), Interp::Linear);
    doc.insert_key(e, prop, s(2.0), AnimValue::Float(b), Interp::Linear);
    doc.set_active(was);
}

/// Key `prop` as a CONSTANT pose (same value at both ends).
fn flat(doc: &mut TimelineDoc, clip: usize, e: u64, prop: PropKind, v: f32) {
    ramp(doc, clip, e, prop, v, v);
}

fn x_of(world: &World, e: u64) -> f32 {
    world
        .get::<Transform>(Entity::from_bits(e))
        .unwrap()
        .translation
        .x
}

fn scale_x_of(world: &World, e: u64) -> f32 {
    world
        .get::<Transform>(Entity::from_bits(e))
        .unwrap()
        .scale
        .x
}

fn lane_with(clip: u16, t0: f64, t1: f64, mode: LaneMode) -> ClipLane {
    let mut lane = ClipLane::new("L");
    lane.mode = mode;
    lane.insert(ClipStrip::new(StripSource::Clip(clip), t0, t1, 2.0));
    lane
}

// ── §3.1 — an empty stack changes nothing ───────────────────────────────────

/// The compatibility gate. Every document that exists today has an empty stack,
/// and must behave EXACTLY as it did — same code path, same values. A feature
/// that is "purely additive" has to prove it, not claim it.
#[test]
fn an_empty_stack_is_the_single_clip_path_value_for_value() {
    let (mut world, mut doc, e) = scene(0.0);
    ramp(&mut doc, 0, e, PropKind::TranslationX, 0.0, 10.0);

    for step in 0..=20 {
        let t = f64::from(step) * 0.1;
        apply_from_doc(&mut world, &mut doc, t);
        #[expect(clippy::cast_possible_truncation, reason = "test fixture")]
        let expected = (t * 5.0) as f32; // the ramp: 0 -> 10 over 2 s
        assert!(
            (x_of(&world, e) - expected).abs() < 1e-5,
            "t={t}: {} != {expected}",
            x_of(&world, e)
        );
    }
    assert!(doc.stack().is_empty(), "and the stack really was empty");
}

// ── §3.2 — the crossfade, end to end ────────────────────────────────────────

/// Two clips, two strips, one lane, dragged into overlap. The sprite must cross
/// from one animation to the other with **no jump and no sag** — and in
/// particular it must never visit a value outside the two clips' range, which is
/// what "sagging toward a default" looks like from the outside.
#[test]
fn overlapping_strips_crossfade_the_scene_with_no_jump_and_no_sag() {
    let (mut world, mut doc, e) = scene(0.0);
    doc.add_clip("Run".to_string()); // clip 1
    ramp(&mut doc, 0, e, PropKind::TranslationX, 100.0, 100.0); // "Walk": holds 100
    ramp(&mut doc, 1, e, PropKind::TranslationX, 200.0, 200.0); // "Run":  holds 200

    let mut lane = ClipLane::new("Base");
    lane.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 2.0)); // [0, 2)
    lane.insert(ClipStrip::new(StripSource::Clip(1), 1.0, 3.0, 2.0)); // [1, 3)  -> 1 s overlap
    doc.stack_mut().push(lane);

    let mut prev = 100.0_f32;
    for step in 0..=60 {
        let t = f64::from(step) * 0.05; // 0 .. 3
        apply_from_doc(&mut world, &mut doc, t);
        let x = x_of(&world, e);

        assert!(
            (100.0..=200.0).contains(&x),
            "t={t}: x={x} left the two clips' range — the blend sagged toward a default"
        );
        assert!(
            (x - prev).abs() < 20.0,
            "t={t}: x jumped {prev} -> {x}; a crossfade has no cliff"
        );
        prev = x;
    }
    assert!(
        (prev - 200.0).abs() < 1e-4,
        "it lands fully on the second clip"
    );
}

/// Mid-overlap the two strips are at exactly half weight each, so the value is
/// the exact mean. (This is the complementary-weight property showing up in the
/// scene rather than in a unit test.)
#[test]
fn mid_overlap_is_the_exact_mean_of_the_two_clips() {
    let (mut world, mut doc, e) = scene(0.0);
    doc.add_clip("Run".to_string());
    flat(&mut doc, 0, e, PropKind::TranslationX, 100.0);
    flat(&mut doc, 1, e, PropKind::TranslationX, 200.0);

    let mut lane = ClipLane::new("Base");
    lane.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 2.0));
    lane.insert(ClipStrip::new(StripSource::Clip(1), 1.0, 3.0, 2.0));
    doc.stack_mut().push(lane);

    apply_from_doc(&mut world, &mut doc, 1.5); // the middle of the overlap
    assert!((x_of(&world, e) - 150.0).abs() < 1e-4);
}

// ── §3.2 (the sparse half) — a lane only touches what it keys ───────────────

/// The masking rule, and the reason this design needs no Avatar Mask: clip B
/// keys Y but not X, so X keeps coming from clip A underneath. A stack that
/// blended X toward a default here would drag the sprite to the origin.
#[test]
fn a_lane_that_does_not_key_a_channel_lets_the_one_below_through() {
    let (mut world, mut doc, e) = scene(0.0);
    doc.add_clip("Bob".to_string());
    flat(&mut doc, 0, e, PropKind::TranslationX, 300.0); // base keys X
    flat(&mut doc, 1, e, PropKind::ScaleX, 2.0); // the lane above keys ONLY ScaleX

    doc.stack_mut()
        .push(lane_with(0, 0.0, 2.0, LaneMode::Override));
    doc.stack_mut()
        .push(lane_with(1, 0.0, 2.0, LaneMode::Override));

    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        (x_of(&world, e) - 300.0).abs() < 1e-4,
        "X came from the lane below, untouched"
    );
    assert!(
        (scale_x_of(&world, e) - 2.0).abs() < 1e-4,
        "and ScaleX from above"
    );
}

// ── §3.4 / §3.5 — additive is a DELTA, and scale multiplies ─────────────────

/// The test that catches "I summed the absolute value". An additive lane whose
/// clip holds a **constant pose** carries no change, and must therefore
/// contribute **nothing at all**.
#[test]
fn an_additive_lane_holding_a_constant_pose_contributes_nothing() {
    let (mut world, mut doc, e) = scene(0.0);
    doc.add_clip("Add".to_string());
    flat(&mut doc, 0, e, PropKind::TranslationX, 300.0);
    flat(&mut doc, 1, e, PropKind::TranslationX, 999.0); // a constant, far away

    doc.stack_mut()
        .push(lane_with(0, 0.0, 2.0, LaneMode::Override));
    doc.stack_mut()
        .push(lane_with(1, 0.0, 2.0, LaneMode::Additive));

    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        (x_of(&world, e) - 300.0).abs() < 1e-4,
        "999 is where the additive clip SITS, not how far it MOVES: it moves 0"
    );
}

#[test]
fn an_additive_lane_adds_how_far_its_clip_travelled() {
    let (mut world, mut doc, e) = scene(0.0);
    doc.add_clip("Add".to_string());
    flat(&mut doc, 0, e, PropKind::TranslationX, 300.0);
    ramp(&mut doc, 1, e, PropKind::TranslationX, 50.0, 90.0); // travels +40 over 2 s

    doc.stack_mut()
        .push(lane_with(0, 0.0, 2.0, LaneMode::Override));
    doc.stack_mut()
        .push(lane_with(1, 0.0, 2.0, LaneMode::Additive));

    apply_from_doc(&mut world, &mut doc, 1.0); // half way: it has travelled +20
    assert!(
        (x_of(&world, e) - 320.0).abs() < 1e-4,
        "300 + 20, not 300 + 70: the delta is measured from the clip's own start"
    );
}

/// **Scale does not add — it multiplies.** Summing two scale clips of 1.0 gives
/// 2.0 (double size) where the honest answer is "no change". This is the bug that
/// forced Blender to invent COMBINE (T47035), and it is one line of algebra away
/// at all times.
#[test]
fn two_additive_scale_clips_of_one_leave_the_scale_at_one() {
    let (mut world, mut doc, e) = scene(0.0);
    doc.add_clip("A".to_string());
    doc.add_clip("B".to_string());
    flat(&mut doc, 0, e, PropKind::ScaleX, 1.0);
    flat(&mut doc, 1, e, PropKind::ScaleX, 1.0);
    flat(&mut doc, 2, e, PropKind::ScaleX, 1.0);

    doc.stack_mut()
        .push(lane_with(0, 0.0, 2.0, LaneMode::Override));
    doc.stack_mut()
        .push(lane_with(1, 0.0, 2.0, LaneMode::Additive));
    doc.stack_mut()
        .push(lane_with(2, 0.0, 2.0, LaneMode::Additive));

    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        (scale_x_of(&world, e) - 1.0).abs() < 1e-4,
        "got {} — the additive lanes SUMMED instead of scaling",
        scale_x_of(&world, e)
    );
}

#[test]
fn an_additive_scale_clip_that_doubles_doubles_the_scale() {
    let (mut world, mut doc, e) = scene(0.0);
    doc.add_clip("Pulse".to_string());
    flat(&mut doc, 0, e, PropKind::ScaleX, 3.0); // the base is 3x
    ramp(&mut doc, 1, e, PropKind::ScaleX, 1.0, 2.0); // the clip doubles over 2 s

    doc.stack_mut()
        .push(lane_with(0, 0.0, 2.0, LaneMode::Override));
    doc.stack_mut()
        .push(lane_with(1, 0.0, 2.0, LaneMode::Additive));

    apply_from_doc(&mut world, &mut doc, 2.0 - 1e-9); // the clip's end: ratio 2.0
    assert!(
        (scale_x_of(&world, e) - 6.0).abs() < 1e-3,
        "3 x 2 = 6, got {}",
        scale_x_of(&world, e)
    );
}

// ── §3.6 — mute, and stacking order ─────────────────────────────────────────

#[test]
fn a_muted_lane_contributes_nothing() {
    let (mut world, mut doc, e) = scene(0.0);
    doc.add_clip("Top".to_string());
    flat(&mut doc, 0, e, PropKind::TranslationX, 300.0);
    flat(&mut doc, 1, e, PropKind::TranslationX, 700.0);

    doc.stack_mut()
        .push(lane_with(0, 0.0, 2.0, LaneMode::Override));
    doc.stack_mut()
        .push(lane_with(1, 0.0, 2.0, LaneMode::Override));
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!((x_of(&world, e) - 700.0).abs() < 1e-4, "the top lane wins");

    doc.stack_mut()[1].muted = true;
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        (x_of(&world, e) - 300.0).abs() < 1e-4,
        "muted, the top lane is GONE — not merely at zero weight"
    );
}

/// The order matters, and the test says which way. (Bottom to top: the last lane
/// in the vector is the one on top, and it is the one that wins.)
#[test]
fn swapping_two_override_lanes_swaps_the_result() {
    let (mut world, mut doc, e) = scene(0.0);
    doc.add_clip("Other".to_string());
    flat(&mut doc, 0, e, PropKind::TranslationX, 300.0);
    flat(&mut doc, 1, e, PropKind::TranslationX, 700.0);

    doc.stack_mut()
        .push(lane_with(0, 0.0, 2.0, LaneMode::Override));
    doc.stack_mut()
        .push(lane_with(1, 0.0, 2.0, LaneMode::Override));
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!((x_of(&world, e) - 700.0).abs() < 1e-4);

    doc.stack_mut().swap(0, 1);
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        (x_of(&world, e) - 300.0).abs() < 1e-4,
        "reordering the stack reorders who wins"
    );
}

// ── R5 — the captured rest pose ─────────────────────────────────────────────

/// A lane easing in from nothing must fade from **where the object was**, not
/// from a type default. With `TranslationX` the type default is 0 — the parent's
/// origin — and a sprite that eases in would fly across the canvas to meet it.
/// This is Rive's Capture Base State and Unreal's Base Pose, and it is why
/// `TargetBinding.rest` exists.
#[test]
fn a_lane_fading_in_from_nothing_fades_from_the_captured_rest_pose() {
    let (mut world, mut doc, e) = scene(500.0); // the animator left it at x = 500
    flat(&mut doc, 0, e, PropKind::TranslationX, 100.0);

    let mut lane = ClipLane::new("Base");
    let mut strip = ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 2.0);
    strip.ease_in = 1.0; // a 1 s fade-in, with nothing underneath
    lane.insert(strip);
    doc.stack_mut().push(lane);

    apply_from_doc(&mut world, &mut doc, 0.0);
    assert!(
        (x_of(&world, e) - 500.0).abs() < 1e-3,
        "at zero weight it must still be where the animator put it, got {}",
        x_of(&world, e)
    );

    apply_from_doc(&mut world, &mut doc, 0.5); // half way through the fade
    let x = x_of(&world, e);
    assert!(
        x > 100.0 && x < 500.0,
        "mid-fade it is between the rest pose and the clip, got {x}"
    );

    apply_from_doc(&mut world, &mut doc, 1.5); // fade over
    assert!(
        (x_of(&world, e) - 100.0).abs() < 1e-4,
        "and then fully on the clip"
    );
}

// ── R2 — nothing keyed, nothing written ─────────────────────────────────────

/// A channel that NO lane keys is never written at all, and the scene keeps it.
/// (Sparsity is the mask.)
#[test]
fn a_channel_no_lane_keys_is_left_to_the_scene() {
    let (mut world, mut doc, e) = scene(0.0);
    flat(&mut doc, 0, e, PropKind::TranslationX, 100.0);
    // Bind ScaleX but never key it in any clip that a strip plays.
    let _ = doc.bind(e, PropKind::ScaleX);
    doc.stack_mut()
        .push(lane_with(0, 0.0, 2.0, LaneMode::Override));

    // The user scales the sprite by hand.
    world
        .get_mut::<Transform>(Entity::from_bits(e))
        .unwrap()
        .scale
        .x = 4.0;

    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        (scale_x_of(&world, e) - 4.0).abs() < 1e-4,
        "the timeline does not own a channel it never keyed"
    );
}

// ── The lane is not a staircase, and the blend must not assume it is ─────────
//
// Both of these were found by an adversarial audit (2026-07-12) and both were
// CATASTROPHIC: a lane's coverage collapsed and the sprite crawled back to its
// rest pose in the middle of a clip that never moves. The cause was one line —
// `blend_out(i)` asked `strips[i + 1]`, the neighbour in SORT order, which is the
// right strip to ask only when the strips form a staircase. Nothing makes them
// one: the body drag has no clamp against its neighbours.

/// **Containment.** A short strip dropped INSIDE a long one. The long strip must
/// hold its value to its own end — the short one makes a hump in its middle, and
/// the middle is not an edge. Before the fix, the long strip faded out for 6 s
/// against a neighbour that had already ended: 500 -> 104.
#[test]
fn a_strip_dropped_inside_another_never_drains_the_lane() {
    let (mut world, mut doc, e) = scene(100.0); // rest = 100
    flat(&mut doc, 0, e, PropKind::TranslationX, 500.0);
    let mut lane = ClipLane::new("L");
    lane.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 10.0, 2.0)); // A
    lane.insert(ClipStrip::new(StripSource::Clip(0), 2.0, 4.0, 2.0)); // B, wholly inside A
    doc.stack_mut().push(lane);

    for t in [3.99, 4.0, 5.0, 8.0, 9.5] {
        apply_from_doc(&mut world, &mut doc, t);
        let x = x_of(&world, e);
        assert!(
            (x - 500.0).abs() < 0.5,
            "the clip is flat at 500 and both strips play it; at t={t} the sprite is at {x} \
             (before the fix it sagged toward the rest pose, reaching 104)"
        );
    }
}

/// **Non-adjacent overlap.** A[0,10), B[1,2), C[8,20). `blend_in(C)` used to look
/// only at `strips[i-1]` = B, whose overlap is negative -> it entered at full
/// weight with no fade, while A had been sagging since t=2. The tell was an 86%
/// sag followed by a POP back to 500 at t=8.
#[test]
fn an_overlap_that_is_not_with_the_sort_order_neighbour_still_crossfades() {
    let (mut world, mut doc, e) = scene(100.0);
    flat(&mut doc, 0, e, PropKind::TranslationX, 500.0);
    let mut lane = ClipLane::new("L");
    lane.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 10.0, 2.0)); // A
    lane.insert(ClipStrip::new(StripSource::Clip(0), 1.0, 2.0, 2.0)); // B, a blip inside A
    lane.insert(ClipStrip::new(StripSource::Clip(0), 8.0, 20.0, 2.0)); // C, overlapping A's tail
    doc.stack_mut().push(lane);

    for t in [3.0, 5.0, 7.0, 7.9, 8.0, 8.1, 12.0] {
        apply_from_doc(&mut world, &mut doc, t);
        let x = x_of(&world, e);
        assert!(
            (x - 500.0).abs() < 0.5,
            "flat clip, full coverage everywhere; at t={t} the sprite is at {x}"
        );
    }
}

/// And the ordinary crossfade — the one the whole model exists for — is untouched
/// by the fix: two strips of DIFFERENT clips, weights summing to exactly 1 through
/// the overlap, no sag toward the rest pose at any point inside it.
#[test]
fn the_plain_crossfade_still_sums_to_one_and_never_sags() {
    let (mut world, mut doc, e) = scene(100.0);
    doc.add_clip("Other".to_string());
    flat(&mut doc, 0, e, PropKind::TranslationX, 0.0);
    flat(&mut doc, 1, e, PropKind::TranslationX, 100.0);
    let mut lane = ClipLane::new("L");
    lane.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 2.0));
    lane.insert(ClipStrip::new(StripSource::Clip(1), 1.0, 3.0, 2.0)); // 1 s of overlap
    doc.stack_mut().push(lane);

    let mut prev = f32::NEG_INFINITY;
    for i in 0..=10 {
        let t = 1.0 + f64::from(i) * 0.1;
        apply_from_doc(&mut world, &mut doc, t);
        let x = x_of(&world, e);
        // Complementary weights: the value only ever travels BETWEEN the two clips'
        // values. Anything outside [0, 100] is the stack falling toward `rest`.
        assert!(
            (-0.01..=100.01).contains(&x),
            "at t={t} the blend left the two clips' values: {x}"
        );
        assert!(
            x >= prev - 0.01,
            "and it climbs monotonically: {prev} -> {x}"
        );
        prev = x;
    }
    assert!(prev > 99.0, "and lands on the incoming clip: {prev}");
}
