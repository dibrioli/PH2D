//! **W1.5 at the product seam:** dragging the playhead backwards must land on
//! the state that tick actually had, bit-exactly, without re-simulating the
//! whole history.
//!
//! `ph2d-physics`'s `checkpoint.rs` gates prove the checkpoint/restore
//! machinery in isolation. These drive the BRIDGE — the thing the shell calls
//! — because a correct ring wired into the wrong place is still a broken
//! scrub ([[feedback_tool_unit_green_integration_dead]]).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

/// Bodies that fall, collide with a floor, and pile up — a trajectory with a
/// *history*, so a scrub that silently replayed from the wrong state would
/// land somewhere visibly different.
fn scene() -> (SimWorld, Vec<Entity>) {
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
    let mut balls = Vec::new();
    for i in 0..9 {
        let col = (i % 3) as f32;
        let row = (i / 3) as f32;
        balls.push(
            sim.world_mut()
                .spawn((
                    RigidBody {
                        kind: BodyKind::Dynamic,
                    },
                    Collider {
                        shape: ColliderShape::Ball { radius: 0.25 },
                        density: 1.0,
                        ..Collider::default()
                    },
                    Transform::from_translation(Vec2::new(col * 0.6 - 0.6, 2.0 + row * 0.7)),
                ))
                .id(),
        );
    }
    (sim, balls)
}

fn poses(sim: &SimWorld, es: &[Entity]) -> Vec<(f32, f32, f32)> {
    es.iter()
        .map(|&e| {
            let t = sim.world().get::<Transform>(e).unwrap();
            (t.translation.x, t.translation.y, t.rotation)
        })
        .collect()
}

/// The state at tick T reached by playing straight there — the truth every
/// scrub is measured against.
fn played_straight_to(target: u64) -> Vec<(f32, f32, f32)> {
    let (mut sim, balls) = scene();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=target {
        bridge.dispatch(&mut sim, true, tick);
    }
    poses(&sim, &balls)
}

/// **Gate 1 (plan §W1.5):** scrubbing backwards is bit-exact.
///
/// Mutation-tested: an `anchor_at_or_before` that returns an anchor *after*
/// the target, or a ring that is not cleared when a body is added, both make
/// this red.
#[test]
fn scrubbing_backwards_reproduces_the_tick_exactly() {
    let (mut sim, balls) = scene();
    let mut bridge = PhysicsBridge::new();

    // Play well past every target, so the ring is populated and the live
    // world is far in the future of each one.
    for tick in 1..=400u64 {
        bridge.dispatch(&mut sim, true, tick);
    }

    // Targets straddle the stride deliberately: exactly on an anchor, just
    // after one, just before one, and inside the settled pile.
    for target in [370u64, 371, 369, 250, 137, 40] {
        let truth = played_straight_to(target);
        bridge.dispatch(&mut sim, false, target);
        assert_eq!(
            poses(&sim, &balls),
            truth,
            "scrubbing back to tick {target} did not reproduce that tick — the artist would be \
             shown a simulation that never happened"
        );
        assert_eq!(bridge.last_stepped(), target);
    }
}

/// **Gate 3 (plan §W1.5):** the scrub is `O(STRIDE)`, not `O(target)`.
///
/// A COUNT, not a stopwatch. The claim is about how much simulation a scrub
/// re-runs, and steps measure exactly that — no `ci-test` profile skew, no
/// flake, and it fails for the real reason rather than for a slow machine.
#[test]
fn a_scrub_replays_a_bounded_number_of_steps_however_far_in_it_lands() {
    let (mut sim, _) = scene();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=2000u64 {
        bridge.dispatch(&mut sim, true, tick);
    }

    let stride = ph2d_physics::checkpoint_stride();

    // A scrub near the very end and one 1500 ticks earlier must cost the
    // same — that is the whole claim of the ring.
    for target in [1990u64, 1500, 900, 500] {
        let before = bridge.steps_taken();
        bridge.dispatch(&mut sim, false, target);
        let replayed = bridge.steps_taken() - before;
        assert!(
            replayed < stride,
            "scrub to tick {target} replayed {replayed} steps; the stride is {stride}, so the \
             ring is not bounding the work (an O(target) replay is the W1 fallback, not the \
             product)"
        );
    }
}

/// The fallback is real and correct: a target older than the cached window
/// falls back to the rest-pose rebuild, which is `O(target)` and exact. This
/// gate pins that the *slow* path is still there and still right — the ring
/// must be deletable.
#[test]
fn a_target_older_than_the_window_still_lands_exactly() {
    let (mut sim, balls) = scene();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=200u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    // Reset: tick 0 always predates the window, so this is the rest path.
    bridge.dispatch(&mut sim, false, 0);
    let (len, _) = bridge.ring_stats();
    assert_eq!(
        len, 0,
        "the rest rebuild must clear the ring (fresh handles)"
    );

    let truth = played_straight_to(63);
    for tick in 1..=63u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    assert_eq!(
        poses(&sim, &balls),
        truth,
        "replaying after a Reset diverged from a straight play"
    );
}

/// **The silent-corruption defense, gated on its own layer.**
///
/// Spawning a body mid-timeline makes every cached state a snapshot of a
/// different world. Restoring one would hand the bridge rapier handles that
/// address a body set the new entity was never in — and the pose published
/// for it would be stale, with nothing looking broken.
///
/// Mutation-tested: removing the `ring.clear()` from `reconcile_structure`
/// makes this red. It needs its own gate because the bit-exactness gate above
/// never adds a body, so that defense would never fire there
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
#[test]
fn adding_a_body_mid_timeline_invalidates_the_cache() {
    let (mut sim, _) = scene();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=200u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let (before, _) = bridge.ring_stats();
    assert!(
        before > 0,
        "the ring must be populated for this to prove anything"
    );

    // A tenth ball drops in — the world is now structurally different.
    let newcomer = sim
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
            Transform::from_translation(Vec2::new(3.0, 6.0)),
        ))
        .id();
    bridge.dispatch(&mut sim, true, 201);

    let (after, _) = bridge.ring_stats();
    assert_eq!(
        after, 0,
        "the ring kept {after} checkpoints of the PREVIOUS body set after a spawn — a scrub \
         would restore a world the new body does not exist in and publish a stale pose for it"
    );

    // And the newcomer actually simulates from here — proof the invalidation
    // left a working world, not a cleared-and-broken one.
    let start = sim
        .world()
        .get::<Transform>(newcomer)
        .unwrap()
        .translation
        .y;
    for tick in 202..=260u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let now = sim
        .world()
        .get::<Transform>(newcomer)
        .unwrap()
        .translation
        .y;
    assert!(
        now < start - 0.5,
        "the newcomer did not fall (y {start} → {now})"
    );
}

/// **Gate 2 (plan §W1.5):** the ring's memory is MEASURED against the budget
/// it claims, not declared (HR-13's amendment).
///
/// The eviction is by BYTES, so a heavier scene gets a shorter window rather
/// than a bigger bill — the ADR-0117 lesson that a count is a multiplier, not
/// a ceiling.
#[test]
fn the_ring_stays_inside_its_share_of_the_physics_budget() {
    let (mut sim, _) = scene();
    let mut bridge = PhysicsBridge::new();
    // Ten minutes of simulated time at 60 Hz — far past what any window can
    // hold, so eviction is definitely exercised.
    for tick in 1..=36_000u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let (len, bytes) = bridge.ring_stats();
    let budget = ph2d_physics::checkpoint_budget_bytes();
    println!(
        "ring after 10 min: {len} checkpoints, {:.2} MB",
        bytes as f64 / 1_048_576.0
    );
    assert!(
        bytes <= budget,
        "the scrub cache grew to {bytes} bytes, past its {budget} byte ceiling"
    );
    assert!(len > 0, "eviction emptied the ring entirely");
}
