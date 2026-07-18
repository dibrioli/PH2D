//! **W2b at the bridge seam:** the world's authored settings have to survive
//! everything that builds a fresh `PhysicsWorld`, and they have to invalidate
//! everything that cached a state simulated under the old ones.
//!
//! A fresh world starts from the ENGINE defaults, and the bridge builds one in
//! three places (construction, `rebuild`, `rebuild_from_rest`). Before W2b only
//! *gravity* was carried across — which was correct while gravity was the only
//! knob, and became a silent reset of nine others the moment there was a panel.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, MAX_DAMPING, MAX_SUBSTEPS, PhysicsBridge, PhysicsSettings,
    RigidBody,
};

/// One ball over a floor — enough for "did this body fall the way the settings
/// say it should", which is the only question in this file.
fn scene() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 50.0,
                half_y: 0.1,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let ball = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 10.0)),
        ))
        .id();
    (sim, ball)
}

fn y_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().translation.y
}

/// Heavy drag — a setting whose effect is obvious in the pose, so "did it
/// survive?" is answerable by looking at the ball rather than at a getter.
fn draggy() -> PhysicsSettings {
    PhysicsSettings {
        linear_damping: MAX_DAMPING,
        ..PhysicsSettings::default()
    }
}

/// Play `ticks` ticks and report where the ball ended up.
fn play(bridge: &mut PhysicsBridge, sim: &mut SimWorld, ball: Entity, ticks: u64) -> f32 {
    for tick in 1..=ticks {
        bridge.dispatch(sim, true, tick);
    }
    y_of(sim, ball)
}

/// **The settings reach the solver at all** — the claim every other one here
/// assumes. Mutation that must bleed: `apply_to` not calling
/// `set_body_defaults`.
#[test]
fn the_authored_settings_reach_the_simulation() {
    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();
    let free = play(&mut bridge, &mut sim, ball, 60);

    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(draggy());
    let dragged = play(&mut bridge, &mut sim, ball, 60);

    assert!(
        dragged > free + 0.5,
        "world drag did not reach the bodies: damped y={dragged}, free y={free}"
    );
}

/// **They survive a REWIND.** This is the bug W2b would have shipped: rapier
/// cannot step backwards, so a backwards clock rebuilds the world from each
/// body's rest description — and a rebuilt world starts from the ENGINE
/// defaults. Scrub back and the artist's settings would evaporate, leaving a
/// simulation that no longer matches the panel they are looking at.
///
/// The oracle is the ball, not `bridge.settings()`: the field surviving proves
/// nothing if it never reached rapier again.
///
/// ⚠️ **The fixture has to force a ring MISS, and the first version didn't.**
/// Scrubbing back to tick 60 after playing to 200 *hits* the checkpoint ring,
/// and a restored checkpoint carries the damping inside the body set — so
/// `rebuild_from_rest` never ran and the gate stayed green with the pre-W2b
/// gravity-only code in place. **Reset** (`target = 0`) is the miss: tick 0 is
/// never recorded, so the ring returns `None` and the rest-pose rebuild is the
/// path taken. The emptied ring is asserted as a precondition, because that is
/// the observable signature of that path having run.
///
/// Mutation that must bleed: `rebuild_from_rest` restoring gravity only (which
/// is literally what it did before W2b).
#[test]
fn the_settings_survive_the_rest_pose_rebuild() {
    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(draggy());
    let straight = play(&mut bridge, &mut sim, ball, 60);

    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(draggy());
    play(&mut bridge, &mut sim, ball, 200);
    // Reset: the clock goes to 0, which the ring never holds → rest rebuild.
    bridge.dispatch(&mut sim, false, 0);
    assert_eq!(
        bridge.ring_stats().0,
        0,
        "fixture precondition: Reset must take the rest-pose rebuild (which \
         clears the ring). A ring HIT restores the body set — damping included \
         — so this gate would prove nothing"
    );
    let after_reset = play(&mut bridge, &mut sim, ball, 60);

    assert_eq!(
        straight.to_bits(),
        after_reset.to_bits(),
        "after Reset and 60 ticks the ball is at y={after_reset}, but playing \
         straight there puts it at y={straight} — the rebuilt world lost the \
         authored settings and re-simulated under the engine defaults"
    );
}

/// **They survive a project load / undo restore** (`rebuild`), for the same
/// reason and by the same mechanism.
///
/// Mutation that must bleed: `rebuild` not re-pushing the settings.
#[test]
fn the_settings_survive_a_rebuild() {
    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(draggy());
    let before = play(&mut bridge, &mut sim, ball, 60);

    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(draggy());
    bridge.rebuild();
    let after = play(&mut bridge, &mut sim, ball, 60);

    assert_eq!(
        before.to_bits(),
        after.to_bits(),
        "a rebuild (project load / undo restore) dropped the world settings: \
         y={after} instead of {before}"
    );
}

/// **Changing the settings invalidates the scrub cache.**
///
/// Every cached state was simulated under the OLD settings. Replaying from one
/// splices two different worlds together and publishes the result with nothing
/// looking broken — the failure mode the W1.5 wave named as the worst kind of
/// wrong. Its own layer, its own gate
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
///
/// Mutation that must bleed: `set_settings` not calling `ring.clear()`.
#[test]
fn changing_the_settings_drops_the_cached_states() {
    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();
    play(&mut bridge, &mut sim, ball, 120);
    assert!(
        bridge.ring_stats().0 > 0,
        "fixture precondition: 120 ticks of play must have cached something, \
         otherwise this gate cannot observe the cache being dropped"
    );

    bridge.set_settings(draggy());
    assert_eq!(
        bridge.ring_stats().0,
        0,
        "the scrub cache still holds states simulated under the OLD settings; \
         a scrub would replay from one and publish a spliced world"
    );
}

/// **Re-publishing the SAME settings is inert.**
///
/// The panel pushes its state every frame, so this is the common case by three
/// orders of magnitude. It cannot be allowed to do work: `set_body_defaults`
/// wakes every body, so a republish-per-frame would mean a stack that can never
/// fall asleep — and it would also clear the scrub cache on every frame, which
/// quietly turns W1.5 back off.
///
/// Mutation that must bleed: dropping the equality early-out.
#[test]
fn republishing_the_same_settings_changes_nothing() {
    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();
    play(&mut bridge, &mut sim, ball, 120);
    let cached = bridge.ring_stats().0;

    // What the panel does every single frame.
    for _ in 0..10 {
        bridge.set_settings(bridge.settings());
    }

    assert_eq!(
        bridge.ring_stats().0,
        cached,
        "re-publishing identical settings dropped the scrub cache — the panel \
         does this every frame, so W1.5's ring would never hold anything"
    );
}

/// **The range is enforced at the door, not at the slider.**
///
/// A slider cannot be the range: a project file written by a future build, or
/// hand-edited, comes in through here too and would hand the solver a number
/// nothing measured (12 sub-steps is already 101.9% of the HR-4 budget at 500
/// bodies; 200 would be seventeen times it).
///
/// Mutation that must bleed: `set_settings` not calling `clamped()`.
#[test]
fn out_of_range_settings_are_clamped_on_the_way_in() {
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(PhysicsSettings {
        substeps: 200,
        linear_damping: -5.0,
        ..PhysicsSettings::default()
    });
    let got = bridge.settings();
    assert_eq!(
        got.substeps, MAX_SUBSTEPS,
        "200 sub-steps reached the solver; the measured ceiling is {MAX_SUBSTEPS}"
    );
    assert_eq!(
        got.linear_damping, 0.0,
        "negative drag reached the solver (it would ACCELERATE bodies)"
    );
}
