//! The bridge folds the optional `Dominance` component into the sim, and a rewind
//! RE-ARMS it (collision priority, W-Dominance).
//!
//! `ph2d-physics` proves `BodyDesc::dominance` reaches the rigid body. This is the
//! ECS half, tested through the OUTCOME that mass cannot fake: a LIGHT ball carrying
//! `Dominance(5)` plows through a HEAVY neutral ball and travels well past the
//! collision point, while the identical LIGHT ball WITHOUT the component bounces off
//! the heavy one. After a scrub back to t=0 and replay it still plows through — which
//! it can only if the priority rode the `BodyDesc` the world rebuilds from.
//!
//! Gravity is zeroed so the collision is the only motion; the launch reuses
//! `InitialVelocity` (W9), the heavy target a `MassOverride` (W-Mass). The observable
//! is the mover's final X (the bridge writes the pose back into `Transform`).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Dominance, InitialVelocity, MassOverride, PhysicsBridge,
    PhysicsSettings, RigidBody,
};

/// A light ball launched right, optionally carrying `Dominance`.
fn mover(sim: &mut SimWorld, dominance: Option<i8>) -> Entity {
    let base = (
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-0.5, 0.0)),
        InitialVelocity {
            linvel: [3.0, 0.0],
            angvel: 0.0,
        },
    );
    match dominance {
        Some(d) => sim.world_mut().spawn((base, Dominance(d))).id(),
        None => sim.world_mut().spawn(base).id(),
    }
}

/// A heavy neutral ball at rest in the mover's path.
fn heavy_target(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.5, 0.0)),
        MassOverride(20.0),
    ));
}

fn zero_gravity() -> PhysicsSettings {
    PhysicsSettings {
        gravity_y: 0.0,
        ..Default::default()
    }
}

fn play(bridge: &mut PhysicsBridge, sim: &mut SimWorld) {
    for tick in 1..=60u64 {
        bridge.dispatch(sim, true, tick);
    }
}

fn x_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().translation.x
}

#[test]
fn the_bridge_folds_dominance_and_a_rewind_preserves_it() {
    // LIGHT mover with Dominance(5) launched into a HEAVY neutral target.
    let mut sim = SimWorld::new();
    let dominant = mover(&mut sim, Some(5));
    heavy_target(&mut sim);

    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    play(&mut bridge, &mut sim);
    let dom_x1 = x_of(&sim, dominant);

    // Plowed through: the light mover treated the heavy target as infinite mass and
    // is well past the collision point (x≈1). Mutating the bridge's
    // `world.get::<Dominance>(e)` to `0` makes it an ordinary light body that bounces
    // off the heavy one — this assertion goes RED.
    assert!(
        dom_x1 > 1.5,
        "the light Dominance(5) mover did not plow through the heavy target (final \
         x={dom_x1}) — the bridge is not folding the dominance into the sim"
    );

    // NEUTRAL control: the identical light mover without the component bounces off
    // the heavy target and stalls near the collision point — the contrast proving the
    // plow-through is the dominance's doing, not the scene.
    let mut sim2 = SimWorld::new();
    let neutral = mover(&mut sim2, None);
    heavy_target(&mut sim2);
    let mut bridge2 = PhysicsBridge::new();
    bridge2.set_settings(zero_gravity());
    play(&mut bridge2, &mut sim2);
    let neutral_x = x_of(&sim2, neutral);
    assert!(
        neutral_x < 1.0,
        "a light neutral mover should have bounced off the heavy target, but it is at \
         x={neutral_x} — the fixture no longer contains the phenomenon"
    );

    // Scrub back to t=0 and replay: the mover must still plow through, which it can
    // only if the dominance rode the `BodyDesc` the rewind rebuilds from.
    bridge.dispatch(&mut sim, false, 0);
    play(&mut bridge, &mut sim);
    let dom_x2 = x_of(&sim, dominant);
    assert!(
        (dom_x1 - dom_x2).abs() < 1e-3,
        "after a rewind to t=0 the dominance was not re-armed (x {dom_x1} → {dom_x2}) \
         — it was read once and lost on scrub"
    );
}
