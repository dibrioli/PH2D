//! **Higiene do par** (W-J8) — *Active*, *Collide Connected* and *Swap A↔B*.
//!
//! The first two are switches the bridge has to carry to the solver and a rewind
//! has to re-arm. The third is the interesting one: a swap has to be
//! **behaviour-preserving**, because the reason to press it is that the pair is
//! labelled the wrong way round — not that the hinge should open the other way.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, MotorMode, PhysicsBridge, PhysicsJoint, RigidBody,
};

fn body(sim: &mut SimWorld, name: &str, kind: BodyKind, at: Vec2, shape: ColliderShape) {
    sim.world_mut().spawn((
        Name::new(name),
        RigidBody { kind },
        Collider {
            shape,
            ..Collider::default()
        },
        Transform::from_translation(at),
    ));
}

fn ball(r: f32) -> ColliderShape {
    ColliderShape::Ball { radius: r }
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity exists")
}

/// A joint entity named `Pin` between `Post` and `Load`, with `j` as its state.
fn joint(sim: &mut SimWorld, j: PhysicsJoint, at: Vec2) {
    sim.world_mut()
        .spawn((Name::new("Pin"), j, Transform::from_translation(at)));
}

fn run(sim: &mut SimWorld, ticks: u64) -> PhysicsBridge {
    let mut bridge = PhysicsBridge::default();
    for t in 1..=ticks {
        bridge.dispatch(sim, true, t);
    }
    bridge
}

fn pose_y(sim: &mut SimWorld, name: &str) -> f32 {
    let e = named(sim, name);
    sim.world().get::<Transform>(e).unwrap().translation.y
}

fn rotation(sim: &mut SimWorld, name: &str) -> f32 {
    let e = named(sim, name);
    sim.world().get::<Transform>(e).unwrap().rotation
}

// ---------------------------------------------------------------- Active ----

/// **The Active switch reaches the solver, and a rewind re-arms it.**
///
/// The rewind half is not ceremony: `rebuild_from_rest` re-spawns every joint
/// from its descriptor, so a switch that lived anywhere but the descriptor would
/// come back ON after a scrub — and the artist would watch a joint they disabled
/// start holding again the moment they dragged the ruler.
#[test]
fn the_bridge_carries_active_to_the_solver_and_a_rewind_re_arms_it() {
    let mut sim = SimWorld::new();
    body(
        &mut sim,
        "Post",
        BodyKind::Static,
        Vec2::new(0.0, 6.0),
        ball(0.05),
    );
    body(
        &mut sim,
        "Load",
        BodyKind::Dynamic,
        Vec2::new(0.0, 5.0),
        ball(0.2),
    );
    joint(
        &mut sim,
        PhysicsJoint {
            body_a: stable_name_id("Post"),
            body_b: stable_name_id("Load"),
            kind: JointKind::Pin,
            active: false,
            ..PhysicsJoint::default()
        },
        Vec2::new(0.0, 6.0),
    );
    let mut bridge = run(&mut sim, 120);
    let fell = pose_y(&mut sim, "Load");
    assert!(
        fell < 0.0,
        "an inactive pin holds nothing; got y = {fell:.3}"
    );

    // Scrub back to the start and replay: the switch has to survive the rebuild.
    bridge.dispatch(&mut sim, true, 0);
    for t in 1..=120 {
        bridge.dispatch(&mut sim, true, t);
    }
    let again = pose_y(&mut sim, "Load");
    assert!(
        (again - fell).abs() < 0.05,
        "a rewind re-arms the switch: {fell:.3} then {again:.3}"
    );
}

/// The control, and it is the one that gives the gate above its teeth: the same
/// rig ACTIVE hangs where it was put.
#[test]
fn an_active_joint_holds() {
    let mut sim = SimWorld::new();
    body(
        &mut sim,
        "Post",
        BodyKind::Static,
        Vec2::new(0.0, 6.0),
        ball(0.05),
    );
    body(
        &mut sim,
        "Load",
        BodyKind::Dynamic,
        Vec2::new(0.0, 5.0),
        ball(0.2),
    );
    joint(
        &mut sim,
        PhysicsJoint {
            body_a: stable_name_id("Post"),
            body_b: stable_name_id("Load"),
            kind: JointKind::Pin,
            ..PhysicsJoint::default()
        },
        Vec2::new(0.0, 6.0),
    );
    run(&mut sim, 120);
    let y = pose_y(&mut sim, "Load");
    assert!(y > 4.0, "an active pin holds it up; got y = {y:.3}");
}

/// **An inactive joint is NOT a broken one** — and they write the same rapier
/// flag, which is exactly why this needs a gate.
///
/// `JointView::broken` was `!joint_is_enabled()`, full stop. The moment an
/// authored switch could disable a joint too, that expression started answering
/// *"is it disabled?"* while being named *"did it break?"* — so turning a joint
/// off would have painted it RED, with the six-pointed burst, telling the artist
/// their rig gave way under load when they had simply disarmed it.
///
/// Both directions are asserted because each alone is satisfiable by a constant:
/// `broken = false` always passes the first, `active = desc.enabled` alone
/// passes neither only when a break is present.
///
/// Mutation: `broken: !enabled` without the `&& j.rest.enabled` → the inactive
/// rig reports broken, red.
#[test]
fn an_inactive_joint_is_not_a_broken_one() {
    // (a) authored inactive: not in force, and NOT broken.
    let mut sim = SimWorld::new();
    body(
        &mut sim,
        "Post",
        BodyKind::Static,
        Vec2::new(0.0, 6.0),
        ball(0.05),
    );
    body(
        &mut sim,
        "Load",
        BodyKind::Dynamic,
        Vec2::new(0.0, 5.0),
        ball(0.2),
    );
    joint(
        &mut sim,
        PhysicsJoint {
            body_a: stable_name_id("Post"),
            body_b: stable_name_id("Load"),
            kind: JointKind::Pin,
            active: false,
            ..PhysicsJoint::default()
        },
        Vec2::new(0.0, 6.0),
    );
    let bridge = run(&mut sim, 60);
    let v = bridge.joint_views().next().expect("a view");
    assert!(!v.active, "the switch says it is not in force");
    assert!(!v.broken, "but it did not BREAK — nobody pulled it apart");

    // (b) authored active, torn apart by a load: in force, and broken.
    let mut sim = SimWorld::new();
    body(
        &mut sim,
        "Post",
        BodyKind::Static,
        Vec2::new(0.0, 6.0),
        ball(0.05),
    );
    body(
        &mut sim,
        "Load",
        BodyKind::Dynamic,
        Vec2::new(0.0, 5.0),
        ball(0.4),
    );
    joint(
        &mut sim,
        PhysicsJoint {
            body_a: stable_name_id("Post"),
            body_b: stable_name_id("Load"),
            kind: JointKind::Rope,
            max_length: 1.0,
            break_enabled: true,
            break_force: 0.5,
            ..PhysicsJoint::default()
        },
        Vec2::new(0.0, 6.0),
    );
    let bridge = run(&mut sim, 120);
    let v = bridge.joint_views().next().expect("a view");
    assert!(v.active, "it was authored in force");
    assert!(v.broken, "and the solver tore it apart");
}

// ------------------------------------------------------- CollideConnected ----

/// **Collide Connected reaches the solver** — the crate roped to a block rests
/// ON it, where the default lets it fall through.
#[test]
fn the_bridge_carries_collide_connected_to_the_solver() {
    let mut y = [0.0f32; 2];
    for (i, collide) in [true, false].into_iter().enumerate() {
        let mut sim = SimWorld::new();
        body(
            &mut sim,
            "Block",
            BodyKind::Static,
            Vec2::new(0.0, 0.0),
            ColliderShape::Cuboid {
                half_x: 1.0,
                half_y: 0.5,
            },
        );
        body(
            &mut sim,
            "Crate",
            BodyKind::Dynamic,
            Vec2::new(0.0, 2.0),
            ColliderShape::Cuboid {
                half_x: 0.4,
                half_y: 0.4,
            },
        );
        joint(
            &mut sim,
            PhysicsJoint {
                body_a: stable_name_id("Block"),
                body_b: stable_name_id("Crate"),
                kind: JointKind::Rope,
                max_length: 4.0,
                collide_connected: collide,
                ..PhysicsJoint::default()
            },
            Vec2::new(0.0, 0.0),
        );
        run(&mut sim, 240);
        y[i] = pose_y(&mut sim, "Crate");
    }
    assert!(
        y[0] > 0.5,
        "collide ON: rests on the block; got {:.3}",
        y[0]
    );
    assert!(y[1] < -3.0, "collide OFF: falls through; got {:.3}", y[1]);
}

// ------------------------------------------------------------------ Swap ----

/// **The pure half: a swap exchanges the two ends, the two anchors, and negates
/// everything measured between them.**
///
/// Stated on the component because that is where the whole operation lives — the
/// shell's edit arm is one line calling this, so a gate that drove the UI would
/// be testing the routing rather than the arithmetic.
#[test]
fn swapping_exchanges_the_pair_and_everything_measured_between_it() {
    let j = PhysicsJoint {
        body_a: 11,
        body_b: 22,
        local_a: [1.0, 2.0],
        local_b: [-3.0, 4.0],
        anchored: true,
        limit_min: -0.2,
        limit_max: 0.6,
        motor_speed: 2.0,
        motor_target: 0.5,
        ..PhysicsJoint::default()
    };
    let s = j.swapped();
    assert_eq!((s.body_a, s.body_b), (22, 11), "the two ends exchange");
    assert_eq!(
        (s.local_a, s.local_b),
        ([-3.0, 4.0], [1.0, 2.0]),
        "and each anchor travels WITH its body — it is stored in that body's frame"
    );
    assert_eq!(
        (s.limit_min, s.limit_max),
        (-0.6, 0.2),
        "the range is of `theta_b - theta_a`, so it MIRRORS"
    );
    assert_eq!(s.motor_speed, -2.0, "and the motor reverses");
    assert_eq!(s.motor_target, -0.5, "as does the servo's target");
    assert!(
        s.anchored,
        "and it stays anchored: the locals are still exactly right, only re-labelled"
    );
}

/// **Swapping twice is the identity.** The invariant that makes the button safe
/// to press while looking for the arrangement you want — and the one an
/// asymmetric compensation would break in silence.
#[test]
fn swapping_twice_is_the_identity() {
    let j = PhysicsJoint {
        body_a: 7,
        body_b: 9,
        local_a: [0.25, -1.5],
        local_b: [3.0, 0.125],
        anchored: true,
        limit_min: -0.75,
        limit_max: 0.25,
        motor_speed: -1.25,
        motor_target: 2.5,
        ..PhysicsJoint::default()
    };
    assert_eq!(j.swapped().swapped(), j);
}

/// **And the whole point: a swap does not change what the joint DOES.**
///
/// Three rigs, each exercising a quantity that a bare swap reverses (measured:
/// the motor to −4.000, the servo to −44.9998°, a stroke to the other end). The
/// swapped rig has to reproduce the authored one.
///
/// ⚠️ The oracle is the TRAJECTORY endpoint of the driven body in each rig, not
/// the joint's own numbers: comparing `swapped()`'s output against `swapped()`'s
/// rule would be the always-green shape this line has been caught by three times.
/// Here the solver is the judge.
///
/// Mutation: drop the negation of `motor_speed` → the wheel rig reads −229° of
/// travel against +229°, red. Drop the limit mirror → the plank rig settles at
/// −34.4° against −11.5°, red.
#[test]
fn a_swap_does_not_change_what_the_joint_does() {
    // (name, build the joint, read the answer)
    #[allow(clippy::type_complexity)]
    let rigs: [(
        &str,
        fn() -> (SimWorld, PhysicsJoint),
        fn(&mut SimWorld) -> f32,
    ); 3] = [
        (
            "motorised wheel",
            || {
                let mut sim = SimWorld::new();
                body(&mut sim, "Post", BodyKind::Static, Vec2::ZERO, ball(0.05));
                body(&mut sim, "Load", BodyKind::Dynamic, Vec2::ZERO, ball(0.5));
                let j = PhysicsJoint {
                    body_a: stable_name_id("Post"),
                    body_b: stable_name_id("Load"),
                    kind: JointKind::Pin,
                    motor_enabled: true,
                    motor_speed: 1.0,
                    motor_max_force: 100.0,
                    ..PhysicsJoint::default()
                };
                (sim, j)
            },
            |sim| rotation(sim, "Load"),
        ),
        (
            "servo holding a place",
            || {
                let mut sim = SimWorld::new();
                body(&mut sim, "Post", BodyKind::Static, Vec2::ZERO, ball(0.05));
                body(&mut sim, "Load", BodyKind::Dynamic, Vec2::ZERO, ball(0.5));
                let j = PhysicsJoint {
                    body_a: stable_name_id("Post"),
                    body_b: stable_name_id("Load"),
                    kind: JointKind::Pin,
                    motor_enabled: true,
                    motor_mode: MotorMode::Position,
                    motor_target: std::f32::consts::FRAC_PI_4,
                    motor_max_force: 100.0,
                    ..PhysicsJoint::default()
                };
                (sim, j)
            },
            |sim| rotation(sim, "Load"),
        ),
        (
            "asymmetric hinge limits",
            || {
                let mut sim = SimWorld::new();
                body(&mut sim, "Post", BodyKind::Static, Vec2::ZERO, ball(0.05));
                body(
                    &mut sim,
                    "Load",
                    BodyKind::Dynamic,
                    Vec2::new(1.0, 0.0),
                    ColliderShape::Cuboid {
                        half_x: 1.0,
                        half_y: 0.1,
                    },
                );
                let j = PhysicsJoint {
                    body_a: stable_name_id("Post"),
                    body_b: stable_name_id("Load"),
                    kind: JointKind::Pin,
                    limits_enabled: true,
                    limit_min: -0.2,
                    limit_max: 0.6,
                    ..PhysicsJoint::default()
                };
                (sim, j)
            },
            |sim| rotation(sim, "Load"),
        ),
    ];

    for (name, build, read) in rigs {
        let mut answers = [0.0f32; 2];
        for (i, swap) in [false, true].into_iter().enumerate() {
            let (mut sim, j) = build();
            joint(&mut sim, if swap { j.swapped() } else { j }, Vec2::ZERO);
            run(&mut sim, 180);
            answers[i] = read(&mut sim);
        }
        assert!(
            (answers[0] - answers[1]).abs() < 0.02,
            "{name}: a swap must not change what the joint does — \
             authored {:.4}, swapped {:.4}",
            answers[0],
            answers[1]
        );
        // And the rig has to CONTAIN the phenomenon: a quantity that never moves
        // is preserved by anything, including a swap that does nothing at all.
        assert!(
            answers[0].abs() > 0.05,
            "{name}: the fixture has to actually move, or it proves nothing; got {:.4}",
            answers[0]
        );
    }
}
