//! **Locking a translation axis pins that axis** (Freeze Position X/Y).
//!
//! The sibling of `lock_rotation`: where that froze the angular DOF, this freezes
//! a linear one. A frozen axis is pinned by TWO independent mechanisms, and each
//! gets its own test (layered defenses need a gate per layer):
//!
//! 1. **The locked-mass bit** — rapier's `LockedAxes` zeroes the axis's inverse
//!    mass, so no FORCE can accelerate the body along it. Tested by pushing with a
//!    force (sideways gravity for X, normal gravity for Y).
//! 2. **The velocity drop** — rapier's `LockedAxes` does NOT project an
//!    explicitly-set initial velocity out of the integrator (it special-cases only
//!    rotation), so `spawn_body` zeroes the locked component itself, or a "frozen"
//!    body launched along the locked axis would drift forever. Tested by launching
//!    a locked body along its locked axis.
//!
//! Red-first and mutation-verified: dropping the `TRANSLATION_LOCKED_X` OR fails
//! (1); dropping the initial-velocity zeroing fails (2); ORing the whole
//! `TRANSLATION_LOCKED` instead of one axis fails the independence check.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

/// A ball with the given locks, initial velocity and gravity; its position after
/// half a second. Everything else is identical, so the lock is the only variable.
fn pose_after(lock_x: bool, lock_y: bool, linvel: [f32; 2], gravity: [f32; 2]) -> (f32, f32) {
    let mut w = PhysicsWorld::new();
    w.set_gravity(gravity[0], gravity[1]);
    let body = w.spawn_body(BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Ball { radius: 0.25 },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel,
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        offset: [0.0, 0.0],
        lock_x,
        lock_y,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
    });
    for _ in 0..30 {
        w.step();
    }
    let p = w.body_pose(body).expect("body exists").translation;
    (p.x, p.y)
}

#[test]
fn an_x_locked_body_ignores_a_sideways_force() {
    // Sideways gravity is a continuous force on X, no initial velocity. This tests
    // the LOCKED-MASS bit: a force must not accelerate a locked axis.
    let (free_x, _) = pose_after(false, false, [0.0, 0.0], [5.0, 0.0]);
    assert!(
        free_x > 0.5,
        "a free body under 5 m/s² sideways gravity should have moved past x=0.5 in \
         0.5 s, but it is at x={free_x} — the fixture no longer contains the phenomenon"
    );
    let (locked_x, _) = pose_after(true, false, [0.0, 0.0], [5.0, 0.0]);
    assert!(
        locked_x.abs() < 1e-4,
        "an X-locked body accelerated under a sideways force (x={locked_x}) — the \
         lock bit did not take (dropping the `TRANSLATION_LOCKED_X` OR reproduces this)"
    );
}

#[test]
fn a_y_locked_body_ignores_gravity() {
    // Normal gravity is a continuous force on Y, no initial velocity — the LOCKED-
    // MASS bit again, on the other axis.
    let (_, free_y) = pose_after(
        false,
        false,
        [0.0, 0.0],
        [0.0, PhysicsWorld::DEFAULT_GRAVITY_Y],
    );
    assert!(
        free_y < -0.5,
        "a free body should have fallen below y=-0.5 in 0.5 s under gravity, but it \
         is at y={free_y} — the fixture no longer contains the phenomenon"
    );
    let (_, locked_y) = pose_after(
        false,
        true,
        [0.0, 0.0],
        [0.0, PhysicsWorld::DEFAULT_GRAVITY_Y],
    );
    assert!(
        locked_y.abs() < 1e-4,
        "a Y-locked body still fell (y={locked_y}) — the lock bit did not take \
         (dropping the `TRANSLATION_LOCKED_Y` OR reproduces this)"
    );
}

#[test]
fn a_frozen_axis_drops_an_authored_launch_and_the_axes_are_independent() {
    // No gravity, launched at 3 m/s on X. A free body glides; an X-locked one must
    // drop the launch (rapier keeps the raw velocity, so `spawn_body` zeroes it).
    let (free_x, _) = pose_after(false, false, [3.0, 0.0], [0.0, 0.0]);
    assert!(
        free_x > 1.0,
        "a free body launched at 3 m/s should have slid past x=1 in 0.5 s, but it is \
         at x={free_x} — the fixture no longer contains the phenomenon"
    );
    let (locked_x, _) = pose_after(true, false, [3.0, 0.0], [0.0, 0.0]);
    assert!(
        locked_x.abs() < 1e-4,
        "an X-locked body kept its launch velocity and drifted (x={locked_x}) — the \
         locked axis did not drop the authored velocity (dropping the `if desc.lock_x \
         {{ 0.0 }}` in `spawn_body` reproduces exactly this)"
    );

    // ⚠️ The two axes are independent, tested with a FORCE on the free axis (not an
    // initial velocity): a Y-locked body under sideways gravity still accelerates on
    // X while staying pinned on Y. A force is what catches an over-broad lock (ORing
    // `TRANSLATION_LOCKED` instead of just `_Y` freezes X's mass too) — a launch
    // velocity would drift through the over-broad bit unnoticed, since rapier keeps
    // the raw velocity and only the per-axis flag drops it.
    let (x_of_y_locked, y_of_y_locked) = pose_after(
        false,
        true,
        [0.0, 0.0],
        [5.0, PhysicsWorld::DEFAULT_GRAVITY_Y],
    );
    assert!(
        x_of_y_locked > 0.5,
        "a Y-locked body did not accelerate on X under a sideways force (x={x_of_y_locked}) \
         — the lock froze more than the Y axis (ORing `TRANSLATION_LOCKED` reproduces this)"
    );
    assert!(
        y_of_y_locked.abs() < 1e-4,
        "the Y lock did not hold while X was free (y={y_of_y_locked})"
    );
}
