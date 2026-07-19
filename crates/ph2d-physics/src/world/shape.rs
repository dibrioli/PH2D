//! The collider silhouette — plain data, no rapier types.
//!
//! Split out of `world.rs` so the shape vocabulary (and the ellipse
//! tessellation that grew with W6) lives in one small place instead of pushing
//! the wrapper past its LOC cap. Re-exported from [`crate::world`], so callers
//! still see `ph2d_physics::ShapeDesc` / `ellipse_vertices`.

/// Collider silhouette for [`crate::world::PhysicsWorld::spawn_body`]. Kept as
/// plain data (no rapier types) so the ECS bridge can build one without a
/// direct `rapier2d` dependency — rapier stays confined to this crate
/// (SKILL §7 "don't couple public API to external types"). **Append-only:**
/// new variants go at the END (Capsule/Triangle/… land later).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ShapeDesc {
    /// Circle of `radius` (world units = meters).
    Ball { radius: f32 },
    /// Axis-aligned box with HALF-extents (rapier convention).
    Cuboid { half_x: f32, half_y: f32 },
    /// Axis-aligned ellipse, `rx`/`ry` the HALF-extents (world units).
    ///
    /// rapier has no native ellipse, so it is realised as a convex
    /// polygon ([`ellipse_vertices`] → `ColliderBuilder::convex_polyline`).
    /// It exists because a [`Ball`](ShapeDesc::Ball) under **non-uniform**
    /// scale is genuinely an ellipse on screen, and the collider must match
    /// the drawn sprite (ADR-0131 W6; the same principle as the collider
    /// outline). A ball under *uniform* scale stays a `Ball` — an exact
    /// circle is cheaper and rounder than any polygon.
    Ellipse { rx: f32, ry: f32 },
}

/// How many vertices approximate an ellipse collider. Shared with the
/// overlay so the wireframe traces the **same** polygon the solver sees
/// (a smoother outline over a coarser collider would be a wireframe that
/// lies). 32 is smooth at any zoom a body is readable at, and matches the
/// overlay's circle tessellation.
pub const ELLIPSE_SEGS: u32 = 32;

/// The vertices of an ellipse with half-extents `rx`/`ry`, in CCW order,
/// as plain `[x, y]` in local space (world units). [`ELLIPSE_SEGS`] of
/// them.
///
/// One door for the tessellation: the collider build (`PhysicsWorld::spawn_body`)
/// and the overlay draw from this **same** function, so they cannot
/// disagree about where the ellipse's edge is.
///
/// Determinism (HR-5): `sin`/`cos` route through `libm::sincosf`
/// (pure-Rust, one implementation across every target), never `f32::sin_cos`
/// (platform-math, differs in the last ulps). This is the exact discipline
/// `Transform::compose` follows, and it is what lets a scaled-ellipse body
/// hash bit-identically on Linux/macOS/Windows in `physics_ecs_c9`.
#[must_use]
pub fn ellipse_vertices(rx: f32, ry: f32) -> Vec<[f32; 2]> {
    (0..ELLIPSE_SEGS)
        .map(|i| {
            let a = f32::from(i as u16) * core::f32::consts::TAU / ELLIPSE_SEGS as f32;
            let (s, c) = libm::sincosf(a);
            [c * rx, s * ry]
        })
        .collect()
}
