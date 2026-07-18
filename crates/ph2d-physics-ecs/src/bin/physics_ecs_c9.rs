#![forbid(unsafe_code)]
//! `physics-ecs-c9` — the ECS-bridged cross-OS determinism harness.
//!
//! Sibling of `ph2d_physics_c9` (which drives the raw wrapper). This one
//! drives the SAME 50-body fixture **through the ECS bridge**: entities
//! carry `RigidBody`/`Collider`, the bridge spawns rapier bodies, steps at
//! the tick, and reads poses back into `Transform`. The hash is over those
//! readback `Transform`s — so it proves OUR code (iteration order, the
//! meters↔rapier boundary, the readback) is bit-identical cross-OS, not
//! just rapier's internal state (ADR-0130 D7).
//!
//! CI runs this on Linux/macOS/Windows and compares the three hashes
//! (`.github/workflows/spike.yml`). Output format (stable, parsed by CI):
//! ```text
//! physics-ecs-c9 step_count: 120
//! physics-ecs-c9 body_count: 51
//! physics-ecs-c9 hash: <hex64>
//! ```

use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

const STEPS: u64 = 120; // 2 s @ 60 Hz — long enough for collisions to develop.
const N_DYNAMIC: u32 = 50;

fn main() {
    let mut sim = SimWorld::new();

    // Static floor: cuboid at y=0, half-thickness 0.1, half-width 50.
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

    // 50 dynamic circles in a 10×5 grid above the floor (same layout as
    // the raw wrapper's c9 fixture).
    for i in 0..N_DYNAMIC {
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
    // Drive it exactly as the shell does on play: one tick forward per
    // frame, sequential.
    for tick in 1..=STEPS {
        bridge.dispatch(&mut sim, true, tick);
    }

    let hash = bridge.deterministic_hash(&sim);
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();

    println!("physics-ecs-c9 step_count: {STEPS}");
    println!("physics-ecs-c9 body_count: {}", bridge.body_count());
    println!("physics-ecs-c9 hash: {hex}");
}
