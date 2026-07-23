//! The bridge folds the optional `AreaTorque` into the sim, and a rewind RE-ARMS it
//! (torque zones, W-AreaTorque).
//!
//! `ph2d-physics` proves `BodyDesc.effector.torque` reaches the world and spins what
//! overlaps it. This is the ECS half, tested through the OUTCOME an artist would check: a
//! box sitting inside a turntable spins up, while an identical box outside it stays still.
//! After a scrub back to t=0 it spins up the same way — which it can only if the torque
//! rode the `BodyDesc` the world rebuilds from. The fixture carries ONLY `AreaTorque` (no
//! `AreaEffector`, no `AreaDrag`), so it also proves its half of the bundle: a body with
//! no force still becomes a zone.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaTorque, BodyKind, Collider, ColliderShape, PhysicsBridge, PhysicsSettings, RigidBody,
};

/// No world gravity, so the only thing acting on a body inside the turntable is the
/// zone's torque — a torque has no "down", and the box must not fall out of the sensor.
fn zero_gravity() -> PhysicsSettings {
    PhysicsSettings {
        gravity_y: 0.0,
        ..Default::default()
    }
}

/// A static SENSOR box carrying a torque — the turntable. `sensor` is a parameter
/// because the coupling ("a solid zone spins nothing") is half the contract.
fn zone(sim: &mut SimWorld, x: f32, torque: f32, sensor: bool) {
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 2.0,
                half_y: 2.0,
            },
            density: 1.0,
            is_sensor: sensor,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(x, 0.0)),
        AreaTorque(torque),
    ));
}

/// A 1x1 box at `(x, 0)`, at rest — so the whole angular-velocity ramp comes from the
/// zone. ⚠️ Big and heavy ENOUGH (moment of inertia 0.167) that a modest torque turns it
/// a fraction of a revolution over the test window: the readback writes `Transform.rotation`,
/// which WRAPS at ±π, so a strong torque on a tiny box spins it many turns and the wrapped
/// value is meaningless. The world-level gates read the raw `angvel` and can push hard; here
/// the oracle is rotation, so the fixture keeps it sub-revolution.
fn spinner(sim: &mut SimWorld, x: f32) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.5,
                },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 0.0)),
        ))
        .id()
}

fn play_to(bridge: &mut PhysicsBridge, sim: &mut SimWorld, tick: u64) {
    let from = bridge.last_stepped();
    for t in (from + 1)..=tick {
        bridge.dispatch(sim, true, t);
    }
}

fn spin_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().rotation
}

#[test]
fn the_bridge_folds_the_torque_zone_and_a_rewind_preserves_it() {
    let mut sim = SimWorld::new();
    // Torque 0.5 turns the 1x1 spinner ~1.49 rad (86 deg) over 60 ticks — a clear turn,
    // well under half a revolution, so `rotation` does not wrap.
    zone(&mut sim, 0.0, 0.5, true);
    let inside = spinner(&mut sim, 0.0);
    let outside = spinner(&mut sim, 8.0);
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    play_to(&mut bridge, &mut sim, 60);

    let spun = spin_of(&sim, inside);
    assert!(
        spun.abs() > 0.3,
        "a body inside the turntable should have turned, but its rotation is {spun} — \
         the bridge is not folding `AreaTorque` into the sim"
    );
    assert!(
        spin_of(&sim, outside).abs() < 1e-4,
        "a body outside the turntable must not turn, rotation={}",
        spin_of(&sim, outside)
    );

    // Scrub back to t=0 and replay: turned the same way, to the same place.
    bridge.dispatch(&mut sim, false, 0);
    play_to(&mut bridge, &mut sim, 60);
    let spun2 = spin_of(&sim, inside);
    assert!(
        (spun2 - spun).abs() < 1e-3,
        "after a rewind the turntable was not re-armed (rotation {spun} -> {spun2}) — the \
         torque was read once and lost on the scrub"
    );
}

#[test]
fn a_solid_torque_zone_spins_nothing() {
    // ⚠️ The coupling, at the only layer where it is a SIM outcome: a solid collider
    // records no intersection, so it has nobody to spin — and it BLOCKS instead. The §11
    // row mirrors this by offering Torque only for a sensor (its seam gate).
    let mut sim = SimWorld::new();
    zone(&mut sim, 0.0, 8.0, false);
    // Beside the solid box, not on it, so this measures the absence of a spin, not a
    // collision.
    let e = spinner(&mut sim, 3.0);
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    play_to(&mut bridge, &mut sim, 60);
    assert!(
        spin_of(&sim, e).abs() < 1e-4,
        "a SOLID zone must not spin anything, rotation={}",
        spin_of(&sim, e)
    );
}
