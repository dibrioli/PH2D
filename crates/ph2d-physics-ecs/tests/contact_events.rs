//! Contact TRANSITIONS — who started touching, who stopped (W-ContactEvents).
//!
//! The sibling of `contacts.rs`, and the distinction is the whole wave: that file
//! proves the STANDING set is republished every dispatch, this one proves the
//! DIFFERENCE between two dispatches is reported — and, far more importantly, that a
//! discontinuous clock move reports NOTHING.
//!
//! ⚠️ The trap these gates exist for: the standing set is recomputed from scratch,
//! so the naive diff turns a scrub over a settled stack into a storm of collisions
//! that never happened. Three of the gates below are about the two boundaries where
//! that is true (a rewind, and disarming the transport's Physics toggle), and each
//! bleeds under its own mutation.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, ContactPhase, PhysicsBridge, RigidBody};

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

/// A bouncy box. `restitution` is what makes a landing END — a dead box lands once
/// and stays, which cannot tell "began and never ended" from "began, ended, began".
fn bouncy_box_at(sim: &mut SimWorld, x: f32, y: f32) -> Entity {
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
                restitution: 0.9,
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

/// Play forward one tick at a time, collecting every transition on the way.
fn play_collecting(
    bridge: &mut PhysicsBridge,
    sim: &mut SimWorld,
    ticks: u64,
) -> Vec<(u64, ContactPhase)> {
    let mut seen = Vec::new();
    let from = bridge.last_stepped();
    for t in (from + 1)..=(from + ticks) {
        bridge.dispatch(sim, true, t);
        for e in bridge.contact_events() {
            seen.push((t, e.phase));
        }
    }
    seen
}

/// A bounce is BEGAN, then ENDED, then BEGAN again — the transitions a gameplay
/// consumer would turn into two impact sounds.
///
/// The standing list alone cannot say this: it reports "touching" on the landing
/// frames and "not touching" in between, and reconstructing the transitions from that
/// is exactly the work this channel exists to do once, deterministically, instead of
/// in every consumer.
#[test]
fn a_bounce_reports_began_then_ended_then_began() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    bouncy_box_at(&mut sim, 0.0, 3.0);
    let mut bridge = PhysicsBridge::new();

    let seen = play_collecting(&mut bridge, &mut sim, 240);
    let phases: Vec<ContactPhase> = seen.iter().map(|(_, p)| *p).collect();

    assert!(
        phases.len() >= 3,
        "a bouncing box must produce at least began/ended/began, got {phases:?}"
    );
    assert_eq!(
        phases[0],
        ContactPhase::Began,
        "the first transition of a falling body is a landing, got {phases:?}"
    );
    assert_eq!(
        phases[1],
        ContactPhase::Ended,
        "a bouncy box leaves the floor, got {phases:?}"
    );
    assert_eq!(
        phases[2],
        ContactPhase::Began,
        "and comes back down, got {phases:?}"
    );
    // The transitions are SPARSE: the pair is touching (or not) for many ticks
    // between them. A channel that fired every frame would be the standing list
    // wearing another name.
    assert!(
        phases.len() < 40,
        "transitions must be sparse, not one per frame; got {} in 240 ticks",
        phases.len()
    );
}

/// **The trap.** Scrubbing the timeline backwards over a settled scene reports
/// nothing at all — nothing began, the artist moved the clock.
///
/// Red-first: with the history kept across the jump, the rebuild diffs a set from
/// tick 240 against a set from tick 20 and calls the difference collisions.
#[test]
fn a_scrub_backwards_is_not_a_hundred_collisions() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    for i in 0..4 {
        bouncy_box_at(&mut sim, i as f32 * 2.0, 3.0);
    }
    let mut bridge = PhysicsBridge::new();
    play_to(&mut bridge, &mut sim, 240);
    assert!(
        !bridge.contacts().is_empty(),
        "fixture: the boxes must be resting on the floor by now"
    );

    // ── Scrub A: backwards into a tick where the boxes are ALSO resting.
    // A naive diff finds the same pairs and stays quiet here by luck, so this half
    // is about the AGE: adopting the set must not stamp it as beginning now.
    bridge.dispatch(&mut sim, false, 150);
    assert!(
        bridge.contact_events().is_empty(),
        "a scrub stepped through no transition, so it reports none; got {:?}",
        bridge.contact_events()
    );
    // ...and the standing set is still published, because "what is touching" is a
    // question about the tick the artist is now looking at.
    assert!(
        !bridge.contacts().is_empty(),
        "the scrub must still publish what touches AT the target tick"
    );
    assert!(
        bridge.contacts().iter().all(|c| c.age_ticks.is_none()),
        "a re-baselined pair has no beginning to count from — else the overlay \
         flashes the whole scene on every scrub"
    );

    // ── Scrub B: backwards into free fall, where NOTHING is touching.
    // ⚠️ This is the half a naive diff cannot survive: four pairs disappear at once,
    // and reporting that difference would announce four departures that never
    // happened. (Fixture note: tick 20 is ~0.33 s in — the boxes are dropped from
    // y = 3 and do not reach the floor until ~tick 43, so the phenomenon really is
    // in this fixture.)
    bridge.dispatch(&mut sim, false, 20);
    assert!(
        bridge.contact_events().is_empty(),
        "scrubbing back into free fall ends nothing — the touches were never \
         un-made, the clock moved; got {:?}",
        bridge.contact_events()
    );
    assert!(
        bridge.contacts().is_empty(),
        "fixture: at tick 20 the boxes are still falling"
    );
}

/// Disarming the transport's Physics toggle CLEARS the marks, and re-arming reports
/// no collisions for what was already touching.
///
/// ⚠️ Red-first, and it was a LIVE defect: `hold` cleared the trigger state with a
/// comment explaining that a stale overlap would light a sensor with nothing inside,
/// and left the contacts — which are read from the narrow phase, and only `step`
/// updates that. So turning physics off left the crosses on screen describing a world
/// the artist could then pull apart by hand.
#[test]
fn disarming_physics_clears_the_marks_and_re_arming_is_silent() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    bouncy_box_at(&mut sim, 0.0, 3.0);
    let mut bridge = PhysicsBridge::new();
    play_to(&mut bridge, &mut sim, 240);
    assert!(
        !bridge.contacts().is_empty(),
        "fixture: the box must be resting before we disarm"
    );

    bridge.hold(&mut sim, 241);
    assert!(
        bridge.contacts().is_empty(),
        "with the solver off there is no live touch to report; a stale cross \
         describes a world that is no longer being simulated"
    );
    assert!(
        bridge.contact_events().is_empty(),
        "and disarming is not itself a departure"
    );

    // Re-arm. The box is still on the floor, so nothing BEGINS.
    bridge.dispatch(&mut sim, true, 242);
    assert!(
        bridge
            .contact_events()
            .iter()
            .all(|e| e.phase != ContactPhase::Began),
        "re-arming re-adopts what is touching in silence; got {:?}",
        bridge.contact_events()
    );
    assert!(
        !bridge.contacts().is_empty(),
        "and the standing set comes straight back"
    );
}

/// The FIRST stepped frame reports what it finds, including a stack the artist
/// authored already resting.
///
/// This is Unity's reading (`OnCollisionEnter` fires on the first `FixedUpdate` for
/// pre-touching bodies) and the only defensible one: the narrow phase had never run,
/// so there is no earlier truth for the pair to be compared against.
#[test]
fn the_first_stepped_frame_reports_what_it_was_authored_touching() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    // Resting exactly on the floor (floor top is y = 0), not falling toward it.
    bouncy_box_at(&mut sim, 0.0, 0.25);
    let mut bridge = PhysicsBridge::new();

    bridge.dispatch(&mut sim, true, 1);

    assert_eq!(
        bridge
            .contact_events()
            .iter()
            .filter(|e| e.phase == ContactPhase::Began)
            .count(),
        1,
        "an authored resting body reports its contact on the first stepped frame; \
         got {:?}",
        bridge.contact_events()
    );
    assert_eq!(
        bridge.contacts()[0].age_ticks,
        Some(0),
        "and it began THIS tick, so the overlay flashes it"
    );
}

/// `age_ticks` counts from the beginning and keeps counting — it is a fact about the
/// SIMULATION, not about how many frames the readback has been looking.
#[test]
fn the_age_counts_ticks_since_the_pair_began() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    bouncy_box_at(&mut sim, 0.0, 0.25);
    let mut bridge = PhysicsBridge::new();

    bridge.dispatch(&mut sim, true, 1);
    let began_at = bridge.last_stepped();
    assert_eq!(bridge.contacts()[0].age_ticks, Some(0));

    play_to(&mut bridge, &mut sim, began_at + 5);
    assert_eq!(
        bridge.contacts()[0].age_ticks,
        Some(5),
        "five ticks later the same pair is five ticks old"
    );
}

/// The channel is DETERMINISTIC: the same authored scene played twice produces the
/// same transitions, in the same order, at the same ticks.
///
/// The order within a frame is fixed (`Ended` then `Began`, each half sorted by
/// entity pair) so a consumer draining the queue behaves the same on every machine —
/// the `BTreeMap` reason the rest of the bridge documents.
#[test]
fn the_transitions_are_deterministic() {
    let run = || {
        let mut sim = SimWorld::new();
        floor(&mut sim);
        for i in 0..3 {
            bouncy_box_at(&mut sim, i as f32 * 2.0, 3.0);
        }
        let mut bridge = PhysicsBridge::new();
        play_collecting(&mut bridge, &mut sim, 200)
    };
    let a = run();
    let b = run();
    assert!(!a.is_empty(), "fixture: the run must produce transitions");
    assert_eq!(a, b, "the same scene must transition identically");
}

/// ⚠️ A `Began` event carries the **impact peak of the landing** (W-ImpactForce), not
/// the load — so a hit sound sizes itself by how hard the pair hit. This is the BRIDGE
/// half: it proves `impact` threads from the world through `ContactReport` →
/// `ContactMemo` → `ContactEvent` distinctly from `impulse`.
///
/// **A bouncy box, and the phase-free signal** (the same one the wrapper's gate uses,
/// one level down). Over a full bounce a box eventually has a Began where it has already
/// lifted off by the tick's end: the event's `impulse` (endpoint load) is ~zero while
/// its `impact` (the captured peak) is not — scene 29 tick 63: load 0.0, impact 0.85. A
/// single landing tick's load is sub-step-phase-dependent and can equal the peak, so the
/// gate collects EVERY Began and asks whether one carries a peak clearly above its own
/// endpoint. Red-first: mutating the bridge to carry `r.impulse` for the impact field
/// (or any of the kernel mutations `contacts.rs` lists) makes `impact == impulse` on
/// every event → no such Began → RED.
#[test]
fn a_began_event_carries_the_impact_of_the_landing() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    bouncy_box_at(&mut sim, 0.0, 3.0);
    let mut bridge = PhysicsBridge::new();

    let mut peak_seen = 0.0f32;
    let mut peak_beats_endpoint = false;
    let from = bridge.last_stepped();
    for t in (from + 1)..=(from + 300) {
        bridge.dispatch(&mut sim, true, t);
        for e in bridge.contact_events() {
            if e.phase == ContactPhase::Began {
                peak_seen = peak_seen.max(e.impact);
                if e.impact > 0.1 && e.impulse < e.impact * 0.5 {
                    peak_beats_endpoint = true;
                }
            }
        }
    }

    assert!(
        peak_seen > 1.0,
        "fixture: a 3 m drop should produce a hard Began, got peak {peak_seen}"
    );
    assert!(
        peak_beats_endpoint,
        "some Began must carry a peak clearly above its own endpoint load — the impact \
         threaded through the bridge, not the load it rebounds to"
    );
}

/// **The headless probe behind smoke scene 29.** Not an assertion — a MEASUREMENT.
///
/// The plan's rule is that a scene's `eprintln!` cites numbers the probe produced,
/// because this module has twice shipped a scene that claimed something the
/// measurement then contradicted. Mirrors the bodies of
/// `shells/desktop/src/physics_smoke_events.rs` exactly; run with:
///
/// ```text
/// cargo test -p ph2d-physics-ecs --test contact_events -- --ignored --nocapture
/// ```
#[test]
#[ignore = "measurement probe for smoke scene 29, not a gate"]
fn probe_scene_29() {
    let mut sim = SimWorld::new();
    // Floor: half 4.0 x 0.2 at y = -1.0, so its top is at y = -0.8.
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
        Transform::from_translation(Vec2::new(0.0, -1.0)),
    ));
    let bouncy = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                restitution: 0.75,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(-2.6, 1.2)),
        ))
        .id();
    let dead = sim
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
                restitution: 0.0,
                friction: 0.6,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 3.4)),
        ))
        .id();
    let mut stack = Vec::new();
    for i in 0..3 {
        stack.push(
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
                        friction: 0.6,
                        ..Collider::default()
                    },
                    Transform::from_translation(Vec2::new(2.6, -0.55 + i as f32 * 0.5)),
                ))
                .id(),
        );
    }

    let mut bridge = PhysicsBridge::new();
    let group = |e: Entity| {
        if e == bouncy {
            "bouncy"
        } else if e == dead {
            "dead"
        } else if stack.contains(&e) {
            "stack"
        } else {
            "floor"
        }
    };
    let mut counts = std::collections::BTreeMap::<&str, usize>::new();
    let mut firsts = std::collections::BTreeMap::<&str, u64>::new();
    println!("--- scene 29, 4 s at 60 Hz ---");
    for t in 1..=240u64 {
        bridge.dispatch(&mut sim, true, t);
        for e in bridge.contact_events() {
            // Name the moving body of the pair (the other side is the floor, or the
            // box below it in the stack).
            let who = if group(e.a) == "floor" {
                group(e.b)
            } else {
                group(e.a)
            };
            *counts.entry(who).or_default() += 1;
            firsts.entry(who).or_insert(t);
            println!(
                "tick {t:>3}  {who:<7} {:?}  load {:.5}  impact {:.5}",
                e.phase, e.impulse, e.impact
            );
        }
    }
    println!("--- totals ---");
    for (who, n) in &counts {
        println!("{who:<7} {n} transitions, first at tick {}", firsts[who]);
    }
}

/// **The headless probe behind smoke scene 30 (the demolition).** A heavy ball fired
/// horizontally into a light box at two speeds: the Began of the ball→box pair carries
/// a bigger `impact` the faster it was fired. Prints the numbers the scene cites.
#[test]
#[ignore = "measurement probe for smoke scene 30, not a gate"]
fn probe_scene_30() {
    // A ball fired at `vx` into a light box sitting on a floor; return the impact of the
    // ball->box Began.
    fn slam(vx: f32) -> f32 {
        use ph2d_physics_ecs::{InitialVelocity, MassOverride};
        let mut sim = SimWorld::new();
        // Floor.
        sim.world_mut().spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 10.0,
                    half_y: 0.2,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, -0.2)),
        ));
        // A light box to hit.
        let box_e = sim
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
                Transform::from_translation(Vec2::new(2.0, 0.25)),
            ))
            .id();
        // A heavy fast ball.
        let ball = sim
            .world_mut()
            .spawn((
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.35 },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(-2.0, 0.35)),
                InitialVelocity {
                    linvel: [vx, 0.0],
                    angvel: 0.0,
                },
                MassOverride(6.0),
            ))
            .id();
        let mut bridge = PhysicsBridge::new();
        let from = bridge.last_stepped();
        for t in (from + 1)..=(from + 180) {
            bridge.dispatch(&mut sim, true, t);
            for e in bridge.contact_events() {
                if e.phase == ContactPhase::Began
                    && ((e.a == ball && e.b == box_e) || (e.a == box_e && e.b == ball))
                {
                    return e.impact;
                }
            }
        }
        0.0
    }

    println!("\n--- scene 30, demolition: impact of ball->box Began ---");
    for vx in [3.0f32, 6.0, 10.0, 16.0] {
        println!("  vx {vx:>5.1} m/s  ->  impact {:.4} N.s", slam(vx));
    }
}

/// **Read-only is still the whole contract.** Draining the transitions does not move
/// a single body.
///
/// The W-Contacts wave earned a byte-identical C9 hash by touching nothing on the
/// stepping path, and this wave adds MEMORY — a map, a flag, a queue — which is
/// exactly the kind of addition that starts writing back the day someone decides an
/// event should wake a body or clear a velocity. The wrapper has the same gate one
/// level down (`ph2d-physics/tests/contacts.rs`); this is the bridge's, over the
/// world the C9 harness actually hashes.
#[test]
fn reading_the_transitions_does_not_move_the_world() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    bouncy_box_at(&mut sim, 0.0, 3.0);
    let mut bridge = PhysicsBridge::new();
    // Mid-collision, where there is something to report — a hash taken over a scene
    // in free fall would be unchanged for the boring reason.
    play_to(&mut bridge, &mut sim, 60);

    let before = bridge.deterministic_hash(&sim);
    let events = bridge.contact_events().len();
    let contacts = bridge.contacts().len();
    let _ = bridge.contact_count(Entity::from_bits(1));
    let after = bridge.deterministic_hash(&sim);

    assert_eq!(
        before, after,
        "reading the contact channel must not perturb the simulation — the C9 hash \
         is what CI compares across three operating systems"
    );
    let _ = (events, contacts);
}

/// Deleting a body ENDS its contacts, and the event still names it.
///
/// The pair ended; *why* is the caller's question, and a consumer that wants to know
/// whether the other side still exists can ask the world. Swallowing the event
/// instead would leave a gameplay listener believing the two are still touching.
#[test]
fn removing_a_body_ends_its_contacts() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    let b = bouncy_box_at(&mut sim, 0.0, 0.25);
    let mut bridge = PhysicsBridge::new();
    play_to(&mut bridge, &mut sim, 30);
    assert!(
        !bridge.contacts().is_empty(),
        "fixture: it must be touching"
    );

    sim.world_mut().entity_mut(b).despawn();
    bridge.dispatch(&mut sim, true, 31);

    let ended: Vec<_> = bridge
        .contact_events()
        .iter()
        .filter(|e| e.phase == ContactPhase::Ended)
        .collect();
    assert_eq!(
        ended.len(),
        1,
        "the pair ended when one side was deleted; got {:?}",
        bridge.contact_events()
    );
    assert!(
        ended[0].a == b || ended[0].b == b,
        "and the event names the body that left"
    );
    assert!(
        bridge.contacts().is_empty(),
        "nothing is touching once one of the two is gone"
    );
}
