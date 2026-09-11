//! The bridge folds the optional `OneWayPlatform` marker into the sim, and a rewind
//! RE-ARMS it (jump-through platforms, W-OneWay).
//!
//! `ph2d-physics` proves `BodyDesc.one_way` reaches the collider and drives rapier's
//! one-way hook. This is the ECS half, tested through the OUTCOME an artist would
//! check: a ball launched UP from under the platform passes clean through and then
//! LANDS on top of it, while the identical platform WITHOUT the marker stops it
//! underneath. After a scrub back to t=0 it still passes through — which it can only if
//! the flag rode the `BodyDesc` the world rebuilds from.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InitialVelocity, OneWayPlatform, PhysicsBridge, RigidBody,
};

/// A static platform at y=0, one-way or solid.
fn platform(sim: &mut SimWorld, one_way: bool) {
    let base = (
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 5.0,
                half_y: 0.1,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    );
    if one_way {
        sim.world_mut().spawn((base, OneWayPlatform));
    } else {
        sim.world_mut().spawn(base);
    };
}

/// A ball under the platform, launched straight up at 8 m/s.
fn jumper(sim: &mut SimWorld) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, -2.0)),
            InitialVelocity {
                linvel: [0.0, 8.0],
                angvel: 0.0,
            },
        ))
        .id()
}

/// Play from wherever the bridge is up to `tick`, tracking the highest y reached.
fn play_tracking_peak(
    bridge: &mut PhysicsBridge,
    sim: &mut SimWorld,
    e: Entity,
    tick: u64,
) -> (f32, f32) {
    let from = bridge.last_stepped();
    let mut peak = f32::NEG_INFINITY;
    for t in (from + 1)..=tick {
        bridge.dispatch(sim, true, t);
        let y = sim.world().get::<Transform>(e).unwrap().translation.y;
        peak = peak.max(y);
    }
    let final_y = sim.world().get::<Transform>(e).unwrap().translation.y;
    (peak, final_y)
}

#[test]
fn the_bridge_folds_one_way_and_a_rewind_preserves_it() {
    let mut sim = SimWorld::new();
    platform(&mut sim, true);
    let ball = jumper(&mut sim);
    let mut bridge = PhysicsBridge::new();
    let (peak, rest) = play_tracking_peak(&mut bridge, &mut sim, ball, 180);

    // Through, then down onto the top face (y ≈ 0.1 + 0.25). Mutating the bridge's
    // `world.get::<OneWayPlatform>(e)` to `false` makes the platform solid and both
    // of these go RED.
    assert!(
        peak > 1.0,
        "a ball launched from below a ONE-WAY platform should pass through it, but its \
         highest point was y={peak} — the bridge is not folding the marker into the sim"
    );
    assert!(
        (rest - 0.35).abs() < 0.1,
        "after passing through, the ball should LAND on the platform (y ≈ 0.35), but it \
         settled at y={rest}"
    );

    // SOLID control: the identical launch is stopped underneath — the contrast proving
    // the pass-through is the marker's doing, not the scene's.
    let mut sim2 = SimWorld::new();
    platform(&mut sim2, false);
    let ball2 = jumper(&mut sim2);
    let mut bridge2 = PhysicsBridge::new();
    let (peak_solid, _) = play_tracking_peak(&mut bridge2, &mut sim2, ball2, 180);
    assert!(
        peak_solid < 0.0,
        "a ball launched at a SOLID platform must be stopped underneath, but it reached \
         y={peak_solid} — the fixture no longer contains the phenomenon"
    );

    // Scrub back to t=0 and replay: still passes through and lands.
    bridge.dispatch(&mut sim, false, 0);
    let (peak2, rest2) = play_tracking_peak(&mut bridge, &mut sim, ball, 180);
    assert!(
        peak2 > 1.0 && (rest2 - rest).abs() < 1e-3,
        "after a rewind the one-way flag was not re-armed (peak {peak} → {peak2}, rest \
         {rest} → {rest2}) — it was read once and lost on scrub"
    );
}
