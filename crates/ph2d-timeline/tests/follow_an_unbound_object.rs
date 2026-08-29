//! **`Follow` follows an object the artist merely PLACED.**
//!
//! ⚠️ Red-first against a report: *"Follow e outros da categoria ruins, não seguem o
//! objeto referido"* (Enio, 2026-07-29). The prop-link machinery needs two lookups —
//! a name to an entity, and an `(entity, prop)` to a value — and BOTH were built
//! exclusively from `doc.bindings()`. So a link could only read a property the
//! timeline already animated, and everything else resolved to the evaluator's total
//! contract: **0.0**. That does not read as *"the link is dead"*; it reads as *"my
//! object jumped to the origin and froze"*, which is exactly what was reported.
//!
//! The measurement that pointed at the cause was the near-miss: a source **bound** on
//! the property with **zero keys** already worked. So the missing ingredient was never
//! a track — it was a BINDING, a document fact with no connection to *"be where that
//! is"* in the artist's head.
//!
//! ⚠️ The oracle is the WORLD after `apply_from_doc` — the pose the artist sees — and
//! never the contents of the link map, which is where the old gates all looked.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, World};
use ph2d_timeline::{PropKind, TimelineDoc, apply_from_doc};

/// A follower at the origin and a source parked at `src_x`. NEITHER is bound.
fn scene(src_x: f32) -> (World, Entity, Entity) {
    let mut w = World::new();
    let follower = w
        .spawn((
            Transform::from_translation(Vec2::ZERO),
            Name::new("Follower"),
        ))
        .id();
    let source = w
        .spawn((
            Transform::from_translation(Vec2::new(src_x, 0.0)),
            Name::new("Ball"),
        ))
        .id();
    (w, follower, source)
}

fn follow(doc: &mut TimelineDoc, e: Entity, src: &str) {
    let tgt = doc.bind(e.to_bits(), PropKind::TranslationX);
    doc.bindings_mut()
        .iter_mut()
        .find(|b| b.target == tgt)
        .expect("just bound")
        .expr = Some(src.to_string());
}

fn x_after_apply(w: &mut World, doc: &mut TimelineDoc, e: Entity) -> f32 {
    apply_from_doc(w, doc, 0.7);
    w.get::<Transform>(e).unwrap().translation.x
}

/// The report, and the number it used to produce: **0.0000**, now **7.0000**.
#[test]
fn a_link_reads_a_source_the_timeline_does_not_animate() {
    let (mut w, follower, _src) = scene(7.0);
    let mut doc = TimelineDoc::default();
    follow(&mut doc, follower, "Ball.x*1 + 0");
    let got = x_after_apply(&mut w, &mut doc, follower);
    assert!(
        (got - 7.0).abs() < 1e-4,
        "a merely-placed source must be readable; got {got} (0.0 is the old bug — the \
         follower teleported to the origin)"
    );
}

/// **A source moved by ANYTHING is followed** — the gizmo, physics, a script. The link
/// reads the world, so it does not care who wrote the pose.
#[test]
fn the_link_follows_the_source_wherever_something_else_puts_it() {
    let (mut w, follower, src) = scene(1.0);
    let mut doc = TimelineDoc::default();
    follow(&mut doc, follower, "Ball.x*1 + 0");
    let mut seen = vec![];
    for x in [1.0_f32, -3.5, 12.25] {
        w.get_mut::<Transform>(src).unwrap().translation.x = x;
        seen.push(x_after_apply(&mut w, &mut doc, follower));
    }
    assert!(
        seen.iter()
            .zip([1.0_f32, -3.5, 12.25])
            .all(|(got, want)| (got - want).abs() < 1e-4),
        "the follower must track the source's live pose; got {seen:?}"
    );
}

/// **The spelling the panel itself uses resolves.**
///
/// ⚠️ `translation_x` is `PropKind::i18n_suffix` — the name this enum gives the
/// property, and the key the panel's own label is looked up by. It parsed fine and
/// resolved to **0.0**, so a link typed in the vocabulary the UI teaches was silently
/// dead. All three spellings must agree.
#[test]
fn every_spelling_of_a_property_reaches_the_same_value() {
    for tail in ["x", "tx", "translationx", "translation_x", "TRANSLATION_X"] {
        let (mut w, follower, _) = scene(4.0);
        let mut doc = TimelineDoc::default();
        follow(&mut doc, follower, &format!("Ball.{tail}*1 + 0"));
        let got = x_after_apply(&mut w, &mut doc, follower);
        assert!(
            (got - 4.0).abs() < 1e-4,
            "`Ball.{tail}` must resolve; got {got}"
        );
    }
}

/// A BOUND source still wins, and still reads its COMPOSED value — the widening must
/// not demote what already worked.
#[test]
fn a_bound_source_still_composes_before_its_reader() {
    use ph2d_anim::{AnimValue, Interp, RationalTime};
    let (mut w, follower, src) = scene(0.0);
    let mut doc = TimelineDoc::default();
    // The source is keyed 0 -> 10 over a second; at t = 0.7 it composes to 7.
    let s = RationalTime::from_seconds;
    doc.insert_key(
        src.to_bits(),
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    doc.insert_key(
        src.to_bits(),
        PropKind::TranslationX,
        s(1.0),
        AnimValue::Float(10.0),
        Interp::Linear,
    );
    follow(&mut doc, follower, "Ball.x*1 + 0");
    let got = x_after_apply(&mut w, &mut doc, follower);
    assert!(
        (got - 7.0).abs() < 1e-4,
        "the reader must see the source's COMPOSED value this frame, not last frame's \
         world; got {got}"
    );
}

/// **An ambiguous name is answered by a rule, not by the query's iteration order.**
///
/// ⚠️ Widening the map to the whole scene makes duplicate `Name`s user-visible, and
/// *"whichever the query yielded first"* is not an answer — the order a `bevy_ecs` query
/// walks its tables is an implementation detail, and a link that changes object when it
/// changes is a link nobody can reason about. The rule is the **lowest entity bits**:
/// arbitrary, but a function of the SCENE.
///
/// ⚠️ The fixture separates the two implementations, and the measurement is what told me
/// how. I assumed `to_bits` put the index in the low half ascending, and built an
/// elaborate despawn/respawn to overtake it — under which the two rules still AGREED and
/// the mutation survived. Measured, `bevy_ecs` gives a LATER spawn a LOWER `to_bits`
/// (`4294967295` then `4294967294`) while the table walk visits it SECOND. So a plain pair
/// is all it takes: first-in-the-walk is `x = 3`, lowest-bits is `x = 9`.
#[test]
fn a_duplicate_name_is_resolved_by_the_rule_not_by_the_query_order() {
    let mut w = World::new();
    let follower = w
        .spawn((Transform::from_translation(Vec2::ZERO), Name::new("F")))
        .id();
    let walked_first = w
        .spawn((
            Transform::from_translation(Vec2::new(3.0, 0.0)),
            Name::new("Twin"),
        ))
        .id();
    let lowest_bits = w
        .spawn((
            Transform::from_translation(Vec2::new(9.0, 0.0)),
            Name::new("Twin"),
        ))
        .id();
    assert!(
        lowest_bits.to_bits() < walked_first.to_bits(),
        "the fixture depends on this and it is measured, not assumed: a later spawn must          carry the lower bits ({} vs {})",
        lowest_bits.to_bits(),
        walked_first.to_bits()
    );

    let mut doc = TimelineDoc::default();
    follow(&mut doc, follower, "Twin.x*1 + 0");
    let got = x_after_apply(&mut w, &mut doc, follower);
    assert!(
        (got - 9.0).abs() < 1e-4,
        "the LOWEST-BITS twin (x = 9) must win, not the one the table walk reaches first \
         (x = 3); got {got}"
    );
    for _ in 0..8 {
        assert!(
            (x_after_apply(&mut w, &mut doc, follower) - got).abs() < 1e-6,
            "an ambiguous link must be stable across frames"
        );
    }
}

/// A name that resolves to NOTHING is still 0 — the total contract. Pinned so the
/// widening cannot be mistaken for "any identifier now means something".
#[test]
fn an_unknown_name_is_still_zero() {
    let (mut w, follower, _) = scene(7.0);
    let mut doc = TimelineDoc::default();
    follow(&mut doc, follower, "NoSuchObject.x*1 + 0");
    let got = x_after_apply(&mut w, &mut doc, follower);
    assert!(got.abs() < 1e-6, "an unresolved link stays 0.0; got {got}");
}

/// **Every property answers to the name the PANEL shows it under.**
///
/// ⚠️ This is the cross-check between two tables that had drifted:
/// `PropKind::i18n_suffix` is the key the panel's label is looked up by, and
/// `from_expr_name` is what a typed link resolves through. `translation_x` was in the
/// first and not the second, so a link typed in the vocabulary the UI teaches parsed
/// cleanly and resolved to 0.0. Checking the tables against EACH OTHER — rather than
/// listing spellings by hand — is what makes a kind added with a label but no spelling
/// impossible to ship.
///
/// `TimeRemap` is the one exception and it is deliberate: it is the timeline's meta-clock,
/// not a scene value a link may read.
#[test]
fn every_prop_answers_to_the_name_the_panel_shows_it_under() {
    let mut checked = 0;
    for i in 0..16u64 {
        let Some(k) = PropKind::from_target(ph2d_timeline::AnimTarget::new(i)) else {
            continue;
        };
        if k == PropKind::TimeRemap {
            assert!(
                PropKind::from_expr_name(k.i18n_suffix()).is_none(),
                "TimeRemap must stay unreadable by a link"
            );
            continue;
        }
        assert_eq!(
            PropKind::from_expr_name(k.i18n_suffix()),
            Some(k),
            "`{}` is the name the panel shows {k:?} under, so a link must accept it",
            k.i18n_suffix()
        );
        checked += 1;
    }
    assert!(checked >= 8, "the sweep must cover the enum; saw {checked}");
}
