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
/// new variants go at the END (Triangle/Polygon land later).
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
    /// **Y-aligned capsule** — a segment of half-length `half_height` with a
    /// circular cap of `radius` at each end. Total half-extent along Y is
    /// therefore `half_height + radius`, the rapier convention.
    ///
    /// This is the **character collider** of 2D: a box catches on tile seams and
    /// ramp corners, a capsule slides over them. Y-aligned only, deliberately —
    /// it is the orientation Unity's `CapsuleCollider2D` and Godot's
    /// `CapsuleShape2D` default to, and a capsule that lies down is a capsule on
    /// a rotated body (the `Transform` already rotates the whole collider). An
    /// axis flag would be a second way to say the same thing.
    Capsule { half_height: f32, radius: f32 },
    /// A capsule under **non-uniform** scale: the caps become elliptical, so it
    /// is no longer a capsule any solver can represent exactly. Realised as a
    /// convex polygon ([`capsule_vertices`]), exactly as
    /// [`Ellipse`](ShapeDesc::Ellipse) is — same reason, same discipline: the
    /// collider must match the sprite that is drawn (ADR-0131 W6). A capsule
    /// under *uniform* scale stays an exact [`Capsule`](ShapeDesc::Capsule).
    ///
    /// (A stadium is the proper name for a rectangle capped by two half-discs.)
    Stadium { half_height: f32, rx: f32, ry: f32 },
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

/// Vertices per cap of a capsule/stadium outline — half of [`ELLIPSE_SEGS`], so
/// the two caps together are as smooth as a circle of the same tessellation.
pub const CAPSULE_CAP_SEGS: u32 = ELLIPSE_SEGS / 2;

/// The vertices of a **stadium** — a Y-aligned segment of half-length
/// `half_height` capped by half-ellipses of radii `rx`/`ry` — in CCW order, as
/// plain `[x, y]` in local space (world units).
///
/// The same one door [`ellipse_vertices`] is: the collider build and the overlay
/// trace this **same** function, so the wireframe cannot describe a different
/// edge than the solver collides with. Pass `rx == ry` for a true (circular-cap)
/// capsule outline — which is what the overlay does for
/// [`ShapeDesc::Capsule`], so the drawn shape matches rapier's exact capsule.
///
/// Determinism (HR-5): `libm::sincosf`, never `f32::sin_cos` — same reasoning as
/// [`ellipse_vertices`], and what keeps a capsule body hashing identically
/// across the three OSes in `physics_ecs_c9`.
///
/// The two caps do **not** share vertices: the top cap ends at `(-rx, +hh)` and
/// the bottom begins at `(-rx, -hh)`, which are different points, so the straight
/// flanks fall out of the ordering rather than needing a special case.
#[must_use]
pub fn capsule_vertices(half_height: f32, rx: f32, ry: f32) -> Vec<[f32; 2]> {
    let cap = |centre_y: f32, start: f32| {
        (0..=CAPSULE_CAP_SEGS).map(move |i| {
            let a = start + f32::from(i as u16) * core::f32::consts::PI / CAPSULE_CAP_SEGS as f32;
            let (s, c) = libm::sincosf(a);
            [c * rx, centre_y + s * ry]
        })
    };
    // Top cap sweeps 0..π (right → up → left) above +hh, bottom cap π..2π
    // (left → down → right) below −hh: CCW overall.
    cap(half_height, 0.0)
        .chain(cap(-half_height, core::f32::consts::PI))
        .collect()
}
