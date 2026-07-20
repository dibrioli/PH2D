//! The bridge folds the optional `LockPositionX`/`LockPositionY` MARKERS into the
//! sim, and a rewind RE-ARMS them (Freeze Position, W-LockPos).
//!
//! `ph2d-physics` proves `BodyDesc::lock_x/lock_y` reach the solver. This is the
//! ECS half: (1) a body carrying `LockPositionX`, launched sideways, does NOT move
//! on X while an identical one without the marker does; and (2) after scrubbing the
//! clock back to t=0 and replaying, the locked body is still pinned — which is the
//! reason the flag rides the `BodyDesc` the world rebuilds from rather than being
//! read once and lost on scrub.
//!
//! The launch reuses `InitialVelocity` (W9); gravity is zeroed so the sideways
//! glide is the only motion. This gate is about whether the marker changes the
//! OUTCOME, and that the two axes are independent.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InitialVelocity, LockPositionX, LockPositionY,
    PhysicsBridge, PhysicsSettings, RigidBody,
};

/// A ball at `(x, 0)` launched at 3 m/s on X. `lock_x` attaches the marker.
fn slider(sim: &mut SimWorld, x: f32, lock_x: bool) -> Entity {
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
            linvel: [3.0, 0.0],
            angvel: 0.0,
        },
    );
    if lock_x {
        sim.world_mut().spawn((base, LockPositionX)).id()
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

fn pos_of(sim: &SimWorld, e: Entity) -> Vec2 {
    sim.world().get::<Transform>(e).unwrap().translation
}

#[test]
fn the_bridge_pins_a_marked_body_on_x_and_a_rewind_re_arms_it() {
    let mut sim = SimWorld::new();
    // Apart on Y so the two balls never touch, both launched along +X.
    let locked = slider(&mut sim, 0.0, true);
    let free = slider(&mut sim, 0.0, false);
    // Nudge the free one onto its own Y lane so they cannot collide.
    sim.world_mut()
        .get_mut::<Transform>(free)
        .unwrap()
        .translation
        .y = 4.0;

    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    for tick in 1..=30u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let locked_x1 = pos_of(&sim, locked).x;
    let free_x = pos_of(&sim, free).x;

    // Folded: the marker pinned X. Mutating the bridge's
    // `world.get::<LockPositionX>(e).is_some()` to `false` lets this body slide too,
    // and this assertion goes RED.
    assert!(
        locked_x1.abs() < 1e-4,
        "the body carrying the LockPositionX marker slid sideways (x={locked_x1}) — \
         the bridge is not folding the marker into the sim"
    );
    // The identical body WITHOUT the marker slides — the contrast proving the launch
    // really does move a free body, so the pin above is the marker's doing.
    assert!(
        free_x > 1.0,
        "the control (no LockPositionX) did NOT slide (x={free_x}) — the fixture no \
         longer contains the phenomenon and the numbers need re-choosing"
    );

    // Scrub back to t=0 and replay: the body must still be pinned, which it can only
    // be if the flag rode the `BodyDesc` the rewind rebuilds from.
    bridge.dispatch(&mut sim, false, 0);
    for tick in 1..=30u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let locked_x2 = pos_of(&sim, locked).x;
    assert_eq!(
        locked_x1, locked_x2,
        "after a rewind to t=0 the X lock was not re-armed (x {locked_x1} → {locked_x2}) \
         — it was read once and lost on scrub"
    );
}

#[test]
fn the_two_position_locks_are_independent() {
    // A Y-locked ball under gravity hangs in the air; an X-locked one launched
    // sideways stays put on X but still falls. Each marker touches only its axis.
    let mut sim = SimWorld::new();
    let y_locked = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 5.0)),
            InitialVelocity {
                linvel: [3.0, 0.0],
                angvel: 0.0,
            },
            LockPositionY,
        ))
        .id();

    let mut bridge = PhysicsBridge::new();
    // Full gravity (the default).
    for tick in 1..=30u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let p = pos_of(&sim, y_locked);
    // Y pinned: gravity could not pull it down.
    assert!(
        (p.y - 5.0).abs() < 1e-4,
        "a LockPositionY body fell (y={}) — the Y lock did not take",
        p.y
    );
    // X free: the sideways launch still moved it. Proves the marker froze ONLY Y
    // (a wiring that ORed `TRANSLATION_LOCKED` would have frozen X too).
    assert!(
        p.x > 1.0,
        "a LockPositionY body lost its X freedom (x={}) — the lock froze more than Y",
        p.x
    );
}
