#![forbid(unsafe_code)]
//! `physics-ecs-c9` — the ECS-bridged cross-OS determinism harness.
//!
//! Sibling of `ph2d_physics_c9` (which drives the raw wrapper). This one
//! drives the same falling-circles fixture **through the ECS bridge** (plus
//! one non-uniformly scaled ball → an ELLIPSE collider, W6, one
//! `GravityScale(0.5)` ball → a per-body gravity multiplier, W8, and one
//! non-uniformly scaled CAPSULE → a STADIUM hull, and one LAUNCHED ball →
//! an `InitialVelocity`, and one CCD ball launched at a thin wall → the CCD
//! solver's sweep, W-CCD): entities carry
//! `RigidBody`/`Collider`, the bridge spawns rapier bodies, steps at the
//! tick, and reads poses back into `Transform`. The hash is over those
//! readback `Transform`s — so it proves OUR code (iteration order, the
//! meters↔rapier boundary, the readback, the libm ellipse tessellation) is
//! bit-identical cross-OS, not just rapier's internal state (ADR-0131 D7).
//!
//! CI runs this on Linux/macOS/Windows and compares the three hashes
//! (`.github/workflows/spike.yml`). Output format (stable, parsed by CI):
//! ```text
//! physics-ecs-c9 step_count: 120
//! physics-ecs-c9 body_count: 57
//! physics-ecs-c9 hash: <hex64>
//! ```

use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Ccd, Collider, ColliderShape, GravityScale, InitialVelocity, PhysicsBridge, RigidBody,
};

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
            ..Collider::default()
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
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(col * 0.6 - 2.7, 5.0 + row * 0.6)),
        ));
    }

    // One NON-uniformly scaled ball (W6): its `Ball` resolves to an ELLIPSE
    // (`scaled_shape`), built from `ellipse_vertices` via libm. Its poses feed
    // the hash, so CI comparing the three OSes proves the ellipse tessellation
    // is bit-identical cross-platform — the only ML-free transcendental on the
    // physics path. A `f32::sin_cos` here would diverge in the last ulps and
    // this hash would split across OSes.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        Transform {
            translation: Vec2::new(0.3, 8.0),
            rotation: 0.0,
            scale: Vec2::new(1.6, 2.4),
            skew_x: 0.0,
            skew_y: 0.0,
        },
    ));

    // One ball with a per-body gravity scale (W8): `GravityScale(0.5)` makes it
    // fall at half rate. Its poses feed the hash, so the multiplier travels the
    // deterministic path (an f32 fold in the bridge's substep integration) and
    // CI proves it is bit-identical across the three OSes — the same guarantee
    // the ellipse gives the tessellation. Set apart on x so it lands on the same
    // floor without touching the grid.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        GravityScale(0.5),
        Transform::from_translation(Vec2::new(4.0, 6.0)),
    ));

    // One NON-uniformly scaled CAPSULE: it degrades to a `Stadium`, whose hull
    // comes from `capsule_vertices` — our own libm tessellation, exactly like
    // the ellipse above. That is what puts the capsule path on the cross-OS
    // hash: an `f32::sin_cos` in there would split the three OSes in the last
    // ulps and this is what would catch it. (A *uniform* capsule is rapier's
    // own exact shape and carries no transcendental of ours.)
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.20,
                radius: 0.15,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform {
            translation: Vec2::new(-4.0, 7.0),
            rotation: 0.0,
            scale: Vec2::new(1.7, 1.1),
            skew_x: 0.0,
            skew_y: 0.0,
        },
    ));

    // One LAUNCHED ball (W9): a nonzero `InitialVelocity` applied at spawn. Its
    // poses feed the hash, so the launch travels the deterministic path (an f32
    // set on the rigid body at build) and CI proves it is bit-identical across
    // the three OSes — same guarantee gravity scale gets. Aimed up-and-left into
    // empty space so it does not perturb the grid.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        InitialVelocity {
            linvel: [-1.5, 3.0],
            angvel: 2.0,
        },
        Transform::from_translation(Vec2::new(-4.0, 4.0)),
    ));

    // One CCD ball (W-CCD): launched fast at a thin static wall so the CCD SOLVER
    // actually runs its conservative-advancement / time-of-impact sweep. That puts
    // the CCD path on the cross-OS hash — an `f32` divergence in the sweep would
    // split the three OSes here, the same guarantee the ellipse gives the
    // tessellation. A body without the `Ccd` marker would tunnel and never invoke
    // the solver, so the marker is load-bearing for this coverage. Placed far off
    // in +x, its own lane, so it does not perturb the grid.
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
        Transform::from_translation(Vec2::new(20.0, 8.0)),
    ));
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.05 },
            density: 1.0,
            ..Collider::default()
        },
        InitialVelocity {
            linvel: [80.0, 0.0],
            angvel: 0.0,
        },
        Ccd,
        Transform::from_translation(Vec2::new(19.0, 8.0)),
    ));

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
