//! `intersecting_body_pairs` reports a sensor overlap — ADR-0131 W7.
//!
//! The low-level half of the trigger primitive: a sensor collider passes
//! through but the narrow phase records its overlaps, and this reads them back
//! as body pairs. The ECS bridge turns those into a trigger state; here we prove
//! the wrapper reports them at all, and that a solid-only pair reports nothing.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

fn body(w: &mut PhysicsWorld, y: f32, body_type: RigidBodyType, is_sensor: bool) {
    w.spawn_body(BodyDesc {
        body_type,
        x: 0.0,
        y,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: 1.0,
            half_y: 1.0,
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor,
    });
}

/// A sensor overlapping a dynamic body is reported as one pair after a step.
/// Mutation-tested: dropping `.sensor(desc.is_sensor)` in `spawn_body` makes the
/// pair a solid contact instead of an intersection, and the pair count drops to
/// zero.
#[test]
fn a_sensor_overlap_is_reported_as_a_body_pair() {
    let mut w = PhysicsWorld::new();
    body(&mut w, 0.0, RigidBodyType::Fixed, true); // a static sensor at origin
    body(&mut w, 0.0, RigidBodyType::Dynamic, false); // a dynamic body inside it
    w.step();

    let pairs = w.intersecting_body_pairs();
    assert_eq!(
        pairs.len(),
        1,
        "a sensor overlapping a body should report exactly one pair, got {pairs:?}"
    );
}

/// Two SOLID overlapping bodies report NO intersection pair — a solid pair is a
/// contact, never an intersection. The control that makes the test above about
/// sensors rather than about "any overlap".
#[test]
fn a_solid_overlap_reports_no_pair() {
    let mut w = PhysicsWorld::new();
    body(&mut w, 0.0, RigidBodyType::Fixed, false);
    body(&mut w, 0.0, RigidBodyType::Dynamic, false);
    w.step();

    assert!(
        w.intersecting_body_pairs().is_empty(),
        "two solid bodies reported an intersection — that should be a contact"
    );
}
