//! The bridge folds the optional `MassOverride` component into the sim, and a
//! rewind RE-ARMS it (manual mass, W-Mass).
//!
//! `ph2d-physics` proves `BodyDesc::mass_override` reaches the collider's mass. This
//! is the ECS half, tested through the OUTCOME rather than a readout: a heavy mover
//! carrying `MassOverride` plows through a light target and travels well past the
//! collision point, while the identical mover WITHOUT the component (auto mass,
//! equal to the target) splits its momentum and stalls near it. After a scrub back
//! to t=0 and replay the heavy mover ends in the same place — which it can only if
//! the mass rode the `BodyDesc` the world rebuilds from rather than being read once
//! and lost on the scrub.
//!
//! Gravity is zeroed so the head-on collision is the only motion; the launch reuses
//! `InitialVelocity` (W9). The observable is the mover's final X — the bridge writes
//! the pose back into `Transform`, and position is the integral of velocity, so a
//! mover that kept its speed has travelled far.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InitialVelocity, MassOverride, PhysicsBridge,
    PhysicsSettings, RigidBody,
};

/// A ball at `(x, 0)` launched at `vx` m/s with an optional mass override.
fn ball(sim: &mut SimWorld, x: f32, vx: f32, mass: Option<f32>) -> Entity {
    let base = (
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(x, 0.0)),
        InitialVelocity {
            linvel: [vx, 0.0],
            angvel: 0.0,
        },
    );
    match mass {
        Some(m) => sim.world_mut().spawn((base, MassOverride(m))).id(),
        None => sim.world_mut().spawn(base).id(),
    }
}

fn zero_gravity() -> PhysicsSettings {
    PhysicsSettings {
        gravity_y: 0.0,
        ..Default::default()
    }
}

/// Play ticks `1..=60` forward.
fn play(bridge: &mut PhysicsBridge, sim: &mut SimWorld) {
    for tick in 1..=60u64 {
        bridge.dispatch(sim, true, tick);
    }
}

fn x_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().translation.x
}

#[test]
fn the_bridge_folds_a_mass_override_and_a_rewind_preserves_it() {
    // HEAVY mover (MassOverride 20 kg) launched right into a light target at rest.
    let mut sim = SimWorld::new();
    let heavy = ball(&mut sim, -0.5, 3.0, Some(20.0));
    let _target = ball(&mut sim, 0.5, 0.0, None);

    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    play(&mut bridge, &mut sim);
    let heavy_x1 = x_of(&sim, heavy);

    // The heavy mover plowed through the light target and is well past the collision
    // point (x≈1). Mutating the bridge's `world.get::<MassOverride>(e)` to `None`
    // makes it light (equal to the target), so it splits momentum and stalls near
    // the midpoint — this assertion goes RED.
    assert!(
        heavy_x1 > 1.5,
        "the heavy mover (MassOverride 20) did not plow through the light target \
         (final x={heavy_x1}) — the bridge is not folding the mass into the sim"
    );

    // AUTO-mass control: the identical scene without the override. Equal masses split
    // the momentum, so the mover stalls near the collision point — the contrast that
    // proves the plow-through above is the override's doing.
    let mut sim2 = SimWorld::new();
    let light = ball(&mut sim2, -0.5, 3.0, None);
    let _t2 = ball(&mut sim2, 0.5, 0.0, None);
    let mut bridge2 = PhysicsBridge::new();
    bridge2.set_settings(zero_gravity());
    play(&mut bridge2, &mut sim2);
    let light_x = x_of(&sim2, light);
    assert!(
        light_x < 1.5,
        "an equal-mass mover should have stalled near the collision point, but it is \
         at x={light_x} — the fixture no longer contains the phenomenon"
    );

    // Scrub back to t=0 and replay: the heavy mover must still be heavy, which it can
    // only be if the override rode the `BodyDesc` the rewind rebuilds from.
    bridge.dispatch(&mut sim, false, 0);
    play(&mut bridge, &mut sim);
    let heavy_x2 = x_of(&sim, heavy);
    assert!(
        (heavy_x1 - heavy_x2).abs() < 1e-3,
        "after a rewind to t=0 the mass override was not re-armed (x {heavy_x1} → \
         {heavy_x2}) — it was read once and lost on scrub"
    );
}
