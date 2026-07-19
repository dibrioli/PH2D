//! W1 zero-alloc gate (HR-3). The steady-state hot path (`dispatch` with no
//! entities spawning or despawning) must not grow the bridge's scratch
//! buffers. Asserted by **capacity stability**, not a global allocation
//! counter — the counter is process-wide and flaky
//! ([[feedback_zero_alloc_gate_capacity_not_global_counter]]); capacity is
//! deterministic and names the exact regression (a realloc).
//!
//! The hot path is reviewed to allocate only via the `seen`/`to_spawn`/
//! `to_remove` `Vec`s and the `bodies` `HashMap`, all reused with
//! `clear()` + `push()` (no `Vec::new`, `Box`, `format!` per frame). This
//! gate proves they never grow past their warmed capacity.
//!
//! Mutation-tested: dropping `self.seen.clear()` in `reconcile_structure`
//! makes `seen` accumulate every frame → its capacity grows → RED.

use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

#[test]
fn steady_state_dispatch_does_not_grow_the_scratch() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 50.0,
                half_y: 0.1,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    for i in 0..30u32 {
        sim.world_mut().spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(
                (i % 6) as f32 * 0.5,
                4.0 + (i / 6) as f32 * 0.5,
            )),
        ));
    }

    let mut bridge = PhysicsBridge::new();

    // ⚠️ A PARENTED body, so the ancestor walk W5 added is actually exercised.
    // Without one, `chain` stays empty and its "allocates nothing per frame"
    // claim is asserted over a code path the fixture never enters
    // ([[feedback_absence_gate_needs_a_presence_sibling]]).
    let rig = sim
        .world_mut()
        .spawn((Transform::from_translation(Vec2::new(3.0, 1.0)),))
        .id();
    let mid = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.5, 0.0)),
            ph2d_ecs::ChildOf(rig),
        ))
        .id();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.2 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 2.0)),
        ph2d_ecs::ChildOf(mid),
    ));

    // Warm: the only spawns (and thus the only scratch growth) happen on
    // the first tick; a few more let capacities settle.
    for tick in 1..=5u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let warm_cap = bridge.scratch_capacity();
    assert!(
        warm_cap >= 31,
        "scratch never held the 31 bodies: {warm_cap}"
    );

    // Steady state: no entity added or removed. The scratch must not grow.
    for tick in 6..=200u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    assert_eq!(
        bridge.scratch_capacity(),
        warm_cap,
        "HR-3 violation: the per-frame scratch reallocated (cap {warm_cap} → {}) \
         — something in the hot path grows instead of reusing clear()+push()",
        bridge.scratch_capacity()
    );
}
