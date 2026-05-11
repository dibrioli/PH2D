//! Determinism replay test for [`propagate_transforms`] (HR-5).
//!
//! Builds a deterministic mixed hierarchy (100 entities with seeded
//! roots and child relationships), runs propagation, hashes the
//! `GlobalTransform` matrix bytes, and asserts the hash is stable
//! across two independent runs in the same process. The CI cross-
//! platform matrix (Linux + Mac) extends this test by uploading
//! hashes and diffing — same logic, distinct hosts.

use ph2d_core::Vec2;
use ph2d_ecs::{
    ChildOf, GlobalTransform, PresentWorld, SimRef, SimWorld, Transform, TransformPropagationState,
    WorklistBuf, propagate_transforms_into_present,
};

/// Deterministic LCG so this test is free of any RNG crate. Same
/// seeds produce identical streams on every architecture/OS.
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next_u32(&mut self) -> u32 {
        // Numerical Recipes LCG constants — known-good full period.
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.state >> 32) as u32
    }
    fn next_f32_unit(&mut self) -> f32 {
        (self.next_u32() as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

fn build_world() -> SimWorld {
    let mut sim = SimWorld::new();
    let mut rng = Lcg::new(0xC0FFEE_u64);

    // 20 roots, each with 4 children → 100 entities total.
    let mut roots = Vec::with_capacity(20);
    for _ in 0..20 {
        let pos = Vec2::new(rng.next_f32_unit() * 5.0, rng.next_f32_unit() * 5.0);
        let rot = rng.next_f32_unit() * std::f32::consts::TAU;
        let scale_factor = 0.5 + rng.next_f32_unit().abs() * 1.5;
        let e = sim
            .world_mut()
            .spawn(Transform {
                translation: pos,
                rotation: rot,
                scale: Vec2::new(scale_factor, scale_factor),
            })
            .id();
        roots.push(e);
    }
    for &root in &roots {
        for _ in 0..4 {
            let pos = Vec2::new(rng.next_f32_unit(), rng.next_f32_unit());
            let rot = rng.next_f32_unit() * std::f32::consts::PI;
            sim.world_mut().spawn((
                Transform {
                    translation: pos,
                    rotation: rot,
                    scale: Vec2::new(1.0, 1.0),
                },
                ChildOf(root),
            ));
        }
    }
    sim
}

fn hash_globals(sim: &mut SimWorld) -> [u8; 32] {
    let mut present = PresentWorld::new();
    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();
    ph2d_ecs::extract!(*sim => present, |sim_w, present_w| {
        propagate_transforms_into_present(sim_w, &mut state, present_w, &mut worklist);
    });

    let mut q = present.world_mut().query::<(&SimRef, &GlobalTransform)>();
    let mut pairs: Vec<(u64, [f32; 9])> = q
        .iter(present.world())
        .map(|(sref, gt)| {
            let m = &gt.matrix;
            (
                sref.0.to_bits(),
                [
                    m.x_axis.x, m.x_axis.y, m.x_axis.z, m.y_axis.x, m.y_axis.y, m.y_axis.z,
                    m.z_axis.x, m.z_axis.y, m.z_axis.z,
                ],
            )
        })
        .collect();
    pairs.sort_by_key(|(e, _)| *e);

    let mut hasher = blake3::Hasher::new();
    for (e, m) in &pairs {
        hasher.update(&e.to_le_bytes());
        for f in m {
            hasher.update(&f.to_le_bytes());
        }
    }
    *hasher.finalize().as_bytes()
}

#[test]
fn same_world_twice_identical_hash() {
    let mut sim_a = build_world();
    let mut sim_b = build_world();
    let h_a = hash_globals(&mut sim_a);
    let h_b = hash_globals(&mut sim_b);
    assert_eq!(
        h_a, h_b,
        "two independently-built identical worlds produced different propagation hashes"
    );
}

#[test]
fn repeated_propagation_same_world_stable() {
    let mut sim = build_world();
    let h1 = hash_globals(&mut sim);
    let h2 = hash_globals(&mut sim);
    assert_eq!(h1, h2, "running propagation twice on the same sim diverged");
}

// blake3 is already a transitive dep via the workspace (ph2d-asset
// pulls it; here we'd need it as a dev-dep). Add it to ph2d-ecs/
// Cargo.toml [dev-dependencies] — see that file.
