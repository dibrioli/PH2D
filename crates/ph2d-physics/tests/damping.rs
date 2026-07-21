//! **A per-body damping override decays that body's own velocities.**
//!
//! Unity's `Rigidbody2D` drag / Godot's `damp` + `damp_mode`, per body. Two knobs
//! (linear + angular) and a mode: `Combine` ADDS to the world's `BodyDefaults` drag,
//! `Replace` IGNORES it. With the default global drag of `0.0` the modes coincide, so
//! the mode is proven under an authored world drag.
//!
//! Red-first and mutation-verified: dropping the `apply_damping_override` call in
//! `spawn_body` leaves every body on the global drag, so a damped body then slides
//! and spins exactly as far as an undamped one and the contrast assertions fail.

use ph2d_physics::{BodyDefaults, BodyDesc, DampingDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

/// A ball launched right at 5 m/s and spinning at 10 rad/s in ZERO gravity (so
/// damping is the only thing that can slow it), with an optional damping override.
/// Returns `(x, |angvel|)` after 1 s.
fn slide_and_spin(global: BodyDefaults, damping: Option<DampingDesc>) -> (f32, f32) {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    w.set_body_defaults(global);
    let h = w.spawn_body(BodyDesc {
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
        linvel: [5.0, 0.0],
        angvel: 10.0,
        ccd: false,
        lock_rotation: false,
        offset: [0.0, 0.0],
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping,
        one_way: false,
    });
    for _ in 0..60 {
        w.step();
    }
    let b = w.bodies().get(h).expect("body exists");
    (b.position().translation.x, b.angvel().abs())
}

fn combine(linear: f32, angular: f32) -> Option<DampingDesc> {
    Some(DampingDesc {
        linear,
        angular,
        replace: false,
    })
}

#[test]
fn linear_damping_slows_a_slide_and_angular_damping_slows_a_spin() {
    // No world drag: the mode is irrelevant, the override is the only source.
    let (x_undamped, spin_undamped) = slide_and_spin(BodyDefaults::rapier(), None);
    let (x_damped, spin_damped) = slide_and_spin(BodyDefaults::rapier(), combine(2.0, 2.0));

    // Undamped in vacuum keeps almost all its speed (5 m/s → ~5 m in 1 s); the damped
    // one is dragged to well under half that. Dropping the spawn apply makes the
    // "damped" body identical to the undamped one and collapses this gap.
    assert!(
        x_damped < x_undamped * 0.6,
        "a linearly-damped ball should slide far less than an undamped one, but \
         damped={x_damped} vs undamped={x_undamped} — the override is not reaching \
         the body (was `apply_damping_override` dropped?)"
    );
    // Nothing decays a spin without angular damping (default 0), so the undamped ball
    // keeps ~all of its 10 rad/s; the damped one is dragged well down.
    assert!(
        spin_undamped > 9.0,
        "sanity: with no angular damping the spin should barely decay (got {spin_undamped})"
    );
    assert!(
        spin_damped < spin_undamped * 0.5,
        "an angularly-damped ball should lose most of its spin, but damped={spin_damped} \
         vs undamped={spin_undamped}"
    );
}

#[test]
fn replace_ignores_the_world_drag_while_combine_adds_to_it() {
    // A thick world drag. A Combine override of 0 rides ON TOP of it (effective = the
    // global), so the ball is dragged down; a Replace override of 0 IGNORES it
    // (effective = 0), so the ball keeps sliding. The mode is the only difference.
    let global = BodyDefaults {
        linear_damping: 3.0,
        ..BodyDefaults::rapier()
    };
    let (x_combine, _) = slide_and_spin(global, combine(0.0, 0.0));
    let (x_replace, _) = slide_and_spin(
        global,
        Some(DampingDesc {
            linear: 0.0,
            angular: 0.0,
            replace: true,
        }),
    );

    // Replace(0) keeps almost all of the 5 m; Combine(0) is dragged to a fraction.
    // Mutating `apply_damping_override` to always combine makes Replace behave like
    // Combine and this gap collapses.
    assert!(
        x_replace > x_combine * 2.0,
        "a Replace(0) override should ignore the world drag and slide far past a \
         Combine(0) one, but replace={x_replace} vs combine={x_combine} — the mode is \
         not honoured"
    );
}

#[test]
fn the_damping_reaches_the_body() {
    // Pinned directly: a Replace override sets the body's damping outright. Mutating
    // `spawn_body` to skip the apply reads back the global default (0), not 1.5/0.75.
    let mut w = PhysicsWorld::new();
    let h = w.spawn_body(BodyDesc {
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
        damping: Some(DampingDesc {
            linear: 1.5,
            angular: 0.75,
            replace: true,
        }),
        one_way: false,
    });
    let b = w.bodies().get(h).expect("body exists");
    assert_eq!(
        b.linear_damping(),
        1.5,
        "the authored linear damping did not reach the body"
    );
    assert_eq!(
        b.angular_damping(),
        0.75,
        "the authored angular damping did not reach the body"
    );
}
