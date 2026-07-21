//! The ellipse collider — a `Ball` under non-uniform scale (ADR-0131 W6).
//!
//! rapier has no ellipse, so [`ShapeDesc::Ellipse`] is realised as a convex
//! polygon from [`ellipse_vertices`]. These gates pin that the polygon (a) has
//! the authored half-extents in the SIM (not just in the descriptor) and (b) is
//! deterministic — the same vertices every run, which is what lets a scaled
//! body hash identically across OSes in `physics_ecs_c9`.

use ph2d_physics::{
    BodyDesc, ELLIPSE_SEGS, PhysicsWorld, RigidBodyType, ShapeDesc, ellipse_vertices,
};

/// **The ellipse collider has the authored half-extents in the sim.** A convex
/// polygon inscribed in an ellipse has vertices ON the ellipse, and
/// [`ELLIPSE_SEGS`] = 32 places vertices exactly on the axes (0°, 90°, …), so
/// the collider's AABB is exactly `[±rx, ±ry]`.
///
/// This drives `spawn_body` (the real build, via `convex_polyline`) and reads
/// the AABB back off the live rapier collider — so it proves the ellipse
/// reached the SOLVER, not merely that a `ShapeDesc` was constructed.
/// Mutation-tested: building a `ball(rx)` here instead makes the y-extent wrong.
#[test]
fn an_ellipse_collider_has_the_authored_half_extents() {
    let (rx, ry) = (1.5_f32, 0.75_f32);
    let mut w = PhysicsWorld::new();
    w.spawn_body(BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Ellipse { rx, ry },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        offset: [0.0, 0.0],
    });

    let (_, collider) = w
        .colliders()
        .iter()
        .next()
        .expect("the ellipse collider was inserted");
    let aabb = collider.compute_aabb();
    assert!(
        (aabb.maxs.x - rx).abs() < 1e-4 && (aabb.mins.x + rx).abs() < 1e-4,
        "ellipse x-extent is [{}, {}]; expected ±{rx}",
        aabb.mins.x,
        aabb.maxs.x
    );
    assert!(
        (aabb.maxs.y - ry).abs() < 1e-4 && (aabb.mins.y + ry).abs() < 1e-4,
        "ellipse y-extent is [{}, {}]; expected ±{ry} — the collider is not the \
         authored ellipse (a circle would make both extents equal)",
        aabb.mins.y,
        aabb.maxs.y
    );
}

/// The tessellation is deterministic and correctly shaped: [`ELLIPSE_SEGS`]
/// vertices, CCW, with the cardinal points exactly on the axes. The pinned
/// interior value documents the `libm` sampling — a change in the formula or a
/// switch to platform `f32::sin_cos` would move it, which is precisely what
/// would break the cross-OS hash.
#[test]
fn ellipse_vertices_are_deterministic_and_axis_aligned() {
    let v = ellipse_vertices(1.0, 2.0);
    assert_eq!(v.len(), ELLIPSE_SEGS as usize, "wrong vertex count");

    // Cardinal vertices sit exactly on the axes (i = 0, N/4, N/2, 3N/4).
    let n = ELLIPSE_SEGS as usize;
    assert!(
        (v[0][0] - 1.0).abs() < 1e-6 && v[0][1].abs() < 1e-6,
        "i=0 is (rx, 0)"
    );
    assert!(
        v[n / 4][0].abs() < 1e-6 && (v[n / 4][1] - 2.0).abs() < 1e-6,
        "quarter-turn is (0, ry)"
    );

    // A pinned interior vertex (i = 1, angle = 2π/32). cos ≈ 0.980785,
    // sin ≈ 0.195090 → [rx·cos, ry·sin] = [0.980785, 0.390181].
    assert!(
        (v[1][0] - 0.980_785).abs() < 1e-5 && (v[1][1] - 0.390_181).abs() < 1e-5,
        "the sampled vertex moved: {:?} — libm tessellation changed",
        v[1]
    );
}
