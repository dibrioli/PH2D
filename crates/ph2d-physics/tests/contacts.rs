//! Contact readback — who hit whom, where, and how hard (ADR-0131 W-Contacts).
//!
//! ⚠️ Two of these fixtures exist because of what the last two waves taught. The
//! near-miss gate uses a **ROUND** body: for a box, the shape and its bounding
//! volume are the same rectangle, so a fixture built from boxes cannot tell "touching"
//! from "nearly touching" — exactly how the force zone's overlap filter survived five
//! gates. And the impulse gate asserts a RATIO down a stack rather than a number: the
//! number is a function of the timestep and the fixture's mass, while "the bottom
//! contact carries four boxes and the top one carries one" is a fact about the scene
//! that physics fixes exactly.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyHandle, RigidBodyType, ShapeDesc};

fn desc(body_type: RigidBodyType, x: f32, y: f32, shape: ShapeDesc) -> BodyDesc {
    BodyDesc {
        body_type,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        offset: [0.0, 0.0],
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
    }
}

/// A wide static floor with its top face at y = 0.
fn floor(w: &mut PhysicsWorld) -> RigidBodyHandle {
    w.spawn_body(desc(
        RigidBodyType::Fixed,
        0.0,
        -0.5,
        ShapeDesc::Cuboid {
            half_x: 10.0,
            half_y: 0.5,
        },
    ))
}

fn ball(w: &mut PhysicsWorld, x: f32, y: f32) -> RigidBodyHandle {
    w.spawn_body(desc(
        RigidBodyType::Dynamic,
        x,
        y,
        ShapeDesc::Ball { radius: 0.25 },
    ))
}

#[test]
fn a_body_resting_on_the_floor_is_reported_as_one_pair_at_the_point_of_contact() {
    let mut w = PhysicsWorld::new();
    let f = floor(&mut w);
    let b = ball(&mut w, 3.0, 0.4);
    for _ in 0..120 {
        w.step();
    }

    let reports = w.contact_reports();
    assert_eq!(
        reports.len(),
        1,
        "a ball on a floor is ONE touching pair, got {reports:?}"
    );
    let r = reports[0];
    assert!(
        (r.body1 == f && r.body2 == b) || (r.body1 == b && r.body2 == f),
        "the report must name the two bodies that touch"
    );
    // The contact is under the ball, on the floor's face — not at either body's
    // centre, which is the answer a report that forgot to transform the point
    // would give (the floor's centre is (0, -0.5), the ball's is (3, ~0.25)).
    assert!(
        (r.point[0] - 3.0).abs() < 0.05 && r.point[1].abs() < 0.05,
        "the contact should be under the ball at the floor's face (3, 0), got {:?}",
        r.point
    );
    assert!(
        r.impulse > 0.0,
        "a body held up by the floor is pushing on it"
    );
}

#[test]
fn a_body_falling_freely_touches_nothing() {
    let mut w = PhysicsWorld::new();
    floor(&mut w);
    ball(&mut w, 0.0, 8.0);
    for _ in 0..10 {
        w.step();
    }
    assert!(
        w.contact_reports().is_empty(),
        "a body in mid-air touches nothing — reporting it would make 'contact' mean \
         'the solver is aware of you'"
    );
}

#[test]
fn a_near_miss_is_not_a_contact() {
    // ⚠️ ROUND bodies, deliberately. Two boxes side by side have shapes identical to
    // their bounding volumes, so a fixture of boxes cannot distinguish "the contact
    // graph knows about this pair" from "these two are touching" — the distinction
    // this gate exists for. Two circles whose AABBs overlap at the corners can be a
    // full 0.29 r apart.
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    // Diagonal offset of (0.40, 0.40): the AABBs overlap (each spans ±0.25, so the
    // x ranges [-0.25, 0.25] and [0.15, 0.65] share a strip), while the circles are
    // 0.566 apart — beyond the 0.5 sum of the radii. (The first draft used 0.35 and
    // the gate went red on its own arithmetic: 0.495 < 0.5, so they really were
    // touching. The product was right; the fixture was not.)
    let a = ball(&mut w, 0.0, 0.0);
    let b = ball(&mut w, 0.40, 0.40);
    for _ in 0..30 {
        w.step();
    }
    assert!(
        w.contact_reports().is_empty(),
        "two circles 0.566 apart (radius 0.25 each) are NOT touching, but their boxes \
         overlap — the report must follow the SHAPES"
    );
    // The positive control: nudge them together and the same scene reports the pair.
    // Without it, a `contact_reports` that always returned nothing would pass above.
    let mut w2 = PhysicsWorld::new();
    w2.set_gravity(0.0, 0.0);
    let c = ball(&mut w2, 0.0, 0.0);
    let d = ball(&mut w2, 0.30, 0.30);
    for _ in 0..30 {
        w2.step();
    }
    assert_eq!(
        w2.contact_reports().len(),
        1,
        "the control: at 0.424 apart the same two circles DO touch"
    );
    let _ = (a, b, c, d);
}

#[test]
fn the_bottom_of_a_stack_carries_more_load_than_the_top() {
    // ⚠️ **What the impulse actually IS, measured.** The first version of this gate
    // asked whether landing from 6 m reports a bigger impulse than resting. It does
    // NOT — measured, the two agree to seven digits (0.010032237 vs 0.010032236),
    // because `step` returns after the solver has already stopped the body: the impact
    // peak lives *between* the substeps and is gone by the time anyone can read it.
    //
    // What the number is instead is the **load this pair is carrying right now**, and
    // that is exactly and beautifully physical: in a stack of four identical boxes the
    // impulses come out 4 : 3 : 2 : 1 from the floor up, because the bottom contact is
    // holding four boxes and the top one is holding one. That is a fact about the
    // SCENE (the same at any timestep), which is what makes it the right oracle — and
    // it is the reading the overlay's spark size means.
    let mut w = PhysicsWorld::new();
    floor(&mut w);
    for i in 0..4 {
        w.spawn_body(desc(
            RigidBodyType::Dynamic,
            0.0,
            0.25 + i as f32 * 0.52,
            ShapeDesc::Cuboid {
                half_x: 0.25,
                half_y: 0.25,
            },
        ));
    }
    for _ in 0..400 {
        w.step();
    }

    let reports = w.contact_reports();
    assert_eq!(reports.len(), 4, "four boxes on a floor make four pairs");
    // Sorted by handle, and the bodies were spawned bottom-up, so report `i` is the
    // i-th joint from the floor: floor–box0, box0–box1, box1–box2, box2–box3.
    let top = reports[3].impulse;
    assert!(top > 0.0, "the top box is being held up by something");
    for (i, expected) in [4.0f32, 3.0, 2.0, 1.0].iter().enumerate() {
        let ratio = reports[i].impulse / top;
        assert!(
            (ratio - expected).abs() < 0.1,
            "the {i}-th contact from the floor carries {expected} boxes, so its impulse \
             should be {expected}x the top's — got {ratio:.3} ({} vs {top})",
            reports[i].impulse
        );
    }
}

#[test]
fn reading_the_contacts_does_not_move_the_world() {
    // ⚠️ The contract of the whole wave: this is a READ. Nothing in `step` calls it,
    // so installing it cannot move a body — which is also why the C9 harness gained
    // no bodies for this wave (there is nothing new on the deterministic path). The
    // hash is asked before and after a full read, on a world mid-collision where
    // there is the most to disturb.
    let mut w = PhysicsWorld::new();
    floor(&mut w);
    for i in 0..5 {
        ball(&mut w, i as f32 * 0.4 - 0.8, 1.0 + i as f32 * 0.6);
    }
    for _ in 0..90 {
        w.step();
    }
    let before = w.deterministic_hash();
    let reports = w.contact_reports();
    assert!(
        !reports.is_empty(),
        "the fixture must be mid-contact, or this proves nothing"
    );
    assert_eq!(
        before,
        w.deterministic_hash(),
        "reading the contacts changed the world"
    );
}

#[test]
fn a_sensor_overlap_is_not_a_contact() {
    // The two channels answer different questions and must not bleed: a sensor
    // produces an INTERSECTION and no contact (it exerts no force, so there is
    // nothing to report an impulse for), and this is the gate that keeps a future
    // "just report everything the narrow phase knows" from collapsing them.
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    w.spawn_body(BodyDesc {
        is_sensor: true,
        ..desc(
            RigidBodyType::Fixed,
            0.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 1.0,
                half_y: 1.0,
            },
        )
    });
    ball(&mut w, 0.0, 0.0);
    for _ in 0..30 {
        w.step();
    }
    assert!(
        w.contact_reports().is_empty(),
        "a body inside a SENSOR is overlapping it, not touching it"
    );
    assert_eq!(
        w.intersecting_body_pairs().len(),
        1,
        "the control: the same pair IS an intersection"
    );
}
