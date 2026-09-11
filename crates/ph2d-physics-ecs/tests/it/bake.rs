//! **W4 — the bake reads the simulation, and puts the scene back.**
//!
//! Three claims, and the third is the one a careless implementation breaks
//! without anyone noticing until an artist loses their place:
//!
//! 1. the trajectory IS the simulation — sample for sample, at the right time;
//! 2. two bakes of the same scene agree (D7, determinism);
//! 3. the clock ends where it started.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, PoseChannel, RigidBody, bake_trajectories,
};

const DT: f64 = 1.0 / 60.0;

/// A ball dropped onto a floor and a second one dropped off-centre onto it, so
/// the scene has contact, rotation and a settle — a trajectory with a history,
/// not a parabola that any wrong implementation could also produce.
fn scene() -> (SimWorld, Vec<Entity>) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 8.0,
                half_y: 0.2,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let mut movers = Vec::new();
    for at in [Vec2::new(0.0, 1.0), Vec2::new(0.18, 2.4)] {
        movers.push(
            sim.world_mut()
                .spawn((
                    RigidBody {
                        kind: BodyKind::Dynamic,
                    },
                    Collider {
                        shape: ColliderShape::Ball { radius: 0.25 },
                        restitution: 0.4,
                        ..Collider::default()
                    },
                    Transform::from_translation(at),
                ))
                .id(),
        );
    }
    (sim, movers)
}

fn pose(sim: &SimWorld, e: Entity) -> (f32, f32, f32) {
    let t = sim.world().get::<Transform>(e).unwrap();
    (t.translation.x, t.translation.y, t.rotation)
}

/// **Every sample is the pose the sim actually had at that tick.**
///
/// An APPEARANCE oracle, not a formula: the reference is the same bridge driven
/// straight through, one tick at a time, which is exactly what the artist sees
/// when they press play. Compared at every tick rather than at the end, because
/// a falling body settles and a wrong trajectory re-converges — an endpoint
/// check would go green over a bake that got the whole middle wrong.
#[test]
fn the_trajectory_is_the_simulation_tick_for_tick() {
    let ticks = 90u64;

    // The reference: play it, writing down the pose each tick.
    let (mut sim, movers) = scene();
    let mut bridge = PhysicsBridge::new();
    let mut expected: Vec<Vec<(f32, f32, f32)>> = vec![Vec::new(); movers.len()];
    bridge.dispatch(&mut sim, false, 0);
    for (i, &e) in movers.iter().enumerate() {
        expected[i].push(pose(&sim, e));
    }
    for tick in 1..=ticks {
        bridge.dispatch(&mut sim, true, tick);
        for (i, &e) in movers.iter().enumerate() {
            expected[i].push(pose(&sim, e));
        }
    }

    // The bake, on a fresh world.
    let (mut sim, movers) = scene();
    let mut bridge = PhysicsBridge::new();
    let baked = bake_trajectories(&mut bridge, &mut sim, &movers, ticks, DT);

    assert_eq!(baked.len(), movers.len());
    for traj in &baked {
        // Matched by ENTITY, never by position: the bake orders its output by
        // entity bits (so a selection has one answer whichever way it was
        // clicked), which is not the order the caller listed them in.
        let i = movers
            .iter()
            .position(|&e| e == traj.entity)
            .expect("the bake invented an entity");
        assert_eq!(
            traj.samples.len(),
            ticks as usize + 1,
            "body {i}: a bake of {ticks} ticks must carry {} samples — the rest \
             pose at t=0 plus one per tick",
            ticks + 1
        );
        for (k, s) in traj.samples.iter().enumerate() {
            assert_eq!(
                s.0,
                k as f64 * DT,
                "body {i} sample {k} is stamped at the wrong time"
            );
            assert_eq!(
                (s.1, s.2, s.3),
                expected[i][k],
                "body {i} diverges from the simulation at tick {k}"
            );
        }
    }
}

/// **The bake leaves the clock where it found it.**
///
/// Red over the obvious implementation, which runs the range and stops at the
/// end: the artist clicks Bake while looking at tick 40 and the scene silently
/// jumps to tick 90. Asserts the POSE and not just the tick number, because a
/// bridge that reported the right tick while holding the wrong world would be
/// the worse bug of the two.
#[test]
fn a_bake_puts_the_scene_back_where_the_artist_left_it() {
    let (mut sim, movers) = scene();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=40 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let before: Vec<_> = movers.iter().map(|&e| pose(&sim, e)).collect();

    let _ = bake_trajectories(&mut bridge, &mut sim, &movers, 90, DT);

    assert_eq!(
        bridge.last_stepped(),
        40,
        "the bake left the clock at the end of its range"
    );
    let after: Vec<_> = movers.iter().map(|&e| pose(&sim, e)).collect();
    assert_eq!(
        after, before,
        "the scene moved: baking is a READING of the simulation, and a reading \
         that changes what it read is not one"
    );
}

/// **The same scene bakes to the same numbers.** (ADR-0131 D7.)
///
/// Not "close" — identical. The whole point of the deterministic sim is that a
/// bake is reproducible, and `f32` equality is the only assertion that would
/// catch a `HashMap` iteration order or a stray transcendental convention.
#[test]
fn baking_twice_gives_the_same_trajectory() {
    let run = || {
        let (mut sim, movers) = scene();
        let mut bridge = PhysicsBridge::new();
        bake_trajectories(&mut bridge, &mut sim, &movers, 60, DT)
    };
    assert_eq!(run(), run(), "two bakes of one scene disagreed");
}

/// **The output order is a property of the SET, not of the click order.**
///
/// The artist selecting A then B and B then A must get the same bake. Ordered
/// by entity bits, the same rule the bridge's `BTreeMap` already follows so the
/// two halves cannot disagree about what "in order" means.
#[test]
fn the_bake_order_does_not_depend_on_the_selection_order() {
    let (mut sim, movers) = scene();
    let mut bridge = PhysicsBridge::new();
    let forward = bake_trajectories(&mut bridge, &mut sim, &movers, 30, DT);

    let mut reversed = movers.clone();
    reversed.reverse();
    let (mut sim, _) = scene();
    let mut bridge = PhysicsBridge::new();
    let backward = bake_trajectories(&mut bridge, &mut sim, &reversed, 30, DT);

    assert_eq!(forward, backward);
}

/// **A body that never moved in a channel yields no track for it.**
///
/// The product-level statement of `channel() -> None`: a box dropped straight
/// down has a constant X, and the bake must not claim that channel. Driven
/// through the real sim rather than a hand-built trajectory, because the claim
/// is about what the SIMULATION produces — if rapier jittered X by an ulp while
/// the body fell, the unit test would still pass and this one would not.
#[test]
fn a_straight_drop_bakes_no_horizontal_track() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 8.0,
                half_y: 0.2,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let e = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.25,
                    half_y: 0.25,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 2.0)),
        ))
        .id();

    let mut bridge = PhysicsBridge::new();
    let baked = bake_trajectories(&mut bridge, &mut sim, &[e], 90, DT);
    let traj = &baked[0];

    assert!(
        traj.channel(PoseChannel::Y).is_some(),
        "the body fell, so Y is a track — if this is None the fixture is not \
         measuring a drop and the assertion below means nothing"
    );
    assert!(
        traj.channel(PoseChannel::X).is_none(),
        "a box dropped straight down produced a horizontal track; baking it \
         would overwrite whatever the artist had animated on X"
    );
}
