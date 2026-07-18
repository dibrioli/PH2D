//! W2b — the world-level body defaults (damping + sleep).
//!
//! Four claims, one per failure mode this feature can have:
//!
//! 1. arriving costs existing art **nothing** (the defaults are rapier's);
//! 2. the numbers actually reach the solver (a knob that does nothing);
//! 3. they reach bodies that **already exist** (the "only new bodies" bug —
//!    the artist edits a world they are looking at, not the next one);
//! 4. sleep is wired too, and its two halves are independent.

use ph2d_physics::{BodyDefaults, BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

/// A dynamic ball dropped from `y`, described in plain data.
fn ball(y: f32) -> BodyDesc {
    BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: 0.0,
        y,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Ball { radius: 0.25 },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
    }
}

/// The height a single ball reaches after `steps` ticks in a world configured
/// by `setup` (run BEFORE the body is spawned unless the test says otherwise).
fn fall_with(defaults: Option<BodyDefaults>, steps: usize) -> f32 {
    let mut w = PhysicsWorld::new();
    if let Some(d) = defaults {
        w.set_body_defaults(d);
    }
    let h = w.spawn_body(ball(10.0));
    for _ in 0..steps {
        w.step();
    }
    w.body_pose(h).expect("body alive").translation.y
}

/// **Claim 1 — applying the defaults at their default value moves nothing.**
///
/// ⚠️ **This gate cannot tell you the defaults are rapier's** — both worlds
/// here read `BodyDefaults::rapier()`, so mutating that function mutates both
/// sides and this stays green. That version was written first and *did* stay
/// green under `linear_damping: 0.05`. The value oracle is the unit test
/// `the_defaults_are_the_ones_rapier_hands_out_untouched`, which compares
/// against a body rapier built and nobody configured.
///
/// What this one still earns: the *machinery* is inert at rest. `spawn_body`
/// now stamps every body, and `set_body_defaults` re-stamps every live body
/// **and wakes it** — three chances to perturb a simulation that must not move.
///
/// The oracle is the trajectory, not the endpoint: a damped fall and an
/// undamped one both end on the floor, and this suite is about a knob whose
/// entire effect is *how you get there*. (Same lesson the W1.5 scrub gate paid
/// for — an endpoint oracle stayed green under a real mutation.)
#[test]
fn applying_the_defaults_at_their_default_value_moves_nothing() {
    let mut untouched = PhysicsWorld::new();
    let mut explicit = PhysicsWorld::new();
    explicit.set_body_defaults(BodyDefaults::rapier());

    let a = untouched.spawn_body(ball(10.0));
    let b = explicit.spawn_body(ball(10.0));

    for tick in 0..240 {
        untouched.step();
        explicit.step();
        let ya = untouched.body_pose(a).unwrap().translation.y;
        let yb = explicit.body_pose(b).unwrap().translation.y;
        assert_eq!(
            ya.to_bits(),
            yb.to_bits(),
            "tick {tick}: setting the defaults to rapier's own values moved the \
             simulation ({ya} vs {yb}) — every existing project just changed"
        );
    }
}

/// **Claim 2 — the damping reaches the solver.**
///
/// Product numbers, not convenient ones: 1 second of fall at the app's own
/// 60 Hz tick and 4 sub-steps. Undamped, the ball falls ~4.9 m; the bar is a
/// full 10 cm of separation so this can never pass on float noise.
///
/// Mutation that must bleed: `apply_to` not calling `set_linear_damping`.
#[test]
fn raising_the_linear_damping_slows_the_fall() {
    let free = fall_with(None, 60);
    let dragged = fall_with(
        Some(BodyDefaults {
            linear_damping: 2.0,
            ..BodyDefaults::rapier()
        }),
        60,
    );
    assert!(
        dragged > free + 0.1,
        "a linear damping of 2.0 should hold the ball UP relative to free fall, \
         but damped y={dragged} vs free y={free}"
    );
}

/// **Claim 3 — the defaults reach bodies that ALREADY exist.**
///
/// This is the bug the feature is most likely to ship with: stamping the
/// defaults only at spawn. The artist types a drag value while looking at a
/// scene full of bodies, and nothing moves — the setting appears dead, and the
/// only way to see it is to delete and re-add every object.
///
/// Mutation that must bleed: `set_body_defaults` storing the value without
/// calling `apply_to_all`.
#[test]
fn the_defaults_reach_bodies_that_already_exist() {
    let mut w = PhysicsWorld::new();
    let h = w.spawn_body(ball(10.0));
    // The body exists FIRST. Only then is the world told about the drag.
    w.set_body_defaults(BodyDefaults {
        linear_damping: 2.0,
        ..BodyDefaults::rapier()
    });
    for _ in 0..60 {
        w.step();
    }
    let after = w.body_pose(h).unwrap().translation.y;
    let free = fall_with(None, 60);
    assert!(
        after > free + 0.1,
        "damping set AFTER the body was spawned did not reach it: y={after}, \
         free fall would be y={free}"
    );
}

/// **Claim 4a — the sleep SPEED THRESHOLD is wired**, isolated from the timer.
///
/// The fixture had to be rebuilt to earn this: a *settled* ball is below any
/// sane threshold and motionless for any sane timer, so whichever of the two
/// lines survived a mutation still decided the verdict — both halves stayed
/// green with the other one deleted
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
///
/// So the threshold gets a body that never stops moving: **free fall, no
/// floor.** Speed climbs from zero, so the threshold alone decides whether the
/// body is ever "slow enough" long enough to qualify. At rapier's 0.4 m/s the
/// ball passes that in three ticks and can never accumulate the delay; at
/// 50 m/s it stays under for ~5 s and falls asleep **in mid-air**, which is
/// also exactly the bug an artist would report if this knob were mis-set.
///
/// Mutation that must bleed: `apply_to` not writing
/// `normalized_linear_threshold` (both cases fall back to 0.4 → both `None`).
#[test]
fn the_sleep_speed_threshold_is_wired() {
    let strict = first_sleep_tick(BodyDefaults::rapier(), Floor::No, 600);
    let generous = first_sleep_tick(
        BodyDefaults {
            sleep_linear_threshold: 50.0,
            ..BodyDefaults::rapier()
        },
        Floor::No,
        600,
    );
    assert_eq!(
        strict, None,
        "at rapier's 0.4 m/s threshold a body in free fall outruns the \
         threshold within three ticks and must never sleep"
    );
    assert!(
        generous.is_some(),
        "at a 50 m/s threshold the falling body is 'slow' for ~5 s and must \
         fall asleep mid-air; it never did, so the threshold never reached rapier"
    );
}

/// **Claim 4b — the sleep DELAY is wired**, isolated from the threshold.
///
/// Same scene, same threshold, two timers: the only thing that can move the
/// answer is the delay. A **differential** oracle on purpose — it asserts the
/// two runs disagree rather than pinning an absolute tick, so it cannot be
/// satisfied by a number that happens to sit on the right side of a bar.
///
/// Mutation that must bleed: `apply_to` not writing `time_until_sleep` (both
/// cases fall back to 2.0 s → identical ticks).
#[test]
fn the_sleep_delay_is_wired() {
    let quick = first_sleep_tick(
        BodyDefaults {
            time_until_sleep: 0.05,
            ..BodyDefaults::rapier()
        },
        Floor::Yes,
        600,
    )
    .expect("a settled ball with a 0.05 s delay must sleep");
    let patient = first_sleep_tick(
        BodyDefaults {
            time_until_sleep: 3.0,
            ..BodyDefaults::rapier()
        },
        Floor::Yes,
        600,
    )
    .expect("a settled ball with a 3 s delay must still sleep within 10 s");
    assert!(
        quick < patient,
        "a 0.05 s delay must put the ball to sleep sooner than a 3 s one, \
         but they slept at tick {quick} and {patient} — the delay is not \
         reaching rapier"
    );
}

/// **Claim 4c — relaxing the sleep settings WAKES what sleep already froze.**
///
/// A sleeping body is not integrated, so it never consults the thresholds
/// again: turning sleep off would be read by nobody and the body would hang in
/// the air forever. `apply_to_all` wakes every body for exactly this reason,
/// and this gate is why that line is allowed to exist — a defense nobody can
/// observe is a comment that lies, and this line's own module already paid for
/// that lesson once (the `ring.clear()` the W1.5 wave deleted).
///
/// The fixture makes the freeze real first: an absurdly generous threshold
/// puts a *falling* ball to sleep in mid-air. Then sleep is turned off.
///
/// Mutation that must bleed: `apply_to_all` not calling `wake_up`.
#[test]
fn relaxing_the_sleep_settings_wakes_a_body_sleep_already_froze() {
    let mut w = PhysicsWorld::new();
    w.set_body_defaults(BodyDefaults {
        sleep_linear_threshold: 50.0,
        time_until_sleep: 0.05,
        ..BodyDefaults::rapier()
    });
    let h = w.spawn_body(ball(1.0));
    // No floor: the only thing that can stop this ball is sleep.
    for _ in 0..120 {
        w.step();
    }
    assert!(
        w.bodies().get(h).expect("alive").is_sleeping(),
        "fixture precondition: the ball must be asleep in mid-air before the \
         settings are relaxed, otherwise this gate proves nothing"
    );
    let frozen_at = w.body_pose(h).unwrap().translation.y;

    // "Never sleep": at rapier's 0.4 m/s a body falling this fast does not qualify.
    w.set_body_defaults(BodyDefaults::rapier());
    for _ in 0..60 {
        w.step();
    }
    let after = w.body_pose(h).unwrap().translation.y;

    assert!(
        after < frozen_at - 0.1,
        "turning sleep off left the ball frozen at y={frozen_at} (now {after}): \
         a sleeping body is never integrated, so the new thresholds reached \
         nobody — `apply_to_all` must wake what it re-describes"
    );
}

/// Whether the fixture has ground to settle on.
#[derive(Copy, Clone, PartialEq)]
enum Floor {
    Yes,
    No,
}

/// Run a ball under `defaults` and report the first tick it is asleep, or
/// `None` if it never slept within `max_ticks`.
fn first_sleep_tick(defaults: BodyDefaults, floor: Floor, max_ticks: u64) -> Option<u64> {
    let mut w = PhysicsWorld::new();
    w.set_body_defaults(defaults);
    if floor == Floor::Yes {
        w.add_static_cuboid(0.0, 0.0, 50.0, 0.1);
    }
    let h = w.spawn_body(ball(1.0));
    for tick in 1..=max_ticks {
        w.step();
        if w.bodies().get(h).expect("body alive").is_sleeping() {
            return Some(tick);
        }
    }
    None
}
