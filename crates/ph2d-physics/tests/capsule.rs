//! The capsule collider — the character shape of 2D (ADR-0131, W-capsule).
//!
//! Two kinds of gate here, deliberately:
//!
//! 1. **Geometry** — the capsule (and the non-uniformly-scaled `Stadium` it
//!    degrades to) has the authored extents *in the sim*, read back off the live
//!    rapier collider rather than off the descriptor.
//! 2. **Behaviour** — a capsule gets past a step that catches a box. That is the
//!    entire reason the shape exists; a geometry-only suite would stay green on
//!    a "capsule" that behaved exactly like a box.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

fn desc(shape: ShapeDesc, x: f32, y: f32, body_type: RigidBodyType) -> BodyDesc {
    BodyDesc {
        body_type,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
    }
}

/// The capsule reaches the solver with the extents the artist authored: `radius`
/// wide, `half_height + radius` tall (the rapier decomposition).
///
/// Mutation-tested: building `capsule_y(radius, half_height)` (arguments
/// swapped) makes both extents wrong.
#[test]
fn a_capsule_collider_has_the_authored_extents() {
    let (half_height, radius) = (0.6_f32, 0.25_f32);
    let mut w = PhysicsWorld::new();
    w.spawn_body(desc(
        ShapeDesc::Capsule {
            half_height,
            radius,
        },
        0.0,
        0.0,
        RigidBodyType::Dynamic,
    ));

    let (_, collider) = w.colliders().iter().next().expect("collider inserted");
    let aabb = collider.compute_aabb();
    assert!(
        (aabb.maxs.x - radius).abs() < 1e-4,
        "capsule half-width is {}, expected the cap radius {radius}",
        aabb.maxs.x
    );
    assert!(
        (aabb.maxs.y - (half_height + radius)).abs() < 1e-4,
        "capsule half-height is {}, expected segment + cap = {}",
        aabb.maxs.y,
        half_height + radius
    );
}

/// A capsule under **non-uniform** scale becomes a `Stadium` — a convex polygon
/// with ELLIPTICAL caps — and it too carries the authored extents. Same
/// discipline as the ellipse: the collider must match the drawn sprite.
#[test]
fn a_stadium_collider_has_the_authored_extents() {
    let (half_height, rx, ry) = (0.4_f32, 0.5_f32, 0.2_f32);
    let mut w = PhysicsWorld::new();
    w.spawn_body(desc(
        ShapeDesc::Stadium {
            half_height,
            rx,
            ry,
        },
        0.0,
        0.0,
        RigidBodyType::Dynamic,
    ));

    let (_, collider) = w.colliders().iter().next().expect("collider inserted");
    let aabb = collider.compute_aabb();
    assert!(
        (aabb.maxs.x - rx).abs() < 1e-4,
        "stadium half-width is {}, expected {rx}",
        aabb.maxs.x
    );
    assert!(
        (aabb.maxs.y - (half_height + ry)).abs() < 1e-4,
        "stadium half-height is {}, expected {}",
        aabb.maxs.y,
        half_height + ry
    );
}

/// **The capsule gets past a step that catches the box** — the reason the shape
/// exists.
///
/// Identical bodies in every way that matters (same total half-extent 0.25, same
/// start, same friction, same push) run at a 0.15 m lip in the ground. The box's
/// flat bottom and square corner hit the lip's vertical face; the capsule's
/// round cap rides over it. Gravity is tilted to supply the push, because a body
/// has no authored initial velocity in this engine — the sideways component is
/// the "walk".
///
/// The oracle is DISTANCE TRAVELLED PAST THE LIP, not "did it move": both bodies
/// slide on the flat approach, so a gate that only asked whether x grew would be
/// green for a box that stopped dead at the step.
#[test]
fn a_capsule_climbs_a_step_that_stops_a_box() {
    let lip_x = 0.0_f32;
    let run = |shape: ShapeDesc| {
        let mut w = PhysicsWorld::new();
        // Push right and down: a tilted gravity is the walk.
        w.set_gravity(6.0, -9.81);
        // Lower ground: top at y = 0.1, spanning x = -10..0.
        w.spawn_body(desc(
            ShapeDesc::Cuboid {
                half_x: 5.0,
                half_y: 0.05,
            },
            -5.0,
            0.05,
            RigidBodyType::Fixed,
        ));
        // Raised ground past the lip: top at y = 0.25, spanning x = 0..10.
        // The 0.15 m difference is the step.
        w.spawn_body(desc(
            ShapeDesc::Cuboid {
                half_x: 5.0,
                half_y: 0.125,
            },
            5.0,
            0.125,
            RigidBodyType::Fixed,
        ));
        // The runner: total half-extent 0.25 either way, resting on the lower
        // ground (top 0.1) so its centre starts at 0.35.
        let body = w.spawn_body(desc(shape, -1.0, 0.35, RigidBodyType::Dynamic));
        for _ in 0..180 {
            w.step();
        }
        w.body_pose(body).expect("body").translation.vector.x
    };

    let box_x = run(ShapeDesc::Cuboid {
        half_x: 0.25,
        half_y: 0.25,
    });
    let capsule_x = run(ShapeDesc::Capsule {
        half_height: 0.10,
        radius: 0.15,
    });

    assert!(
        capsule_x > lip_x,
        "the capsule never got past the lip (x = {capsule_x}) — it is not \
         behaving like a capsule"
    );
    assert!(
        capsule_x > box_x + 0.5,
        "the capsule ({capsule_x}) did not clear the step meaningfully further \
         than the box ({box_x}) — the shape makes no difference, which is the \
         only thing this wave is for"
    );
}
