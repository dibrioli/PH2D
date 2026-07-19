//! **A physics body can be a CHILD** (ADR-0131 W5) — the solver speaks WORLD,
//! `Transform` is LOCAL, and for four waves the bridge treated them as the same
//! thing.
//!
//! For a root entity they *are* the same thing, which is why every gate, smoke
//! scene and demo that came before missed this: they all used root bodies.
//! Parent a physics object to anything in the Hierarchy — a gesture the app
//! fully supports — and both directions break at once. Measured before the fix:
//! a body authored at local `(0, 4)` under a parent at `(5, 0)` **simulated at
//! x = 0 while it drew at x = 5**. Nothing errored; the collider simply was not
//! where the sprite was.
//!
//! Five sites read or wrote a pose in the wrong space (spawn/rest, settle, the
//! kinematic aim, the joint anchor, and the readback). Each gets its own gate
//! here, because a single "the child ends up in the right place" assertion stays
//! green over most of them ([[feedback_layered_defenses_need_per_layer_gates]]).

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, SimWorld, Transform, parent_world_transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

const PARENT_X: f32 = 5.0;
const DROP_HEIGHT: f32 = 4.0;
const BALL_R: f32 = 0.3;
/// The floor spans `PARENT_X ± this`, so a body simulating at its *local* x = 0
/// misses it entirely and falls forever. That gap is the gate.
const FLOOR_HALF_X: f32 = 1.0;
const FLOOR_TOP: f32 = 0.0;

/// Where the artist actually SEES the entity: its local pose composed with the
/// parent chain, which is what the renderer draws.
fn drawn_at(sim: &SimWorld, e: Entity) -> (f32, f32, f32) {
    let local = *sim.world().get::<Transform>(e).unwrap();
    let t = Transform::compose(parent_world_transform(sim.world(), e), local);
    (t.translation.x, t.translation.y, t.rotation)
}

fn ball(kind: BodyKind) -> (RigidBody, Collider) {
    (
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Ball { radius: BALL_R },
            ..Collider::default()
        },
    )
}

/// A floor under `PARENT_X`, a rig at `(PARENT_X, 0)` rotated by `rot`, and a
/// ball hanging off it — drawn directly above the floor, with local coordinates
/// that are NOT above it.
///
/// ⚠️ **`rot` is not decoration, and leaving it out is how this fixture lies.**
/// The ball is authored at `R(-rot) · (0, DROP_HEIGHT)`, so correct composition
/// puts it over the floor whatever the rig's rotation is — which means an
/// implementation that composed only the parent's TRANSLATION would swing it off
/// the (deliberately narrow) floor and miss. Author the ball at a plain
/// `(0, DROP_HEIGHT)` instead and the polarity flips: dropping the rotation
/// LANDS it and handling it correctly misses. The scene this fixture mirrors
/// shipped with exactly that inversion
/// ([[feedback_moving_the_law_is_half_the_fix_the_fixture_must_contain_it]]).
fn parented_scene_rot(kind: BodyKind, rot: f32) -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: FLOOR_HALF_X,
                half_y: 0.2,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(PARENT_X, FLOOR_TOP - 0.2)),
    ));
    let parent = sim
        .world_mut()
        .spawn((Transform {
            translation: Vec2::new(PARENT_X, 0.0),
            rotation: rot,
            ..Transform::IDENTITY
        },))
        .id();
    let (rb, col) = ball(kind);
    let (sin_r, cos_r) = (rot.sin(), rot.cos());
    let child = sim
        .world_mut()
        .spawn((
            rb,
            col,
            Transform::from_translation(Vec2::new(DROP_HEIGHT * sin_r, DROP_HEIGHT * cos_r)),
            ChildOf(parent),
        ))
        .id();
    (sim, parent, child)
}

/// The un-rotated case — still worth having on its own, because it is the one
/// where "compose" and "add the parent's translation" agree, so it isolates the
/// space conversion from the rotation handling.
fn parented_scene(kind: BodyKind) -> (SimWorld, Entity, Entity) {
    parented_scene_rot(kind, 0.0)
}

/// A rig rotated far enough that composing only the translation misses the
/// floor by more than its half-width (`0.45 rad` swings the ball 1.3 m).
const RIG_ROT: f32 = 0.45;

/// **The body lands on the floor it is drawn above.**
///
/// The product-level statement of the whole wave. Before the fix the ball was
/// described to the solver at its LOCAL `(0, 4)`, so it fell down the x = 0 line
/// and sailed past a floor that only exists at x = 5 — forever.
#[test]
fn a_parented_body_falls_onto_the_floor_it_is_drawn_above() {
    // Both rigs, because they fail differently: the un-rotated one catches a
    // bridge that ignores the hierarchy at all, the rotated one catches a
    // bridge that composes only the parent's translation.
    for rot in [0.0, RIG_ROT] {
        let (mut sim, _p, child) = parented_scene_rot(BodyKind::Dynamic, rot);
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, true, 180);

        let (x, y, _) = drawn_at(&sim, child);
        assert!(
            (x - PARENT_X).abs() < 0.05,
            "rig rotated {rot}: the ball drifted to x = {x}; it should still be \
             above the floor at {PARENT_X}"
        );
        assert!(
            (y - (FLOOR_TOP + BALL_R)).abs() < 0.05,
            "rig rotated {rot}: the ball came to rest at y = {y}, not on the floor at {}",
            FLOOR_TOP + BALL_R
        );
    }
}

/// **What is drawn IS what is simulated** — the invariant the bug broke.
///
/// Checked every tick, not at the end: a readback that writes world into local
/// makes the drawn pose run away from the solver by one parent-offset per
/// frame, and an endpoint-only oracle over a body at rest can miss it
/// ([[feedback_a_snapshot_must_be_a_fixed_point_of_the_systems]] — the W1.5
/// lesson about damped systems re-converging).
#[test]
fn the_drawn_pose_and_the_simulated_pose_never_diverge() {
    let (mut sim, _p, child) = parented_scene_rot(BodyKind::Dynamic, RIG_ROT);
    let mut bridge = PhysicsBridge::new();

    let mut worst = 0.0f32;
    for tick in 1..=180 {
        bridge.dispatch(&mut sim, true, tick);
        let (dx, dy, _) = drawn_at(&sim, child);
        let (sx, sy, _) = bridge.body_pose(child).expect("the body exists");
        worst = worst.max((dx - sx).abs()).max((dy - sy).abs());
    }
    assert!(
        worst < 1e-3,
        "the sprite and the collider disagree by {worst} m — the body is not where it is drawn"
    );
}

/// **A paused child body is not teleported every frame.**
///
/// `settle` teleports only when the AUTHORED pose differs from the body's, and
/// `set_body_pose` zeroes the velocity. Comparing a LOCAL authored pose against
/// a WORLD body pose makes every child look permanently "moved by hand": it is
/// re-teleported and re-stilled on every paused frame, so Pause → Play restarts
/// the fall from a standstill and the pose is pinned to its local coordinates.
#[test]
fn a_paused_child_body_is_not_dragged_to_its_local_coordinates() {
    let (mut sim, _p, child) = parented_scene(BodyKind::Dynamic);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 60);
    let moving = drawn_at(&sim, child);

    // Now pause: many frames at the same tick, which is the `settle` path.
    for _ in 0..30 {
        bridge.dispatch(&mut sim, false, 60);
    }
    let after = drawn_at(&sim, child);
    assert!(
        (after.0 - moving.0).abs() < 1e-4 && (after.1 - moving.1).abs() < 1e-4,
        "pausing moved the body from {moving:?} to {after:?} — settle compared \
         a local pose against a world one"
    );
}

/// **A parented KINEMATIC platform carries its cargo.**
///
/// The aim is a world-space target. Driving a parented platform from its local
/// coordinates sends it along a path nobody authored — and because a kinematic
/// body pushes what it touches, the cargo goes with it.
#[test]
fn a_parented_kinematic_platform_carries_its_cargo() {
    let mut sim = SimWorld::new();
    let parent = sim
        .world_mut()
        .spawn((Transform::from_translation(Vec2::new(PARENT_X, 0.0)),))
        .id();
    let platform = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 1.0,
                    half_y: 0.2,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            ChildOf(parent),
        ))
        .id();
    // Cargo resting on the platform, in WORLD (a root body).
    let cargo = sim
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
            Transform::from_translation(Vec2::new(PARENT_X, 0.45)),
        ))
        .id();

    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 1);
    let start = drawn_at(&sim, cargo).0;

    // Slide the platform a metre along +x, in its LOCAL frame, over 60 ticks.
    for tick in 2..=61 {
        let f = (tick - 1) as f32 / 60.0;
        sim.world_mut()
            .get_mut::<Transform>(platform)
            .unwrap()
            .translation
            .x = f;
        bridge.dispatch(&mut sim, true, tick);
    }

    let travelled = drawn_at(&sim, cargo).0 - start;
    assert!(
        travelled > 0.8,
        "the cargo travelled {travelled} m of the platform's 1.0 m — a parented \
         platform aimed in the wrong space leaves its load behind"
    );
}

/// **A parent that cannot be inverted leaves the pose ALONE.**
///
/// Scaling a parent to zero collapses the subtree, so no local pose maps back
/// to the solver's world one. Dividing anyway writes `±inf`/`NaN`, and one
/// non-finite field poisons the whole subtree's `GlobalTransform` through
/// propagation — a scene-wide corruption caused by one body under one bad
/// parent. Refusing keeps the damage at "this body does not move".
#[test]
fn a_degenerate_parent_never_poisons_the_transform() {
    let mut sim = SimWorld::new();
    let parent = sim
        .world_mut()
        .spawn((Transform {
            scale: Vec2::new(0.0, 1.0),
            ..Transform::IDENTITY
        },))
        .id();
    let (rb, col) = ball(BodyKind::Dynamic);
    let child = sim
        .world_mut()
        .spawn((
            rb,
            col,
            Transform::from_translation(Vec2::new(0.0, DROP_HEIGHT)),
            ChildOf(parent),
        ))
        .id();

    let before = *sim.world().get::<Transform>(child).unwrap();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 120);
    let after = *sim.world().get::<Transform>(child).unwrap();

    assert!(after.is_finite(), "a non-finite pose was stored: {after:?}");
    assert_eq!(
        after, before,
        "the readback wrote through a parent it cannot invert"
    );
}

/// **A root body is byte-identical to what it was before this wave.**
///
/// `compose(IDENTITY, local)` and `inverse_compose(IDENTITY, world)` are both
/// exact, so routing root bodies through the new conversion must change
/// nothing. Without this, the wave could pay for child support with a silent
/// drift in the case that is 99% of every scene.
#[test]
fn a_root_body_is_unchanged_by_the_conversion() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 4.0,
                half_y: 0.2,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -0.2)),
    ));
    let (rb, col) = ball(BodyKind::Dynamic);
    let root_body = sim
        .world_mut()
        .spawn((rb, col, Transform::from_translation(Vec2::new(0.0, 4.0))))
        .id();

    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 180);

    let t = *sim.world().get::<Transform>(root_body).unwrap();
    let (sx, sy, sr) = bridge.body_pose(root_body).unwrap();
    assert_eq!(
        (t.translation.x, t.translation.y, t.rotation),
        (sx, sy, sr),
        "a root body's local Transform must still BE the solver's world pose, exactly"
    );
}

/// **A parented JOINT anchors where it is drawn.**
///
/// The joint entity's `Transform` *is* the anchor on body A, and a joint object
/// can be parented like anything else in the Hierarchy. Read locally, the pin
/// lands at the joint's local coordinates read as world ones — so the plank
/// hangs from a point the artist never marked, and the marker the overlay draws
/// sits somewhere else again.
///
/// Its own gate because the other five sites can all be correct while this one
/// is wrong: nothing else in the bridge reads the joint entity's pose.
#[test]
fn a_parented_joint_anchors_where_it_is_drawn() {
    use ph2d_ecs::Name;
    use ph2d_physics_ecs::{JointKind, PhysicsJoint};

    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Hook"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.05 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(PARENT_X, 6.0)),
    ));
    let plank = sim
        .world_mut()
        .spawn((
            Name::new("Plank"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.1,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(PARENT_X + 0.5, 5.0)),
        ))
        .id();

    // A rig node the artist parented the joint under. The joint's LOCAL pose is
    // (0, 5) — its WORLD pose is (PARENT_X, 5), the plank's left end.
    let rig = sim
        .world_mut()
        .spawn((Transform::from_translation(Vec2::new(PARENT_X, 0.0)),))
        .id();
    let joint = sim
        .world_mut()
        .spawn((
            Name::new("Pin"),
            PhysicsJoint {
                kind: JointKind::Pin,
                body_a: stable_name_id("Hook"),
                body_b: stable_name_id("Plank"),
                ..PhysicsJoint::default()
            },
            Transform::from_translation(Vec2::new(0.0, 5.0)),
            ChildOf(rig),
        ))
        .id();
    let anchor = drawn_at(&sim, joint);
    assert!(
        (anchor.0 - PARENT_X).abs() < 1e-5,
        "fixture is wrong: the joint should be drawn at x = {PARENT_X}"
    );

    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 120);

    // A pinned plank swings, but its left end stays on the anchor: the pin is a
    // constraint, so the distance from the anchor is the oracle, not the pose.
    let (px, py, _) = drawn_at(&sim, plank);
    let reach = ((px - anchor.0).powi(2) + (py - anchor.1).powi(2)).sqrt();
    assert!(
        reach < 0.75,
        "the plank ended {reach:.3} m from the anchor it is pinned to (max half-length \
         plus slack ≈ 0.5) — the joint attached at its LOCAL coordinates, not where it is drawn"
    );
}
