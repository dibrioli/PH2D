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
use ph2d_ecs::{Entity, Name, Transform, World};
use ph2d_timeline::{
    ClipLane, ClipStrip, LaneMode, PoseSample, PropKind, StripSource, TimelineDoc, apply_from_doc,
    apply_from_doc_except, autokey_props, autokey_props_solo, key_time, key_value_in_active_clip,
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
    let mut p: PoseSample = [None; 7];
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

        let plan = autokey_props(&doc, e, t, &pose_x(shown), &pose_x(shown), true, false);
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

    let plan = autokey_props(&doc, e, 1.0, &pose_x(want), &pose_x(150.0), true, false);
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

    let plan = autokey_props(&doc, e, 1.0, &pose_x(350.0), &pose_x(200.0), true, false);
    assert!(plan.keys.is_empty(), "nothing may be written");
    assert_eq!(
        plan.refused,
        vec![PropKind::TranslationX],
        "and the refusal is REPORTED, not swallowed"
    );

    assert_eq!(
        key_value_in_active_clip(&doc, e, PropKind::TranslationX, 350.0, 1.0),
        Err(ph2d_timeline::KeyRefusal::Overridden),
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
        key_value_in_active_clip(&doc, e, PropKind::TranslationX, 42.0, 1.0),
        Ok(42.0),
        "the track IS the scene"
    );
    assert_eq!(
        key_time(&doc, e, 1.0),
        Some(1.0),
        "and the clock is the clock"
    );

    let plan = autokey_props(&doc, e, 1.0, &pose_x(42.0), &pose_x(100.0), true, false);
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
            // Opacity lives on `Sprite`, Morph on `VecMorph`, and Position needs a path
            // on its binding — none of which this bare Transform entity has got.
            PropKind::Opacity | PropKind::TimeRemap | PropKind::Morph | PropKind::Position => {
                continue;
            }
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
        key_value_in_active_clip(&doc, e, PropKind::TranslationX, 130.0, 0.0),
        Err(ph2d_timeline::KeyRefusal::Overridden),
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

    let stored = key_value_in_active_clip(&doc, e, PropKind::TranslationX, 130.0, 1.0)
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
    let plan = autokey_props_solo(&doc, e, 1.0, &pose_x(350.0), &pose_x(100.0), true, false);
    assert_eq!(
        plan.keys,
        vec![(PropKind::TranslationX, 350.0)],
        "solo armazena a própria pose — não há blend para inverter"
    );
    assert!(plan.refused.is_empty(), "solar é o que garante um lugar");

    // Sobre a curva soloada (100): nada se moveu, nada keya — mesmo com o BLEND
    // mostrando 200 aqui.
    let still = autokey_props_solo(&doc, e, 1.0, &pose_x(100.0), &pose_x(100.0), true, false);
    assert!(
        still.is_empty(),
        "pose == curva soloada: nenhuma key fantasma"
    );
    // Controle positivo: o diff do ARRANGE vê a MESMA pose como movida — é o
    // fantasma que o solo existe para não ver.
    assert!(
        !autokey_props(&doc, e, 1.0, &pose_x(100.0), &pose_x(100.0), true, false).is_empty(),
        "sem o fenômeno no fixture, mutar o solo para ler o blend ficaria verde"
    );
}

/// Stamp a per-clip expression on clip `clip`'s channel `(e, prop)`, binding the target.
fn set_expr(doc: &mut TimelineDoc, clip: usize, e: u64, prop: PropKind, src: &str) {
    let tgt = doc.bind(e, prop);
    doc.set_clip_expr(clip, tgt, Some(src.to_string()));
}

/// **Gate #11a (W5) — `value + g(time)` KEYS and PRE-COMPENSATES**, both non-stacked and
/// stacked. The most-used AE idiom: an expression that offsets the keyed value. Keying `want`
/// must store the value the offset needs so the composed scene lands EXACTLY on `want` —
/// `stored = want - g(t)`, not `want` raw (which would show `want + g(t)`).
///
/// Non-stacked (C3): `value + time*10` at t=1 has g=10, so keying 50 stores 40, and the scene
/// round-trips to 50. Stacked: `value + 100` on a full Override lane stores 50 for want 150.
/// Mutation (revert the probe-through-expr in `eval_frame` / the C3 invert): the solve measures
/// `p.value` raw and stores `want` un-compensated (40 -> 50 / 50 -> 150), RED.
#[test]
fn value_plus_g_of_time_keys_and_pre_compensates() {
    // ── Non-stacked (C3): the common keyed animation with no strips. ──
    let (mut world, mut doc, e) = scene();
    set_expr(&mut doc, 0, e, PropKind::TranslationX, "value + time*10");
    apply_from_doc(&mut world, &mut doc, 1.0); // scene = rest(0) + 10 = 10
    let stored = key_value_in_active_clip(&doc, e, PropKind::TranslationX, 50.0, 1.0)
        .expect("an affine expression keys");
    assert!(
        (stored - 40.0).abs() < 1e-3,
        "value + time*10 pre-compensates g(1)=10: store 50 - 10 = 40, not 50; got {stored}"
    );
    // Round-trip: the stored value, composed back through the expression, shows `want`.
    let t_key = key_time(&doc, e, 1.0).expect("the key has a home");
    doc.insert_key(
        e,
        PropKind::TranslationX,
        s(t_key),
        AnimValue::Float(stored),
        Interp::Hold,
    );
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        (x_of(&world, e) - 50.0).abs() < 1e-3,
        "the pre-compensated key round-trips to 50, got {}",
        x_of(&world, e)
    );

    // ── Stacked: a full Override lane playing the expression-driven clip. ──
    let (mut world, mut doc, e) = scene();
    set_expr(&mut doc, 0, e, PropKind::TranslationX, "value + 100");
    doc.stack_mut().push(lane(0, LaneMode::Override, 1.0));
    apply_from_doc(&mut world, &mut doc, 1.0); // scene = rest(0) + 100 = 100
    let stored = key_value_in_active_clip(&doc, e, PropKind::TranslationX, 150.0, 1.0)
        .expect("a full Override lane inverts through the expression");
    assert!(
        (stored - 50.0).abs() < 1e-3,
        "value + 100 pre-compensates: store 150 - 100 = 50 through the blend, not 150; got {stored}"
    );
}

/// **Gate #11b (W5) — a PURE formula refuses `ExpressionDriven` (not `Overridden`).** A
/// value-INDEPENDENT expression (`wiggle`) offers no stored value that changes the pose — the
/// solve is degenerate in `value` (A ~ 0). It must refuse, and the reason is the FORMULA
/// (clean/rewrite it), never the lane stack — distinct from `Overridden`, whose fix is a lane.
///
/// Both non-stacked and stacked. Mutation (`refusal_for` returns `Overridden` unconditionally):
/// the reason lies about what is wrong, RED.
#[test]
#[allow(
    non_snake_case,
    reason = "the gate name mirrors the ADR-0152 refusal variant"
)]
fn a_pure_formula_refuses_ExpressionDriven() {
    use ph2d_timeline::KeyRefusal;
    // Non-stacked: a pure wiggle drives the channel.
    let (mut world, mut doc, e) = scene();
    set_expr(&mut doc, 0, e, PropKind::TranslationX, "wiggle(2, 20)");
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert_eq!(
        key_value_in_active_clip(&doc, e, PropKind::TranslationX, 42.0, 1.0),
        Err(KeyRefusal::ExpressionDriven),
        "a value-independent formula refuses with the FORMULA reason, not Overridden"
    );

    // Stacked: the same, through a full Override lane — `invert_stack` sees A ~ 0 and, because a
    // formula drives the clip, names `ExpressionDriven` rather than the lane's `Overridden`.
    let (mut world, mut doc, e) = scene();
    set_expr(&mut doc, 0, e, PropKind::TranslationX, "wiggle(2, 20)");
    doc.stack_mut().push(lane(0, LaneMode::Override, 1.0));
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert_eq!(
        key_value_in_active_clip(&doc, e, PropKind::TranslationX, 42.0, 1.0),
        Err(KeyRefusal::ExpressionDriven),
        "stacked too: the fix is the formula, not the lane"
    );
}

/// **Gate #11c (W5) — a `value`-NON-LINEAR formula refuses.** `value*value` is affine at no
/// two points, so the two-point solve would draw a confident WRONG line (store `want`, land at
/// `want*want`). The THIRD probe catches it: `f(0.5)` strays from the line `f(0)..f(1)`, and the
/// key is refused instead of moving the object somewhere nobody asked for.
///
/// Mutation (skip the third probe in `solve_affine`): the non-linear solve returns a bogus
/// stored value (`Ok`) instead of refusing, RED.
#[test]
fn a_value_nonlinear_formula_refuses() {
    use ph2d_timeline::KeyRefusal;
    // Non-stacked.
    let (mut world, mut doc, e) = scene();
    set_expr(&mut doc, 0, e, PropKind::TranslationX, "value * value");
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert_eq!(
        key_value_in_active_clip(&doc, e, PropKind::TranslationX, 4.0, 1.0),
        Err(KeyRefusal::ExpressionDriven),
        "a non-linear formula has no single stored value: refuse (the 3rd probe strays)"
    );

    // Stacked: the same non-affinity through the blend.
    let (mut world, mut doc, e) = scene();
    set_expr(&mut doc, 0, e, PropKind::TranslationX, "value * value");
    doc.stack_mut().push(lane(0, LaneMode::Override, 1.0));
    apply_from_doc(&mut world, &mut doc, 1.0);
    assert_eq!(
        key_value_in_active_clip(&doc, e, PropKind::TranslationX, 4.0, 1.0),
        Err(KeyRefusal::ExpressionDriven),
        "stacked too: the composition is non-affine in the stored value"
    );
}

/// **Gate #12 (W6, C2) — the autokey mints NO phantom key on a PROP-LINKED channel.** Follower
/// drives its X by `Sprite.x` (a per-clip prop-link) on a full Override lane; Sprite is keyed to
/// 30. The apply composes Follower to 30 and PERSISTS it. The autokey diff reads that persisted
/// value, so `shown == world` and a still, paused scene keys NOTHING.
///
/// Why a prop-link and not a local expression: `shown_value`'s stacked re-derivation is
/// single-entity and has NO graph, so `Sprite.x` resolves to 0 there — `shown` would differ
/// from the world every frame. Mutation (`persisted_shown` returns `None`): the diff falls back
/// to that graph-less re-derivation, sees 0 != 30, and mints a phantom key, RED.
#[test]
#[allow(
    non_snake_case,
    reason = "the gate name mirrors the ADR-0152 W6 gate list"
)]
fn auto_key_mints_no_phantom_key_on_a_PROP_LINKED_channel() {
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
    // Clip 0: Sprite keyed X = 30, Follower drives X by `Sprite.x`. A full Override lane plays
    // it, so `shown_value` takes the STACKED branch where the prop-link needs the graph.
    doc.insert_key(
        sprite,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(30.0),
        Interp::Hold,
    );
    let ftgt = doc.bind(follower, PropKind::TranslationX);
    doc.set_clip_expr(0, ftgt, Some("Sprite.x".into()));
    doc.stack_mut().push(lane(0, LaneMode::Override, 1.0));

    apply_from_doc(&mut world, &mut doc, 1.0);
    let fx = x_of(&world, follower);
    assert!(
        (fx - 30.0).abs() < 1e-3,
        "the follower composed to Sprite.x = 30, got {fx}"
    );

    // The live pose IS what the apply wrote: the autokey must key nothing.
    let live = pose_x(fx);
    let plan = autokey_props(&doc, follower, 1.0, &live, &live, true, false);
    assert!(
        plan.is_empty(),
        "a prop-linked channel sitting on the composed value keys nothing (a phantom key means \
         `shown` was re-derived without the graph)"
    );
}

/// **Gate #13 (W6, C2) — a SKIPPED entity is left alone but READABLE by a prop-link.** Dragged
/// is keyed to 10 but the user displaced it to 42 (a gizmo drag), so the apply SKIPS it. Reader
/// drives its X by `Dragged.x`. The skip must leave Dragged's live 42 untouched AND a prop-link
/// must read that live 42 (the seed from the world), never Dragged's document 10.
///
/// Mutation (the compose loop ignores `skip`): Dragged is composed to its document 10,
/// overwriting the displaced 42 — it DRIFTS while paused, and the prop-link reads 10, RED.
#[test]
fn a_skipped_entity_is_left_alone_but_readable_by_a_prop_link() {
    let mut world = World::new();
    let dragged = world
        .spawn((Transform::default(), Name::new("Dragged")))
        .id()
        .to_bits();
    let reader = world
        .spawn((Transform::default(), Name::new("Reader")))
        .id()
        .to_bits();
    let mut doc = TimelineDoc::new();
    doc.insert_key(
        dragged,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(10.0), // Dragged's DOCUMENT value
        Interp::Hold,
    );
    let rtgt = doc.bind(reader, PropKind::TranslationX);
    doc.set_clip_expr(0, rtgt, Some("Dragged.x".into())); // Reader.x = Dragged.x

    // The user displaced Dragged to 42 (owns it this frame).
    world
        .get_mut::<Transform>(Entity::from_bits(dragged))
        .unwrap()
        .translation
        .x = 42.0;

    apply_from_doc_except(&mut world, &mut doc, 1.0, |bits| bits == dragged);

    assert!(
        (x_of(&world, dragged) - 42.0).abs() < 1e-3,
        "a skipped entity is left alone (its displaced 42, not the document 10); got {}",
        x_of(&world, dragged)
    );
    assert!(
        (x_of(&world, reader) - 42.0).abs() < 1e-3,
        "a prop-link reads the skipped source's LIVE pose (42), not its document 10; got {}",
        x_of(&world, reader)
    );
}
