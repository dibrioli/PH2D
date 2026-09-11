//! The bridge folds `Collider.offset` into the sim, and a rewind preserves it
//! (W-Offset).
//!
//! `ph2d-physics` proves `BodyDesc::offset` reaches the collider. This is the ECS
//! half: (1) a body whose collider is offset upward rests LOWER than a centred
//! one through the bridge, and (2) after scrubbing the clock back to t=0 and
//! replaying, it still rests there — the offset rides the `BodyDesc` the world
//! rebuilds from (it is on the `Collider`, which `body_desc` reads every frame).
//!
//! Mutating `scale::body_desc`'s `offset: [col.offset[0] * .., ..]` to `[0.0, 0.0]`
//! makes the offset body rest at the centred height, and assertion (1) goes RED.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

/// Drop a ball onto a floor with the given collider offset; return where its
/// entity's `Transform` settles. `x` keeps two drops apart.
fn scene(x: f32, offset: [f32; 2]) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
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
                shape: ColliderShape::Ball { radius: 0.5 },
                density: 1.0,
                offset,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 6.0)),
        ))
        .id();
    (sim, ball)
}

fn y_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().translation.y
}

#[test]
fn the_bridge_folds_the_collider_offset_and_a_rewind_preserves_it() {
    let (mut sim, centred) = scene(-3.0, [0.0, 0.0]);
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=300u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let centred_y = y_of(&sim, centred);

    let (mut sim2, raised) = scene(3.0, [0.0, 2.0]);
    let mut bridge2 = PhysicsBridge::new();
    for tick in 1..=300u64 {
        bridge2.dispatch(&mut sim2, true, tick);
    }
    let raised_y1 = y_of(&sim2, raised);

    // Folded: the collider offset 2 m up makes the body centre rest ~2 m lower.
    assert!(
        (raised_y1 - (centred_y - 2.0)).abs() < 0.05,
        "the body with a +2 m collider offset should rest ~2 m below the centred \
         one (≈{}), but it is at {raised_y1} — the bridge is not folding \
         Collider.offset into the sim",
        centred_y - 2.0
    );

    // Scrub back to t=0 and replay: it must still rest there.
    bridge2.dispatch(&mut sim2, false, 0);
    for tick in 1..=300u64 {
        bridge2.dispatch(&mut sim2, true, tick);
    }
    let raised_y2 = y_of(&sim2, raised);
    assert_eq!(
        raised_y1, raised_y2,
        "after a rewind the collider offset was not preserved (y {raised_y1} → \
         {raised_y2})"
    );
}
