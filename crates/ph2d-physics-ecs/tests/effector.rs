//! The bridge folds the optional `AreaEffector` into the sim, and a rewind RE-ARMS
//! it (force zones, W-Area).
//!
//! `ph2d-physics` proves `BodyDesc.effector` reaches the world and pushes what
//! overlaps it. This is the ECS half, tested through the OUTCOME an artist would
//! check: a box falling past a wind column is blown sideways, while an identical box
//! outside it falls straight. After a scrub back to t=0 it is blown the same way —
//! which it can only if the force rode the `BodyDesc` the world rebuilds from.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{AreaEffector, BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

/// A static SENSOR box carrying a force — the zone. `sensor` is a parameter because
/// the coupling ("a solid zone pushes nothing") is half the contract.
fn zone(sim: &mut SimWorld, x: f32, force: [f32; 2], sensor: bool) {
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 1.0,
                half_y: 3.0,
            },
            density: 1.0,
            is_sensor: sensor,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(x, 0.0)),
        AreaEffector { force },
    ));
}

/// A box dropped from above, at `x`.
fn faller(sim: &mut SimWorld, x: f32) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.2,
                    half_y: 0.2,
                },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 2.5)),
        ))
        .id()
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
fn the_bridge_folds_the_force_zone_and_a_rewind_preserves_it() {
    let mut sim = SimWorld::new();
    zone(&mut sim, 0.0, [1.0, 0.0], true);
    let inside = faller(&mut sim, 0.0);
    // ⚠️ The control sits UPWIND, not merely far away. Downwind it was rammed by the
    // very body the column had just launched (a body that "must fall straight" ended
    // 2.7 m from where it started) — the fixture, not the product: a control has to be
    // out of the experiment's way, and here the way is a direction.
    let outside = faller(&mut sim, -6.0);
    let mut bridge = PhysicsBridge::new();
    play_to(&mut bridge, &mut sim, 60);

    let blown = x_of(&sim, inside);
    assert!(
        blown > 0.3,
        "a body falling through the wind column should be carried sideways, but it is \
         at x={blown} — the bridge is not folding `AreaEffector` into the sim"
    );
    assert!(
        (x_of(&sim, outside) + 6.0).abs() < 1e-4,
        "a body outside the column must fall straight, but it is at x={}",
        x_of(&sim, outside)
    );

    // Scrub back to t=0 and replay: blown the same way, to the same place.
    bridge.dispatch(&mut sim, false, 0);
    play_to(&mut bridge, &mut sim, 60);
    let blown2 = x_of(&sim, inside);
    assert!(
        (blown2 - blown).abs() < 1e-3,
        "after a rewind the zone was not re-armed (x {blown} -> {blown2}) — the force \
         was read once and lost on the scrub"
    );
}

#[test]
fn a_solid_zone_pushes_nothing() {
    // ⚠️ The coupling, at the only layer where it is observable as a SIM outcome:
    // a solid collider records no intersection, so it has nobody to push — and it
    // BLOCKS instead. The §11 rows mirror this by only offering Force for a sensor
    // (its seam gate), and `effector::zone_force` states it once for both.
    let mut sim = SimWorld::new();
    zone(&mut sim, 0.0, [8.0, 0.0], false);
    // Dropped beside the solid box, not onto it, so this measures the absence of a
    // push rather than a collision.
    let e = faller(&mut sim, 1.6);
    let mut bridge = PhysicsBridge::new();
    play_to(&mut bridge, &mut sim, 60);
    assert!(
        (x_of(&sim, e) - 1.6).abs() < 1e-4,
        "a SOLID zone must not push: the body should have fallen straight, x={}",
        x_of(&sim, e)
    );
}

#[test]
fn the_bridge_folds_the_area_drag_and_a_rewind_preserves_it() {
    // The medium half (W-AreaDrag), through the outcome an artist would check: a box
    // dropped into a pool arrives at the bottom slower than the identical box dropped
    // beside it. Two separate components reach the world as ONE effect, so this also
    // proves the bundling — a body carrying only `AreaDrag` and no `AreaEffector` still
    // becomes a zone.
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 1.0,
                half_y: 2.0,
            },
            is_sensor: true,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        ph2d_physics_ecs::AreaDrag(6.0),
    ));
    let in_pool = faller(&mut sim, 0.0);
    let in_air = faller(&mut sim, 6.0);
    let mut bridge = PhysicsBridge::new();
    play_to(&mut bridge, &mut sim, 60);

    fn y(sim: &SimWorld, e: Entity) -> f32 {
        sim.world().get::<Transform>(e).unwrap().translation.y
    }
    let (wet, dry) = (y(&sim, in_pool), y(&sim, in_air));
    assert!(
        wet > dry + 0.5,
        "the box that fell through the pool should be held up relative to the one in \
         open air ({wet} vs {dry}) — the bridge is not folding `AreaDrag` into the sim"
    );

    // Scrub to t=0 and replay: the same resistance, so the drag rode the `BodyDesc`.
    bridge.dispatch(&mut sim, false, 0);
    play_to(&mut bridge, &mut sim, 60);
    assert!(
        (y(&sim, in_pool) - wet).abs() < 1e-3,
        "after a rewind the pool was not re-armed ({wet} -> {})",
        y(&sim, in_pool)
    );
}
