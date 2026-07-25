//! **W3 — joints at the ECS seam.**
//!
//! The engine's own gates (`ph2d-physics/tests/joints.rs`) prove the solver
//! does what a pin, a spring and a rope are supposed to do. These prove the
//! part that only exists here: that a joint authored as an **entity**, naming
//! its bodies by **name**, survives everything the editor does to entities —
//! an undo that respawns them all with fresh bits, a project load, a Reset
//! that rebuilds the world from rest, a scrub backwards through the cache.
//!
//! That list is the reason the whole design is what it is, so it is the list
//! the gates walk.

use ph2d_core::Vec2;
use ph2d_ecs::scene::{ComponentRegistry, WorldSnapshot, snapshot_to_world, world_to_snapshot};
use ph2d_ecs::{
    Entity, Name, SimWorld, Transform, TransformPropagationState, WorklistBuf, stable_name_id,
};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
    register_physics_components,
};

/// A hook (static) above, a plank (dynamic) to its right, and a joint entity
/// pinning the plank's LEFT END to a point one metre below the hook.
///
/// The plank's centre is nowhere near the anchor on purpose — that is what
/// makes "the anchor is the joint's own Transform" a claim with teeth.
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

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// Run `ticks` of play and report how far the plank's centre stayed from the
/// pin at its **worst** moment. A rigid plank pinned at its end never leaves
/// 0.50 m; an unbound one falls away immediately.
fn worst_drift(sim: &mut SimWorld, bridge: &mut PhysicsBridge, from: u64, ticks: u64) -> f32 {
    let mut worst = 0.0f32;
    for tick in from + 1..=from + ticks {
        bridge.dispatch(sim, true, tick);
        worst = worst.max((dist(pos(sim, "Plank"), [0.0, 5.0]) - 0.5).abs());
    }
    worst
}

/// **The product claim: a joint entity binds two bodies.**
///
/// And its own control, in the same run — the plank must SWING (its lowest
/// point is well under the pin) as well as stay attached. "Held together" is
/// satisfied perfectly by "frozen".
#[test]
fn a_joint_entity_binds_the_two_bodies_it_names() {
    let mut sim = pendulum();
    let mut bridge = PhysicsBridge::new();
    let mut lowest = f32::MAX;
    for tick in 1..=240 {
        bridge.dispatch(&mut sim, true, tick);
        let p = pos(&mut sim, "Plank");
        assert!(
            (dist(p, [0.0, 5.0]) - 0.5).abs() < 0.02,
            "tick {tick}: the plank left its pin — centre {p:?} is {:.3} m from \
             the anchor (0, 5), and a rigid plank pinned at its end is 0.500",
            dist(p, [0.0, 5.0])
        );
        lowest = lowest.min(p[1]);
    }
    assert_eq!(bridge.joint_count(), 1);
    assert!(
        lowest < 4.6,
        "the plank never swung (lowest y={lowest:.3} against a pin at 5.0) — \
         a pendulum that does not fall is not pinned, it is frozen"
    );
}

/// **The anchor is the joint entity's own `Transform`.**
///
/// Move the joint object and the pivot moves with it. This is what makes the
/// Inspector's existing Position fields the authoring UI for an anchor, with
/// no widget written for it.
#[test]
fn moving_the_joint_object_moves_the_pivot() {
    let mut sim = pendulum();
    // Put the pin at the plank's RIGHT end instead (its centre is at x=0.5,
    // half-length 0.5, so the right end is x=1.0).
    let pin = named(&mut sim, "Pin");
    sim.world_mut()
        .get_mut::<Transform>(pin)
        .expect("transform")
        .translation = Vec2::new(1.0, 5.0);

    let mut bridge = PhysicsBridge::new();
    for tick in 1..=240 {
        bridge.dispatch(&mut sim, true, tick);
        let p = pos(&mut sim, "Plank");
        assert!(
            (dist(p, [1.0, 5.0]) - 0.5).abs() < 0.02,
            "tick {tick}: the plank should hang from (1, 5) — where the joint \
             object is — but its centre {p:?} is {:.3} m from there",
            dist(p, [1.0, 5.0])
        );
    }
}

/// **The joint survives an undo — the whole reason the bodies are named.**
///
/// This drives the real path: `world_to_snapshot` → despawn everything →
/// `snapshot_to_world`, which is exactly what Ctrl+Z does and which hands
/// every entity **fresh bits**. A joint that had stored `Entity::to_bits()`
/// would come back pointing at nothing (or, worse, at whichever object
/// inherited those recycled bits).
#[test]
fn the_joint_survives_the_respawn_an_undo_performs() {
    let mut registry = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut registry);
    register_physics_components(&mut registry);

    let mut sim = pendulum();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=60 {
        bridge.dispatch(&mut sim, true, tick);
    }

    let bits_before = named(&mut sim, "Plank").to_bits();

    // --- the undo round-trip ---
    let mut snap = WorldSnapshot::new();
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::default();
    world_to_snapshot(sim.world(), &mut prop, &mut worklist, &registry, &mut snap)
        .expect("snapshot");
    let editable: Vec<Entity> = {
        let mut q = sim
            .world_mut()
            .query_filtered::<Entity, bevy_ecs::prelude::With<Transform>>();
        q.iter(sim.world()).collect()
    };
    for e in editable {
        let _ = sim.world_mut().despawn(e);
    }
    snapshot_to_world(sim.world_mut(), &snap, &registry).expect("restore");
    bridge.rebuild(); // what the shell does after a restore

    let bits_after = named(&mut sim, "Plank").to_bits();
    assert_ne!(
        bits_before, bits_after,
        "the fixture is not exercising anything: the restore handed back the \
         SAME entity bits, so a joint storing bits would have survived by luck"
    );

    // The pin is still a pin, all the way through another swing.
    let drift = worst_drift(&mut sim, &mut bridge, 0, 240);
    assert_eq!(bridge.joint_count(), 1, "the joint did not come back");
    assert!(
        drift < 0.02,
        "after the undo's respawn the plank drifted {drift:.3} m from its pin"
    );
}

/// **Reset re-attaches the joints in the same call.**
///
/// A backwards clock rebuilds the world from the bodies' rest descriptions and
/// then replays forward. If the joints were left to come back "next frame",
/// those replayed steps would run on a scene that had fallen apart — and it
/// would look like a one-frame glitch nobody could catch.
#[test]
fn a_reset_rebuilds_the_joints_with_the_bodies() {
    let mut sim = pendulum();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=120 {
        bridge.dispatch(&mut sim, true, tick);
    }
    // Reset: target 0 is never in the checkpoint ring, so this is the
    // rest-pose rebuild path (the one that shipped in W1) and not the cache.
    bridge.dispatch(&mut sim, false, 0);
    assert_eq!(bridge.joint_count(), 1, "Reset dropped the joint");
    assert!(
        dist(pos(&mut sim, "Plank"), [0.5, 5.0]) < 0.001,
        "Reset did not put the plank back where it was authored"
    );
    let drift = worst_drift(&mut sim, &mut bridge, 0, 180);
    assert!(
        drift < 0.02,
        "after a Reset the plank drifted {drift:.3} m from its pin — the \
         rebuild handed out fresh body handles and the joint kept the old ones"
    );
}

/// **A scrub backwards through the checkpoint cache keeps the joints.**
///
/// The sibling of the gate above, on the other path: the ring restores rapier's
/// arenas wholesale, joints included. It is cheap to assert and it is the half
/// that would break if `ImpulseJointSet` ever left the checkpoint.
#[test]
fn a_scrub_backwards_keeps_the_joints() {
    let mut sim = pendulum();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=200 {
        bridge.dispatch(&mut sim, true, tick);
    }
    bridge.dispatch(&mut sim, false, 150);
    assert_eq!(bridge.joint_count(), 1);
    let drift = worst_drift(&mut sim, &mut bridge, 150, 120);
    assert!(drift < 0.02, "after a scrub the plank drifted {drift:.3} m");
}

/// **A half-authored joint is not a joint, and does not panic.**
///
/// The artist creates the object and picks the second body a moment later.
/// Between those two moments there is a `PhysicsJoint` naming one body, and
/// the bridge has to sail straight past it.
#[test]
fn a_joint_that_names_only_one_body_is_inert() {
    let mut sim = pendulum();
    let pin = named(&mut sim, "Pin");
    sim.world_mut()
        .get_mut::<PhysicsJoint>(pin)
        .expect("joint")
        .body_b = 0;
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=120 {
        bridge.dispatch(&mut sim, true, tick);
    }
    assert_eq!(
        bridge.joint_count(),
        0,
        "an unfinished joint was built anyway"
    );
    // And the unbound plank simply falls.
    assert!(
        pos(&mut sim, "Plank")[1] < 3.0,
        "with no joint the plank should be in free fall"
    );
}

/// **A joint naming a body that is not there goes dormant — and comes back.**
///
/// Deleting a body (or renaming it) detaches its joints rather than breaking
/// them; put the name back and the joint re-attaches by itself. This is the
/// same healing the timeline's bindings get, and it falls out of resolving by
/// name every frame instead of remembering a handle.
#[test]
fn a_joint_whose_body_is_renamed_detaches_and_re_attaches() {
    let mut sim = pendulum();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 1);
    assert_eq!(bridge.joint_count(), 1);

    let plank = named(&mut sim, "Plank");
    *sim.world_mut().get_mut::<Name>(plank).expect("name") = Name::new("Renamed");
    bridge.dispatch(&mut sim, true, 2);
    assert_eq!(
        bridge.joint_count(),
        0,
        "renaming a body must detach the joint that named it — the id IS the \
         name, and pretending otherwise would leave the joint bound to a body \
         the artist believes they have renamed"
    );

    *sim.world_mut().get_mut::<Name>(plank).expect("name") = Name::new("Plank");
    bridge.dispatch(&mut sim, true, 3);
    assert_eq!(
        bridge.joint_count(),
        1,
        "putting the name back must re-attach the joint"
    );
}

/// **A scene with joints hashes the same twice** (HR-5, the ECS-side check).
#[test]
fn a_jointed_scene_hashes_the_same_twice() {
    fn run() -> [u8; 32] {
        let mut sim = pendulum();
        let mut bridge = PhysicsBridge::new();
        for tick in 1..=120 {
            bridge.dispatch(&mut sim, true, tick);
        }
        bridge.deterministic_hash(&sim)
    }
    assert_eq!(run(), run());
}

/// **Editing the anchor at rest moves the pivot.**
///
/// The sibling of `moving_the_joint_object_moves_the_pivot`, and it covers the
/// case that one cannot: there the anchor was moved *before* the joint ever
/// existed, so the initial spawn read it. Here the joint has already been
/// built and simulated, the clock is back at zero, and the artist drags the
/// pin — the joint has to be re-described, exactly as a body is.
///
/// ⚠️ Since the gold-standard anchor (ADR-0131) the anchor is authored BODY-LOCAL
/// and a body move no longer re-derives it — so REPOSITIONING the pivot is an
/// explicit gesture: write the joint's `Transform` AND set `anchored = false`
/// (exactly what the shell's anchor-dot drag and Inspector Position commit do).
/// The next reconcile re-derives the body-local anchors from the new pivot.
#[test]
fn editing_the_anchor_at_rest_moves_the_pivot() {
    let mut sim = pendulum();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=120 {
        bridge.dispatch(&mut sim, true, tick);
    }
    bridge.dispatch(&mut sim, false, 0); // back to rest — where authoring happens

    let pin = named(&mut sim, "Pin");
    {
        let mut j = sim.world_mut().get_mut::<PhysicsJoint>(pin).expect("joint");
        j.anchored = false; // the reposition gesture requests a re-derive
    }
    sim.world_mut()
        .get_mut::<Transform>(pin)
        .expect("transform")
        .translation = Vec2::new(1.0, 5.0);

    for tick in 1..=240 {
        bridge.dispatch(&mut sim, true, tick);
        let p = pos(&mut sim, "Plank");
        assert!(
            (dist(p, [1.0, 5.0]) - 0.5).abs() < 0.02,
            "tick {tick}: the pin was dragged to (1, 5) with the clock at rest, \
             so the plank must now hang from there; its centre {p:?} is {:.3} m \
             away — the joint was never re-described",
            dist(p, [1.0, 5.0])
        );
    }
}

/// **A body respawned underneath a joint takes the joint with it.**
///
/// Editing a collider at tick 0 re-describes the body, which means rapier
/// hands out a **fresh handle** — and the joint is still holding the old one.
/// A joint bound to a handle into an arena that has moved on is attached to
/// nothing, and nothing about it looks broken.
#[test]
fn a_body_re_spawned_at_rest_takes_its_joints_with_it() {
    let mut sim = pendulum();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 1);
    bridge.dispatch(&mut sim, false, 0);

    // The artist thickens the plank — a collider edit at rest, which is the
    // one thing that respawns a body without despawning its entity.
    let plank = named(&mut sim, "Plank");
    sim.world_mut()
        .get_mut::<Collider>(plank)
        .expect("collider")
        .shape = ColliderShape::Cuboid {
        half_x: 0.5,
        half_y: 0.2,
    };

    let drift = worst_drift(&mut sim, &mut bridge, 0, 240);
    assert_eq!(bridge.joint_count(), 1, "the joint vanished with the edit");
    assert!(
        drift < 0.02,
        "the plank was re-spawned by a collider edit and drifted {drift:.3} m \
         from its pin — the joint kept a handle into the old arena"
    );
}

/// **The simulation does not depend on ECS archetype order** (HR-5).
///
/// The joint query yields entities in *archetype* order, and rapier assigns
/// joint handles — and therefore solves the joints — in insertion order. Float
/// addition is not associative, so two scenes that differ only in the order the
/// joints happened to be handed over produce different numbers.
///
/// Archetypes diverge for ordinary reasons: one joint carries a `Name` and
/// another does not, and bevy stores them in different tables. This builds the
/// same chain twice — once with every joint named, once with only the middle
/// one named — and requires the same hash. The bridge sorts by entity before
/// spawning, which is what makes the sim a function of the entity SET.
#[test]
fn the_chain_hashes_the_same_however_the_archetypes_fall() {
    fn chain(name_them_all: bool) -> [u8; 32] {
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
            Transform::from_translation(Vec2::new(0.0, 10.0)),
        ));
        let mut prev = "Hook".to_string();
        for i in 0..4 {
            let y = 9.5 - i as f32 * 0.5;
            let link = format!("Link{i}");
            sim.world_mut().spawn((
                Name::new(link.clone()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.2 },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(0.0, y)),
            ));
            let joint = PhysicsJoint {
                body_a: stable_name_id(&prev),
                body_b: stable_name_id(&link),
                kind: JointKind::Pin,
                ..PhysicsJoint::default()
            };
            let at = Transform::from_translation(Vec2::new(0.0, y + 0.25));
            // The only difference between the two runs: whether this joint
            // entity carries a `Name`, which decides which table it lands in.
            if name_them_all || i == 2 {
                sim.world_mut()
                    .spawn((Name::new(format!("Pin{i}")), joint, at));
            } else {
                sim.world_mut().spawn((joint, at));
            }
            prev = link;
        }
        let mut bridge = PhysicsBridge::new();
        for tick in 1..=120 {
            bridge.dispatch(&mut sim, true, tick);
        }
        assert_eq!(bridge.joint_count(), 4, "the chain is not fully linked");
        bridge.deterministic_hash(&sim)
    }
    assert_eq!(
        chain(true),
        chain(false),
        "the same chain simulated differently because its joint entities were \
         stored in different archetypes — the solver's joint order is leaking \
         ECS storage layout into the physics"
    );
}

/// **A dormant joint must not destroy the scrub cache.**
///
/// Found by audit, and measured before the fix: with one half-authored joint
/// in the scene, `joints_to_remove` was non-empty on **every** frame — a joint
/// that could not be built was queued for removal whether or not it had ever
/// been built — so the ring was cleared every frame and a scrub back to tick
/// 150 replayed **150** steps instead of 0.
///
/// Reachable by the most ordinary gestures: delete one of two jointed bodies,
/// rename one, or start a joint and pick the second body a moment later.
///
/// The oracle is the **step count**, which is the quantity the claim is about
/// and is immune to the machine the test runs on (the W1.5 argument).
#[test]
fn a_dormant_joint_does_not_destroy_the_scrub_cache() {
    fn steps_to_scrub(with_dormant: bool) -> u64 {
        let mut sim = pendulum();
        if with_dormant {
            // A joint object the artist has begun and not finished.
            sim.world_mut().spawn((
                Name::new("HalfDone"),
                PhysicsJoint {
                    body_a: stable_name_id("Hook"),
                    body_b: 0,
                    ..PhysicsJoint::default()
                },
                Transform::from_translation(Vec2::new(0.0, 5.0)),
            ));
        }
        let mut bridge = PhysicsBridge::new();
        for tick in 1..=200 {
            bridge.dispatch(&mut sim, true, tick);
        }
        let before = bridge.steps_taken();
        bridge.dispatch(&mut sim, false, 150);
        bridge.steps_taken() - before
    }
    let control = steps_to_scrub(false);
    let dormant = steps_to_scrub(true);
    assert!(
        control <= 10,
        "the fixture is not exercising the cache: even without a dormant \
         joint the scrub replayed {control} steps"
    );
    assert_eq!(
        dormant, control,
        "one unfinished joint object made a backwards scrub replay {dormant} \
         steps instead of {control} — the checkpoint ring is being cleared \
         every frame, and W1.5 is dead for as long as that joint exists"
    );
}

/// **A joint pins the same place on the body after a Reset as it did live.**
///
/// The other audit finding, and the deeper one: `JointRef::rest` used to store
/// the anchors in WORLD coordinates, while `spawn_joint` re-derived the local
/// pair from wherever the bodies stood. So the live spawn and the
/// rebuild-from-rest were two pieces of code answering *"where on the body is
/// this pinned?"* differently — measured at **1.611 m** live and **0.642 m**
/// after a Reset, a pin walking 0.969 m along the body with no user action.
///
/// Joining mid-swing is the reachable path: nothing gates the gesture on the
/// clock, and the module doc claimed a rest-only rule that did not exist.
#[test]
fn a_joint_made_mid_swing_survives_a_reset_unchanged() {
    let mut sim = SimWorld::new();
    // A hook and a free plank — no joint yet.
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
    let mut bridge = PhysicsBridge::new();
    // Let it FALL for a while, so the bodies are nowhere near their rest pose.
    for tick in 1..=40 {
        bridge.dispatch(&mut sim, true, tick);
    }
    // …and only now does the artist join them.
    sim.world_mut().spawn((
        Name::new("Pin"),
        PhysicsJoint {
            body_a: stable_name_id("Hook"),
            body_b: stable_name_id("Plank"),
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(0.0, 5.0)),
    ));
    for tick in 41..=140 {
        bridge.dispatch(&mut sim, true, tick);
    }
    assert_eq!(bridge.joint_count(), 1);
    let live = dist(pos(&mut sim, "Plank"), [0.0, 6.0]);

    // Reset, then replay to the same tick. The constraint must be the SAME
    // one — the pin cannot move along the body because the clock went home.
    bridge.dispatch(&mut sim, false, 0);
    for tick in 1..=140 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let replayed = dist(pos(&mut sim, "Plank"), [0.0, 6.0]);
    assert!(
        (live - replayed).abs() < 0.05,
        "the plank hung {live:.3} m from the hook live and {replayed:.3} m \
         after a Reset — the joint was re-derived against a different pose, so \
         the pin walked {:.3} m along the body with nobody touching it",
        (live - replayed).abs()
    );
}

/// **A joint loaded with poison does not poison the simulation.**
///
/// The Inspector sanitises what it writes, but `PhysicsJoint` is `serde` and
/// travels in the project file, so the Inspector is not the only door. Audit
/// measured it on the unclamped version: `stiffness = NaN` drove the body's
/// pose to `(NaN, NaN)` within 120 steps, and `readback` wrote that straight
/// into the entity's `Transform` — from where it flows into the cross-OS
/// determinism hash.
#[test]
fn a_joint_carrying_nan_does_not_poison_the_pose() {
    let mut sim = pendulum();
    let pin = named(&mut sim, "Pin");
    {
        let mut j = sim.world_mut().get_mut::<PhysicsJoint>(pin).expect("joint");
        j.kind = JointKind::Spring;
        j.stiffness = f32::NAN;
        j.damping = f32::NAN;
        j.rest_length = f32::NAN;
    }
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=120 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let p = pos(&mut sim, "Plank");
    assert!(
        p[0].is_finite() && p[1].is_finite(),
        "a NaN spring constant reached the solver and the plank is at {p:?} — \
         that NaN is now in the Transform and in the determinism hash"
    );
}

/// **Inverted hinge limits are a wide hinge, never a weld.**
///
/// `limit_min` and `limit_max` are authored independently, so `min > max` is
/// one keystroke away — and rapier, handed the pair that way, froze the plank
/// solid (measured: `rot 0.000` after 180 steps). A hinge the artist believes
/// is limited to ±45° silently being a weld has no symptom to search for.
#[test]
fn inverted_limits_do_not_weld_the_hinge() {
    let mut sim = pendulum();
    let pin = named(&mut sim, "Pin");
    {
        let mut j = sim.world_mut().get_mut::<PhysicsJoint>(pin).expect("joint");
        j.limits_enabled = true;
        // The artist typed the two boxes in the order that reads naturally and
        // got them the wrong way round.
        j.limit_min = 0.8;
        j.limit_max = -0.8;
    }
    let mut bridge = PhysicsBridge::new();
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for tick in 1..=180 {
        bridge.dispatch(&mut sim, true, tick);
        let e = named(&mut sim, "Plank");
        let r = sim.world().get::<Transform>(e).expect("t").rotation;
        lo = lo.min(r);
        hi = hi.max(r);
    }
    assert!(
        hi - lo > 0.3,
        "the plank only ever turned through {:.3} rad — inverted limits welded \
         a hinge the artist believes is limited to ±46°",
        hi - lo
    );
}
