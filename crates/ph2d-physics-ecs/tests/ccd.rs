//! The bridge folds the optional `Ccd` MARKER into the sim, and a rewind
//! RE-ARMS it (ADR-0131 W-CCD).
//!
//! `ph2d-physics` proves `BodyDesc::ccd` reaches the solver and stops a tunnel.
//! This is the ECS half: (1) a fast body carrying the marker is stopped by a
//! thin wall while an identical one WITHOUT it tunnels clean through, and (2)
//! after scrubbing the clock back to t=0 and replaying, the marked body is still
//! stopped — which is the entire reason the flag rides the `BodyDesc` the world
//! rebuilds from, rather than being read once and lost on scrub.
//!
//! Gravity is zeroed so the horizontal launch is the only motion, and each ball
//! is aimed straight at its own thin wall (two lanes, so the balls never meet).
//! The launch itself reuses `InitialVelocity` (W9) — this gate is about whether
//! the `Ccd` marker changes the OUTCOME, not about the launch.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Ccd, Collider, ColliderShape, InitialVelocity, PhysicsBridge, PhysicsSettings,
    RigidBody,
};

/// A 0.04 m-thick, 2 m-tall static wall centred at `(0, y)`.
fn wall(sim: &mut SimWorld, y: f32) {
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.02,
                half_y: 1.0,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, y)),
    ));
}

/// A small ball one metre left of the wall in lane `y`, launched right at
/// 200 m/s. `ccd` attaches the `Ccd` marker (its presence is the flag).
fn ball(sim: &mut SimWorld, y: f32, ccd: bool) -> Entity {
    let base = (
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.05 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-1.0, y)),
        InitialVelocity {
            linvel: [200.0, 0.0],
            angvel: 0.0,
        },
    );
    if ccd {
        sim.world_mut().spawn((base, Ccd)).id()
    } else {
        sim.world_mut().spawn(base).id()
    }
}

fn zero_gravity() -> PhysicsSettings {
    PhysicsSettings {
        gravity_y: 0.0,
        ..Default::default()
    }
}

fn x_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().translation.x
}

#[test]
fn the_bridge_makes_a_marked_body_continuous_and_a_rewind_re_arms_it() {
    let mut sim = SimWorld::new();
    wall(&mut sim, 0.0);
    wall(&mut sim, 3.0);
    let continuous = ball(&mut sim, 0.0, true);
    let discrete = ball(&mut sim, 3.0, false);

    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    for tick in 1..=30u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let continuous_x1 = x_of(&sim, continuous);
    let discrete_x = x_of(&sim, discrete);

    // Folded: the marker made the body continuous, so the wall stopped it on the
    // near side. Mutating the bridge's `world.get::<Ccd>(e).is_some()` to `false`
    // (ignore the marker) makes this body tunnel too, and this assertion goes RED.
    assert!(
        continuous_x1 < 0.0,
        "the body carrying the Ccd marker tunnelled (x={continuous_x1}) — the \
         bridge is not folding the marker into the sim"
    );
    // The identical body WITHOUT the marker tunnels — the contrast that proves the
    // scene really does reproduce tunnelling, so the stop above is the marker's doing.
    assert!(
        discrete_x > 0.5,
        "the control (no Ccd marker) did NOT tunnel (x={discrete_x}) — the fixture \
         no longer contains the phenomenon and the numbers need re-choosing"
    );

    // Scrub back to t=0 and replay: the body must still be stopped, which it can
    // only be if the CCD flag rode the `BodyDesc` the rewind rebuilds from.
    bridge.dispatch(&mut sim, false, 0);
    for tick in 1..=30u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let continuous_x2 = x_of(&sim, continuous);
    assert_eq!(
        continuous_x1, continuous_x2,
        "after a rewind to t=0 the CCD flag was not re-armed (x {continuous_x1} → \
         {continuous_x2}) — it was read once and lost on scrub"
    );
}
