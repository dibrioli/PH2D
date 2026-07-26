//! **Which anchors get a handle** — the publish rule, headless.

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::{Locked, Name, Transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::Sprite;

fn camera() -> Camera2d {
    Camera2d {
        center: [0.0, 0.0],
        height_world: 10.0,
        ..Camera2d::default()
    }
}

fn window() -> WindowSize {
    WindowSize {
        width: 1000,
        height: 1000,
    }
}

fn body(sim: &mut SimWorld, name: &str, kind: BodyKind, at: [f32; 2]) {
    sim.world_mut().spawn((
        Name::new(name.to_string()),
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.3,
                half_y: 0.3,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(at[0], at[1])),
    ));
}

fn joint(sim: &mut SimWorld, name: &str, a: &str, b: &str, at: [f32; 2]) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(name.to_string()),
            PhysicsJoint {
                body_a: stable_name_id(a),
                body_b: stable_name_id(b),
                kind: JointKind::Rope,
                max_length: 2.0,
                ..PhysicsJoint::default()
            },
            Transform::from_translation(Vec2::new(at[0], at[1])),
        ))
        .id()
}

/// One post, one arm, one rope between them — plus a plain sprite that is not a
/// joint and must never grow a handle.
fn rig() -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    body(&mut sim, "Post", BodyKind::Static, [0.0, 6.0]);
    body(&mut sim, "Arm", BodyKind::Dynamic, [1.0, 5.0]);
    sim.world_mut().spawn((
        Name::new("JustASprite".to_string()),
        Transform::from_translation(Vec2::new(-3.0, 0.0)),
        Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]),
    ));
    let j = joint(&mut sim, "Link", "Post", "Arm", [0.0, 6.0]);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge, j)
}

/// **A joint nobody selected still gets its handles** — the whole of W-J2b.
///
/// A joint has no sprite, so a canvas click cannot reach it; when the handles
/// were selection-gated the only way to see them was to find the joint in the
/// Hierarchy first. Nothing here is selected, and both ends are offered.
///
/// Mutation-tested: re-adding a `selection` guard makes this return an empty
/// list and the gate goes red.
#[test]
fn an_unselected_joint_still_publishes_both_handles() {
    let (sim, bridge, j) = rig();
    let handles = joint_anchor_handles(&sim, &bridge, true);
    assert_eq!(
        handles.len(),
        2,
        "the rope's two ends must both be grabbable with nothing selected, got {handles:?}"
    );
    assert!(handles.iter().all(|h| h.key == j.to_bits()));
    assert!(handles.iter().any(|h| h.side == PointSide::A));
    assert!(handles.iter().any(|h| h.side == PointSide::B));
}

/// **Two joints are four handles, and a plain sprite is none of them.** The
/// list is per-joint, so the count is the fact that says the rule reads the
/// whole scene rather than one entity.
#[test]
fn every_joint_in_the_scene_is_offered_and_a_sprite_is_not() {
    let (mut sim, mut bridge, _) = rig();
    body(&mut sim, "Post2", BodyKind::Static, [4.0, 6.0]);
    body(&mut sim, "Arm2", BodyKind::Dynamic, [5.0, 5.0]);
    let j2 = joint(&mut sim, "Link2", "Post2", "Arm2", [4.0, 6.0]);
    bridge.dispatch(&mut sim, false, 0);

    let handles = joint_anchor_handles(&sim, &bridge, true);
    assert_eq!(handles.len(), 4, "two joints, two ends each: {handles:?}");
    assert_eq!(
        handles.iter().filter(|h| h.key == j2.to_bits()).count(),
        2,
        "the second joint must be offered too — it is the one that proves the rule is not \
         'the selected joint' wearing another name"
    );
}

/// **The order is deterministic.** It decides which of two overlapping joints
/// paints on top, and that must not depend on archetype layout.
#[test]
fn the_handle_order_is_stable() {
    let (sim, bridge, _) = rig();
    let a = joint_anchor_handles(&sim, &bridge, true);
    let b = joint_anchor_handles(&sim, &bridge, true);
    assert_eq!(a, b);
    let mut sorted = a.clone();
    sorted.sort_by_key(|h| (h.key, h.side));
    assert_eq!(a, sorted, "handles must come out sorted by (entity, side)");
}

/// **No handles while the clock runs.** During play the overlay draws the
/// SOLVER's anchors — the live ones — and a grabbable dot authoring against a
/// swinging pose would write an anchor nobody chose. `sync_joint_pivots` has
/// claimed this in its own doc since W-AnchorFollow; nothing enforced it until
/// W-J2.
///
/// Mutation-tested: dropping the `at_rest` guard returns the two handles.
#[test]
fn the_handles_are_not_offered_while_the_clock_runs() {
    let (sim, bridge, _) = rig();
    assert!(joint_anchor_handles(&sim, &bridge, false).is_empty());
}

/// **A locked joint is not offered a handle.** `open_drag` refuses a locked
/// entity, and a dot that paints, registers a hit and then declines the gesture
/// is worse than one that is not there.
#[test]
fn a_locked_joint_offers_nothing_to_grab() {
    let (mut sim, bridge, j) = rig();
    sim.world_mut().entity_mut(j).insert(Locked);
    assert!(joint_anchor_handles(&sim, &bridge, true).is_empty());
}

/// **A dormant joint keeps its A handle and is refused a B one.** Half-authored
/// (a body renamed, deleted, or never picked) is precisely the joint the artist
/// is in the middle of fixing, and its authored pivot is the `Transform` the
/// seed will convert. B has no body, so no anchor, so no handle.
#[test]
fn a_dormant_joints_a_end_is_offered_and_its_b_end_is_not() {
    let mut sim = SimWorld::new();
    body(&mut sim, "Post", BodyKind::Static, [0.0, 6.0]);
    let j = joint(&mut sim, "Link", "Post", "NoSuchBody", [0.0, 6.0]);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);

    let handles = joint_anchor_handles(&sim, &bridge, true);
    assert_eq!(handles.len(), 1, "only the A end, got {handles:?}");
    assert_eq!(handles[0].side, PointSide::A);
    assert_eq!(handles[0].key, j.to_bits());
    assert_eq!(handles[0].world, [0.0, 6.0]);
}

/// **Nothing to grab publishes no view.** An empty list would paint nothing and
/// register nothing anyway; `None` says so at the boundary instead of shipping
/// an empty struct every frame of every scene without joints.
#[test]
fn an_empty_scene_publishes_no_view() {
    assert!(build_point_view(Vec::new(), &camera(), window(), None).is_none());
    let v = build_point_view(
        vec![PointHandle {
            key: 1,
            side: PointSide::A,
            world: [1.0, 2.0],
        }],
        &camera(),
        window(),
        Some([3.0, 3.0]),
    )
    .expect("a handle publishes a view");
    assert_eq!(v.handles.len(), 1);
    assert_eq!(v.snap_world, Some([3.0, 3.0]));
}

/// **A hit id resolves back to the joint and the end it was painted for.** This
/// is the half that makes several joints safe: the ids are keyed, so the map is
/// the only thing that knows them, and it is filled by the same pass that
/// registers them.
#[test]
fn a_hit_id_resolves_to_its_joint_and_side() {
    let (sim, bridge, j) = rig();
    let handles = joint_anchor_handles(&sim, &bridge, true);
    let mut map = std::collections::BTreeMap::new();
    for h in &handles {
        map.insert(ph2d_editor::gizmo::point_handle_id(h.key, h.side), *h);
    }
    for h in &handles {
        let id = ph2d_editor::gizmo::point_handle_id(h.key, h.side);
        let (e, side) = resolve_anchor_hit(&map, id).expect("a painted handle resolves");
        assert_eq!(e, j);
        assert_eq!(
            side == JointSide::A,
            h.side == PointSide::A,
            "the resolved end must be the end that was painted"
        );
    }
    assert!(resolve_anchor_hit(&map, ph2d_editor::NodeId(7)).is_none());
}
