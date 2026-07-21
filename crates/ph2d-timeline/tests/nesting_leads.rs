//! **A container strip rides the same speed, leads and loop seam as a clip strip.**
//!
//! Every lead/seam gate in this suite drives strips whose source is a CLIP; the eval
//! could grow a `match` arm that forgets `StripSource::Container` and all of them would
//! stay green. This file drives the composition through the REAL apply (the
//! `clip_stack_eval.rs` doctrine): a container of 3 clips in 2 lanes, instanced three
//! times — the middle one stretched (`add_strip_to` derives `speed = slice/span`), the
//! gaps crossed by `lead_in`, the end crossing the LOOP seam by `lead_out`.
//!
//! It is also the `PH2D_NEST_SMOKE=2` demo scene, pinned: if these facts stop holding,
//! the smoke narrates a product that no longer exists.
//!
//! Mutation (2026-07-20): forcing the stretched strip's `speed` back to `1.0` bleeds
//! with the right diagnosis ("slow instance at its apex … got y=0").

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{PropKind, StackHost, StripSource, TimelineDoc, apply_from_doc};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn key(doc: &mut TimelineDoc, e: u64, prop: PropKind, t: f64, v: f32) {
    doc.upsert_key(e, prop, s(t), AnimValue::Float(v), Interp::Linear);
}

fn pose(world: &World, e: u64) -> (f32, f32) {
    let tr = world.get::<Transform>(Entity::from_bits(e)).unwrap();
    (tr.translation.x, tr.translation.y)
}

#[test]
fn a_container_strip_rides_the_same_speed_leads_and_loop_seam() {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();
    let mut doc = TimelineDoc::new();

    // Exactly the smoke's construction.
    doc.rename_clip(0, "Rise".to_string());
    key(&mut doc, e, PropKind::TranslationY, 0.0, 0.0);
    key(&mut doc, e, PropKind::TranslationY, 1.1, 1.2);
    let fall = doc.add_clip("Fall".to_string());
    doc.set_active(fall);
    key(&mut doc, e, PropKind::TranslationY, 0.0, 1.2);
    key(&mut doc, e, PropKind::TranslationY, 0.9, 0.0);
    key(&mut doc, e, PropKind::TranslationY, 1.0, 0.18);
    key(&mut doc, e, PropKind::TranslationY, 1.1, 0.0);
    let drift = doc.add_clip("Drift".to_string());
    doc.set_active(drift);
    key(&mut doc, e, PropKind::TranslationX, 0.0, -1.5);
    key(&mut doc, e, PropKind::TranslationX, 2.0, 1.5);
    doc.set_active(0);

    let jump = doc.add_container("Jump".to_string());
    let host = StackHost::Container(jump);
    let clip_of = |i: usize| StripSource::Clip(u16::try_from(i).expect("cabe"));
    let body = doc.add_lane_in(host, "Body".to_string()).unwrap();
    doc.add_strip_to(host, body, clip_of(0), 0.0, 1.1).unwrap();
    doc.add_strip_to(host, body, clip_of(fall), 0.9, 2.0).unwrap();
    let side = doc.add_lane_in(host, "Drift".to_string()).unwrap();
    doc.add_strip_to(host, side, clip_of(drift), 0.0, 2.0).unwrap();

    let lane = doc.add_lane("Jumps".to_string()).unwrap();
    let src = StripSource::Container(u16::try_from(jump).expect("cabe"));
    doc.add_strip_to(StackHost::Document, lane, src, 0.0, 2.0).unwrap();
    let slow = doc.add_strip_to(StackHost::Document, lane, src, 3.0, 7.0).unwrap();
    let last = doc.add_strip_to(StackHost::Document, lane, src, 8.0, 10.0).unwrap();
    assert!(
        (doc.strip_in(StackHost::Document, lane, slow).unwrap().speed - 0.5).abs() < 1e-9,
        "esticar a strip deriva speed 0.5"
    );
    doc.strip_in_mut(StackHost::Document, lane, slow).unwrap().lead_in = 0.6;
    {
        let s = doc.strip_in_mut(StackHost::Document, lane, last).unwrap();
        s.lead_in = 0.6;
        s.lead_out = 0.5;
    }
    doc.set_active_loop_for(false, Some((0.0, 10.6)));

    // Fact 1: the jump plays — start pose, apex (crossfade region), landing.
    apply_from_doc(&mut world, &mut doc, 0.0);
    let (x, y) = pose(&world, e);
    assert!((x + 1.5).abs() < 1e-3 && y.abs() < 1e-3, "start pose, got ({x},{y})");
    apply_from_doc(&mut world, &mut doc, 1.0);
    let (_, y) = pose(&world, e);
    assert!(y > 1.0, "apex through the interior crossfade, got y={y}");

    // Fact 2: the middle instance is SLOW MOTION — t=5.0 is 2 s in, container time 1.0.
    apply_from_doc(&mut world, &mut doc, 5.0);
    let (x, y) = pose(&world, e);
    assert!(y > 1.0, "slow instance at its apex after 2 s of scene time, got y={y}");
    assert!(x.abs() < 0.1, "drift at its midpoint, got x={x}");

    // Fact 3: the gap holds, then the lead-in travels to the next start.
    apply_from_doc(&mut world, &mut doc, 2.2);
    let (x, _) = pose(&world, e);
    assert!((x - 1.5).abs() < 1e-3, "held at instance A's end pose, got x={x}");
    apply_from_doc(&mut world, &mut doc, 2.7);
    let (x, _) = pose(&world, e);
    assert!(x > -1.5 + 0.05 && x < 1.5 - 0.05, "travelling across the gap, got x={x}");

    // Fact 4: the last strip's lead-out crosses the LOOP seam back to the first pose.
    apply_from_doc(&mut world, &mut doc, 10.25);
    let (x, _) = pose(&world, e);
    assert!(x > -1.5 + 0.05 && x < 1.5 - 0.05, "mid seam crossing, got x={x}");
    apply_from_doc(&mut world, &mut doc, 10.55);
    let (x, y) = pose(&world, e);
    assert!((x + 1.5).abs() < 0.05 && y.abs() < 0.05, "arrived at the seam target, got ({x},{y})");
}
