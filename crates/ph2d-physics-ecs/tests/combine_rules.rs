//! The bridge folds the optional `MaterialCombine` component into the sim, and a
//! rewind RE-ARMS it (collision-material combine, W-Material).
//!
//! `ph2d-physics` proves `BodyDesc.material` reaches the collider. This is the ECS
//! half, tested through the OUTCOME: a superball (restitution 1.0) carrying
//! `MaterialCombine{restitution: Max}` bounces off a plain, DEAD floor (restitution
//! 0.0) back to nearly its drop height, while the identical superball WITHOUT the
//! component averages with the floor (effective 0.5) and returns to only about a
//! quarter of it. After a scrub back to t=0 and replay it still bounces high — which
//! it can only if the combine rule rode the `BodyDesc` the world rebuilds from.
//!
//! The observable is the ball's peak height after the bounce (the bridge writes the
//! solved pose back into `Transform`).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, CombineRule, MaterialCombine, PhysicsBridge, RigidBody,
};

/// A plain static floor at y=0 — DEFAULT restitution 0.0 and DEFAULT (Average)
/// combine, so the falling ball's own rule is the only thing that varies.
fn floor(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 5.0,
                half_y: 0.1,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
}

/// A superball (restitution 1.0) dropped from y=3, optionally carrying a
/// restitution combine rule.
fn superball(sim: &mut SimWorld, restitution_rule: Option<CombineRule>) -> Entity {
    let base = (
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            restitution: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 3.0)),
    );
    match restitution_rule {
        Some(rule) => sim
            .world_mut()
            .spawn((
                base,
                MaterialCombine {
                    restitution: rule,
                    friction: CombineRule::Average,
                },
            ))
            .id(),
        None => sim.world_mut().spawn(base).id(),
    }
}

/// Play 150 ticks (2.5 s) — long enough to fall, bounce, and reach the rebound
/// apex — and return the peak height the ball reaches AFTER it first comes down
/// near the floor (so the initial drop is excluded from the peak).
fn peak_rebound_height(bridge: &mut PhysicsBridge, sim: &mut SimWorld, ball: Entity) -> f32 {
    let mut bounced = false;
    let mut peak = 0.0f32;
    for tick in 1..=150u64 {
        bridge.dispatch(sim, true, tick);
        let y = sim.world().get::<Transform>(ball).unwrap().translation.y;
        if y < 0.8 {
            bounced = true;
        }
        if bounced {
            peak = peak.max(y);
        }
    }
    peak
}

#[test]
fn the_bridge_folds_the_combine_rule_and_a_rewind_preserves_it() {
    // MAX-combine superball on a dead floor: bounces back near its drop height.
    let mut sim = SimWorld::new();
    floor(&mut sim);
    let ball = superball(&mut sim, Some(CombineRule::Max));
    let mut bridge = PhysicsBridge::new();
    let max_apex = peak_rebound_height(&mut bridge, &mut sim, ball);

    // A perfect bounce returns most of the 2.65 m drop. Mutating the bridge's
    // `world.get::<MaterialCombine>(e)` to ignore the component (always default
    // Average) makes this ball average with the dead floor and rebound to only
    // ~1 m — this assertion goes RED.
    assert!(
        max_apex > 2.0,
        "a Max-combine superball should bounce off the dead floor back near its drop \
         height, but its rebound apex was y={max_apex} — the bridge is not folding the \
         combine rule into the sim"
    );

    // NEUTRAL control: the identical superball WITHOUT the component averages with
    // the dead floor (effective restitution 0.5) and returns to only ~a quarter of
    // the drop — the contrast proving the high bounce is the rule's doing.
    let mut sim2 = SimWorld::new();
    floor(&mut sim2);
    let ball2 = superball(&mut sim2, None);
    let mut bridge2 = PhysicsBridge::new();
    let neutral_apex = peak_rebound_height(&mut bridge2, &mut sim2, ball2);
    assert!(
        neutral_apex < 1.5,
        "a superball averaging with a dead floor should return to about a quarter of \
         its drop, but its apex was y={neutral_apex} — the fixture no longer contains \
         the phenomenon"
    );
    assert!(
        max_apex > neutral_apex + 1.0,
        "the Max-combine ball ({max_apex}) should clearly out-bounce the averaging one \
         ({neutral_apex})"
    );

    // Scrub back to t=0 and replay: it must still bounce high, which it can only if
    // the combine rule rode the `BodyDesc` the rewind rebuilds from.
    bridge.dispatch(&mut sim, false, 0);
    let max_apex2 = peak_rebound_height(&mut bridge, &mut sim, ball);
    assert!(
        (max_apex - max_apex2).abs() < 0.2,
        "after a rewind to t=0 the combine rule was not re-armed (apex {max_apex} → \
         {max_apex2}) — it was read once and lost on scrub"
    );
}
