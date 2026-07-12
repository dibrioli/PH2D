//! ADR-0115 A8/A9: authoring a key while a clip stack is driving the scene.
//!
//! Two things must hold, and the first one is a REGRESSION GUARD:
//!
//! 1. A pose that is exactly what the stack produced must key **nothing**. The
//!    diff compares the world against what the apply WROTE — the blend — and not
//!    against the active clip's raw curve, which under a stack differs every
//!    single frame. Reading the curve at a time the apply never used is the exact
//!    shape of the bug that minted a key per frame on 2026-07-12; reading a
//!    *different curve entirely* would be the same bug, one order of magnitude
//!    louder.
//!
//! 2. A pose the animator DID make must round-trip: the value stored in the
//!    active clip, once the stack re-evaluates it, must put the object back where
//!    they left it — or the key must be refused. Never a third outcome.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{
    ClipLane, ClipStrip, LaneMode, PoseSample, PropKind, TimelineDoc, apply_from_doc,
    autokey_props, key_time, key_value_in_active_clip,
};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn scene() -> (World, TimelineDoc, u64) {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();
    (world, TimelineDoc::new(), e)
}

fn flat(doc: &mut TimelineDoc, clip: usize, e: u64, prop: PropKind, v: f32) {
    let was = doc.active_index();
    doc.set_active(clip);
    doc.insert_key(e, prop, s(0.0), AnimValue::Float(v), Interp::Linear);
    doc.insert_key(e, prop, s(2.0), AnimValue::Float(v), Interp::Linear);
    doc.set_active(was);
}

fn lane(clip: u16, mode: LaneMode, weight: f64) -> ClipLane {
    let mut l = ClipLane::new("L");
    l.mode = mode;
    l.weight = weight;
    l.insert(ClipStrip::new(clip, 0.0, 2.0, 2.0));
    l
}

fn x_of(world: &World, e: u64) -> f32 {
    world
        .get::<Transform>(Entity::from_bits(e))
        .unwrap()
        .translation
        .x
}

fn pose_x(x: f32) -> PoseSample {
    let mut p: PoseSample = [None; 6];
    p[0] = Some(x); // PropKind::ALL[0] == TranslationX
    p
}

/// **The regression guard.** A stack drives the object; the animator touches
/// nothing; auto-key must write nothing. The diff has to read the blend — the
/// number the apply actually wrote — because the active clip's own curve says
/// something different at every instant, and comparing against THAT would mint a
/// key per frame with the object standing perfectly still.
#[test]
fn a_pose_sitting_on_the_blend_keys_nothing() {
    let (mut world, mut doc, e) = scene();
    doc.add_clip("Top".to_string());
    flat(&mut doc, 0, e, PropKind::TranslationX, 100.0); // the clip being edited
    flat(&mut doc, 1, e, PropKind::TranslationX, 200.0);

    doc.stack_mut().push(lane(0, LaneMode::Override, 1.0));
    doc.stack_mut().push(lane(1, LaneMode::Override, 0.5)); // blend = 150

    for step in 0..=20 {
        let t = f64::from(step) * 0.1;
        apply_from_doc(&mut world, &mut doc, t);
        let shown = x_of(&world, e);
        assert!((shown - 150.0).abs() < 1e-4, "the stack shows the blend");

        let plan = autokey_props(&doc, e, t, &pose_x(shown), &pose_x(shown), true);
        assert!(
            plan.is_empty(),
            "t={t}: auto-key wrote {:?} for an object nobody touched \
             (the diff read the clip's curve, not the blend)",
            plan.keys
        );
    }
}

/// The animator drags the object to 180 while a lane above is pulling it halfway
/// toward 200. The key stored in the clip they are editing must be
/// **pre-compensated** — and the proof is the round trip: write the key, let the
/// stack re-evaluate, and the object must be at 180.
#[test]
fn a_key_authored_under_a_stack_round_trips_through_the_blend() {
    let (mut world, mut doc, e) = scene();
    doc.add_clip("Top".to_string());
    flat(&mut doc, 0, e, PropKind::TranslationX, 100.0);
    flat(&mut doc, 1, e, PropKind::TranslationX, 200.0);
    doc.stack_mut().push(lane(0, LaneMode::Override, 1.0));
    doc.stack_mut().push(lane(1, LaneMode::Override, 0.5));

    apply_from_doc(&mut world, &mut doc, 1.0); // shows 150
    let want = 180.0_f32;

    let plan = autokey_props(&doc, e, 1.0, &pose_x(want), &pose_x(150.0), true);
    assert_eq!(plan.keys.len(), 1, "one key, no refusal: {plan:?}");
    let (prop, stored) = plan.keys[0];

    // The naive answer would be 180 — the pose itself. The right one is 160,
    // because the lane above will drag it half way to 200 again.
    assert!(
        (stored - 160.0).abs() < 1e-3,
        "stored {stored}, but 180 is the POSE, not what the clip must hold"
    );

    let t = key_time(&doc, e, 1.0).expect("the clip plays exactly once");
    doc.insert_key(e, prop, s(t), AnimValue::Float(stored), Interp::Linear);
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        (x_of(&world, e) - want).abs() < 1e-3,
        "the object must end up where the animator dragged it, got {}",
        x_of(&world, e)
    );
}

/// **Refuse, never lie.** An `Override` lane at full weight above the clip being
/// edited owns the channel outright: no value in that clip can change what the
/// animator sees. The honest answers are "refuse" and "move the object behind
/// their back" — and only one of those is acceptable. (Blender's new layered
/// system reaches the same verdict: *"Blender will simply reject keying and issue
/// an error."*)
#[test]
fn a_pose_the_active_clip_cannot_express_is_refused_not_faked() {
    let (mut world, mut doc, e) = scene();
    doc.add_clip("Top".to_string());
    flat(&mut doc, 0, e, PropKind::TranslationX, 100.0);
    flat(&mut doc, 1, e, PropKind::TranslationX, 200.0);
    doc.stack_mut().push(lane(0, LaneMode::Override, 1.0));
    doc.stack_mut().push(lane(1, LaneMode::Override, 1.0)); // full override on top

    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        (x_of(&world, e) - 200.0).abs() < 1e-4,
        "the top lane owns it"
    );

    let plan = autokey_props(&doc, e, 1.0, &pose_x(350.0), &pose_x(200.0), true);
    assert!(plan.keys.is_empty(), "nothing may be written");
    assert_eq!(
        plan.refused,
        vec![PropKind::TranslationX],
        "and the refusal is REPORTED, not swallowed"
    );

    assert_eq!(
        key_value_in_active_clip(&doc, e, PropKind::TranslationX, 350.0),
        None,
        "the manual K path refuses on the same rule"
    );
}

/// A clip playing **twice at once** offers two homes for "key it here". Picking
/// one silently would drop the key somewhere the animator never looked. Blender
/// hits the same wall and documents it: keyframe remapping only works when the
/// action *"occurs once in the current frame"*.
#[test]
fn a_clip_playing_twice_at_once_has_no_single_place_to_key() {
    let (mut world, mut doc, e) = scene();
    flat(&mut doc, 0, e, PropKind::TranslationX, 100.0);

    let mut l = ClipLane::new("Base");
    l.insert(ClipStrip::new(0, 0.0, 2.0, 2.0));
    l.insert(ClipStrip::new(0, 1.0, 3.0, 2.0)); // the SAME clip, overlapping itself
    doc.stack_mut().push(l);

    apply_from_doc(&mut world, &mut doc, 1.5); // inside the overlap: it plays twice
    assert_eq!(
        key_time(&doc, e, 1.5),
        None,
        "two occurrences, no single answer"
    );

    apply_from_doc(&mut world, &mut doc, 0.5); // before the overlap: it plays once
    assert!(
        key_time(&doc, e, 0.5).is_some(),
        "one occurrence, one answer"
    );
}

/// Without a stack, nothing above changes: the key stored is the pose itself, and
/// it lands at the entity's own clock. The whole feature is additive or it is
/// nothing.
#[test]
fn with_no_stack_the_authoring_rules_are_exactly_what_they_were() {
    let (mut world, mut doc, e) = scene();
    flat(&mut doc, 0, e, PropKind::TranslationX, 100.0);
    apply_from_doc(&mut world, &mut doc, 1.0);

    assert_eq!(
        key_value_in_active_clip(&doc, e, PropKind::TranslationX, 42.0),
        Some(42.0),
        "the track IS the scene"
    );
    assert_eq!(
        key_time(&doc, e, 1.0),
        Some(1.0),
        "and the clock is the clock"
    );

    let plan = autokey_props(&doc, e, 1.0, &pose_x(42.0), &pose_x(100.0), true);
    assert_eq!(plan.keys, vec![(PropKind::TranslationX, 42.0)]);
    assert!(plan.refused.is_empty());
}

// ── A9 / R7 — the invariant, made executable ────────────────────────────────

/// **INVARIANT: every `PropKind` is a blendable scalar.**
///
/// Mid-crossfade, every channel must land on the exact mean of the two clips —
/// i.e. it *interpolated*. That is true today because all seven properties are
/// `f32`.
///
/// If someone adds a **discrete** property (a sprite-sheet frame index, a
/// visibility flag, a Flip drawing, a z-order), this test is the tripwire: a
/// drawing cannot be half-way between two drawings, and `AnimValue::lerp` would
/// happily "blend" a `Bool` by stepping at `t < 0.5` and a mismatched pair by
/// silently returning the second — both wrong, both silent. A discrete channel
/// needs a Replace-only path through the stack, and this test is where you will
/// be forced to notice.
#[test]
fn every_prop_kind_interpolates_and_a_discrete_one_would_break_this() {
    for &prop in &PropKind::ALL {
        let (mut world, mut doc, e) = scene();
        doc.add_clip("B".to_string());
        // Two clips a long way apart, so a mid-crossfade STEP is unmistakable.
        flat(&mut doc, 0, e, prop, 1.0);
        flat(&mut doc, 1, e, prop, 3.0);

        let mut l = ClipLane::new("Base");
        l.insert(ClipStrip::new(0, 0.0, 2.0, 2.0));
        l.insert(ClipStrip::new(1, 1.0, 3.0, 2.0)); // 1 s overlap
        doc.stack_mut().push(l);

        apply_from_doc(&mut world, &mut doc, 1.5); // dead centre of the crossfade

        let xf = world.get::<Transform>(Entity::from_bits(e)).unwrap();
        let got = match prop {
            PropKind::TranslationX => xf.translation.x,
            PropKind::TranslationY => xf.translation.y,
            PropKind::Rotation => xf.rotation,
            PropKind::ScaleX => xf.scale.x,
            PropKind::ScaleY => xf.scale.y,
            // Opacity lives on `Sprite`, which this bare Transform entity has not
            // got; the blend math is the same code and is covered above.
            PropKind::Opacity | PropKind::TimeRemap => continue,
        };
        assert!(
            (got - 2.0).abs() < 1e-4,
            "{prop:?} mid-crossfade is {got}, not the mean 2.0 — it did not \
             interpolate. If this is a DISCRETE channel, it must not go through \
             the blend at all (ADR-0115 R7)."
        );
    }
}
