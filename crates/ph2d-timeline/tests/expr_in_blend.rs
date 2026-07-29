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
use ph2d_ecs::{Entity, Name, Transform, World};
use ph2d_timeline::{
    ClipLane, ClipStrip, LaneMode, PropKind, StackHost, StripSource, TimelineDoc,
    apply_active_clip, apply_from_doc, pose_at,
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

/// **Gate #8 (W3) — a prop-link reads the FADED source, in the same frame.** Sprite.x
/// crossfades 0 -> 100 (faded to 50 mid-overlap); Follower drives its own X by `Sprite.x`.
/// The frame composes in TOPOLOGICAL order — the source (Sprite, no prop-link) before its
/// reader (Follower) — so Follower reads Sprite's ALREADY-composed 50 with no one-frame lag.
///
/// Follower's binding is CREATED FIRST, so the NATURAL order would compose it before its
/// source; only the topological order gets it right. Mutation: make `topo_order` return the
/// natural order -> Follower composes before Sprite, reads an empty `LinkFrame` -> 0, RED.
#[test]
fn a_prop_link_reads_the_faded_source() {
    let mut world = World::new();
    let follower = world
        .spawn((Transform::default(), Name::new("Follower")))
        .id()
        .to_bits();
    let sprite = world
        .spawn((Transform::default(), Name::new("Sprite")))
        .id()
        .to_bits();
    let mut doc = TimelineDoc::new();
    doc.add_clip("SpriteB".into()); // clip 1
    doc.add_clip("Follow".into()); // clip 2

    // Follower's binding first (index 0) — natural order would compose it before Sprite.
    set_expr(&mut doc, 2, follower, PropKind::TranslationX, "Sprite.x");
    // Sprite.x crossfades 0 -> 100 on lane A: at the midpoint it is the FADED 50.
    flat(&mut doc, 0, sprite, PropKind::TranslationX, 0.0);
    flat(&mut doc, 1, sprite, PropKind::TranslationX, 100.0);
    let mut lane_a = ClipLane::new("A");
    lane_a.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 2.0));
    lane_a.insert(ClipStrip::new(StripSource::Clip(1), 1.0, 3.0, 2.0));
    doc.stack_mut().push(lane_a);
    // Follower plays clip 2 at full weight on lane B (so its OWN strip does not fade — this
    // gate isolates the source fade; the reader's own fade is exercised elsewhere).
    let mut lane_b = ClipLane::new("B");
    lane_b.insert(ClipStrip::new(StripSource::Clip(2), 0.0, 3.0, 2.0));
    doc.stack_mut().push(lane_b);

    apply_from_doc(&mut world, &mut doc, 1.5);
    let s = x_of(&world, sprite);
    let f = x_of(&world, follower);
    assert!(
        (s - 50.0).abs() < 1e-3,
        "Sprite crossfades to the faded 50; got {s}"
    );
    assert!(
        (f - 50.0).abs() < 1e-3,
        "Follower's `Sprite.x` reads Sprite's FADED value (50) IN THE SAME FRAME; got {f} \
         (0 = the reader composed before its source)"
    );
}

/// **Gate #14 (W3) — the scene evaluates in DEPENDENCY order.** An acyclic chain
/// `A = B.x`, `B = C.x`, `C = 50` composes C then B then A in ONE frame, so A reads B reads
/// C fresh with no lag; and a re-scrub at the same instant agrees. A genuine CONTRACTIVE
/// cycle `P = 0.5*Q + 10`, `Q = P` reads its back edge from last frame (the one-frame-delay)
/// and STABILIZES at its fixed point (20) instead of exploding.
///
/// Mutation: `topo_order` returns the natural order -> A composes before its source and reads
/// 0 on the first frame, RED.
#[test]
fn the_scene_evaluates_in_dependency_order() {
    // Acyclic chain, non-stacked. A's binding is created FIRST, so only the topological
    // order composes the source (C) before B before A.
    let mut world = World::new();
    let a = world
        .spawn((Transform::default(), Name::new("A")))
        .id()
        .to_bits();
    let b = world
        .spawn((Transform::default(), Name::new("B")))
        .id()
        .to_bits();
    let c = world
        .spawn((Transform::default(), Name::new("C")))
        .id()
        .to_bits();
    let mut doc = TimelineDoc::new();
    set_expr(&mut doc, 0, a, PropKind::TranslationX, "B.x");
    set_expr(&mut doc, 0, b, PropKind::TranslationX, "C.x");
    flat(&mut doc, 0, c, PropKind::TranslationX, 50.0);

    apply_from_doc(&mut world, &mut doc, 0.0);
    assert!((x_of(&world, c) - 50.0).abs() < 1e-3);
    assert!(
        (x_of(&world, b) - 50.0).abs() < 1e-3,
        "B reads C fresh; got {}",
        x_of(&world, b)
    );
    assert!(
        (x_of(&world, a) - 50.0).abs() < 1e-3,
        "A reads B reads C fresh in ONE frame, no lag; got {}",
        x_of(&world, a)
    );
    apply_from_doc(&mut world, &mut doc, 0.0);
    assert!(
        (x_of(&world, a) - 50.0).abs() < 1e-3,
        "a re-scrub at the same instant agrees (acyclic)"
    );

    // A CONTRACTIVE cycle stabilizes at its fixed point, never exploding.
    let mut world = World::new();
    let p = world
        .spawn((Transform::default(), Name::new("P")))
        .id()
        .to_bits();
    let q = world
        .spawn((Transform::default(), Name::new("Q")))
        .id()
        .to_bits();
    let mut doc = TimelineDoc::new();
    set_expr(&mut doc, 0, p, PropKind::TranslationX, "0.5 * Q.x + 10");
    set_expr(&mut doc, 0, q, PropKind::TranslationX, "P.x");
    for _ in 0..80 {
        apply_from_doc(&mut world, &mut doc, 0.0);
    }
    let pv = x_of(&world, p);
    assert!(
        pv.is_finite(),
        "the cycle stayed finite (did not explode); got {pv}"
    );
    assert!(
        (pv - 20.0).abs() < 0.5,
        "the contractive cycle converges to its fixed point 20; got {pv}"
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

/// **Gate #18 (W3 follow-up) — a per-clip PROP-LINK resolves in the KEYS solo view.** Editing
/// the active clip alone (`apply_active_clip`, the panel's Keys tab): Follower drives its X by
/// the per-clip expression `Sprite.x`, Sprite keyed flat at 100. The solo now composes in
/// TOPOLOGICAL order and hands `solo_source_value` a populated `LinkFrame`, so `Sprite.x` reads
/// Sprite's composed 100 in the SAME frame — the scene apply's double-fade, now reachable while
/// editing keys. Before this the view threaded an EMPTY `LinkFrame`, so the link read 0.
///
/// Follower's binding is created FIRST, so the natural order would compose it before its source;
/// only the topological order gets it right. Mutation: thread an empty `LinkFrame` (drop
/// `build_names`/`topo_order` in `apply_active_clip`) -> `Sprite.x` -> 0, RED.
#[test]
fn the_keys_solo_resolves_a_per_clip_prop_link() {
    let mut world = World::new();
    // Reader FIRST (binding index 0): the natural order would compose it before its source.
    let follower = world
        .spawn((Transform::default(), Name::new("Follower")))
        .id()
        .to_bits();
    let sprite = world
        .spawn((Transform::default(), Name::new("Sprite")))
        .id()
        .to_bits();
    let mut doc = TimelineDoc::new();
    // Active clip 0: Follower driven by `Sprite.x` (per-clip expr), Sprite keyed flat at 100.
    set_expr(&mut doc, 0, follower, PropKind::TranslationX, "Sprite.x");
    flat(&mut doc, 0, sprite, PropKind::TranslationX, 100.0);
    assert!(
        doc.stack().is_empty(),
        "the Keys solo is the NON-STACKED view"
    );

    apply_active_clip(&mut world, &mut doc, 0.0, |_| false);
    let s = x_of(&world, sprite);
    assert!(
        (s - 100.0).abs() < 1e-3,
        "Sprite is keyed flat at 100; got {s}"
    );
    let f = x_of(&world, follower);
    assert!(
        (f - 100.0).abs() < 1e-3,
        "Follower's per-clip `Sprite.x` reads Sprite's composed 100 in the KEYS solo; got {f} \
         (0 = the view threaded an empty LinkFrame)"
    );
}

/// **Gate #19 (W4) — the clean separation: a per-clip SOURCE fades, a global TRANSFORM does
/// not.** ADR-0145's two roles on ONE channel. Clip 0 drives X by the PER-CLIP expression
/// `100` (a pure lane source); clip 1 keys X flat at 0; the two strips crossfade over [1,2) —
/// so the COMPOSED X fades to 50 mid-overlap (the per-clip half is a faded lane source). A
/// GLOBAL `value * 2` (`binding.expr`) then transforms that composed value AS A CHANNEL
/// FORMULA — applied at FULL wherever the composition covers, on the cut clock, NEVER weighted
/// by the overlap. Mid-overlap X = 2 * 50 = 100; at full weight X = 2 * 100 = 200.
///
/// This is the separation as one statement: the SAME channel carries a faded source and an
/// un-faded transform, and both land. Mutation A (per-clip stops being a faded source):
/// disable the `Expr` arm in `clip_anim_source` -> clip 0 contributes nothing -> composed X is
/// the keyed 0 -> the global gives 0, not 100. Mutation B (the global transform stops running):
/// drop the `run` body / `Expr` write in `expr_pass` -> X stays the composed 50, not 100.
#[test]
fn a_per_clip_source_fades_and_a_global_transform_does_not() {
    let (mut world, mut doc, e) = scene(0.0);
    doc.add_clip("Zero".into()); // clip 1
    // Per-clip lane source on clip 0 (pure expr) + keyed 0 on clip 1 -> composed X crossfades.
    set_expr(&mut doc, 0, e, PropKind::TranslationX, "100");
    flat(&mut doc, 1, e, PropKind::TranslationX, 0.0);
    // A GLOBAL channel transform on the SAME channel: value * 2, applied post-composition.
    let tgt = doc.bind(e, PropKind::TranslationX);
    doc.bindings_mut()
        .iter_mut()
        .find(|b| b.target == tgt)
        .unwrap()
        .expr = Some("value * 2".into());

    let mut lane = ClipLane::new("Base");
    lane.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 2.0));
    lane.insert(ClipStrip::new(StripSource::Clip(1), 1.0, 3.0, 2.0));
    doc.stack_mut().push(lane);

    // Full weight (only clip 0 plays): composed = the per-clip 100, the global doubles it -> 200.
    apply_from_doc(&mut world, &mut doc, 0.5);
    let full = x_of(&world, e);
    assert!(
        (full - 200.0).abs() < 1e-3,
        "at full weight the global transform doubles the per-clip 100 -> 200; got {full}"
    );

    // Mid overlap: the per-clip source FADES to 50, the global transform doubles it at FULL -> 100.
    apply_from_doc(&mut world, &mut doc, 1.5);
    let mid = x_of(&world, e);
    assert!(
        (mid - 100.0).abs() < 1e-3,
        "the per-clip source fades to 50 and the global transform doubles it at full (100); \
         got {mid} (0 = per-clip stopped fading; 50 = the global did not transform)"
    );
}

/// **Gate #20 (W6, §3) — the onion ghost EVALUATES a local expression at its OWN time.** The
/// object is driven by `time*10`; the live scene sits at t=1 (x=10). The onion samples the
/// NEIGHBOUR pose at t=3, and must show the expression there (30), not the live pose (10).
/// `pose_at` reads the SAME door the apply does (`solo_source_value`) with a degenerate
/// LinkFrame — a LOCAL expression ignores the frame and ghosts EXACTLY.
///
/// Mutation (revert `pose_at` to sampling the raw track): the expression-driven channel has no
/// keyed track, so the loop skips it and `pose_at` returns the live pose (10) at every ghost
/// time, RED.
#[test]
fn the_onion_ghost_evaluates_a_local_expression() {
    let (mut world, mut doc, e) = scene(0.0);
    set_expr(&mut doc, 0, e, PropKind::TranslationX, "time*10");
    apply_from_doc(&mut world, &mut doc, 1.0); // the live scene: x = 10
    assert!((x_of(&world, e) - 10.0).abs() < 1e-3, "live pose is 10");

    let ghost = pose_at(&world, &doc, e, 3.0).expect("the entity has a Transform");
    assert!(
        (ghost.translation.x - 30.0).abs() < 1e-3,
        "the onion ghost evaluates `time*10` at ITS OWN time (3 -> 30), not the live pose (10); \
         got {} (a raw-track sample would return the live pose)",
        ghost.translation.x
    );
}

/// **Gate #15 (W7) — the cross-OS hash of a `wiggle` + prop-link scene.** A deterministic
/// fingerprint of a scene driven by `wiggle` and a prop-link, folded over 121 frames. The
/// nextest matrix runs this on Linux/macOS/Windows; a divergent hash on any OS fails there,
/// which is the point — it pins that the driven path is bit-reproducible across platforms.
///
/// ⚠️ **ONLY `wiggle` (Noise = an integer hash of the f32 bits) and arithmetic — NEVER
/// `sin`/`cos`.** The std transcendentals (`ph2d-expr/eval.rs:42-43`) are not bit-identical
/// across platforms; a fingerprint including them would diverge between OSes (ADR-0146 §5.15).
#[test]
fn the_cross_os_hash_of_wiggle_plus_prop_link() {
    let mut world = World::new();
    let src = world
        .spawn((Transform::default(), Name::new("Src")))
        .id()
        .to_bits();
    let follower = world
        .spawn((Transform::default(), Name::new("Follower")))
        .id()
        .to_bits();
    let mut doc = TimelineDoc::new();
    // This gate sweeps `t` to 6.0 to fold 121 DISTINCT wiggle frames. A clip with NO authored
    // duration is UNBOUNDED (0 = infinite, Enio 2026-07-28): `clip_cut` never clamps the solo
    // clock, so all 121 frames stay distinct on their own. No override is authored here on
    // purpose — that is the very behaviour the hash pins.
    // Src wiggles on X and Y; Follower reads Src.x and adds its own wiggle (prop-link + wiggle).
    set_expr(&mut doc, 0, src, PropKind::TranslationX, "wiggle(3, 20)");
    set_expr(&mut doc, 0, src, PropKind::TranslationY, "wiggle(5, 8)");
    set_expr(
        &mut doc,
        0,
        follower,
        PropKind::TranslationX,
        "Src.x + wiggle(2, 5)",
    );

    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for i in 0..=120 {
        let t = f64::from(i) * 0.05;
        apply_from_doc(&mut world, &mut doc, t);
        for e in [src, follower] {
            let xf = *world.get::<Transform>(Entity::from_bits(e)).unwrap();
            for v in [xf.translation.x, xf.translation.y] {
                for byte in v.to_bits().to_le_bytes() {
                    h ^= u64::from(byte);
                    h = h.wrapping_mul(0x0000_0100_0000_01b3);
                }
            }
        }
    }
    assert_eq!(
        h, CROSS_OS_HASH,
        "the wiggle + prop-link scene diverged (a cross-OS divergence, or a regression)"
    );
}

/// Pinned on `line/anim` at ADR-0146 W7. Wiggle is an integer hash (cross-OS); the fold is
/// integer arithmetic. A divergence on any OS in the nextest matrix is a real defect.
/// ⚠️ **MOVED 2026-07-29** (`0x6ed2_84e3_8f4f_28f9` -> here), and the reason is a
/// deliberate semantic change, not a drift: `wiggle` now lowers onto a **smooth value
/// noise** instead of the raw hash, because with the hash its `freq` argument was a
/// SEED — measured, 494 to 509 zero-crossings per second across a 32x sweep, i.e. no
/// frequency at all (`ph2d-expr-parse::smooth_noise`, and the Enio smoke that named
/// it: *"a velocidade em shake nunca foi velocidade"*).
///
/// ⚠️ What this gate GUARDS is untouched: the lowering adds only `floor`, `fract`
/// (`x - floor(x)`), `mix` (`a*(1-t) + b*t`) and `+ - *` — every one of them
/// IEEE-754 exact, and Rust never contracts to FMA. No transcendental entered the
/// path, so the scene is still bit-reproducible on Linux/macOS/Windows, which is the
/// whole claim. A pin that moved for any OTHER reason is a bug.
const CROSS_OS_HASH: u64 = 0x8d0c_3807_61f8_141a;

/// **Gate #6 (W7, Hole B / C4) — a MULTI-CHANNEL keyed fade co-resident with an expression is
/// byte stable.** Gate #5 pins TranslationY under `scheduled`; the byte-identity fingerprint
/// (#1/#4) is TranslationX-only. So a mutation that perturbed the KEYED composition — or the
/// per-channel `write_prop` — of Rotation / Scale ONLY under `scheduled` would slip past all of
/// them. This folds Rotation, ScaleX, ScaleY (the channels the other fingerprints never touch)
/// across a crossfade while an X expression forces the two-phase path.
///
/// The other half of Hole B (C4): `read_prop` was deliberately NOT extended for Position/Morph
/// (they seed 0 in a cycle — a documented limitation), so the rest-capture path stays
/// byte-identical and needs no separate inert-ness gate. Mutation: any change that moves a non-X
/// keyed crossfade under `scheduled`, incl. swapping the Scale axes in `write_prop`.
#[test]
fn a_multi_channel_keyed_fade_co_resident_with_an_expression_is_byte_stable() {
    let (h, range) = multichannel_coresident_fingerprint();
    assert!(
        range > 5.0,
        "the co-resident scene went inert (range {range}); it must exercise the fade"
    );
    assert_eq!(
        h, MULTICHANNEL_FINGERPRINT,
        "a non-X keyed crossfade moved WHILE an expression drove X. Re-pin in the same commit if \
         intended."
    );
}

/// Rotation / ScaleX / ScaleY keyed crossfades over [1,2), sampled while X is expression-driven.
fn multichannel_coresident_fingerprint() -> (u64, f32) {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();
    let mut doc = TimelineDoc::new();
    doc.add_clip("B".into()); // clip 1

    // X: expression-driven (forces scheduled == true).
    set_expr(&mut doc, 0, e, PropKind::TranslationX, "time*5");
    // Rotation / ScaleX / ScaleY: keyed crossfades across two clips.
    for (prop, a, b) in [
        (PropKind::Rotation, 0.0, 1.0),
        (PropKind::ScaleX, 1.0, 2.0),
        (PropKind::ScaleY, 3.0, 4.0),
    ] {
        ramp(&mut doc, 0, e, prop, a, b);
        ramp(&mut doc, 1, e, prop, a + 10.0, b + 10.0);
    }
    let mut lane = ClipLane::new("Base");
    lane.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 2.0));
    lane.insert(ClipStrip::new(StripSource::Clip(1), 1.0, 3.0, 2.0));
    doc.stack_mut().push(lane);

    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for i in 0..=60 {
        let t = f64::from(i) * 0.05;
        apply_from_doc(&mut world, &mut doc, t);
        let xf = *world.get::<Transform>(Entity::from_bits(e)).unwrap();
        for v in [xf.rotation, xf.scale.x, xf.scale.y] {
            lo = lo.min(v);
            hi = hi.max(v);
            for byte in v.to_bits().to_le_bytes() {
                h ^= u64::from(byte);
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
    }
    (h, hi - lo)
}

/// Pinned on `line/anim` at ADR-0146 W7 (Hole B). Non-X keyed crossfades under `scheduled`.
const MULTICHANNEL_FINGERPRINT: u64 = 0x97e1_9fe3_ea22_4329;

/// **Cost measurement (ADR-0146 W7, `#[ignore]`).** The named trigger from the plan: HUNDREDS of
/// prop-link channels. Each frame the scheduler topo-sorts the graph, then re-evaluates every
/// channel through `solo_source_value` — parsing each expression afresh (~335 ns, caching
/// measured-and-rejected in `expr_pass.rs`). This builds a CHAIN of `N` prop-links (each reads
/// the previous — the deepest topo order) and reports ms/frame. The FORMULA-FREE path is
/// untouched (gate #3 pins zero-alloc). Run:
/// `cargo test -p ph2d-timeline --test expr_in_blend measure -- --ignored --nocapture`.
///
/// **Measured (line/anim, DEBUG, `N = 300`): 1.86 ms/frame** — 11% of a 60 fps frame, for 300
/// channels each parsing a prop-link + a wiggle every frame and topo-sorted into one deep chain.
/// A real scene has tens of expression channels, not hundreds in a chain, so this is the ceiling
/// of the named trigger, not a typical cost; release is several times cheaper. No cap is
/// warranted — the number is named, and the formula-free path pays nothing (gate #3).
#[test]
#[ignore = "cost measurement, run manually"]
fn measure_hundreds_of_prop_link_channels() {
    const N: usize = 300;
    let mut world = World::new();
    let mut doc = TimelineDoc::new();
    let src = world
        .spawn((Transform::default(), Name::new("N0")))
        .id()
        .to_bits();
    set_expr(&mut doc, 0, src, PropKind::TranslationX, "wiggle(2, 5)");
    for i in 1..N {
        let e = world
            .spawn((Transform::default(), Name::new(format!("N{i}"))))
            .id()
            .to_bits();
        // Ni.x = N(i-1).x + wiggle: a chain of prop-links, wiggle-only (no transcendental).
        set_expr(
            &mut doc,
            0,
            e,
            PropKind::TranslationX,
            &format!("N{}.x + wiggle(2, 5)", i - 1),
        );
    }

    // Warm one frame, then time 100.
    apply_from_doc(&mut world, &mut doc, 0.0);
    let frames = 100u32;
    let start = std::time::Instant::now();
    for i in 0..frames {
        apply_from_doc(&mut world, &mut doc, f64::from(i) * 0.05);
    }
    let per_frame_ms = start.elapsed().as_secs_f64() * 1000.0 / f64::from(frames);
    println!("PROP-LINK COST: {N} chained prop-link channels = {per_frame_ms:.3} ms/frame");
    assert!(
        world
            .get::<Transform>(Entity::from_bits(src))
            .unwrap()
            .translation
            .x
            .is_finite()
    );
}

/// **A per-clip expression FREEZES at the clip's authored cut** (Enio: *"expressões não estão
/// obedecendo a duração dos clips e tocam fora da área válida"*). A clip authored to 2 s,
/// driving X by the pure expression `time*10` over a strip `[0,5)`: within the cut `time`
/// advances; PAST the cut it holds the cut's `time` (2 s) — the SAME `cut_source` (`clip_cut`)
/// the keyed pass rides (`stack_frames::collect_frame`), so the formula respects the veil that
/// darkens the dead zone. With NO authored duration there is no cut, and it plays the strip.
///
/// This is why Bug 5 was a consequence of Bug 4: the leak is a clip with `length_override:
/// None` (nothing cuts it), not an expr that ignores the cut.
///
/// Mutation: drop the `cut_source` clamp in `collect_frame` -> the expr extrapolates past the
/// cut (30, not the frozen 20), and the first assert on the authored clip fails.
#[test]
fn a_per_clip_expression_freezes_at_the_clips_authored_cut() {
    let build = |dur: Option<f64>| {
        let (world, mut doc, e) = scene(0.0);
        set_expr(&mut doc, 0, e, PropKind::TranslationX, "time*10");
        doc.set_clip_length_override(0, dur);
        doc.set_scene_length(Some(5.0)); // the scene does not cut first
        let mut lane = ClipLane::new("Base");
        lane.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 5.0, 5.0));
        doc.stack_mut().push(lane);
        (world, doc, e)
    };

    // Authored to 2 s: t=1 -> 10; t=3 (past the cut) -> FROZEN at the 2 s pose = 20.
    let (mut w, mut doc, e) = build(Some(2.0));
    apply_from_doc(&mut w, &mut doc, 1.0);
    assert!(
        (x_of(&w, e) - 10.0).abs() < 1e-3,
        "within the cut: time*10 = 10"
    );
    apply_from_doc(&mut w, &mut doc, 3.0);
    assert!(
        (x_of(&w, e) - 20.0).abs() < 1e-3,
        "past the 2 s cut the expr FREEZES at time=2 (-> 20), never extrapolates: {}",
        x_of(&w, e)
    );

    // No authored duration -> no cut -> the whole strip plays (t=3 -> 30).
    let (mut w2, mut doc2, e2) = build(None);
    apply_from_doc(&mut w2, &mut doc2, 3.0);
    assert!(
        (x_of(&w2, e2) - 30.0).abs() < 1e-3,
        "no override = no cut, the expr plays the strip: {}",
        x_of(&w2, e2)
    );
}
