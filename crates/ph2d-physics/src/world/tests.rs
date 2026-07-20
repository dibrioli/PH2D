//! Unit tests for [`super::PhysicsWorld`], split out of `world.rs` to keep that
//! file under the workspace LOC cap. A child module (`super::*` reaches the
//! wrapper's private surface exactly as an inline `mod tests` did).

use super::*;

#[test]
fn empty_world_has_zero_bodies() {
    let w = PhysicsWorld::new();
    assert_eq!(w.body_snapshots().len(), 0);
    assert_eq!(w.step_count(), 0);
}

#[test]
fn step_advances_counter() {
    let mut w = PhysicsWorld::new();
    w.step();
    w.step();
    w.step();
    assert_eq!(w.step_count(), 3);
}

#[test]
fn dt_default_is_60hz() {
    let w = PhysicsWorld::new();
    // 1/60 ≈ 0.01666...; allow tiny f32 rounding.
    assert!((w.dt() - 1.0 / 60.0).abs() < 1e-6);
}

#[test]
fn falling_body_hits_floor() {
    let mut w = PhysicsWorld::new();
    // Floor at y=0, half-thickness 0.1.
    w.add_static_cuboid(0.0, 0.0, 50.0, 0.1);
    // Ball at y=10 (10 m above floor).
    let (ball, _) = w.add_dynamic_circle(0.0, 10.0, 0.5, 1.0);
    // Step long enough to settle (gravity 9.81 m/s²; free-fall
    // from 10m takes ~1.43s; settling takes a few more).
    for _ in 0..600 {
        w.step();
    }
    let pose = w.body_pose(ball).expect("ball still exists");
    // Ball center should be near floor + radius (~ 0.1 + 0.5 = 0.6).
    assert!(
        pose.translation.y >= 0.5 && pose.translation.y <= 1.0,
        "ball settled at y={}, expected ~0.6",
        pose.translation.y
    );
}

#[test]
fn hash_is_stable_across_runs_in_same_process() {
    // Same fixture, same hash. Cross-OS test lives in the bin
    // (tests/spike-style C9 extension) — this is a sanity check
    // that the hashing function itself is deterministic on one
    // OS, not affected by allocation order or HashMap iteration.
    let h1 = run_50_body_fixture();
    let h2 = run_50_body_fixture();
    assert_eq!(h1, h2);
}

fn run_50_body_fixture() -> [u8; 32] {
    let mut w = PhysicsWorld::new();
    w.add_static_cuboid(0.0, 0.0, 50.0, 0.1);
    for i in 0..50 {
        let row = (i / 10) as f32;
        let col = (i % 10) as f32;
        w.add_dynamic_circle(col * 0.6 - 2.7, 5.0 + row * 0.6, 0.25, 1.0);
    }
    for _ in 0..120 {
        w.step();
    }
    w.deterministic_hash()
}
