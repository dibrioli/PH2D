//! W1 determinism gate (local proxy for the cross-OS CI gate). The
//! ECS-bridged hash must be a pure function of the SET of readback poses —
//! independent of the `HashMap` iteration order of the handle map. Two
//! fresh runs of the identical fixture must hash identically.
//!
//! The only nondeterminism our code can introduce is `HashMap` iteration
//! (its `RandomState` seed differs per instance). `deterministic_hash`
//! sorts the pose rows to neutralise it. Mutation-tested: removing the
//! `rows.sort_unstable()` in `deterministic_hash` makes the two runs use
//! different orders → different blake3 → RED. (With 51 bodies the two
//! random orders coincide with probability ~1/51!, i.e. never.)
//!
//! The true cross-OS byte-identity is proven by the `physics-ecs-c9`
//! harness compared across Linux/macOS/Windows in CI (spike.yml).

use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

/// Build the 50-body fixture, drive 120 ticks through the bridge, return
/// the ECS-bridged hash. A fresh `SimWorld` + `PhysicsBridge` each call, so
/// the handle map is a different `HashMap` instance (different seed).
fn run_fixture() -> [u8; 32] {
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
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    for i in 0..50u32 {
        let row = (i / 10) as f32;
        let col = (i % 10) as f32;
        sim.world_mut().spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                density: 1.0,
            },
            Transform::from_translation(Vec2::new(col * 0.6 - 2.7, 5.0 + row * 0.6)),
        ));
    }
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=120u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    bridge.deterministic_hash(&sim)
}

#[test]
fn the_ecs_bridged_hash_is_independent_of_handle_map_order() {
    let a = run_fixture();
    let b = run_fixture();
    assert_eq!(
        a, b,
        "two fresh runs of the identical fixture hashed differently — \
         HashMap iteration order leaked into the digest (the sort is gone)"
    );
}

/// A non-empty world hashes to something; a mutation that never steps or
/// never reads back would leave every pose at the spawn height, but the
/// *hash* would still be stable — so this is only a liveness sanity check,
/// NOT the determinism proof (that is the equality above).
#[test]
fn the_hash_is_nonzero_for_a_populated_world() {
    assert_ne!(run_fixture(), [0u8; 32]);
}
