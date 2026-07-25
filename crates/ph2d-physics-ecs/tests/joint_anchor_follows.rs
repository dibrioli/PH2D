//! **The gold-standard joint anchor: it is authored BODY-LOCAL, so it follows
//! the body** (ADR-0131, the Wave-1 column).
//!
//! The old model stored the anchor as a single WORLD point (the joint entity's
//! `Transform`) and re-derived the body-local anchor every reconcile against the
//! bodies' poses. Moving a body left the world point put and slid the local
//! anchor along the body — measured at 2 m of slide on a body 0.2 m tall. The
//! fix stores `local_a`/`local_b` per body (rapier/Box2D/Unity's native pair), so
//! a body move carries its anchor by construction, and a reposition gesture
//! (dot / Position / re-pick) is the ONLY thing that re-derives it.
//!
//! These gates pin all three halves: a body move must NOT slide the anchor
//! (solver side), the display pivot must FOLLOW the body (the dot), and marking
//! the joint un-anchored must RE-DERIVE from the new pivot (reposition).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
};

/// A static Hook above, a dynamic Plank to its right, pinned at the plank's LEFT
/// end (one metre below the hook). The plank's centre is 0.5 m right of the
/// anchor, so "where on the plank is the pin" is a claim with teeth.
fn pendulum() -> SimWorld {
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
        Transform::from_translation(Vec2::new(0.0, 6.0)),
    ));
    sim.world_mut().spawn((
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
        Transform::from_translation(Vec2::new(0.5, 5.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Pin"),
        PhysicsJoint {
            body_a: stable_name_id("Hook"),
            body_b: stable_name_id("Plank"),
            kind: JointKind::Pin,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(0.0, 5.0)),
    ));
    sim
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity exists")
}

fn pos(sim: &mut SimWorld, name: &str) -> [f32; 2] {
    let e = named(sim, name);
    let t = sim.world().get::<Transform>(e).expect("transform");
    [t.translation.x, t.translation.y]
}

fn set_plank_y(sim: &mut SimWorld, y: f32) {
    let e = named(sim, "Plank");
    sim.world_mut()
        .get_mut::<Transform>(e)
        .expect("t")
        .translation
        .y = y;
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// Where the pin sits RELATIVE to the plank — the invariant a body move must not
/// change. `joint_anchors` returns `(world_a on Hook, world_b on Plank)`; the
/// second is the plank's anchor in world, and its offset from the plank's centre
/// is where on the body the pin is glued.
fn anchor_rel_to_plank(sim: &mut SimWorld, bridge: &PhysicsBridge) -> [f32; 2] {
    let world_b = bridge.joint_anchors().next().expect("one joint").1;
    let p = pos(sim, "Plank");
    [world_b[0] - p[0], world_b[1] - p[1]]
}

/// **A body move does not slide the anchor** (the solver side of the fix).
///
/// Mutation-tested: forcing `reconcile_joints` to re-derive every reconcile
/// (ignore `joint.anchored`, always take the seed branch) reproduces the old
/// slide — the relative anchor moves +2 m — and this goes red.
#[test]
fn the_anchor_follows_the_body_when_it_moves() {
    let mut sim = pendulum();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0); // seed the body-local anchors at rest

    let rel_before = anchor_rel_to_plank(&mut sim, &bridge);

    // Author: drag the plank 2 m down on the canvas (paused).
    set_plank_y(&mut sim, 3.0);
    bridge.dispatch(&mut sim, false, 0);

    let rel_after = anchor_rel_to_plank(&mut sim, &bridge);
    let slide = dist(rel_before, rel_after);
    assert!(
        slide < 1e-4,
        "the pin must stay glued to the same spot on the plank, but it slid {slide:.3} m \
         (before {rel_before:?}, after {rel_after:?})"
    );
}

fn set_hook_y(sim: &mut SimWorld, y: f32) {
    let e = named(sim, "Hook");
    sim.world_mut()
        .get_mut::<Transform>(e)
        .expect("t")
        .translation
        .y = y;
}

/// **The display pivot follows body A** (the dot side of the fix). The joint's
/// `Transform.translation` — what the anchor dot and Inspector Position read — is
/// synced to `bodyA · local_a` at rest, so it tracks the body its anchor is glued
/// to. The SINGLE dot follows body A on purpose: a Pin's two anchors are one
/// point at rest and diverge only when a body is moved, and showing BOTH ends is
/// the second draggable handle of a later wave; for now the one dot is body A's.
///
/// Moving body A (the hook) here — repositioning the whole pendulum's pivot — must
/// carry the dot with it. Mutation-tested: removing the `sync_joint_pivots` call
/// leaves the joint's `Transform` frozen at its authored value while the solver's
/// anchor follows, so the dot would sit where the pivot no longer is — red.
#[test]
fn the_display_pivot_follows_body_a() {
    let mut sim = pendulum();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);

    // The pin was authored at (0, 5); after seeding, the display pivot is there.
    let pivot_before = pos(&mut sim, "Pin");
    assert!(
        dist(pivot_before, [0.0, 5.0]) < 1e-4,
        "the display pivot should start at the authored anchor, got {pivot_before:?}"
    );

    // Move body A (the hook) 2 m down. The pin is glued to the hook, so the world
    // pivot must move down with it: (0, 5) -> (0, 3).
    set_hook_y(&mut sim, 4.0);
    bridge.dispatch(&mut sim, false, 0);

    let pivot_after = pos(&mut sim, "Pin");
    assert!(
        dist(pivot_after, [0.0, 3.0]) < 1e-4,
        "the display pivot must follow body A to (0, 3), got {pivot_after:?}"
    );
}

/// **Re-authoring the pivot re-glues the bodies** (the reposition side). A
/// gesture that moves the pivot (dot drag / Position edit / re-pick) writes the
/// joint's `Transform` and sets `anchored = false`; the next reconcile re-derives
/// the body-local anchors from the new pivot, so the pin moves to the new spot on
/// the body.
///
/// Mutation-tested: if reconcile ignored `anchored` and NEVER re-derived (always
/// took the stored-local branch), the reposition would be dropped and the pin
/// would stay put — this goes red.
#[test]
fn re_authoring_the_pivot_re_glues_the_bodies() {
    let mut sim = pendulum();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0); // seed at (0, 5): rel = (-0.5, 0)

    // Reposition the pivot to the plank's RIGHT end (its centre is at x=0.5, so
    // the right end is x=1.0). Simulate the shell gesture: write the pivot into
    // the joint's Transform and mark it un-anchored.
    let joint = named(&mut sim, "Pin");
    {
        let mut t = sim.world_mut().get_mut::<Transform>(joint).expect("t");
        t.translation = Vec2::new(1.0, 5.0);
    }
    {
        let mut j = sim.world_mut().get_mut::<PhysicsJoint>(joint).expect("j");
        j.anchored = false;
    }
    bridge.dispatch(&mut sim, false, 0);

    // The pin is now glued to the plank's right end: rel = (+0.5, 0).
    let rel = anchor_rel_to_plank(&mut sim, &bridge);
    assert!(
        dist(rel, [0.5, 0.0]) < 1e-4,
        "re-authoring the pivot to the plank's right end should re-glue there \
         (rel = (0.5, 0)), got {rel:?}"
    );
}
