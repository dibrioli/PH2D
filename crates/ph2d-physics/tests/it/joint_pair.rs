//! **The pair's two switches** (W-J8) — *is this constraint in force?* and *do
//! the bodies it joins still bump into each other?*
//!
//! Both were already rapier's, and both were being answered for the artist:
//! `enabled` was never written and `contacts_enabled` was hardcoded `false` on
//! the strength of one W3 measurement. These gates pin what each switch buys,
//! with the numbers from `measure_joint_pair.rs`.

use ph2d_physics::{BodyDesc, JointDesc, JointKind, MotorDesc, MotorMode, PhysicsWorld, ShapeDesc};

type Handle = ph2d_physics::RigidBodyHandle;

fn body(
    world: &mut PhysicsWorld,
    kind: ph2d_physics::RigidBodyType,
    x: f32,
    y: f32,
    shape: ShapeDesc,
) -> Handle {
    world.spawn_body(BodyDesc {
        body_type: kind,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    })
}

fn dynamic(world: &mut PhysicsWorld, x: f32, y: f32, shape: ShapeDesc) -> Handle {
    body(world, ph2d_physics::RigidBodyType::Dynamic, x, y, shape)
}

fn fixed(world: &mut PhysicsWorld, x: f32, y: f32, shape: ShapeDesc) -> Handle {
    body(world, ph2d_physics::RigidBodyType::Fixed, x, y, shape)
}

fn steps(world: &mut PhysicsWorld, n: usize) {
    for _ in 0..n {
        world.step();
    }
}

/// Angular velocity of a body — the RATE, because a rotation angle wraps at ±π
/// and stops meaning anything past one turn.
fn angvel(world: &PhysicsWorld, h: Handle) -> f32 {
    world
        .body_snapshots()
        .into_iter()
        .find(|s| s.handle_index == h.into_raw_parts().0)
        .map(|s| s.angvel)
        .unwrap_or(0.0)
}

/// **An inactive joint holds nothing — and is still THERE.**
///
/// The two halves are one gate on purpose: "holds nothing" alone is also true of
/// a joint that was never built, and building-then-disabling is exactly what
/// separates *disabled* from *deleted*. If the bridge ever "optimised" this by
/// skipping the spawn, the load would still fall and only the second assertion
/// would notice — and the canvas would silently lose the joint the artist is
/// still authoring.
#[test]
fn an_inactive_joint_holds_nothing_and_is_still_there() {
    let mut world = PhysicsWorld::new();
    let hook = fixed(&mut world, 0.0, 0.0, ShapeDesc::Ball { radius: 0.05 });
    let load = dynamic(&mut world, 0.0, -1.0, ShapeDesc::Ball { radius: 0.2 });
    let joint = world
        .spawn_joint(
            hook,
            load,
            JointDesc {
                kind: JointKind::Pin,
                anchor_a: [0.0, 0.0],
                anchor_b: [0.0, 1.0],
                enabled: false,
                ..Default::default()
            },
        )
        .expect("an inactive joint is still SPAWNED");
    steps(&mut world, 120);

    let y = world.body_pose(load).map(|p| p.translation.y).unwrap();
    assert!(
        y < -5.0,
        "an inactive pin holds nothing, so the load falls; got y = {y:.3}"
    );
    assert_eq!(
        world.joint_count(),
        1,
        "and the joint is still in the world"
    );
    assert!(
        world.joint_anchors(joint).is_some(),
        "and still answers where it attaches — the canvas draws from this"
    );
    assert_eq!(
        world.joint_is_enabled(joint),
        Some(false),
        "and reports itself not in force"
    );
}

/// The control: the same rig, active, holds. Without it "the load fell" proves
/// nothing about the switch (an unanchored joint would fall too).
#[test]
fn the_same_joint_active_holds() {
    let mut world = PhysicsWorld::new();
    let hook = fixed(&mut world, 0.0, 0.0, ShapeDesc::Ball { radius: 0.05 });
    let load = dynamic(&mut world, 0.0, -1.0, ShapeDesc::Ball { radius: 0.2 });
    world
        .spawn_joint(
            hook,
            load,
            JointDesc {
                kind: JointKind::Pin,
                anchor_a: [0.0, 0.0],
                anchor_b: [0.0, 1.0],
                ..Default::default()
            },
        )
        .expect("joint");
    steps(&mut world, 120);
    let y = world.body_pose(load).map(|p| p.translation.y).unwrap();
    assert!(
        (y + 1.0).abs() < 0.01,
        "an active pin holds the load a metre down; got y = {y:.3}"
    );
}

/// **Collide Connected is what makes a jointed pair BUMP.**
///
/// MEASURED: a crate roped to a static block rests **on** it at `y = 0.899` with
/// contacts on, and falls straight **through** it to the rope's full 4 m
/// (`y = −4.000`) with them off. The rope is identical in both — only the switch
/// moves — so this is the switch and nothing else.
///
/// Mutation: `joint.contacts_enabled = false` hardcoded back → the ON row falls
/// through and the first assertion goes red.
#[test]
fn collide_connected_lets_a_jointed_pair_bump() {
    let mut y = [0.0f32; 2];
    for (i, contacts) in [true, false].into_iter().enumerate() {
        let mut world = PhysicsWorld::new();
        let base = fixed(
            &mut world,
            0.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 1.0,
                half_y: 0.5,
            },
        );
        let crated = dynamic(
            &mut world,
            0.0,
            2.0,
            ShapeDesc::Cuboid {
                half_x: 0.4,
                half_y: 0.4,
            },
        );
        world
            .spawn_joint(
                base,
                crated,
                JointDesc {
                    kind: JointKind::Rope,
                    max_length: 4.0,
                    contacts_enabled: contacts,
                    ..Default::default()
                },
            )
            .expect("joint");
        steps(&mut world, 240);
        y[i] = world.body_pose(crated).map(|p| p.translation.y).unwrap();
    }
    assert!(
        y[0] > 0.5,
        "contacts ON: the crate rests on the block it is roped to; got {:.3}",
        y[0]
    );
    assert!(
        y[1] < -3.0,
        "contacts OFF: it falls through, down to the rope's length; got {:.3}",
        y[1]
    );
}

/// **And the default is OFF because ON defeats a motor pinned inside its load.**
///
/// The rig that bought the default in W3, with its number: a hub pinned inside
/// the plank it drives, told 4 rad/s, reaches a relative **4.000** with contacts
/// off and **0.000** with them on — the solver spending the whole budget on a
/// permanent interpenetration.
///
/// This is why the switch has a default rather than being a question the artist
/// is asked: the common case (a chain link, a hub in a plank) overlaps by
/// construction.
#[test]
fn contacts_on_defeats_a_motor_pinned_inside_its_own_load() {
    let mut relative = [0.0f32; 2];
    for (i, contacts) in [false, true].into_iter().enumerate() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(0.0, 0.0);
        let hub = dynamic(&mut world, 0.0, 0.0, ShapeDesc::Ball { radius: 0.25 });
        let plank = dynamic(
            &mut world,
            0.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 1.0,
                half_y: 0.15,
            },
        );
        world
            .spawn_joint(
                hub,
                plank,
                JointDesc {
                    kind: JointKind::Pin,
                    motor: Some(MotorDesc {
                        mode: MotorMode::Velocity,
                        speed: 4.0,
                        target: 0.0,
                        max_force: 100.0,
                    }),
                    contacts_enabled: contacts,
                    ..Default::default()
                },
            )
            .expect("joint");
        steps(&mut world, 120);
        relative[i] = angvel(&world, plank) - angvel(&world, hub);
    }
    assert!(
        (relative[0] - 4.0).abs() < 0.1,
        "contacts OFF: the motor keeps its word (4 rad/s); got {:.3}",
        relative[0]
    );
    assert!(
        relative[1].abs() < 0.5,
        "contacts ON: it is defeated by the interpenetration; got {:.3}",
        relative[1]
    );
}
