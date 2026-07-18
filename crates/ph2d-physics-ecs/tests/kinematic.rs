//! **W4 — `BodyKind::Kinematic`: the pose is an INPUT.**
//!
//! `Dynamic` and `Static` between them cover "the solver decides" and "nothing
//! decides". The kind this wave adds covers the third case, which is the one a
//! baked body is in: **the scene decides, and the solver is told**.
//!
//! The gate that matters is not that a kinematic body holds still — `Static`
//! does that too, and a broken drive stage would pass it. It is that a
//! kinematic body **pushes**: it carries what rests on it and shoves what
//! stands in its way, because rapier derives its velocity from the pose it was
//! aimed at. Teleporting the body instead (`set_body_pose`, which zeroes the
//! velocity) looks identical on screen right up to the first contact, and then
//! leaves the load behind. That difference is what these tests measure.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

const PLATFORM_HALF_X: f32 = 1.0;
const PLATFORM_HALF_Y: f32 = 0.2;
const BOX_HALF: f32 = 0.25;

fn spawn_body(sim: &mut SimWorld, kind: BodyKind, shape: ColliderShape, at: Vec2) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody { kind },
            Collider {
                shape,
                ..Collider::default()
            },
            Transform::from_translation(at),
        ))
        .id()
}

fn pose(sim: &SimWorld, e: Entity) -> (f32, f32, f32) {
    let t = sim.world().get::<Transform>(e).unwrap();
    (t.translation.x, t.translation.y, t.rotation)
}

fn set_x(sim: &mut SimWorld, e: Entity, x: f32) {
    sim.world_mut()
        .get_mut::<Transform>(e)
        .unwrap()
        .translation
        .x = x;
}

/// A box resting on a platform. The platform's kind is the variable.
fn platform_scene(kind: BodyKind) -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let platform = spawn_body(
        &mut sim,
        kind,
        ColliderShape::Cuboid {
            half_x: PLATFORM_HALF_X,
            half_y: PLATFORM_HALF_Y,
        },
        Vec2::new(0.0, 0.0),
    );
    // Resting ON it: bottom of the box exactly at the top of the platform.
    let cargo = spawn_body(
        &mut sim,
        BodyKind::Dynamic,
        ColliderShape::Cuboid {
            half_x: BOX_HALF,
            half_y: BOX_HALF,
        },
        Vec2::new(0.0, PLATFORM_HALF_Y + BOX_HALF),
    );
    (sim, platform, cargo)
}

/// **A kinematic body does not fall, and the solver never writes its pose.**
///
/// The first half of the contract. Red before the variant existed (there was
/// no third kind to author); red again if `body_desc` mapped it to `Dynamic`,
/// which is the mapping typo this catches — it would drop 1.2 m in 50 ticks.
#[test]
fn a_kinematic_body_ignores_gravity_and_keeps_its_authored_pose() {
    let mut sim = SimWorld::new();
    let e = spawn_body(
        &mut sim,
        BodyKind::Kinematic,
        ColliderShape::Ball { radius: 0.25 },
        Vec2::new(1.5, 3.0),
    );
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=50 {
        bridge.dispatch(&mut sim, true, tick);
    }
    assert_eq!(
        pose(&sim, e),
        (1.5, 3.0, 0.0),
        "the solver moved a body whose pose the SCENE owns — `readback` must \
         skip every kind but Dynamic"
    );
}

/// **A kinematic body goes exactly where the `Transform` says, tick by tick.**
///
/// The drive stage, measured on its own: no contacts, so nothing but the aim
/// can put the body where it ends up. This is what a baked curve does to a
/// body once the keys are driving it.
#[test]
fn a_kinematic_body_follows_the_transform_it_is_given() {
    let mut sim = SimWorld::new();
    let e = spawn_body(
        &mut sim,
        BodyKind::Kinematic,
        ColliderShape::Ball { radius: 0.25 },
        Vec2::new(0.0, 1.0),
    );
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=20 {
        // The scene writes the pose — a curve would write exactly here.
        set_x(&mut sim, e, tick as f32 * 0.05);
        bridge.dispatch(&mut sim, true, tick);
        assert_eq!(
            pose(&sim, e).0,
            tick as f32 * 0.05,
            "tick {tick}: the readback overwrote a pose the scene authored"
        );
    }
}

/// **A kinematic platform CARRIES what rests on it.**
///
/// The gate the variant exists for, and the one that separates it from every
/// cheaper thing that looks the same standing still:
///
/// - `Static` — the platform never moves in rapier at all, so the cargo stays
///   put and travels **0**.
/// - drive stage missing — identical to the above.
/// - `set_body_pose` instead of aiming — the platform teleports with **zero
///   velocity**, so the contact has no tangential motion to read, friction has
///   nothing to transmit, and the cargo is left behind.
///
/// The oracle is what the artist would SEE: the cargo arrives near the far end
/// of the platform's travel, not near where it started. The bar is deliberately
/// loose (friction is a real coefficient, not a weld — the box slips a little)
/// but the fossa between "carried" and "left behind" is the platform's whole
/// 1.0 m of travel.
#[test]
fn a_kinematic_platform_carries_what_rests_on_it() {
    let travel = 1.0f32;
    let ticks = 120u64;

    let (mut sim, platform, cargo) = platform_scene(BodyKind::Kinematic);
    let mut bridge = PhysicsBridge::new();
    // Let it settle onto the platform first, so the contact exists before the
    // ride starts. A box dropped and dragged in the same instant would measure
    // the settling, not the carrying.
    for tick in 1..=30 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let start_x = pose(&sim, cargo).0;
    for i in 1..=ticks {
        set_x(&mut sim, platform, travel * i as f32 / ticks as f32);
        bridge.dispatch(&mut sim, true, 30 + i);
    }
    let carried = pose(&sim, cargo).0 - start_x;

    assert!(
        carried > travel * 0.5,
        "the cargo travelled {carried:.3} m of the platform's {travel:.3} m — a \
         kinematic body that does not push is a Static body with a nicer name"
    );

    // The control: the SAME scene with a static platform, whose Transform is
    // moved identically. Nothing must ride along — this is what proves the
    // measurement above is reading the drive stage and not gravity, drift, or
    // a lucky initial nudge.
    let (mut sim, platform, cargo) = platform_scene(BodyKind::Static);
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=30 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let start_x = pose(&sim, cargo).0;
    for i in 1..=ticks {
        set_x(&mut sim, platform, travel * i as f32 / ticks as f32);
        bridge.dispatch(&mut sim, true, 30 + i);
    }
    let unmoved = (pose(&sim, cargo).0 - start_x).abs();
    assert!(
        unmoved < travel * 0.1,
        "a STATIC platform carried its cargo {unmoved:.3} m — the control is \
         supposed to be the thing that does not happen"
    );
}

/// **The readback never writes a pose the SCENE owns.**
///
/// `readback` asks `BodyKind::solver_owns_pose` and skips everything else. The
/// obvious place to test that is a kinematic body — and it is the wrong place:
/// the drive stage puts the body exactly where the `Transform` said, and the
/// wrapper's final sub-step is the target *bit-exactly*, so writing it back
/// writes the identical number. Deleting the guard passes such a gate.
///
/// The case where it is observable is a **static** body moved during PLAY.
/// `settle` only runs while paused, so rapier still holds the old pose; without
/// the guard the readback pushes that stale pose back over the artist's edit,
/// and the wall the artist just dragged snaps home every frame.
#[test]
fn the_readback_does_not_erase_a_scene_owned_pose() {
    let mut sim = SimWorld::new();
    let wall = spawn_body(
        &mut sim,
        BodyKind::Static,
        ColliderShape::Cuboid {
            half_x: 0.2,
            half_y: 1.0,
        },
        Vec2::new(0.0, 0.0),
    );
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=10 {
        bridge.dispatch(&mut sim, true, tick);
    }
    // The artist drags the wall while the clock is running.
    set_x(&mut sim, wall, 4.0);
    for tick in 11..=20 {
        bridge.dispatch(&mut sim, true, tick);
    }
    assert_eq!(
        pose(&sim, wall).0,
        4.0,
        "the readback pushed rapier's stale pose over an edit the artist made \
         — a body the solver does not own must not be written back"
    );
}

/// **A kinematic body shoves a dynamic one out of its way.**
///
/// The other half of "pushes": not friction along a surface but a head-on
/// contact. A ram sweeps across a resting ball; the ball must end up beyond
/// where the ram's leading face reached, i.e. it was moved rather than
/// tunnelled through or ignored.
#[test]
fn a_kinematic_body_shoves_a_dynamic_one() {
    let mut sim = SimWorld::new();
    // A floor to rest on, so the ball's motion is horizontal and unambiguous.
    spawn_body(
        &mut sim,
        BodyKind::Static,
        ColliderShape::Cuboid {
            half_x: 20.0,
            half_y: 0.1,
        },
        Vec2::new(0.0, 0.0),
    );
    let ram = spawn_body(
        &mut sim,
        BodyKind::Kinematic,
        ColliderShape::Cuboid {
            half_x: 0.2,
            half_y: 0.5,
        },
        Vec2::new(-2.0, 0.6),
    );
    let ball = spawn_body(
        &mut sim,
        BodyKind::Dynamic,
        ColliderShape::Ball { radius: 0.25 },
        Vec2::new(0.0, 0.35),
    );

    let mut bridge = PhysicsBridge::new();
    for tick in 1..=30 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let rest_x = pose(&sim, ball).0;
    // Sweep the ram from x=-2 to x=0 — it arrives where the ball is standing.
    for i in 1..=100u64 {
        set_x(&mut sim, ram, -2.0 + 2.0 * i as f32 / 100.0);
        bridge.dispatch(&mut sim, true, 30 + i);
    }
    let pushed = pose(&sim, ball).0 - rest_x;
    assert!(
        pushed > 0.1,
        "the ram passed through the ball ({pushed:.3} m of displacement) — a \
         kinematic body must carry momentum into the contact, not haunt it"
    );
}
