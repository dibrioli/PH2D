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
    ClipLane, ClipStrip, LaneMode, PoseSample, PropKind, StripSource, TimelineDoc, apply_from_doc,
    autokey_props, autokey_props_solo, key_time, key_value_in_active_clip,
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
    l.insert(ClipStrip::new(StripSource::Clip(clip), 0.0, 2.0, 2.0));
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
    l.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 2.0));
    l.insert(ClipStrip::new(StripSource::Clip(0), 1.0, 3.0, 2.0)); // the SAME clip, overlapping itself
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
        l.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 2.0));
        l.insert(ClipStrip::new(StripSource::Clip(1), 1.0, 3.0, 2.0)); // 1 s overlap
        doc.stack_mut().push(l);

        apply_from_doc(&mut world, &mut doc, 1.5); // dead centre of the crossfade

        let xf = world.get::<Transform>(Entity::from_bits(e)).unwrap();
        let got = match prop {
            PropKind::TranslationX => xf.translation.x,
            PropKind::TranslationY => xf.translation.y,
            PropKind::Rotation => xf.rotation,
            PropKind::ScaleX => xf.scale.x,
            PropKind::ScaleY => xf.scale.y,
            // Opacity lives on `Sprite` and Morph on `VecMorph`, neither of which this bare
            // Transform entity has got; the blend math is the same code and is covered above.
            PropKind::Opacity | PropKind::TimeRemap | PropKind::Morph => continue,
        };
        assert!(
            (got - 2.0).abs() < 1e-4,
            "{prop:?} mid-crossfade is {got}, not the mean 2.0 — it did not \
             interpolate. If this is a DISCRETE channel, it must not go through \
             the blend at all (ADR-0115 R7)."
        );
    }
}

// ── A key can move the reference it is measured against ─────────────────────

/// **The additive key that was silently thrown away** (audit, 2026-07-12).
///
/// An additive strip measures its delta against its clip's OWN value at `src_in`.
/// Key at the strip's first frame — where an animator starts posing — and the key
/// you write IS the value at `src_in`: the delta comes out zero, the pose is lost,
/// and (worse) every OTHER frame of that lane translates by the value you just
/// invented. The probe held the reference fixed, so the solve reported full
/// influence where the truth is none, and nothing refused.
///
/// The probe now models the WRITE, so `A` really is 0 and the key is refused. A
/// refusal is a correct answer here — the additive delta at a clip's own first
/// frame is zero BY DEFINITION (Maya: "relative to its first frame"), so no value
/// in the clip can produce that pose. What is NOT acceptable is the third outcome:
/// writing a key and moving the object anyway.
#[test]
fn an_additive_key_at_the_strips_first_frame_is_refused_not_lost() {
    let (mut world, mut doc, e) = scene();
    // rest = 100, clip ramps 0 -> 10 over its 2 s.
    if let Some(mut t) = world.get_mut::<Transform>(Entity::from_bits(e)) {
        t.translation.x = 100.0;
    }
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
        s(2.0),
        AnimValue::Float(10.0),
        Interp::Linear,
    );
    let mut l = ClipLane::new("Add");
    l.mode = LaneMode::Additive;
    l.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 4.0, 2.0));
    doc.stack_mut().push(l);

    // The strip's FIRST frame: the delta is definitionally zero, so the sprite sits
    // at its rest pose.
    apply_from_doc(&mut world, &mut doc, 0.0);
    assert!((x_of(&world, e) - 100.0).abs() < 1e-3);

    // The animator drags it to 130. There is no value the clip could hold that
    // would produce 130 here — keying it moves the reference too.
    assert_eq!(
        key_value_in_active_clip(&doc, e, PropKind::TranslationX, 130.0),
        None,
        "no value in the clip reaches this pose: refuse, do not invent one \
         (before the fix it returned Some(30.0), the pose came out at 100 anyway, \
          and every other frame of the lane shifted by -30)"
    );

    // And the rest of the strip is untouched — nothing was written.
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        (x_of(&world, e) - 105.0).abs() < 1e-3,
        "the lane still reads +5 at its midpoint: {}",
        x_of(&world, e)
    );
}

/// Away from the reference the additive inversion is exact, and it must stay so —
/// the fix must not turn a working case into a refusal. Keying mid-strip writes a
/// value that reproduces the pose the animator posed.
#[test]
fn an_additive_key_away_from_the_reference_still_round_trips_exactly() {
    let (mut world, mut doc, e) = scene();
    if let Some(mut t) = world.get_mut::<Transform>(Entity::from_bits(e)) {
        t.translation.x = 100.0;
    }
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
        s(2.0),
        AnimValue::Float(10.0),
        Interp::Linear,
    );
    let mut l = ClipLane::new("Add");
    l.mode = LaneMode::Additive;
    l.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 4.0, 2.0));
    doc.stack_mut().push(l);
    apply_from_doc(&mut world, &mut doc, 1.0); // mid-strip; the pose is 105

    let stored = key_value_in_active_clip(&doc, e, PropKind::TranslationX, 130.0)
        .expect("mid-strip the clip DOES have influence");
    let t_key = key_time(&doc, e, 1.0).expect("and the key has a home");
    doc.insert_key(
        e,
        PropKind::TranslationX,
        s(t_key),
        AnimValue::Float(stored),
        Interp::Linear,
    );

    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        (x_of(&world, e) - 130.0).abs() < 1e-2,
        "the key must reproduce the pose it was authored from: {}",
        x_of(&world, e)
    );
}

/// **Soloed, o MESMO documento keya — e a pose na própria curva não keya nada.**
///
/// A metade de crate do report de 2026-07-22 (*"a partir do momento que eu crio uma
/// strip numa lane, não consigo mais criar keys com autokey"*): a vista Keys dirige a
/// cena só pelo clip ativo, então o auto-key dela diffa contra a curva SOLOADA (não o
/// blend) e armazena a pose CRUA (sem inverso) — `autokey_props_solo`, gêmea da
/// `key_authoring_solo` do K. Julgada contra o blend, uma pose parada sobre a própria
/// curva lia como movida todo frame, e o mapa da strip recusava: a parede de toasts.
#[test]
fn soloed_the_same_pose_keys_and_the_on_curve_pose_keys_nothing() {
    let (mut world, mut doc, e) = scene();
    doc.add_clip("Top".to_string());
    flat(&mut doc, 0, e, PropKind::TranslationX, 100.0);
    flat(&mut doc, 1, e, PropKind::TranslationX, 200.0);
    doc.stack_mut().push(lane(0, LaneMode::Override, 1.0));
    doc.stack_mut().push(lane(1, LaneMode::Override, 1.0)); // dona do canal no ARRANGE
    apply_from_doc(&mut world, &mut doc, 1.0); // prima o scratch p/ o controle positivo

    // Arrastada para fora da curva soloada: a key aterrissa, CRUA, sem recusa.
    let plan = autokey_props_solo(&doc, e, 1.0, &pose_x(350.0), &pose_x(100.0), true);
    assert_eq!(
        plan.keys,
        vec![(PropKind::TranslationX, 350.0)],
        "solo armazena a própria pose — não há blend para inverter"
    );
    assert!(plan.refused.is_empty(), "solar é o que garante um lugar");

    // Sobre a curva soloada (100): nada se moveu, nada keya — mesmo com o BLEND
    // mostrando 200 aqui.
    let still = autokey_props_solo(&doc, e, 1.0, &pose_x(100.0), &pose_x(100.0), true);
    assert!(
        still.is_empty(),
        "pose == curva soloada: nenhuma key fantasma"
    );
    // Controle positivo: o diff do ARRANGE vê a MESMA pose como movida — é o
    // fantasma que o solo existe para não ver.
    assert!(
        !autokey_props(&doc, e, 1.0, &pose_x(100.0), &pose_x(100.0), true).is_empty(),
        "sem o fenômeno no fixture, mutar o solo para ler o blend ficaria verde"
    );
}
