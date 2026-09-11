//! **Locking rotation pins a body's orientation** (Freeze Rotation, W-LockRot).
//!
//! A free body given an angular velocity keeps spinning; a rotation-locked one
//! ignores it entirely — there is no angular degree of freedom to turn. That is
//! the whole point of the flag (a character stays upright, a crate does not
//! roll), and it is the sharpest thing to test: a box spun at t=0 either turns
//! or it does not.
//!
//! Red-first and mutation-verified: mutating `spawn_body`'s `.locked_axes(...)`
//! to always pass `LockedAxes::empty()` makes the locked body spin too, and the
//! "did not rotate" assertion goes RED.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

/// Spawn a box at the origin with an initial spin of 5 rad/s and return its
/// orientation after half a second. `lock` toggles rotation lock; everything
/// else is identical. Gravity is zeroed so the only motion is the spin.
fn rotation_after_spin(lock: bool) -> f32 {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    let body = w.spawn_body(BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: 0.5,
            half_y: 0.5,
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        // A 5 rad/s spin: over 0.5 s a free body turns ~2.5 rad (well under π, so
        // the angle does not wrap), a locked one turns nothing.
        angvel: 5.0,
        ccd: false,
        lock_rotation: lock,
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
    for _ in 0..30 {
        w.step();
    }
    w.body_pose(body).expect("body exists").rotation.angle()
}

#[test]
fn a_free_body_spins_and_a_rotation_locked_one_does_not() {
    // Free: the initial spin turns it well past a radian.
    let free = rotation_after_spin(false);
    assert!(
        free.abs() > 1.0,
        "a free body given a 5 rad/s spin should have turned (|rotation| > 1), \
         but it is at {free} rad — if this is no longer true the fixture no \
         longer contains the phenomenon and the numbers need re-choosing"
    );

    // Locked: the angular DOF is gone, so the spin does nothing at all.
    let locked = rotation_after_spin(true);
    assert!(
        locked.abs() < 1e-4,
        "a rotation-locked body still rotated (|rotation| = {locked}) — the lock \
         did not take (mutating `.locked_axes(...)` to always `empty()` reproduces \
         exactly this)"
    );
}
