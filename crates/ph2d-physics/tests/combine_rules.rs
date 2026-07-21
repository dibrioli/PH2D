//! **A collider's material combine rule decides whose bounce wins.**
//!
//! rapier resolves a contact by combining the two colliders' restitution with
//! `rule1.max(rule2)` over the enum order `Average < Min < Multiply < Max`, so a
//! superball (restitution 1.0) set to `Max` bounces off ANY floor — even a dead
//! one (restitution 0.0) — while the default `Average` returns it to only a
//! quarter of its drop height. This is the whole reason to expose the rule: with
//! `Average`, an artist who sets Bounce = 1.0 on a ball and drops it on a normal
//! floor gets a feeble bounce and no way to fix it from the ball alone.
//!
//! Red-first and mutation-verified: dropping `.restitution_combine_rule(...)` in
//! `spawn_body` falls back to rapier's default `Average`, so the `Max` ball then
//! rebounds no higher than the `Average` one and the contrast assertion fails.

use ph2d_physics::{BodyDesc, CoefficientCombineRule, CombineRules, PhysicsWorld, ShapeDesc};

/// Drop a bouncy ball (restitution 1.0) onto a plain floor whose own restitution
/// is the default 0.0, with the given restitution COMBINE rule, and return the
/// peak upward speed the ball reaches after the bounce.
///
/// The peak upward velocity is the rebound speed, which scales with the
/// *effective* restitution the combine rule produces: `Max` → ~the impact speed,
/// `Average` → ~half of it, `Min` → ~zero. It is a clean oracle because before
/// the bounce the vertical velocity is negative (falling) and only the bounce
/// makes it positive.
fn peak_rebound_speed(rule: CoefficientCombineRule) -> f32 {
    let mut w = PhysicsWorld::new();
    // Plain static floor — DEFAULT restitution 0.0 and DEFAULT (Average) combine.
    // The ball's rule is the only thing that varies, and because rapier takes the
    // higher-priority of the two, the ball alone decides the bounce.
    w.add_static_cuboid(0.0, 0.0, 5.0, 0.1);
    let ball = w.spawn_body(BodyDesc {
        body_type: ph2d_physics::RigidBodyType::Dynamic,
        x: 0.0,
        y: 3.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Ball { radius: 0.25 },
        // A perfect superball; the floor is dead. Under `Average` the pair is 0.5,
        // under `Max` it is 1.0, under `Min` it is 0.0 — that spread is the test.
        restitution: 1.0,
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
        material: CombineRules {
            restitution: rule,
            friction: CoefficientCombineRule::Average,
        },
        damping: None,
        one_way: false,
    });
    // 2 s @ 60 Hz: long enough to fall from 3 m, bounce, and reach the rebound peak.
    let mut peak_up = 0.0f32;
    for _ in 0..120 {
        w.step();
        let vy = w.bodies().get(ball).expect("ball exists").linvel().y;
        peak_up = peak_up.max(vy.max(0.0));
    }
    peak_up
}

#[test]
fn max_combine_bounces_off_a_dead_floor_and_min_stays_dead() {
    let max = peak_rebound_speed(CoefficientCombineRule::Max);
    let average = peak_rebound_speed(CoefficientCombineRule::Average);
    let min = peak_rebound_speed(CoefficientCombineRule::Min);

    // Max returns nearly all the impact speed; Average returns about half. The
    // 1.5× margin is comfortably inside the 2× the physics predicts, and it is
    // exactly what a dropped `.restitution_combine_rule` erases (the `Max` ball
    // would fall back to `Average` and this contrast would collapse).
    assert!(
        max > average * 1.5,
        "a Max-combine superball should rebound far faster than an Average one off \
         the same dead floor, but max={max} vs average={average} — the combine rule \
         is not reaching the collider (did `.restitution_combine_rule` get dropped?)"
    );
    // Average clearly bounces; Min is dead (the smaller of 1.0 and 0.0 is 0.0), so
    // it barely leaves the floor. The gap proves the rule is honoured in BOTH
    // directions, not just that `Max` is large.
    assert!(
        average > min + 1.0,
        "an Average-combine ball should clearly out-bounce a Min-combine one \
         (average={average} vs min={min})"
    );
    assert!(
        min < 0.5,
        "a Min-combine ball on a dead floor should not bounce (min={min})"
    );
}

#[test]
fn the_combine_rule_reaches_the_collider() {
    // The fold pinned directly: the body's collider carries the authored rules.
    // Mutating `spawn_body` to drop either `_combine_rule` call makes this read
    // back `Average` and fail.
    let mut w = PhysicsWorld::new();
    let h = w.spawn_body(BodyDesc {
        body_type: ph2d_physics::RigidBodyType::Dynamic,
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Ball { radius: 0.25 },
        restitution: 0.5,
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
        material: CombineRules {
            restitution: CoefficientCombineRule::Max,
            friction: CoefficientCombineRule::Min,
        },
        damping: None,
        one_way: false,
    });
    // The collider attached to this body is the last one inserted; find it.
    let body = w.bodies().get(h).expect("body exists");
    let col_handle = body.colliders()[0];
    let col = w.colliders().get(col_handle).expect("collider exists");
    assert_eq!(
        col.restitution_combine_rule(),
        CoefficientCombineRule::Max,
        "the authored restitution combine rule did not reach the collider"
    );
    assert_eq!(
        col.friction_combine_rule(),
        CoefficientCombineRule::Min,
        "the authored friction combine rule did not reach the collider"
    );
}
