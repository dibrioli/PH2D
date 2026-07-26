//! **Criar apontando** — os gates da W-J4.
//!
//! O que estes provam não é "um joint nasceu": é **onde as âncoras nascem**. Pela
//! seleção não há pontos a oferecer e a política de semeadura decide; pelo gesto
//! há dois, e eles são o que o artista quis dizer. Um gate que só contasse joints
//! ficaria verde sobre um gesto que joga os dois pontos no lixo.

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, PhysicsJoint, RigidBody};

fn body(sim: &mut SimWorld, name: &str, at: [f32; 2]) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(name.to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(at[0], at[1])),
        ))
        .id()
}

fn joint_of(sim: &SimWorld, e: Entity) -> PhysicsJoint {
    *sim.world().get::<PhysicsJoint>(e).expect("joint")
}

/// The world point an anchor lands at, through the bridge's own door — the same
/// answer the canvas handle and the solver read.
fn anchor(sim: &mut SimWorld, joint: Entity, side: ph2d_physics_ecs::JointSide) -> [f32; 2] {
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(sim, false, 0);
    bridge
        .joint_anchor_world(sim, joint, side)
        .expect("anchor resolves")
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    (b[0] - a[0]).hypot(b[1] - a[1])
}

/// **The anchors are born AT the gesture's points, not at the centres.**
///
/// The headline. A rope drawn from the top-left corner of A to the bottom-right
/// corner of B must attach THERE — the seed policy would put B's end at the
/// body's centre, which is the *"anchors are born in centres"* failure this wave
/// exists to remove.
///
/// Mutation-tested: passing `None` for `at` (the selection route's shape) makes
/// B's anchor land on the body's centre, and this goes red by 0.5 m.
#[test]
fn a_drawn_joint_anchors_at_the_two_points_of_the_gesture() {
    let mut sim = SimWorld::new();
    let a = body(&mut sim, "A", [0.0, 0.0]);
    let b = body(&mut sim, "B", [3.0, 0.0]);
    let press = [-0.5, 0.5]; // A's top-left corner
    let release = [3.5, -0.5]; // B's bottom-right corner

    let j = crate::render_loop::inspector_joint::create_joint_at(
        &mut sim,
        a.to_bits(),
        b.to_bits(),
        JointKind::Rope,
        Some((press, release)),
    )
    .expect("the gesture creates a joint");

    assert!(
        joint_of(&sim, j).anchored,
        "a joint born from a gesture is ALREADY anchored — going through the seed \
         would throw the two points away"
    );
    let wa = anchor(&mut sim, j, ph2d_physics_ecs::JointSide::A);
    let wb = anchor(&mut sim, j, ph2d_physics_ecs::JointSide::B);
    assert!(
        dist(wa, press) < 1e-4,
        "the A anchor must sit where the press was, got {wa:?}"
    );
    assert!(
        dist(wb, release) < 1e-4,
        "the B anchor must sit where the release was, got {wb:?} (the seed policy \
         would put it at B's centre, {:?})",
        [3.0, 0.0]
    );
}

/// **A shared-point kind is ONE place, and it is the press.** Two bodies in one
/// spot is what a pin is, so the release only names the partner.
#[test]
fn a_drawn_pin_puts_both_anchors_on_the_press_point() {
    for kind in [JointKind::Pin, JointKind::Weld] {
        let mut sim = SimWorld::new();
        let a = body(&mut sim, "A", [0.0, 0.0]);
        let b = body(&mut sim, "B", [1.0, 0.0]);
        let press = [0.4, 0.2];
        let j = crate::render_loop::inspector_joint::create_joint_at(
            &mut sim,
            a.to_bits(),
            b.to_bits(),
            kind,
            Some((press, [1.4, -0.3])),
        )
        .expect("joint");
        let wa = anchor(&mut sim, j, ph2d_physics_ecs::JointSide::A);
        let wb = anchor(&mut sim, j, ph2d_physics_ecs::JointSide::B);
        assert!(
            dist(wa, press) < 1e-4,
            "{kind:?} A at the press, got {wa:?}"
        );
        assert!(
            dist(wb, press) < 1e-4,
            "{kind:?} B must share the press point too, got {wb:?}"
        );
    }
}

/// **The selection route is UNCHANGED.** `create_joint` still seeds, so the
/// button behaves exactly as it did before the gesture existed — the two routes
/// are one function with an `Option`, and the `None` arm must be the old
/// behaviour to the letter.
#[test]
fn the_selection_route_still_seeds_its_anchors() {
    let mut sim = SimWorld::new();
    let a = body(&mut sim, "A", [0.0, 0.0]);
    let b = body(&mut sim, "B", [3.0, 0.0]);
    let j = crate::render_loop::inspector_joint::create_joint(
        &mut sim,
        a.to_bits(),
        b.to_bits(),
        JointKind::Rope,
    )
    .expect("joint");
    assert!(
        !joint_of(&sim, j).anchored,
        "the selection route has no points to offer, so the SEED must still run"
    );
    let wb = anchor(&mut sim, j, ph2d_physics_ecs::JointSide::B);
    assert!(
        dist(wb, [3.0, 0.0]) < 1e-4,
        "and the seed puts a rope's B end at the body's centre, got {wb:?}"
    );
}

/// **The chain is N−1 joints over the selection's ORDER.**
///
/// Mutation-tested: `windows(2)` → joining every body to the FIRST makes a star
/// (still 3 joints for 4 bodies) and the per-pair check below goes red.
#[test]
fn a_chain_of_four_bodies_is_three_joints_linked_in_order() {
    let mut sim = SimWorld::new();
    let names = ["L0", "L1", "L2", "L3"];
    let order: Vec<u64> = names
        .iter()
        .enumerate()
        .map(|(i, n)| body(&mut sim, n, [i as f32, 0.0]).to_bits())
        .collect();

    let (made, last) = join_chain(&mut sim, &order, JointKind::Rope);
    assert_eq!(made, 3, "four bodies chain with three joints");
    assert!(last.is_some(), "the last link is selectable");

    // Each joint binds CONSECUTIVE bodies — a star would bind everything to L0.
    let mut q = sim.world_mut().query::<&PhysicsJoint>();
    let pairs: Vec<(u64, u64)> = q.iter(sim.world()).map(|j| (j.body_a, j.body_b)).collect();
    for i in 0..3 {
        let want = (stable_name_id(names[i]), stable_name_id(names[i + 1]));
        assert!(
            pairs.contains(&want),
            "the chain must link {} to {}; got {pairs:?}",
            names[i],
            names[i + 1]
        );
    }
}

/// **Two bodies through the chain door is exactly one joint** — the pre-W-J4
/// behaviour of the button, unchanged, which is what makes it safe for the same
/// button to serve both counts.
#[test]
fn two_bodies_still_make_one_joint() {
    let mut sim = SimWorld::new();
    let order = vec![
        body(&mut sim, "A", [0.0, 0.0]).to_bits(),
        body(&mut sim, "B", [1.0, 0.0]).to_bits(),
    ];
    assert_eq!(join_chain(&mut sim, &order, JointKind::Pin).0, 1);
}

/// **A body cannot be joined to itself, by either route.** The gesture's own
/// refusal is a toast at the release; this pins the door underneath it, so a
/// path that skipped the toast still cannot make a dormant joint.
#[test]
fn a_body_is_never_joined_to_itself() {
    let mut sim = SimWorld::new();
    let a = body(&mut sim, "A", [0.0, 0.0]);
    assert!(
        crate::render_loop::inspector_joint::create_joint_at(
            &mut sim,
            a.to_bits(),
            a.to_bits(),
            JointKind::Pin,
            Some(([0.0, 0.0], [0.0, 0.0])),
        )
        .is_none()
    );
    let order = vec![a.to_bits(), a.to_bits()];
    assert_eq!(join_chain(&mut sim, &order, JointKind::Pin).0, 0);
}

/// **The rubber band is the gesture's only mark, and it dies with it.** No
/// gesture, no band — otherwise a stale line would describe a joint nobody is
/// making.
#[test]
fn the_band_exists_only_while_the_gesture_does() {
    assert!(band(None).is_none());
    let d = JointDraw {
        body_a: Entity::from_bits(1),
        from: [1.0, 2.0],
        to: [3.0, 4.0],
    };
    assert_eq!(band(Some(d)), Some(([1.0, 2.0], [3.0, 4.0])));
}

/// **A gesture whose body A was deleted underneath it draws nothing.**
///
/// The band is anchored to a body, so if that body goes the line describes a
/// place that is no longer anything. (Deleting mid-drag is reachable: the
/// Hierarchy's delete is a key away.)
#[test]
fn a_gesture_whose_body_vanished_draws_no_band() {
    let mut sim = SimWorld::new();
    let a = body(&mut sim, "A", [0.0, 0.0]);
    let d = Some(JointDraw {
        body_a: a,
        from: [0.0, 0.0],
        to: [1.0, 1.0],
    });
    assert!(body_alive(&sim, d), "alive while the body is there");
    sim.world_mut().despawn(a);
    assert!(!body_alive(&sim, d), "and not once it is gone");
    // …and with no gesture at all the question is vacuously true (nothing to
    // invalidate), which is what lets the caller ask it unconditionally.
    assert!(body_alive(&sim, None));
}
