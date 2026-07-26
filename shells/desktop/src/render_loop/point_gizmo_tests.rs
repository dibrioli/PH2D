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
    assert!(handles.iter().any(|h| h.kind == PointHandleKind::AnchorA));
    assert!(handles.iter().any(|h| h.kind == PointHandleKind::AnchorB));
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
    sorted.sort_by_key(|h| (h.key, h.kind));
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
    assert_eq!(handles[0].kind, PointHandleKind::AnchorA);
    assert_eq!(handles[0].key, j.to_bits());
    assert_eq!(handles[0].world, [0.0, 6.0]);
}

/// **Nothing to grab publishes no view.** An empty list would paint nothing and
/// register nothing anyway; `None` says so at the boundary instead of shipping
/// an empty struct every frame of every scene without joints.
#[test]
fn an_empty_scene_publishes_no_view() {
    assert!(build_point_view(Vec::new(), &camera(), window(), None, false).is_none());
    let v = build_point_view(
        vec![PointHandle {
            key: 1,
            kind: PointHandleKind::AnchorA,
            world: [1.0, 2.0],
        }],
        &camera(),
        window(),
        Some([3.0, 3.0]),
        false,
    )
    .expect("a handle publishes a view");
    assert_eq!(v.handles.len(), 1);
    assert_eq!(v.snap_world, Some([3.0, 3.0]));
    assert!(!v.inert, "nothing armed => the handles are grabbable");
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
        map.insert(ph2d_editor::gizmo::point_handle_id(h.key, h.kind), *h);
    }
    for h in &handles {
        let id = ph2d_editor::gizmo::point_handle_id(h.key, h.kind);
        let (e, kind) = resolve_anchor_hit(&map, id).expect("a painted handle resolves");
        assert_eq!(e, j);
        assert_eq!(kind, h.kind, "the resolved kind must be the one painted");
        assert_eq!(
            anchor_side(kind) == Some(JointSide::A),
            h.kind == PointHandleKind::AnchorA,
            "an anchor kind must map back to its side"
        );
    }
    assert!(resolve_anchor_hit(&map, ph2d_editor::NodeId(7)).is_none());
}

// ── W-J3: the parameter grips ────────────────────────────────────────────────

/// A hinge with limits + a spring, so both grip families are on screen at once.
fn param_rig() -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    body(&mut sim, "HingePost", BodyKind::Static, [0.0, 0.0]);
    body(&mut sim, "HingeArm", BodyKind::Dynamic, [1.0, 0.0]);
    sim.world_mut().spawn((
        Name::new("Hinge".to_string()),
        PhysicsJoint {
            body_a: stable_name_id("HingePost"),
            body_b: stable_name_id("HingeArm"),
            kind: JointKind::Pin,
            limits_enabled: true,
            limit_min: -0.5,
            limit_max: 0.9,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    body(&mut sim, "SpringPost", BodyKind::Static, [5.0, 0.0]);
    body(&mut sim, "SpringBob", BodyKind::Dynamic, [6.0, 0.0]);
    sim.world_mut().spawn((
        Name::new("Spring".to_string()),
        PhysicsJoint {
            body_a: stable_name_id("SpringPost"),
            body_b: stable_name_id("SpringBob"),
            kind: JointKind::Spring,
            rest_length: 1.0,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(5.0, 0.0)),
    ));
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge)
}

fn kinds(hs: &[PointHandle]) -> Vec<PointHandleKind> {
    let mut k: Vec<_> = hs.iter().map(|h| h.kind).collect();
    k.sort_unstable();
    k
}

/// **Each joint gets the grips its own kind has**, and no others: a hinge with
/// limits gets two walls, a spring gets its ring. A parameter that a joint does
/// not use is not a grip that quietly authors an unused field.
#[test]
fn a_hinge_gets_two_walls_and_a_spring_gets_its_ring() {
    let (_sim, bridge) = param_rig();
    let hs = joint_param_handles(&bridge, &camera(), window(), true, true);
    assert_eq!(
        kinds(&hs),
        vec![
            PointHandleKind::LimitMin,
            PointHandleKind::LimitMax,
            PointHandleKind::Length
        ],
        "got {hs:?}"
    );
}

/// **A free hinge has no walls to grip.** `limits_enabled` off means the arc is
/// not drawn, so there is nothing there to grab.
#[test]
fn a_hinge_without_limits_has_no_walls() {
    let (mut sim, mut bridge) = param_rig();
    let hinge = sim
        .world_mut()
        .query::<(Entity, &Name)>()
        .iter(sim.world())
        .find(|(_, n)| n.as_str() == "Hinge")
        .map(|(e, _)| e)
        .expect("the hinge");
    if let Some(mut c) = sim.world_mut().get_mut::<PhysicsJoint>(hinge) {
        c.limits_enabled = false;
    }
    bridge.dispatch(&mut sim, false, 0);
    let hs = joint_param_handles(&bridge, &camera(), window(), true, true);
    assert_eq!(kinds(&hs), vec![PointHandleKind::Length], "got {hs:?}");
}

/// **The grips follow the joint overlay's visibility.**
///
/// They are grips on ITS geometry — the arc, the ring. With `B` off there is no
/// arc on screen, and a dot that moves an invisible line is a control the artist
/// cannot reason about. (The ANCHOR dots are drawn by the gizmo itself, so they
/// are not gated: they are always visible.)
///
/// Mutation-tested: dropping the `show_overlay` guard leaves the grips live with
/// nothing drawn under them.
#[test]
fn the_parameter_grips_are_not_offered_with_the_overlay_hidden() {
    let (_sim, bridge) = param_rig();
    assert!(joint_param_handles(&bridge, &camera(), window(), false, true).is_empty());
    // …nor while the clock runs, for the reason the anchors are not.
    assert!(joint_param_handles(&bridge, &camera(), window(), true, false).is_empty());
}

/// **A wall's grip re-projects to the pixel the arc drew it at.** The handle
/// carries WORLD (one projection serves every kind), and the arc lives at a
/// fixed SCREEN radius, so the publish unprojects — a round trip that must land
/// back where it started or the hit rect drifts off the tick as you zoom.
#[test]
fn a_walls_grip_round_trips_through_world_back_to_its_pixel() {
    let (_sim, bridge) = param_rig();
    let (cam, win) = (camera(), window());
    let hs = joint_param_handles(&bridge, &cam, win, true, true);
    for h in hs.iter().filter(|h| h.kind != PointHandleKind::Length) {
        let v = bridge
            .joint_views()
            .find(|v| v.entity.to_bits() == h.key)
            .expect("the joint");
        let l = v.limits.expect("limits");
        let i = usize::from(h.kind == PointHandleKind::LimitMax);
        let drawn = crate::render_loop::physics_overlay_joint_glyphs::limit_end_screen(
            &cam, win, v.anchor_a, v.angle_a, l[i],
        );
        let (sx, sy) = cam.world_to_screen(h.world, win);
        let d = (f64::from(sx) - drawn.x).hypot(f64::from(sy) - drawn.y);
        assert!(
            d < 0.05,
            "the {:?} grip re-projected {d:.4} px away from the wall it grips",
            h.kind
        );
    }
}
