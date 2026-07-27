//! **ADR-0146 W1: a per-clip expression is a first-class LANE SOURCE that fades.**
//!
//! The pre-W1 engine applied a per-clip expression in a SEPARATE post-composition pass
//! that OVERWROTE the composed value — so it could not crossfade, could not sum on an
//! additive lane, could not fade out with a `lead_out`, and went QUIET when its clip
//! played twice (`sole_strip_of` refused). W1 makes the expression the value a clip
//! contributes at the blend's single sample site, so fade / overlap / additive / nesting
//! are inherited from the same machinery a keyed track flows through.
//!
//! The oracle is the WORLD after `apply_from_doc` — the composed pose an artist sees, not
//! an intermediate. Each gate names the mutation it dies to; the fade fingerprint
//! (`fade_fingerprint.rs`, gate #1) proves the formula-free path is byte-untouched.

use ph2d_anim::{AnimTarget, AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{
    ClipLane, ClipStrip, LaneMode, PropKind, StackHost, StripSource, TimelineDoc, apply_from_doc,
};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

/// One sprite at `x0`, and an empty document.
fn scene(x0: f32) -> (World, TimelineDoc, u64) {
    let mut world = World::new();
    let e = world
        .spawn(Transform {
            translation: Vec2::new(x0, 0.0),
            ..Default::default()
        })
        .id()
        .to_bits();
    (world, TimelineDoc::new(), e)
}

fn x_of(world: &World, e: u64) -> f32 {
    world
        .get::<Transform>(Entity::from_bits(e))
        .unwrap()
        .translation
        .x
}

/// Key `prop` of `e` in clip `clip` as a CONSTANT pose.
fn flat(doc: &mut TimelineDoc, clip: usize, e: u64, prop: PropKind, v: f32) {
    let was = doc.active_index();
    doc.set_active(clip);
    doc.insert_key(e, prop, s(0.0), AnimValue::Float(v), Interp::Hold);
    doc.set_active(was);
}

/// Key `prop` of `e` in clip `clip` as a ramp `a` (t=0) -> `b` (t=2).
fn ramp(doc: &mut TimelineDoc, clip: usize, e: u64, prop: PropKind, a: f32, b: f32) {
    let was = doc.active_index();
    doc.set_active(clip);
    doc.insert_key(e, prop, s(0.0), AnimValue::Float(a), Interp::Linear);
    doc.insert_key(e, prop, s(2.0), AnimValue::Float(b), Interp::Linear);
    doc.set_active(was);
}

/// Stamp a per-clip expression `src` on clip `clip`'s channel `(e, prop)`, binding the
/// target if needed. Returns the target (the same one every clip shares for that channel).
fn set_expr(doc: &mut TimelineDoc, clip: usize, e: u64, prop: PropKind, src: &str) -> AnimTarget {
    let tgt = doc.bind(e, prop);
    doc.set_clip_expr(clip, tgt, Some(src.to_string()));
    tgt
}

/// **Gate #4 — an expression FADES with its strip.** Clip 0 drives X by the constant
/// expression `100` (a PURE expression: no keys, yet it covers the channel); clip 1 keys X
/// flat at 0. Two strips crossfade over `[1,2)`. At full weight the expression is 100; mid
/// overlap it crossfades to 50 (half weight each) — a pre-W1 OVERWRITE would pin it at 100.
///
/// Mutation: disable the `Expr` arm in `clip_anim_source` (the expr stops being a source)
/// -> clip 0 has no track, contributes nothing -> x collapses to the keyed 0 / rest.
#[test]
fn an_expression_fades_with_its_strip() {
    let (mut world, mut doc, e) = scene(0.0);
    doc.add_clip("Zero".into()); // clip 1
    set_expr(&mut doc, 0, e, PropKind::TranslationX, "100");
    flat(&mut doc, 1, e, PropKind::TranslationX, 0.0);

    let mut lane = ClipLane::new("Base");
    lane.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 2.0));
    lane.insert(ClipStrip::new(StripSource::Clip(1), 1.0, 3.0, 2.0));
    doc.stack_mut().push(lane);

    apply_from_doc(&mut world, &mut doc, 0.5);
    let full = x_of(&world, e);
    assert!(
        (full - 100.0).abs() < 1e-3,
        "expr at full weight = 100, got {full}"
    );

    apply_from_doc(&mut world, &mut doc, 1.5);
    let mid = x_of(&world, e);
    assert!(
        (mid - 50.0).abs() < 1e-3,
        "the expression must CROSSFADE to 50 mid-overlap (an overwrite pins it at 100); got {mid}"
    );

    apply_from_doc(&mut world, &mut doc, 2.5);
    let past = x_of(&world, e);
    assert!(
        past.abs() < 1e-3,
        "past the overlap only the keyed 0 stands; got {past}"
    );
}

/// **Gate #9 — an expression SELF-CROSSFADES.** The SAME clip (with the time-dependent
/// expression `time*10`) plays in two overlapping strips. At the overlap midpoint the two
/// instances read different clip times and CROSSFADE — the pre-W1 engine refused this
/// (`sole_strip_of` -> `PlaysTwice`) and went quiet.
///
/// Mutation: disable the `Expr` arm -> both instances contribute nothing -> x = rest (0).
#[test]
fn an_expression_self_crossfades() {
    let (mut world, mut doc, e) = scene(0.0);
    set_expr(&mut doc, 0, e, PropKind::TranslationX, "time*10");

    let mut lane = ClipLane::new("Base");
    lane.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 2.0)); // clip time == timeline
    lane.insert(ClipStrip::new(StripSource::Clip(0), 1.0, 3.0, 2.0)); // clip time == timeline - 1
    doc.stack_mut().push(lane);

    // At t=1.5: strip A reads clip time 1.5 (E=15), strip B reads 0.5 (E=5). The blend is
    // strictly BETWEEN — both instances drive, and it is their crossfade, not either alone.
    apply_from_doc(&mut world, &mut doc, 1.5);
    let x = x_of(&world, e);
    assert!(
        x > 5.5 && x < 14.5,
        "the same clip playing twice must CROSSFADE its two expression instances (5 < x < 15); \
         got {x} (0 = the pre-W1 PlaysTwice quiet)"
    );
    assert!(
        (x - 10.0).abs() < 1e-3,
        "and at the exact midpoint it is their mean (10); got {x}"
    );
}

/// **Gate #7 — an ADDITIVE expression contributes a DELTA.** A base override lane keys X
/// flat at 5; an additive lane on top carries the constant expression `100`. An additive
/// contribution is `E(t) - E(src_in)`, so a CONSTANT expression contributes exactly 0 —
/// the whole point of an additive lane, and the test that catches "I summed the absolute
/// value". A moving expression then contributes its travel.
///
/// Mutation: force the additive-reference base to 0 -> the constant expr contributes
/// `100 - 0 = 100` and the sprite jumps to 105.
#[test]
fn an_additive_expression_contributes_a_delta() {
    let (mut world, mut doc, e) = scene(0.0);
    doc.add_clip("Add".into()); // clip 1

    // Base override lane: clip 0 keys X flat at 5.
    flat(&mut doc, 0, e, PropKind::TranslationX, 5.0);
    let mut base = ClipLane::new("Base");
    base.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 4.0, 2.0));
    doc.stack_mut().push(base);

    // Additive lane: clip 1 carries a CONSTANT expression.
    set_expr(&mut doc, 1, e, PropKind::TranslationX, "100");
    let mut add = ClipLane::new("Add");
    add.mode = LaneMode::Additive;
    add.insert(ClipStrip::new(StripSource::Clip(1), 0.0, 4.0, 2.0));
    doc.stack_mut().push(add);

    apply_from_doc(&mut world, &mut doc, 1.0);
    let constant = x_of(&world, e);
    assert!(
        (constant - 5.0).abs() < 1e-3,
        "a CONSTANT additive expression contributes 0 -> the base (5) stands; got {constant} \
         (105 = the base-is-0 bug summing the absolute value)"
    );

    // A MOVING additive expression contributes its travel from the strip's first frame.
    let tgt = doc.bind(e, PropKind::TranslationX);
    doc.set_clip_expr(1, tgt, Some("time*10".into()));
    apply_from_doc(&mut world, &mut doc, 1.0);
    let moving = x_of(&world, e);
    assert!(
        moving > 5.0 + 1.0,
        "a MOVING additive expression adds its delta over the base; got {moving} (should exceed 5)"
    );
}

/// **Gate #10 — a `lead_out` FADES an expression OUT.** Clip 0 keys X as a ramp 0->10 and
/// drives it by `value + 100` (E ranges 100->110 — an expression riding its keys, the
/// common case). Its strip has a 1 s `lead_out`: the clip plays fully then fades out in the
/// gap AFTER its authored end. The pre-W1 overwrite could only switch off at the boundary.
///
/// Mutation: disable the `Expr` arm -> the ramp (0->10) drives directly, so nothing near
/// 100 ever appears; the "inside > 100" assertion fails.
///
/// (A PURE expression — no keys at all — has no authored length for the `lead_out` region;
/// gate #4 covers the keyless crossfade, this covers the keyed lead_out fade.)
#[test]
fn a_lead_out_fades_an_expression_out() {
    let (mut world, mut doc, e) = scene(0.0);
    ramp(&mut doc, 0, e, PropKind::TranslationX, 0.0, 10.0);
    set_expr(&mut doc, 0, e, PropKind::TranslationX, "value + 100");

    // A following strip (clip 1, keyed flat 0) gives the lead_out a target to cross the gap
    // TO — the outward fade travels from clip 0's last frame toward it (a lone lead_out
    // strip with nothing after just holds).
    doc.add_clip("Next".into()); // clip 1
    flat(&mut doc, 1, e, PropKind::TranslationX, 0.0);
    let lane = doc.add_lane("A".into()).expect("lane");
    let strip = doc
        .add_strip_to(StackHost::Document, lane, StripSource::Clip(0), 0.0, 3.0)
        .expect("strip");
    doc.strip_mut(lane, strip).expect("strip mut").lead_out = 1.0;
    doc.add_strip_to(StackHost::Document, lane, StripSource::Clip(1), 4.0, 6.0)
        .expect("next strip");

    // Inside the strip: full weight, the expression rides the ramp (> 100).
    apply_from_doc(&mut world, &mut doc, 1.0);
    let inside = x_of(&world, e);
    assert!(
        inside > 100.0,
        "inside the strip the expression rides its keys (> 100); got {inside}"
    );

    // In the lead_out gap [3,4): the expression's clamped value (~110) FADES OUT toward
    // rest — strictly below the un-faded value and above rest.
    apply_from_doc(&mut world, &mut doc, 3.5);
    let fading = x_of(&world, e);
    assert!(
        fading > 5.0 && fading < 105.0,
        "in the lead_out gap the expression must FADE OUT (5 < x < 105, below the clamped ~110); \
         got {fading}"
    );
}

/// **Gate #16 (C1) — an expression drives a NON-STACKED document.** The common case: a
/// keyed animation with NO strips at all. Its per-clip expression never reaches the blend
/// (`eval_frame` iterates an empty stack), so without the SECOND sample site
/// (`solo_source_value`) it would go undriven in silence. No test before this covered it —
/// they all `add_lane`/`add_strip_to`.
///
/// Mutation: disable the `Expr` arm in `clip_anim_source` -> the non-stacked expression
/// channel stops driving (`time*10` stays 0; `value + 100` collapses to the bare ramp).
#[test]
fn an_expression_drives_a_non_stacked_document() {
    // A pure expression, no stack and no keys.
    let (mut world, mut doc, e) = scene(0.0);
    set_expr(&mut doc, 0, e, PropKind::TranslationX, "time*10");
    assert!(doc.stack().is_empty(), "this gate is the NON-STACKED path");
    apply_from_doc(&mut world, &mut doc, 1.0);
    let pure = x_of(&world, e);
    assert!(
        (pure - 10.0).abs() < 1e-3,
        "a pure expression drives a strip-less document; got {pure}"
    );

    // An expression riding keys, still no stack.
    let (mut world, mut doc, e) = scene(0.0);
    ramp(&mut doc, 0, e, PropKind::TranslationX, 0.0, 10.0);
    set_expr(&mut doc, 0, e, PropKind::TranslationX, "value + 100");
    assert!(doc.stack().is_empty());
    apply_from_doc(&mut world, &mut doc, 1.0);
    let riding = x_of(&world, e);
    assert!(
        (riding - 105.0).abs() < 1e-3,
        "value + 100 rides the ramp (5 at t=1) to 105 with no stack; got {riding}"
    );
}

/// **Gate #5 (Hole A) — a keyed, faded channel CO-RESIDENT with an expression is byte
/// stable.** The real-world common case: X is expression-driven (so `scheduled == true`
/// and the whole two-pass path runs), while Y is a keyed crossfade. Gate #1 only exercises
/// the formula-free `!scheduled` path, so a mutation that perturbs the KEYED composition
/// only under `scheduled` would slip past it. This pins Y's faded curve under `scheduled`.
///
/// Mutation: any change that moves the keyed crossfade of Y when an expression is present.
#[test]
fn a_keyed_fade_co_resident_with_an_expression_is_byte_stable() {
    let (h, samples) = coresident_fingerprint();
    let (lo, hi) = samples
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), &(_, y)| {
            (lo.min(y), hi.max(y))
        });
    assert!(
        hi - lo > 5.0,
        "the co-resident scene went inert (Y range {lo}..{hi}); it must exercise the fade"
    );
    assert_eq!(
        h, CORESIDENT_FINGERPRINT,
        "the keyed crossfade of Y moved WHILE an expression drove X (scheduled==true). \
         If intended, re-pin in the same commit.\nsamples (t, y) = {samples:?}"
    );
}

/// Y keyed 0->10 / 20->30 crossfaded over [1,2), sampled while X is expression-driven.
fn coresident_fingerprint() -> (u64, Vec<(f64, f32)>) {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();
    let mut doc = TimelineDoc::new();
    doc.add_clip("B".into()); // clip 1

    // X: expression-driven (forces scheduled == true).
    set_expr(&mut doc, 0, e, PropKind::TranslationX, "time*5");
    // Y: a keyed crossfade across two clips.
    ramp(&mut doc, 0, e, PropKind::TranslationY, 0.0, 10.0);
    ramp(&mut doc, 1, e, PropKind::TranslationY, 20.0, 30.0);
    let mut lane = ClipLane::new("Base");
    lane.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 2.0));
    lane.insert(ClipStrip::new(StripSource::Clip(1), 1.0, 3.0, 2.0));
    doc.stack_mut().push(lane);

    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut samples = Vec::new();
    for i in 0..=60 {
        let t = f64::from(i) * 0.05;
        apply_from_doc(&mut world, &mut doc, t);
        let y = world
            .get::<Transform>(Entity::from_bits(e))
            .unwrap()
            .translation
            .y;
        samples.push((t, y));
        for byte in y.to_bits().to_le_bytes() {
            h ^= u64::from(byte);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    (h, samples)
}

/// Pinned on `line/anim` at ADR-0146 W1. Captures Y's keyed crossfade under `scheduled`.
const CORESIDENT_FINGERPRINT: u64 = 0x7a19_b02a_890c_015b;
