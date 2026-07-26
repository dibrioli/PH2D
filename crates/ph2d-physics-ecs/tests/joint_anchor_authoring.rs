//! **Authoring a joint anchor, per end** — the bridge's anchor door (W-J2).
//!
//! W-AnchorFollow made the anchor authored body-local state and gave the A end a
//! grabbable dot. Body B's anchor stayed whatever the seed policy produced, and
//! nothing in the editor could move it. These gates pin the door that fixes that,
//! and — the part with teeth — that authoring ONE end leaves the other alone.
//!
//! The failure they exist to catch is not a crash: it is a silent reset. The
//! `anchored` sentinel is joint-wide, so any reposition that clears it re-derives
//! BOTH locals from the seed policy. Before the B handle existed that was
//! invisible (B had no authored value to lose); with it, dragging the A dot would
//! throw away the anchor the artist had just placed on the other body, with every
//! other test in the suite still green.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, JointSide, PhysicsBridge, PhysicsJoint,
    RigidBody, ShapeDesc,
};

/// A static Post and a dynamic Arm, roped together. A **Rope** on purpose: it is
/// the two-ended kind, so the seed policy puts B at the arm's own centre and any
/// authored B value is visibly different from it.
fn rig() -> SimWorld {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Post"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.2,
                half_y: 0.2,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Arm"),
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
        Transform::from_translation(Vec2::new(1.0, 5.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Link"),
        PhysicsJoint {
            body_a: stable_name_id("Post"),
            body_b: stable_name_id("Arm"),
            kind: JointKind::Rope,
            max_length: 2.0,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(0.0, 6.0)),
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

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// **Authoring the A anchor does not move the B anchor.** The headline of the
/// wave: the two ends are independent, so a gesture on one is not a gesture on
/// both.
///
/// Mutation-tested: making `set_joint_anchor_world` clear `PhysicsJoint::anchored`
/// (the old reposition mechanism) makes the next reconcile re-seed B from the
/// policy — measured, B snaps back to the arm's centre, 0.400 m away — and this
/// goes red.
#[test]
fn authoring_the_a_anchor_leaves_the_b_anchor_where_it_was() {
    let mut sim = rig();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let joint = named(&mut sim, "Link");

    // Place B away from the arm's centre — the left end of the arm.
    bridge.set_joint_anchor_world(&mut sim, joint, JointSide::B, [0.6, 5.0]);
    bridge.dispatch(&mut sim, false, 0);
    let b_before = bridge
        .joint_anchor_world(&sim, joint, JointSide::B)
        .expect("authored B anchor");
    assert!(
        dist(b_before, [0.6, 5.0]) < 1e-4,
        "the B anchor must land where it was put, got {b_before:?}"
    );

    // Now author the OTHER end.
    bridge.set_joint_anchor_world(&mut sim, joint, JointSide::A, [0.0, 6.3]);
    bridge.dispatch(&mut sim, false, 0);

    let b_after = bridge
        .joint_anchor_world(&sim, joint, JointSide::B)
        .expect("B anchor still authored");
    let moved = dist(b_before, b_after);
    assert!(
        moved < 1e-4,
        "authoring the A end moved the B end by {moved:.3} m ({b_before:?} -> {b_after:?}) \
         — the joint-wide re-seed is back"
    );
    let a_after = bridge
        .joint_anchor_world(&sim, joint, JointSide::A)
        .expect("A anchor");
    assert!(
        dist(a_after, [0.0, 6.3]) < 1e-4,
        "and the A end must be where it was put, got {a_after:?}"
    );
}

/// **The authored B anchor is what the SOLVER holds** — not just what the
/// component says. A rope hangs from the point it is tied to, so tying it to the
/// arm's left end must swing the arm differently from tying it to the centre.
///
/// Mutation-tested: dropping the `local_b` write in the door leaves the seeded
/// centre anchor and the two runs become identical.
#[test]
fn the_solver_honours_the_authored_b_anchor() {
    let settle = |anchor_b: Option<[f32; 2]>| -> [f32; 2] {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        let joint = named(&mut sim, "Link");
        if let Some(p) = anchor_b {
            bridge.set_joint_anchor_world(&mut sim, joint, JointSide::B, p);
        }
        for tick in 1..=180 {
            bridge.dispatch(&mut sim, true, tick);
        }
        let arm = named(&mut sim, "Arm");
        let t = sim.world().get::<Transform>(arm).expect("arm pose");
        [t.translation.x, t.translation.y]
    };
    let centre_tied = settle(None);
    let end_tied = settle(Some([1.5, 5.0])); // the arm's RIGHT end
    let apart = dist(centre_tied, end_tied);
    assert!(
        apart > 0.2,
        "tying the rope to the arm's end must hang it differently from tying it to \
         its centre, but the two settled {apart:.4} m apart ({centre_tied:?} vs {end_tied:?})"
    );
}

/// **An authored anchor survives a rewind.** The rewind rebuilds the world from
/// each body's rest description and re-attaches the joints from the component, so
/// an anchor that only lived in the live rapier joint would silently revert on the
/// first scrub.
#[test]
fn an_authored_anchor_survives_a_rewind() {
    let mut sim = rig();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let joint = named(&mut sim, "Link");
    bridge.set_joint_anchor_world(&mut sim, joint, JointSide::B, [1.4, 5.0]);
    for tick in 1..=40 {
        bridge.dispatch(&mut sim, true, tick);
    }
    // Scrub back to the start: the world is rebuilt from rest.
    bridge.dispatch(&mut sim, true, 0);
    let b = bridge
        .joint_anchor_world(&sim, joint, JointSide::B)
        .expect("B anchor after rewind");
    assert!(
        dist(b, [1.4, 5.0]) < 1e-3,
        "the authored B anchor must come back with the rewind, got {b:?}"
    );
}

/// **The A end of a joint whose bodies are not resolved is still authorable** —
/// through the same door, as the world `Transform` the first seed will convert.
///
/// This is not a nicety: a joint is created before its bodies are picked in the
/// canvas-pick flow (W-JointAuthoring), and placing its pivot beforehand is how
/// the seed knows where to glue it. Losing that would be a regression from the
/// single-handle world.
#[test]
fn the_a_end_of_a_dormant_joint_is_authorable_and_b_is_not() {
    let mut sim = SimWorld::new();
    let joint = sim
        .world_mut()
        .spawn((
            Name::new("Link"),
            PhysicsJoint::default(), // names no bodies
            Transform::from_translation(Vec2::new(1.0, 1.0)),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);

    assert_eq!(
        bridge.joint_anchor_world(&sim, joint, JointSide::A),
        Some([1.0, 1.0]),
        "a dormant joint's A anchor is its authored pivot"
    );
    assert_eq!(
        bridge.joint_anchor_world(&sim, joint, JointSide::B),
        None,
        "there is no body B, so there is no B anchor to draw a handle for"
    );
    assert!(bridge.set_joint_anchor_world(&mut sim, joint, JointSide::A, [2.5, -1.0]));
    assert_eq!(
        bridge.joint_anchor_world(&sim, joint, JointSide::A),
        Some([2.5, -1.0])
    );
    assert!(
        !bridge.set_joint_anchor_world(&mut sim, joint, JointSide::B, [2.5, -1.0]),
        "writing the B end of a bodyless joint must refuse rather than invent a frame"
    );
}

/// **The snap targets are the collider's own points, in world** — through the
/// body's rest pose and its collider offset, so a rotated body's corners are
/// where the outline draws them and not where an axis-aligned guess would put
/// them.
///
/// Mutation-tested: dropping the pose transform leaves the targets in local space
/// (the far corner lands at `(0.5, 0.1)` instead of on the rotated body) and this
/// goes red.
#[test]
fn the_snap_targets_sit_on_the_rotated_offset_collider() {
    let mut sim = rig();
    // Rotate the arm a quarter turn and push its collider up 0.3 m.
    let arm = named(&mut sim, "Arm");
    {
        let mut t = sim.world_mut().get_mut::<Transform>(arm).expect("t");
        t.rotation = std::f32::consts::FRAC_PI_2;
    }
    {
        let mut c = sim.world_mut().get_mut::<Collider>(arm).expect("c");
        c.offset = [0.0, 0.3];
    }
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let joint = named(&mut sim, "Link");

    let mut out = [[0.0f32; 2]; ShapeDesc::MAX_SNAP_POINTS];
    let n = bridge.joint_snap_targets(&sim, joint, JointSide::B, &mut out);
    assert_eq!(n, 9, "a cuboid offers nine snap points");

    // The arm sits at (1, 5), rotated 90°, collider offset (0, 0.3) in body local
    // → the collider centre is at (1 - 0.3, 5).
    let centre = out[0];
    assert!(
        dist(centre, [0.7, 5.0]) < 1e-4,
        "the collider centre must be the rotated offset position, got {centre:?}"
    );
    // Its half-extents are (0.5, 0.1); rotated 90°, the long axis runs along Y.
    let far = out[1..n]
        .iter()
        .copied()
        .max_by(|a, b| {
            dist(*a, centre)
                .partial_cmp(&dist(*b, centre))
                .expect("finite")
        })
        .expect("corners");
    let reach = dist(far, centre);
    let expected = (0.5f32 * 0.5 + 0.1 * 0.1).sqrt();
    assert!(
        (reach - expected).abs() < 1e-4,
        "the farthest corner must be a corner of THIS collider ({expected:.4} m from its \
         centre), got {reach:.4} m at {far:?}"
    );
    // And it is genuinely rotated: the long reach is along Y, not X.
    assert!(
        (far[1] - centre[1]).abs() > (far[0] - centre[0]).abs(),
        "a body turned a quarter turn must offer its corners turned too, got {far:?}"
    );
}

/// **The displayed pivot agrees with the authored A anchor.** The dot, the
/// Inspector's Position field and the stored local are three views of one value;
/// the sync writes the pivot from the same door the handle reads.
#[test]
fn the_display_pivot_agrees_with_the_authored_a_anchor() {
    let mut sim = rig();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let joint = named(&mut sim, "Link");
    bridge.set_joint_anchor_world(&mut sim, joint, JointSide::A, [-0.15, 6.2]);
    bridge.dispatch(&mut sim, false, 0);

    let t = sim.world().get::<Transform>(joint).expect("pivot");
    let pivot = [t.translation.x, t.translation.y];
    let anchor = bridge
        .joint_anchor_world(&sim, joint, JointSide::A)
        .expect("A anchor");
    assert!(
        dist(pivot, anchor) < 1e-4,
        "the drawn pivot {pivot:?} and the authored anchor {anchor:?} must be one value"
    );
    assert!(dist(pivot, [-0.15, 6.2]) < 1e-4);
}
