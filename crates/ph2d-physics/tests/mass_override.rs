//! **An explicit mass override replaces the density-derived mass** (Unity's manual
//! `Rigidbody2D.mass`, W-Mass).
//!
//! Two roads to one quantity: without an override the mass is `density × area`
//! (rapier computes it from the collider); with one, the mass IS the number, and
//! density is ignored. The sharpest test reads the body's computed mass straight
//! back from rapier — no collision dynamics to make it chaotic.
//!
//! Red-first and mutation-verified: mutating `spawn_body` to always take the
//! `.density(...)` branch makes the overridden body weigh its density-derived mass
//! instead of the authored one, and the `== 10.0` assertion goes RED. A behavioural
//! sibling proves the override actually MATTERS (a heavier body accelerates less
//! under the same force).

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

/// Spawn a ball (r=0.5, density=1) with the given mass override and return its
/// rapier-computed mass. `None` = auto (density × area).
fn mass_of(mass_override: Option<f32>) -> f32 {
    let mut w = PhysicsWorld::new();
    let h = w.spawn_body(BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: 0.0,
        y: 0.0,
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
        offset: [0.0, 0.0],
        lock_x: false,
        lock_y: false,
        mass_override,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
    });
    w.bodies().get(h).expect("body exists").mass()
}

#[test]
fn a_manual_mass_overrides_the_density_derived_mass() {
    // Auto: mass = density × area = 1.0 × π × 0.5² ≈ 0.785. This is the fixture's
    // own contained phenomenon — if it drifts, the numbers need re-choosing.
    let auto = mass_of(None);
    let expected_auto = std::f32::consts::PI * 0.25;
    assert!(
        (auto - expected_auto).abs() < 1e-3,
        "auto mass should be density×area ≈ {expected_auto}, but it is {auto}"
    );

    // Manual: the mass IS the authored number, regardless of density/area. Mutating
    // `spawn_body` to always `.density(...)` makes this weigh `auto` instead → RED.
    let manual = mass_of(Some(10.0));
    assert!(
        (manual - 10.0).abs() < 1e-4,
        "an explicit mass override did not take: the body weighs {manual}, not 10.0 \
         (mutating `spawn_body` to always take the `.density(...)` branch reproduces this)"
    );
    // And it is genuinely different from the auto mass, so the override is not a
    // no-op that happens to match.
    assert!(
        (manual - auto).abs() > 1.0,
        "the override and auto masses are indistinguishable ({manual} vs {auto})"
    );
}

/// Mass changes BEHAVIOUR, not just a readout — proven through a head-on collision.
/// Gravity cannot show it (all masses fall at `g`) and the wrapper has no impulse
/// API, but a collision does: momentum is conserved, so a heavy mover barely slows
/// while an equal-mass one splits its momentum with the target.
#[test]
fn a_heavier_body_dominates_a_head_on_collision() {
    fn mover_speed_after_collision(mover_mass: Option<f32>) -> f32 {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        // Mover: launched right at 3 m/s, its mass under test.
        let mover = w.spawn_body(BodyDesc {
            body_type: RigidBodyType::Dynamic,
            x: -1.5,
            y: 0.0,
            rotation: 0.0,
            density: 1.0,
            shape: ShapeDesc::Ball { radius: 0.25 },
            restitution: 0.0,
            friction: 0.5,
            layer: 0,
            is_sensor: false,
            gravity_scale: 1.0,
            linvel: [3.0, 0.0],
            angvel: 0.0,
            ccd: false,
            lock_rotation: false,
            offset: [0.0, 0.0],
            lock_x: false,
            lock_y: false,
            mass_override: mover_mass,
            dominance: 0,
            material: Default::default(),
            damping: None,
            one_way: false,
        });
        // Target: light (auto mass ≈ 0.196), at rest in the mover's path.
        w.spawn_body(BodyDesc {
            body_type: RigidBodyType::Dynamic,
            x: 1.5,
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
            damping: None,
            one_way: false,
        });
        for _ in 0..60 {
            w.step();
        }
        w.bodies().get(mover).expect("mover exists").linvel().x
    }

    // Heavy mover (20 kg) plows through the light target and keeps most of its
    // speed (inelastic momentum: 20·3/(20+0.196) ≈ 2.97 m/s).
    let heavy = mover_speed_after_collision(Some(20.0));
    assert!(
        heavy > 2.0,
        "a heavy mover should keep most of its speed through a light target, but it \
         is at {heavy} m/s — mass is not affecting the collision"
    );
    // Auto-mass mover (≈0.196 kg, equal to the target): the inelastic collision
    // splits the momentum, so it ends near 1.5 m/s. The contrast proves the heavy
    // result above is the override's doing, not the scene.
    let light = mover_speed_after_collision(None);
    assert!(
        light < 2.0,
        "an equal-mass mover should have slowed well below 2 m/s in the collision, \
         but it is at {light} m/s — the fixture no longer contains the phenomenon"
    );
}
