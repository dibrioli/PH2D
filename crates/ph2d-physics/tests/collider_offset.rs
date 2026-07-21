//! **A collider offset places the collider off the body centre** (W-Offset).
//!
//! `BodyDesc::offset` becomes the collider's translation relative to its rigid
//! body, so the shape a body collides with is no longer centred on it — the feet
//! of a character below its sprite, an off-centre hitbox. The sharpest test is
//! where a body comes to REST: a body whose collider sits well above its centre
//! settles with its centre far below the floor, because it is the COLLIDER that
//! lands, not the body origin.
//!
//! Red-first and mutation-verified: mutating `spawn_body`'s
//! `.translation(Vector2::new(desc.offset[0], desc.offset[1]))` to a zero vector
//! makes the offset body rest at the same height as the centred one, and the
//! "rests lower" assertion goes RED.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

/// Drop a ball (radius 0.5) from y=6 onto a floor at y=0 with the given collider
/// offset, and return where its CENTRE settles. `x` keeps the two drops apart.
fn rest_y_with_offset(x: f32, offset: [f32; 2]) -> f32 {
    let mut w = PhysicsWorld::new();
    // Wide static floor at y=0 (top ≈ 0.1).
    w.spawn_body(BodyDesc {
        body_type: RigidBodyType::Fixed,
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: 20.0,
            half_y: 0.1,
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    });
    let body = w.spawn_body(BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x,
        y: 6.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Ball { radius: 0.5 },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset,
    });
    for _ in 0..300 {
        w.step();
    }
    w.body_pose(body).expect("body exists").translation.y
}

#[test]
fn a_collider_offset_moves_where_the_body_rests() {
    // Centred: the ball's collider is on the body centre, so the centre rests at
    // floor + radius ≈ 0.1 + 0.5 = 0.6.
    let centred = rest_y_with_offset(-3.0, [0.0, 0.0]);
    assert!(
        (centred - 0.6).abs() < 0.05,
        "a centred ball should rest at y≈0.6, but it is at {centred}"
    );

    // Offset the collider 2 m ABOVE the centre: now the collider lands on the
    // floor while the body CENTRE sits 2 m below it (≈ 0.6 - 2 = -1.4). This is
    // the whole point — the body rests where its COLLIDER touches, not its origin.
    let raised_collider = rest_y_with_offset(3.0, [0.0, 2.0]);
    assert!(
        (raised_collider - (centred - 2.0)).abs() < 0.05,
        "a body whose collider is offset 2 m up should rest 2 m LOWER than a \
         centred one (≈{}), but it is at {raised_collider} — the offset did not \
         reach the collider (mutating `.translation(...)` to zero reproduces this)",
        centred - 2.0
    );
}
