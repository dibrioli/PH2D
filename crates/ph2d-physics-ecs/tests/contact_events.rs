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
use ph2d_physics_ecs::{
    BodyKind, CONTACT_FLASH_TICKS, Collider, ColliderShape, ContactPhase, PhysicsBridge, RigidBody,
};

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
        bridge.contact_flashes().is_empty(),
        "a scrub leaves no flash lit — else the overlay flashes the whole scene when \
         the artist drags the ruler"
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
        bridge.contact_flashes().len(),
        1,
        "and it began THIS tick, so the overlay flashes it"
    );
}

/// A begin-flash lives a FIXED span and then is gone — even though the pair keeps
/// touching the whole time.
///
/// The flash marks a BEGINNING (an event), not the duration of a touch, which is why it
/// rides its own channel and not the standing contact. The old flash rode
/// `BodyContact.age_ticks`, so it lasted exactly as long as the pair touched: a short
/// bounce under-flashed and a fast touch never flashed at all. Here the box rests for
/// the whole test, and the flash still expires on schedule.
#[test]
fn the_begin_flash_decays_over_a_fixed_span_even_while_the_pair_keeps_touching() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    bouncy_box_at(&mut sim, 0.0, 0.25); // authored resting on the floor
    let mut bridge = PhysicsBridge::new();

    bridge.dispatch(&mut sim, true, 1);
    assert_eq!(
        bridge.contact_flashes().len(),
        1,
        "the authored resting contact flashes on the first stepped tick"
    );
    assert_eq!(bridge.contact_flashes()[0].age_ticks, 0, "and it is new");

    // Play well past the flash's life. The box never leaves the floor.
    play_to(&mut bridge, &mut sim, CONTACT_FLASH_TICKS + 3);
    assert!(
        !bridge.contacts().is_empty(),
        "fixture: the box is still resting on the floor the whole time"
    );
    assert!(
        bridge.contact_flashes().is_empty(),
        "the flash is spent after {CONTACT_FLASH_TICKS} ticks even though the pair is \
         still touching — a flash is a beginning, not a duration"
    );
}

/// A discontinuity puts out a LIVE flash — a scrub or a disarm is not a moment to be
/// lighting up.
///
/// The existing scrub gate settles for many ticks first, so no flash is live when it
/// jumps; this one disarms while the flash is still burning. Red-first: without the
/// flash-clear in `discard_contact_history`, the spark survives the discontinuity and
/// hangs on screen over a world the artist can now pull apart by hand.
#[test]
fn a_discontinuity_puts_out_a_live_flash() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    bouncy_box_at(&mut sim, 0.0, 0.25); // authored resting → flashes on tick 1
    let mut bridge = PhysicsBridge::new();

    bridge.dispatch(&mut sim, true, 1);
    assert_eq!(
        bridge.contact_flashes().len(),
        1,
        "fixture: the authored contact flashes on the first stepped tick, while it is live"
    );

    // Disarm the transport's Physics toggle while the flash is still burning.
    bridge.hold(&mut sim, 2);
    assert!(
        bridge.contact_flashes().is_empty(),
        "a discontinuity puts the live flash out"
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

/// **The wave (W-TickContacts), red-first.** A touch that begins AND ends between two
/// dispatch endpoints fires an event — because the diff now runs per TICK, not per
/// dispatch, over the sub-step union the world already captures for the impact peak.
///
/// The construction makes the fast touch *unavoidable* rather than hoping a drop lands
/// sub-tick (which is phase-dependent — measured, a 3 m drop's first landing is
/// invisible while a 6 m drop's is not): play tick-by-tick until the ball is just above
/// the floor, then ask for ONE dispatch that spans the whole landing-and-rebound. The
/// ball is airborne at both endpoints, so the end-of-dispatch sample front A took sees
/// nothing.
///
/// Red-first: with the per-dispatch diff, `dispatch(55)` from tick 40 diffs the tick-55
/// state (airborne) against the tick-40 state (airborne) and reports no Began, even
/// though the ball plainly bounced in between. With the per-tick diff, the landing tick
/// enters the union and a Began fires.
#[test]
fn a_touch_between_two_dispatch_endpoints_still_fires() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    // A ball, high restitution, so the landing ENDS (a dead body would rest and the
    // touch would persist to the endpoint, hiding the phenomenon).
    let ball = sim
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
            Transform::from_translation(Vec2::new(0.0, 3.0)),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();

    // Fall tick-by-tick until the ball is just above the floor (still airborne).
    play_to(&mut bridge, &mut sim, 40);
    assert!(
        bridge.contacts().is_empty(),
        "fixture: at tick 40 the ball is still falling"
    );

    // ONE dispatch spanning the landing (~tick 44) and the rebound.
    bridge.dispatch(&mut sim, true, 55);
    assert!(
        bridge.contacts().is_empty(),
        "fixture: by tick 55 the ball has bounced back up — the touch is INVISIBLE to \
         a diff that samples only the dispatch endpoints"
    );

    let began: Vec<_> = bridge
        .contact_events()
        .iter()
        .filter(|e| e.phase == ContactPhase::Began && (e.a == ball || e.b == ball))
        .collect();
    assert!(
        !began.is_empty(),
        "the ball landed between the endpoints, so a Began must fire; got {:?}",
        bridge.contact_events()
    );
    assert!(
        began.iter().any(|e| e.impact > 0.5),
        "and it carries the impact of a real landing, not a graze; got {began:?}"
    );
}

/// **The wave in the ordinary case: normal ONE-tick-per-dispatch play.** A hard landing
/// that the solver resolves and rebounds within a single tick fires a `Began`, because
/// the diff runs over the tick's sub-step union, not its settled end.
///
/// Red-first, and phase-free by construction: an 8 m drop is fast enough that its FIRST
/// landing is sub-tick at every alignment (measured: with the old per-dispatch diff it
/// fired NOTHING in the first ~100 ticks; the first reported event, if any, was a much
/// later slow bounce). Here it lands at ~tick 76 with an impact near 5 N·s.
#[test]
fn a_fast_landing_fires_during_normal_one_tick_play() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    let ball = sim
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
            Transform::from_translation(Vec2::new(0.0, 8.0)),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();

    // Play 100 ticks one at a time — the FIRST landing is ~tick 76.
    let mut hardest = 0.0f32;
    for t in 1..=100u64 {
        bridge.dispatch(&mut sim, true, t);
        for e in bridge.contact_events() {
            if e.phase == ContactPhase::Began && (e.a == ball || e.b == ball) {
                hardest = hardest.max(e.impact);
            }
        }
    }
    assert!(
        hardest > 3.0,
        "an 8 m drop's first landing is sub-tick, and the old diff missed it entirely; \
         the per-tick union must fire a Began carrying its impact — got hardest {hardest}"
    );
}

/// **Measurement probe for the fast-bounce gate and smoke scene 31.** Drops a bouncy
/// ball from several heights, ONE tick per dispatch (the real play cadence), and prints
/// the tick and impact of the FIRST landing's Began — the landing the old per-dispatch
/// diff missed above ~3 m.
#[test]
#[ignore = "measurement probe, not a gate"]
fn probe_fast_bounces() {
    println!("\n--- fast bounces, 1 tick/dispatch, first-landing Began ---");
    for h in [1.2f32, 3.0, 6.0, 8.0, 10.0] {
        let mut sim = SimWorld::new();
        floor(&mut sim);
        let ball = sim
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
                Transform::from_translation(Vec2::new(0.0, h)),
            ))
            .id();
        let mut bridge = PhysicsBridge::new();
        let mut first: Option<(u64, f32)> = None;
        let mut total = 0;
        for t in 1..=180u64 {
            bridge.dispatch(&mut sim, true, t);
            for e in bridge.contact_events() {
                if e.phase == ContactPhase::Began && (e.a == ball || e.b == ball) {
                    total += 1;
                    first.get_or_insert((t, e.impact));
                }
            }
        }
        match first {
            Some((t, imp)) => println!(
                "drop {h:>5.1} m -> first landing Began at tick {t}, impact {imp:.3} N.s; \
                 {total} begans in 180 ticks"
            ),
            None => println!("drop {h:>5.1} m -> NO Began in 180 ticks"),
        }
    }
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
