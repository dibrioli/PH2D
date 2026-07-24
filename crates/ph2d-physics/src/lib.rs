#![forbid(unsafe_code)]
//! ph2d-physics — Rapier2D wrapper with `enhanced-determinism` ON
//! (M10, HR-5).
//!
//! ### Determinism contract
//!
//! - Cargo features pin `rapier2d` with `enhanced-determinism` ON and
//!   `parallel` / `simd-stable` / `simd-nightly` OFF. Those three
//!   features all break bit-identical reproducibility across Linux /
//!   macOS / Windows because they reorder floating-point summation
//!   per-platform (LLVM auto-vectorizer) or per-thread (work stealing).
//! - The fixed-step `dt` lives in [`PhysicsWorld::dt`]; never call
//!   `step()` with an external dt — the integrator's accuracy +
//!   determinism both depend on a fixed step.
//! - [`PhysicsWorld::deterministic_hash`] returns a blake3 over the
//!   sorted `(handle_index, position, rotation, linvel, angvel)` of
//!   every body. Used by the C9 cross-OS gate in CI.
//!
//! ### What this crate is NOT (yet)
//!
//! M10 shipped the wrapper + the cross-OS determinism gate; the ECS
//! integration it said was waiting for a real scene is now
//! `ph2d-physics-ecs`, and joints landed with it (see [`world::joints`]).
//! Continuous Collision Detection and force fields still work via the
//! underlying Rapier API but have no convenience helpers here.

pub mod world;

pub use world::checkpoint::{PhysicsCheckpoint, PhysicsCheckpointRing};
pub use world::contacts::{ContactReport, PeakKey, PeakSample};
pub use world::defaults::BodyDefaults;
pub use world::effector::{zone_force_world, zone_force_world_at, zone_spin_sign};
pub use world::joints::{JointDesc, JointKind, MotorDesc};
pub use world::layers::{LayerMatrix, MAX_LAYERS};
pub use world::{
    AreaEffect, BodyDesc, CAPSULE_CAP_SEGS, CombineRules, DampingDesc, ELLIPSE_SEGS, PhysicsWorld,
    ShapeDesc, capsule_vertices, ellipse_vertices,
};

/// The checkpoint ring's stride, in ticks — the bound on how many steps a
/// backwards scrub ever replays. See [`world::checkpoint::PhysicsCheckpointRing`]
/// for why it is 10 and not 1.
pub fn checkpoint_stride() -> u64 {
    world::checkpoint::STRIDE
}

/// The ring's byte ceiling (its slice of the HR-13 physics budget).
pub fn checkpoint_budget_bytes() -> usize {
    world::checkpoint::DEFAULT_BUDGET_BYTES
}

// Re-export the few rapier types callers will reach for, so a
// downstream crate doesn't need a direct rapier2d dep just to
// receive a body handle. Keeping the surface narrow per the
// SKILL §7 anti-pattern "Acoplar API pública a tipos externos".
pub use rapier2d::dynamics::{
    CoefficientCombineRule, ImpulseJointHandle, RigidBodyHandle, RigidBodyType,
};
pub use rapier2d::geometry::ColliderHandle;
