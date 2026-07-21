//! **Dominance is a collision priority, orthogonal to mass** (W-Dominance).
//!
//! rapier treats the STRICTLY higher-dominance body as infinite mass to the lower
//! one, so it bulldozes through and is never pushed back. The sharpest test is the
//! one mass CANNOT reproduce: a LIGHT body with high dominance plows through a HEAVY
//! body with the default dominance. Under mass alone the light one would bounce off
//! the heavy one; dominance flips it.
//!
//! Red-first and mutation-verified: mutating `spawn_body` to drop `.dominance_group`
//! (always the default 0) makes the light mover an ordinary light body that bounces
//! off the heavy target, and the "plowed through" assertion goes RED. A readback
//! sibling pins the fold directly.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

fn ball(
    w: &mut PhysicsWorld,
    x: f32,
    vx: f32,
    mass_override: Option<f32>,
    dominance: i8,
) -> rapier2d::dynamics::RigidBodyHandle {
    w.spawn_body(BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x,
        y: 0.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Ball { radius: 0.25 },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [vx, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        offset: [0.0, 0.0],
        lock_x: false,
        lock_y: false,
        mass_override,
        dominance,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
    })
}

#[test]
fn a_light_high_dominance_body_plows_through_a_heavy_one() {
    // The mover is LIGHT (auto mass ≈ 0.2) but HIGH dominance; the target is HEAVY
    // (20 kg) and neutral dominance. Zero gravity — the collision is the only motion.
    fn mover_x_after(mover_dominance: i8) -> f32 {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let mover = ball(&mut w, -0.5, 3.0, None, mover_dominance);
        ball(&mut w, 0.5, 0.0, Some(20.0), 0);
        for _ in 0..60 {
            w.step();
        }
        w.body_pose(mover).expect("mover exists").translation.x
    }

    // High dominance: the light mover treats the heavy target as if IT had infinite
    // mass, so it plows through and travels well past the collision point.
    let dominant = mover_x_after(5);
    assert!(
        dominant > 1.5,
        "a light HIGH-dominance mover should plow through the heavy target, but it is \
         at x={dominant} — dominance is not overriding the mass (dropping `.dominance_group` \
         reproduces exactly this)"
    );
    // Neutral dominance: the SAME light mover bounces off the heavy target (mass
    // wins), stalling near the collision point. The contrast proves it is dominance,
    // not the scene, that let the light body win above.
    let neutral = mover_x_after(0);
    assert!(
        neutral < 1.0,
        "a light neutral-dominance mover should have bounced off the heavy target, but \
         it is at x={neutral} — the fixture no longer contains the phenomenon"
    );
}

#[test]
fn the_dominance_group_reaches_the_body() {
    // The fold, pinned directly: the body carries the authored dominance. Mutating
    // `spawn_body` to drop `.dominance_group` makes this read 0.
    let mut w = PhysicsWorld::new();
    let h = ball(&mut w, 0.0, 0.0, None, 7);
    assert_eq!(
        w.bodies().get(h).expect("body exists").dominance_group(),
        7,
        "the authored dominance did not reach the rigid body"
    );
    // A neutral body is byte-identical to before this existed (rapier's default 0).
    let n = ball(&mut w, 3.0, 0.0, None, 0);
    assert_eq!(w.bodies().get(n).expect("body exists").dominance_group(), 0);
}
