//! The bridge folds the optional `DampingOverride` into the sim, a rewind RE-ARMS it,
//! and — the reason this one is different — a change to the GLOBAL drag mid-play does
//! NOT clobber a per-body override (W-Damping).
//!
//! `ph2d-physics` proves `BodyDesc.damping` reaches the body. This is the ECS half:
//! a body carrying a linear `DampingOverride` slides visibly less than an identical
//! one without it, and after a scrub back to t=0 it still does. The clobber test is
//! the one that pins the bridge's re-stamp pass: a `Replace` override must keep
//! ignoring the world drag even after the artist raises the global drag while playing
//! — which `PhysicsWorld::set_body_defaults` would otherwise stamp over.
//!
//! Gravity is zeroed so damping is the only thing that can slow the body; the launch
//! reuses `InitialVelocity` (W9). The observable is the body's final X.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, DampMode, DampingOverride, InitialVelocity, PhysicsBridge,
    PhysicsSettings, RigidBody,
};

/// A ball launched right at 5 m/s, optionally carrying a damping override.
fn slider(sim: &mut SimWorld, damping: Option<DampingOverride>) -> Entity {
    let base = (
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        InitialVelocity {
            linvel: [5.0, 0.0],
            angvel: 0.0,
        },
    );
    match damping {
        Some(d) => sim.world_mut().spawn((base, d)).id(),
        None => sim.world_mut().spawn(base).id(),
    }
}

fn zero_gravity(linear_damping: f32) -> PhysicsSettings {
    PhysicsSettings {
        gravity_y: 0.0,
        linear_damping,
        ..Default::default()
    }
}

fn play_to(bridge: &mut PhysicsBridge, sim: &mut SimWorld, tick: u64) {
    let from = bridge.last_stepped();
    for t in (from + 1)..=tick {
        bridge.dispatch(sim, true, t);
    }
}

fn x_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().translation.x
}

#[test]
fn the_bridge_folds_damping_and_a_rewind_preserves_it() {
    // A body with a thick linear drag (Combine, no world drag → the override IS the
    // drag) launched into empty space.
    let mut sim = SimWorld::new();
    let damped = slider(
        &mut sim,
        Some(DampingOverride {
            linear: 3.0,
            angular: 0.0,
            mode: DampMode::Combine,
        }),
    );
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity(0.0));
    play_to(&mut bridge, &mut sim, 60);
    let x_damped = x_of(&sim, damped);

    // The undamped control keeps almost all of its 5 m. Mutating the bridge to ignore
    // the component makes `x_damped` match it and this assertion goes RED.
    let mut sim2 = SimWorld::new();
    let plain = slider(&mut sim2, None);
    let mut bridge2 = PhysicsBridge::new();
    bridge2.set_settings(zero_gravity(0.0));
    play_to(&mut bridge2, &mut sim2, 60);
    let x_plain = x_of(&sim2, plain);
    assert!(
        x_plain > 4.0,
        "sanity: an undamped ball in vacuum should keep sliding (got {x_plain})"
    );
    assert!(
        x_damped < x_plain * 0.6,
        "a linearly-damped body should slide far less than an undamped one, but \
         damped={x_damped} vs plain={x_plain} — the bridge is not folding the override"
    );

    // Scrub back to t=0 and replay: still damped, which it can only be if the override
    // rode the `BodyDesc` the rewind rebuilds from.
    bridge.dispatch(&mut sim, false, 0);
    play_to(&mut bridge, &mut sim, 60);
    let x_damped2 = x_of(&sim, damped);
    assert!(
        (x_damped - x_damped2).abs() < 1e-3,
        "after a rewind the damping was not re-armed (x {x_damped} → {x_damped2})"
    );
}

#[test]
fn a_global_drag_change_mid_play_does_not_clobber_a_replace_override() {
    // Two identical sliders in empty space: one with a Replace(0) override (which
    // IGNORES the world drag outright), one plain (which the world drag governs).
    let mut sim = SimWorld::new();
    let replace = slider(
        &mut sim,
        Some(DampingOverride {
            linear: 0.0,
            angular: 0.0,
            mode: DampMode::Replace,
        }),
    );
    let plain = slider(&mut sim, None);

    let mut bridge = PhysicsBridge::new();
    // No world drag at first — both slide freely.
    bridge.set_settings(zero_gravity(0.0));
    play_to(&mut bridge, &mut sim, 30);

    // The artist raises the GLOBAL drag mid-play. `set_body_defaults` stamps it onto
    // EVERY live body — the Replace override included — which is the clobber. The
    // bridge's re-stamp pass restores the override on the next dispatch.
    bridge.set_settings(zero_gravity(8.0));
    play_to(&mut bridge, &mut sim, 90);

    let x_replace = x_of(&sim, replace);
    let x_plain = x_of(&sim, plain);

    // The plain body was dragged to a near-stop after the change; the Replace body
    // kept sliding because it ignores the world drag. Removing the re-stamp pass lets
    // the global-drag clobber survive, so the Replace body slows just like the plain
    // one and this gap collapses.
    assert!(
        x_replace > x_plain + 1.0,
        "a Replace override must keep ignoring the world drag after a mid-play global \
         change, but replace={x_replace} vs plain={x_plain} — the global drag clobbered \
         the override (is the re-stamp pass missing?)"
    );
}
