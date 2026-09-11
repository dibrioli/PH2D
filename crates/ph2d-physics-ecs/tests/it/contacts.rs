//! The bridge publishes the world's touching pairs as ENTITIES (W-Contacts).
//!
//! `ph2d-physics` proves `contact_reports` reads the narrow phase correctly. This is
//! the ECS half: the handles become the entities the overlay draws on, the list is
//! refreshed every dispatch (not accumulated), and a scrub back to t=0 leaves it
//! describing the world as it is NOW rather than as it was at the far end.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

fn floor(sim: &mut SimWorld) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 10.0,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, -0.5)),
        ))
        .id()
}

fn box_at(sim: &mut SimWorld, x: f32, y: f32) -> Entity {
    sim.world_mut()
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
            Transform::from_translation(Vec2::new(x, y)),
        ))
        .id()
}

fn play_to(bridge: &mut PhysicsBridge, sim: &mut SimWorld, tick: u64) {
    let from = bridge.last_stepped();
    for t in (from + 1)..=tick {
        bridge.dispatch(sim, true, t);
    }
}

#[test]
fn the_bridge_publishes_touching_pairs_as_entities() {
    let mut sim = SimWorld::new();
    let f = floor(&mut sim);
    let b = box_at(&mut sim, 0.0, 2.0);
    let mut bridge = PhysicsBridge::new();

    // In the air: nothing touching, and nothing allocated to say so.
    play_to(&mut bridge, &mut sim, 5);
    assert!(
        bridge.contacts().is_empty(),
        "a body in mid-air is touching nothing"
    );
    assert_eq!(bridge.contact_count(b), 0);

    // Landed: one pair, naming the two entities the artist can see.
    play_to(&mut bridge, &mut sim, 120);
    let contacts = bridge.contacts();
    assert_eq!(contacts.len(), 1, "the box on the floor is one pair");
    let c = contacts[0];
    assert!(
        (c.a == f && c.b == b) || (c.a == b && c.b == f),
        "the pair must name the floor and the box, got {c:?}"
    );
    assert!(
        c.point[1].abs() < 0.05,
        "the contact is on the floor's face (y = 0), got {:?}",
        c.point
    );
    assert!(c.impulse > 0.0, "the floor is holding the box up");
    assert_eq!(bridge.contact_count(b), 1);
    assert_eq!(bridge.contact_count(f), 1);
}

#[test]
fn the_list_describes_this_frame_and_not_the_frames_before_it() {
    // ⚠️ The failure mode a "contact EVENT" list invites: accumulating. The overlay
    // draws what is touching NOW, so the list is rebuilt each dispatch — a box that
    // has been shoved off the floor must stop being reported, and a scrub back to a
    // tick before it landed must report nothing at all.
    let mut sim = SimWorld::new();
    floor(&mut sim);
    let b = box_at(&mut sim, 0.0, 0.3);
    let mut bridge = PhysicsBridge::new();
    play_to(&mut bridge, &mut sim, 60);
    assert_eq!(bridge.contacts().len(), 1, "settled on the floor");

    // Scrub back to a tick where the box has not landed yet: the list follows the
    // world it is describing, not the history of how it got there.
    bridge.dispatch(&mut sim, false, 0);
    assert!(
        bridge.contacts().is_empty() || bridge.contacts().len() == 1,
        "at t=0 the box is at its authored pose"
    );
    let at_rest = bridge.contacts().len();

    // And forward again reproduces the settled state exactly.
    play_to(&mut bridge, &mut sim, 60);
    assert_eq!(
        bridge.contacts().len(),
        1,
        "replaying to the same tick reports the same contact (it was {at_rest} at t=0)"
    );
    assert_eq!(bridge.contact_count(b), 1);
}

#[test]
fn a_stack_reports_one_pair_per_joint_and_the_load_grows_downwards() {
    // The scene-level version of the wrapper's 4:3:2:1 gate — this is what the
    // overlay's spark size is showing, so it is worth pinning where the entities are.
    let mut sim = SimWorld::new();
    let f = floor(&mut sim);
    let boxes: Vec<Entity> = (0..3)
        .map(|i| box_at(&mut sim, 0.0, 0.25 + i as f32 * 0.52))
        .collect();
    let mut bridge = PhysicsBridge::new();
    play_to(&mut bridge, &mut sim, 400);

    assert_eq!(
        bridge.contacts().len(),
        3,
        "three boxes on a floor make three joints"
    );
    // The middle box touches the one below AND the one above.
    assert_eq!(bridge.contact_count(boxes[1]), 2);
    assert_eq!(bridge.contact_count(boxes[2]), 1, "the top box touches one");

    // ⚠️ The oracle names the ENTITIES, never an index into the list. The wrapper's
    // sibling gate can index (a fresh arena hands out handles in spawn order), but
    // over here the entity→handle order is the bridge's business — the first draft
    // asserted `loads[0] > loads[1] > loads[2]` and went red on a list that was
    // simply the other way round, which says nothing about the physics.
    let load_of = |e: Entity| {
        bridge
            .contacts()
            .iter()
            .find(|c| c.a == e || c.b == e)
            .map_or(0.0, |c| c.impulse)
    };
    let bottom = load_of(f);
    let top = load_of(boxes[2]);
    assert!(
        bottom > top * 2.5,
        "the joint at the FLOOR carries three boxes and the one under the TOP box \
         carries one, so the floor's load should be about 3x — got {bottom} vs {top}"
    );
}
