//! W1 persistence + lifecycle gates.
//!
//! 1. **Round-trip seam:** `RigidBody`/`Collider`, once registered, survive
//!    `world_to_snapshot` → `snapshot_to_world` (undo + save travel on that
//!    snapshot). This exercises the seam the registration promises — a
//!    dropped `reg.register` line makes the component vanish → RED.
//! 2. **rebuild:** the derived world is forgotten on project load / undo
//!    restore, then re-derived from components.
//! 3. **self-heal:** when entities respawn with fresh bits (undo/load), the
//!    dangling handle map is reconciled — stale bodies dropped, new spawned.

use ph2d_core::Vec2;
use ph2d_ecs::scene::{
    ComponentRegistry, WorldSnapshot, register_ecs_components, snapshot_to_world, world_to_snapshot,
};
use ph2d_ecs::{SimWorld, Transform, TransformPropagationState, WorklistBuf};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody, register_physics_components,
};

fn drop_ball(sim: &mut SimWorld) -> ph2d_ecs::Entity {
    sim.world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.0, 3.0)),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                density: 1.0,
            },
        ))
        .id()
}

#[test]
fn physics_components_survive_a_world_snapshot_round_trip() {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    register_physics_components(&mut reg);

    let mut src = SimWorld::new();
    src.world_mut().spawn((
        Transform::from_translation(Vec2::new(1.0, 2.0)),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 3.0,
                half_y: 0.5,
            },
            density: 2.0,
        },
    ));

    let mut state = TransformPropagationState::new(src.world_mut());
    let mut worklist = WorklistBuf::new();
    let mut snap = WorldSnapshot::new();
    world_to_snapshot(src.world(), &mut state, &mut worklist, &reg, &mut snap).expect("snapshot");

    let mut dst = SimWorld::new();
    let spawned = snapshot_to_world(dst.world_mut(), &snap, &reg).expect("restore");
    assert_eq!(spawned.len(), 1);
    let e = spawned[0];

    let rb = *dst
        .world()
        .get::<RigidBody>(e)
        .expect("RigidBody survived the round trip (registered?)");
    let col = *dst
        .world()
        .get::<Collider>(e)
        .expect("Collider survived the round trip (registered?)");
    assert_eq!(rb.kind, BodyKind::Static);
    assert_eq!(col.density, 2.0);
    assert!(
        matches!(col.shape, ColliderShape::Cuboid { half_x, half_y } if half_x == 3.0 && half_y == 0.5)
    );
}

#[test]
fn rebuild_drops_the_derived_world_and_reconcile_re_derives() {
    let mut sim = SimWorld::new();
    drop_ball(&mut sim);
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=10u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    assert_eq!(bridge.body_count(), 1);
    assert_eq!(bridge.last_stepped(), 10);

    bridge.rebuild();
    assert_eq!(bridge.body_count(), 0, "rebuild kept a stale body");
    assert_eq!(bridge.last_stepped(), 0, "rebuild kept the old tick");

    // Next dispatch re-derives from the still-present components.
    bridge.dispatch(&mut sim, false, 0);
    assert_eq!(
        bridge.body_count(),
        1,
        "reconcile did not re-derive the body"
    );
}

#[test]
fn reconcile_self_heals_when_entities_respawn_with_new_bits() {
    // Undo/load despawns then respawns entities with fresh bits; the old
    // handle map dangles. reconcile must drop the stale body and spawn the
    // new one — never keep both.
    let mut sim = SimWorld::new();
    let e1 = drop_ball(&mut sim);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    assert_eq!(bridge.body_count(), 1);

    sim.world_mut().despawn(e1);
    let e2 = drop_ball(&mut sim);
    assert_ne!(e1.to_bits(), e2.to_bits(), "respawn should give new bits");

    bridge.dispatch(&mut sim, false, 0);
    assert_eq!(
        bridge.body_count(),
        1,
        "self-heal failed: a stale body was kept or the new one was missed"
    );
}
