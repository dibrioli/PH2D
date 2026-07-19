//! **Sensors / triggers** — ADR-0131 W7. A sensor collider passes through (no
//! contact forces) but the solver reports what overlaps it, which the bridge
//! publishes as a trigger state. These gates drive the real sim: a body falls
//! THROUGH a sensor and is detected, a solid collider blocks and never
//! triggers, a scene with no sensors reports nothing, and disarming physics
//! clears the state.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

fn floor(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 50.0,
                half_y: 0.2,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
}

/// A static box at `y`, either a sensor or a solid platform.
fn bar(sim: &mut SimWorld, y: f32, half_y: f32, is_sensor: bool) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 1.0,
                    half_y,
                },
                is_sensor,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, y)),
        ))
        .id()
}

fn ball(sim: &mut SimWorld) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 5.0)),
        ))
        .id()
}

fn run(mut sim: SimWorld, ticks: u64) -> (SimWorld, PhysicsBridge) {
    let mut bridge = PhysicsBridge::new();
    for t in 1..=ticks {
        bridge.dispatch(&mut sim, true, t);
    }
    (sim, bridge)
}

/// **A sensor detects a body inside it but does not block it.**
///
/// The sensor sits at `y ≈ 1.05` (spanning 0.3..1.8, clear of the floor top at
/// 0.2). The ball drops from 5, PASSES THROUGH the sensor, and rests on the
/// floor at `y ≈ 0.5` — where it still overlaps the sensor's lower edge, so at
/// rest the sensor is triggered with the ball inside it.
///
/// Mutation-tested twice over: dropping `.sensor(desc.is_sensor)` in `spawn_body`
/// turns the bar solid, so the ball rests ON it (≈ 2.1, not 0.5) AND produces a
/// contact instead of an intersection (nothing triggers); dropping the
/// `rebuild_triggers` call leaves the trigger state empty.
#[test]
fn a_sensor_detects_a_body_inside_it_but_does_not_block_it() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    let sensor = bar(&mut sim, 1.05, 0.75, true);
    let b = ball(&mut sim);

    let (_, bridge) = run(sim, 300);

    let ball_y = bridge.body_pose(b).expect("ball has a body").1;
    assert!(
        (ball_y - 0.5).abs() < 0.1,
        "the ball rested at y={ball_y}; a sensor must NOT block it — it should pass \
         through and land on the floor at ≈ 0.5, not rest on the bar (≈ 2.1)"
    );
    assert!(
        bridge.is_triggered(sensor),
        "the sensor did not fire with a body inside it"
    );
    assert!(
        bridge.bodies_inside(sensor).contains(&b),
        "the ball is inside the sensor but not in its bodies_inside list: {:?}",
        bridge.bodies_inside(sensor)
    );
    assert_eq!(
        bridge.triggered_sensors(),
        vec![sensor],
        "exactly the sensor should be triggered"
    );
}

/// **A solid collider blocks the ball and never triggers** — the control that
/// gives the sensor test its meaning. Same geometry, `is_sensor = false`: the
/// ball rests ON the bar, and nothing is a trigger.
#[test]
fn a_solid_collider_blocks_and_never_triggers() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    let platform = bar(&mut sim, 1.05, 0.75, false);
    let b = ball(&mut sim);

    let (_, bridge) = run(sim, 300);

    let ball_y = bridge.body_pose(b).expect("ball has a body").1;
    assert!(
        ball_y > 1.5,
        "the ball rested at y={ball_y}; a SOLID bar must block it (rest ≈ 2.1), not \
         let it fall through"
    );
    assert!(
        !bridge.is_triggered(platform),
        "a solid collider reported a trigger — a solid pair produces a contact, never \
         an intersection"
    );
    assert!(
        bridge.triggered_sensors().is_empty(),
        "a scene whose only bar is solid has a triggered sensor: {:?}",
        bridge.triggered_sensors()
    );
}

/// **A scene with no sensors reports no triggers** — the no-cost guard. The
/// `intersecting_body_pairs` query is empty, so `rebuild_triggers` returns
/// before it allocates, and a non-trigger scene pays nothing.
#[test]
fn a_scene_with_no_sensors_reports_no_triggers() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    let _ = ball(&mut sim);

    let (_, bridge) = run(sim, 300);
    assert!(
        bridge.triggered_sensors().is_empty(),
        "a scene with no sensors reported triggers: {:?}",
        bridge.triggered_sensors()
    );
}

/// **Disarming physics clears the trigger state.** With the solver off (`hold`),
/// no sim runs, so a lingering "something is inside" would light a sensor that
/// nothing is inside anymore.
#[test]
fn disarming_physics_clears_the_triggers() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    let sensor = bar(&mut sim, 1.05, 0.75, true);
    let _ = ball(&mut sim);

    let mut bridge = PhysicsBridge::new();
    for t in 1..=300 {
        bridge.dispatch(&mut sim, true, t);
    }
    assert!(
        bridge.is_triggered(sensor),
        "precondition: the sensor should be triggered before disarming"
    );

    bridge.hold(&mut sim, 301);
    assert!(
        !bridge.is_triggered(sensor),
        "the trigger state survived a hold (physics disarmed) — it must clear"
    );
    assert!(bridge.triggered_sensors().is_empty());
}
