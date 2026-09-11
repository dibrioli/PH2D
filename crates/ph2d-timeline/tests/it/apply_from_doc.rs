//! W1.T3 — the document-driven apply pass, headless: bindings in a `TimelineDoc`
//! drive real entities' `Transform`, unbound props survive, empty tracks are
//! skipped, and a dead entity is flagged `missing` (never a silent no-op).

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Transform, World};
use ph2d_timeline::{PropKind, TimelineDoc, apply_from_doc};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

#[test]
fn doc_drives_bound_props_and_leaves_others() {
    let mut w = World::new();
    // Authored x=0, y=5. Only X is animated; Y must survive.
    let e = w
        .spawn(Transform::from_translation(Vec2::new(0.0, 5.0)))
        .id();
    let bits = e.to_bits();

    let mut doc = TimelineDoc::new();
    doc.insert_key(
        bits,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    doc.insert_key(
        bits,
        PropKind::TranslationX,
        s(2.0),
        AnimValue::Float(10.0),
        Interp::Hold,
    );

    // Midpoint of the 2 s ramp → x = 5.
    apply_from_doc(&mut w, &mut doc, 1.0);
    let xf = *w.get::<Transform>(e).unwrap();
    assert!((xf.translation.x - 5.0).abs() < 1e-5, "X animated to 5.0");
    assert!((xf.translation.y - 5.0).abs() < 1e-5, "unbound Y kept");
    assert!(
        !doc.binding_for(bits, PropKind::TranslationX)
            .unwrap()
            .missing
    );
}

#[test]
fn empty_track_does_not_clobber_property() {
    let mut w = World::new();
    let e = w
        .spawn(Transform::from_translation(Vec2::new(3.0, 0.0)))
        .id();
    let mut doc = TimelineDoc::new();
    // Bind but never key it: apply must leave the property untouched.
    doc.bind(e.to_bits(), PropKind::TranslationX);
    apply_from_doc(&mut w, &mut doc, 1.0);
    let xf = *w.get::<Transform>(e).unwrap();
    assert!(
        (xf.translation.x - 3.0).abs() < 1e-5,
        "empty track left X alone"
    );
}

#[test]
fn dead_entity_is_flagged_missing() {
    let mut w = World::new();
    let e = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
    let bits = e.to_bits();
    let mut doc = TimelineDoc::new();
    doc.insert_key(
        bits,
        PropKind::Rotation,
        s(0.0),
        AnimValue::Float(1.0),
        Interp::Hold,
    );
    w.despawn(e);

    apply_from_doc(&mut w, &mut doc, 0.0); // must not panic
    assert!(
        doc.binding_for(bits, PropKind::Rotation).unwrap().missing,
        "dead entity → missing badge, not a silent no-op"
    );
}

#[test]
fn two_entities_same_prop_stay_independent() {
    // The doc allocates distinct targets, so two sprites animating TranslationX
    // do not collide on one track.
    let mut w = World::new();
    let a = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
    let b = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
    let mut doc = TimelineDoc::new();
    doc.insert_key(
        a.to_bits(),
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(7.0),
        Interp::Hold,
    );
    doc.insert_key(
        b.to_bits(),
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(-2.0),
        Interp::Hold,
    );
    assert_ne!(
        doc.binding_for(a.to_bits(), PropKind::TranslationX)
            .unwrap()
            .target,
        doc.binding_for(b.to_bits(), PropKind::TranslationX)
            .unwrap()
            .target,
    );

    apply_from_doc(&mut w, &mut doc, 0.0);
    assert!((w.get::<Transform>(a).unwrap().translation.x - 7.0).abs() < 1e-5);
    assert!((w.get::<Transform>(b).unwrap().translation.x + 2.0).abs() < 1e-5);
}
