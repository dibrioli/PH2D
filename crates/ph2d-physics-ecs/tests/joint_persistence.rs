//! **W3 — joints in the project file, and why `PROJECT_SCHEMA` did NOT move.**
//!
//! The wave plan said to bump 21 → 22. The count says zero, and the count is
//! what decides (*"o valor se CONTA, não se escolhe"*).
//!
//! A component's blob in the snapshot is keyed by
//! `stable_type_id = blake3(canonical_name)[..8]` — derived from the **name**,
//! not from a position in the registry. So registering
//! `ph2d::physics::PhysicsJoint` mints a brand-new id and **moves nothing**:
//! every blob in every project already on disk keeps the id it was written
//! with. That is the opposite of W2c, which appended a `layer` field *inside*
//! `Collider`, where postcard is positional and the bump was mandatory.
//!
//! Bumping anyway is not free and not neutral: a schema mismatch **refuses the
//! whole file** (`project.rs`), so it would throw away every project the artist
//! has saved — to improve the error message in the one direction that cannot
//! work either way (an older build reading a file that has joints in it).
//!
//! These two gates are that reasoning made falsifiable. If a future change
//! really does move the layout, the first one goes red and the bump is owed.

use ph2d_core::Vec2;
use ph2d_ecs::scene::{
    ComponentRegistry, WorldSnapshot, snapshot_to_world, stable_type_id, world_to_snapshot,
};
use ph2d_ecs::{
    Entity, Name, SimWorld, Transform, TransformPropagationState, WorklistBuf, stable_name_id,
};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody,
    register_physics_components,
};

/// The registry as this build knows it.
fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    register_physics_components(&mut reg);
    reg
}

/// The registry as the build BEFORE this wave knew it — everything except the
/// joint. Writing a snapshot through this is how a project file from schema 21
/// is reproduced without keeping a binary fixture around.
fn registry_before_w3() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    reg.register::<RigidBody>("ph2d::physics::RigidBody");
    reg.register::<Collider>("ph2d::physics::Collider");
    reg
}

fn snapshot(sim: &mut SimWorld, reg: &ComponentRegistry) -> WorldSnapshot {
    let mut snap = WorldSnapshot::new();
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::default();
    world_to_snapshot(sim.world(), &mut prop, &mut worklist, reg, &mut snap).expect("snapshot");
    snap
}

fn scene(with_joint: bool) -> SimWorld {
    let mut sim = SimWorld::new();
    for (name, y) in [("Hook", 6.0f32), ("Plank", 5.0)] {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, y)),
        ));
    }
    if with_joint {
        sim.world_mut().spawn((
            Name::new("Pin"),
            PhysicsJoint {
                body_a: stable_name_id("Hook"),
                body_b: stable_name_id("Plank"),
                kind: JointKind::Rope,
                max_length: 3.25,
                ..PhysicsJoint::default()
            },
            Transform::from_translation(Vec2::new(0.0, 6.0)),
        ));
    }
    sim
}

/// **A project saved before joints existed still opens.**
///
/// This is the claim the missing schema bump rests on, driven end to end: a
/// snapshot written by a registry that has never heard of `PhysicsJoint`, read
/// back by the registry that has. Nothing in it may shift.
#[test]
fn a_project_written_before_joints_existed_still_loads() {
    let mut before = scene(false);
    let old_file = snapshot(&mut before, &registry_before_w3());

    let mut sim = SimWorld::new();
    snapshot_to_world(sim.world_mut(), &old_file, &registry())
        .expect("a file written before joints existed must still load");

    let mut q = sim.world_mut().query::<(&Name, &Collider, &Transform)>();
    let rows: Vec<_> = q
        .iter(sim.world())
        .map(|(n, c, t)| (n.as_str().to_string(), c.shape, t.translation.y))
        .collect();
    assert_eq!(rows.len(), 2, "the old file lost entities on the way in");
    for (name, shape, y) in rows {
        assert_eq!(
            shape,
            ColliderShape::Ball { radius: 0.25 },
            "{name}'s collider came back wrong — a blob was read with the \
             wrong type id, which is exactly the layout break a schema bump \
             would have been owed for"
        );
        assert!(y == 6.0 || y == 5.0, "{name} came back at y={y}");
    }
}

/// **Registering the joint moved no other component's id.**
///
/// The mechanism behind the gate above, asserted directly: the ids are a hash
/// of the canonical name, so they are a property of the name and of nothing
/// else — not of how many components happen to be registered, nor of the order
/// they were registered in.
#[test]
fn registering_the_joint_moves_no_other_components_id() {
    let before = registry_before_w3();
    let after = registry();
    for name in [
        "ph2d::ecs::Transform",
        "ph2d::ecs::Name",
        "ph2d::physics::RigidBody",
        "ph2d::physics::Collider",
    ] {
        let id = stable_type_id(name);
        assert!(
            before.get_by_id(id).is_some() && after.get_by_id(id).is_some(),
            "{name} does not resolve to the same id on both sides of W3 — every \
             project file on disk is keyed by that number"
        );
    }
    assert!(
        before
            .get_by_id(stable_type_id("ph2d::physics::PhysicsJoint"))
            .is_none(),
        "the 'before' registry is not actually before anything"
    );
}

/// **A joint round-trips through the snapshot with every parameter intact.**
///
/// The other half: the component is registered, so it travels in the
/// `WorldSnapshot` — which is undo AND save — with no code written on either
/// side. The gate reads back a **non-default** joint, because a default one
/// would be reproduced just as well by a component that was silently dropped
/// and re-created (the `Locked`/`GroupedChildren`/`VecPathRef` bug).
#[test]
fn a_joint_round_trips_through_the_snapshot_with_its_parameters() {
    let reg = registry();
    let mut sim = scene(true);
    let file = snapshot(&mut sim, &reg);

    let mut back = SimWorld::new();
    snapshot_to_world(back.world_mut(), &file, &reg).expect("load");

    let mut q = back.world_mut().query::<(Entity, &PhysicsJoint)>();
    let found: Vec<PhysicsJoint> = q.iter(back.world()).map(|(_, j)| *j).collect();
    assert_eq!(found.len(), 1, "the joint did not survive the round trip");
    let j = found[0];
    assert_eq!(
        j.kind,
        JointKind::Rope,
        "the joint came back as another kind"
    );
    assert_eq!(j.max_length, 3.25, "the rope's length was not preserved");
    assert_eq!(j.body_a, stable_name_id("Hook"));
    assert_eq!(j.body_b, stable_name_id("Plank"));
    assert!(
        j.names_two_bodies(),
        "the restored joint no longer names two bodies"
    );
}
