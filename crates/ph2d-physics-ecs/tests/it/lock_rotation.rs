//! The bridge folds the optional `LockRotation` MARKER into the sim, and a
//! rewind RE-ARMS it (Freeze Rotation, W-LockRot).
//!
//! `ph2d-physics` proves `BodyDesc::lock_rotation` reaches the solver. This is
//! the ECS half: (1) a spinning body carrying the marker does NOT rotate while an
//! identical one without it does, and (2) after scrubbing the clock back to t=0
//! and replaying, the locked body is still pinned — which is the reason the flag
//! rides the `BodyDesc` the world rebuilds from rather than being read once and
//! lost on scrub.
//!
//! The spin itself reuses `InitialVelocity` (W9); gravity is zeroed so the spin is
//! the only motion. This gate is about whether the marker changes the OUTCOME.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InitialVelocity, LockRotation, PhysicsBridge,
    PhysicsSettings, RigidBody,
};

/// A box at `(x, 0)` spun at 5 rad/s. `lock` attaches the `LockRotation` marker.
fn spinner(sim: &mut SimWorld, x: f32, lock: bool) -> Entity {
    let base = (
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
        InitialVelocity {
            linvel: [0.0, 0.0],
            angvel: 5.0,
        },
    );
    if lock {
        sim.world_mut().spawn((base, LockRotation)).id()
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

fn rot_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().rotation
}

#[test]
fn the_bridge_pins_a_marked_body_and_a_rewind_re_arms_it() {
    let mut sim = SimWorld::new();
    // Set apart on x so the two boxes never touch.
    let locked = spinner(&mut sim, 0.0, true);
    let free = spinner(&mut sim, 4.0, false);

    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    for tick in 1..=30u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let locked_rot1 = rot_of(&sim, locked);
    let free_rot = rot_of(&sim, free);

    // Folded: the marker pinned the orientation. Mutating the bridge's
    // `world.get::<LockRotation>(e).is_some()` to `false` makes this body spin too,
    // and this assertion goes RED.
    assert!(
        locked_rot1.abs() < 1e-4,
        "the body carrying the LockRotation marker rotated (rot={locked_rot1}) — \
         the bridge is not folding the marker into the sim"
    );
    // The identical body WITHOUT the marker spins — the contrast proving the scene
    // really does turn a free body, so the pin above is the marker's doing.
    assert!(
        free_rot.abs() > 1.0,
        "the control (no LockRotation) did NOT spin (rot={free_rot}) — the fixture \
         no longer contains the phenomenon and the numbers need re-choosing"
    );

    // Scrub back to t=0 and replay: the body must still be pinned, which it can
    // only be if the flag rode the `BodyDesc` the rewind rebuilds from.
    bridge.dispatch(&mut sim, false, 0);
    for tick in 1..=30u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let locked_rot2 = rot_of(&sim, locked);
    assert_eq!(
        locked_rot1, locked_rot2,
        "after a rewind to t=0 the rotation lock was not re-armed (rot {locked_rot1} \
         → {locked_rot2}) — it was read once and lost on scrub"
    );
}
