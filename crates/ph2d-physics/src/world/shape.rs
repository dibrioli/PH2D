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

impl ShapeDesc {
    /// **How far out is this local point, as a fraction of the way from the shape's
    /// CENTRE to its BOUNDARY along the ray through it** — `0` at the centre, exactly
    /// `1` on the boundary in *every* direction, `> 1` outside (W-AreaFalloff).
    ///
    /// `p` is in the shape's own local frame (the collider's, so an offset collider
    /// measures from where it actually sits). This is the ruler a force zone's falloff
    /// reads, and the reason it is *this* measure rather than a raw distance:
    ///
    /// - **It needs no second number.** A falloff radius of its own would be a length
    ///   the artist has to keep in step with the zone's size — the recurring "two doors
    ///   to one quantity" failure. Here the zone's own silhouette IS the extent.
    /// - **It reaches zero exactly at the boundary, on every side.** A body leaving the
    ///   area therefore leaves through a push that has already faded to nothing, instead
    ///   of stepping off a cliff — which is the artefact a fade exists to remove.
    /// - **Its iso-contours are the shape SCALED about its centre**, because the measure
    ///   is invariant under any linear map (`t(Sp; S·shape) == t(p; shape)` — scaling a
    ///   ray through the origin scales its boundary point by the same factor). That is
    ///   what lets the overlay draw the half-strength ring by simply halving the shape,
    ///   through the same `scaled_shape` door the outline uses, with no second geometry;
    ///   and it is why the measure composes with W6's collider scaling for free.
    ///
    /// Three closed forms and no iteration: a `Ball` is an `Ellipse` with equal radii, a
    /// `Capsule` is a `Stadium` with equal cap radii, and every `Stadium` is a
    /// unit-radius capsule seen through `diag(rx, ry)` — so the two round families
    /// collapse into one kernel each. ⚠️ **No `hypot`, no transcendental** (law 6): `hypot`
    /// is the platform's libm and is not pinned across OSes, and this number reaches the
    /// impulses the `physics_ecs_c9` hash compares between Linux, macOS and Windows.
    /// Everything here is `+ - * /` and `sqrt`, all correctly rounded by IEEE-754.
    ///
    /// A degenerate shape (a zero half-extent) answers `0` — *do not attenuate*. There is
    /// no interior to measure across, and the honest failure for an undefined ruler is to
    /// leave the value it would have scaled exactly as it was.
    #[must_use]
    pub fn radial_fraction(self, p: [f32; 2]) -> f32 {
        let [x, y] = p;
        match self {
            Self::Ball { radius } => Self::ellipse_fraction(x, y, radius, radius),
            Self::Ellipse { rx, ry } => Self::ellipse_fraction(x, y, rx, ry),
            Self::Cuboid { half_x, half_y } => {
                if half_x <= 0.0 || half_y <= 0.0 {
                    return 0.0;
                }
                // The ray from the centre leaves a box through whichever slab it
                // saturates first, so the fraction is the larger of the two — and this
                // is exactly `1` on the whole rectangle, corners included.
                (x.abs() / half_x).max(y.abs() / half_y)
            }
            Self::Capsule {
                half_height,
                radius,
            } => Self::stadium_fraction(x, y, half_height, radius, radius),
            Self::Stadium {
                half_height,
                rx,
                ry,
            } => Self::stadium_fraction(x, y, half_height, rx, ry),
        }
    }

    /// Room for the largest [`snap_points`](ShapeDesc::snap_points) answer — a
    /// `Cuboid`'s nine (centre, four corners, four edge midpoints).
    pub const MAX_SNAP_POINTS: usize = 9;

    /// **The points on this shape an artist aims at** — its centre and its
    /// extremes, in the shape's own local frame. Returns how many were written.
    ///
    /// This is the candidate set a joint anchor snaps to (W-J2), and it is the
    /// COLLIDER's, never the sprite quad's: a joint attaches to a *body*, and a
    /// body's shape is the thing the solver collides with. Snapping to the
    /// picture would put the dot where the physics is not.
    ///
    /// The vocabulary is deliberately the same one the pivot handle already
    /// snaps to (`ph2d_editor::pivot_snap_candidates`: centre / corners / edge
    /// mids) — a second answer to *"what can I snap a point to?"* is how two
    /// handles in the same editor come to feel like different programs. What
    /// differs is that a round shape has no corners, and inventing some would
    /// offer the artist a point that is **not on the body**:
    ///
    /// | shape | points |
    /// |---|---|
    /// | `Cuboid` | centre · 4 corners · 4 edge midpoints (9) |
    /// | `Ball` / `Ellipse` | centre · 4 cardinal rim points (5) |
    /// | `Capsule` / `Stadium` | centre · 2 cap centres · 2 poles · 2 barrel sides (7) |
    ///
    /// A capsule's **cap centres** earn their place: they are where a limb
    /// pivots, and they are invisible on the outline, so they are exactly the
    /// point an artist cannot hit by eye.
    ///
    /// A degenerate shape (a zero half-extent) still answers — the extremes
    /// simply collapse onto the centre, and the nearest-wins search that consumes
    /// this treats duplicates as one point.
    #[must_use]
    pub fn snap_points(self, out: &mut [[f32; 2]; Self::MAX_SNAP_POINTS]) -> usize {
        out[0] = [0.0, 0.0];
        match self {
            Self::Ball { radius } => Self::cardinal_rim(out, radius, radius),
            Self::Ellipse { rx, ry } => Self::cardinal_rim(out, rx, ry),
            Self::Cuboid { half_x, half_y } => {
                out[1] = [-half_x, half_y];
                out[2] = [half_x, half_y];
                out[3] = [-half_x, -half_y];
                out[4] = [half_x, -half_y];
                out[5] = [0.0, half_y];
                out[6] = [half_x, 0.0];
                out[7] = [0.0, -half_y];
                out[8] = [-half_x, 0.0];
                9
            }
            Self::Capsule {
                half_height,
                radius,
            } => Self::stadium_points(out, half_height, radius, radius),
            Self::Stadium {
                half_height,
                rx,
                ry,
            } => Self::stadium_points(out, half_height, rx, ry),
        }
    }

    /// Centre plus the four cardinal points of an axis-aligned ellipse.
    fn cardinal_rim(out: &mut [[f32; 2]; Self::MAX_SNAP_POINTS], rx: f32, ry: f32) -> usize {
        out[1] = [0.0, ry];
        out[2] = [rx, 0.0];
        out[3] = [0.0, -ry];
        out[4] = [-rx, 0.0];
        5
    }

    /// Centre, the two cap centres, the two poles and the two barrel sides.
    fn stadium_points(
        out: &mut [[f32; 2]; Self::MAX_SNAP_POINTS],
        half_height: f32,
        rx: f32,
        ry: f32,
    ) -> usize {
        out[1] = [0.0, half_height];
        out[2] = [0.0, -half_height];
        out[3] = [0.0, half_height + ry];
        out[4] = [0.0, -half_height - ry];
        out[5] = [rx, 0.0];
        out[6] = [-rx, 0.0];
        7
    }

    /// The fraction for an axis-aligned ellipse: normalise each axis by its own radius
    /// and the boundary becomes the unit circle, so the fraction is just the length of
    /// the normalised point. (A circle is the case `rx == ry`.)
    fn ellipse_fraction(x: f32, y: f32, rx: f32, ry: f32) -> f32 {
        if rx <= 0.0 || ry <= 0.0 {
            return 0.0;
        }
        let (u, v) = (x / rx, y / ry);
        (u * u + v * v).sqrt()
    }

    /// The fraction for a stadium — a Y-aligned segment of half-length `half_height`
    /// capped by half-ellipses of radii `rx`/`ry`.
    ///
    /// Normalising by `(rx, ry)` turns it into a **unit-radius** capsule of half-height
    /// `half_height / ry`, and the measure is invariant under that map, so one kernel
    /// serves both the exact capsule and the scaled stadium.
    ///
    /// From the centre, the ray leaves either through a straight flank (`|u| = 1`) or
    /// through a cap (the unit circle about `(0, ±h)`). The two cases are complementary:
    /// the flank applies while `dv <= h·du`, and outside it `h·du < dv <= 1`, which is
    /// precisely what keeps the cap's discriminant `1 − h²du²` non-negative — so neither
    /// branch needs a guard beyond the degenerate-size one.
    fn stadium_fraction(x: f32, y: f32, half_height: f32, rx: f32, ry: f32) -> f32 {
        if rx <= 0.0 || ry <= 0.0 {
            return 0.0;
        }
        let (u, v) = (x / rx, y / ry);
        let h = (half_height / ry).max(0.0);
        let len = (u * u + v * v).sqrt();
        if len <= 0.0 {
            return 0.0;
        }
        let (du, dv) = (u.abs() / len, v.abs() / len);
        let boundary = if dv <= h * du {
            1.0 / du
        } else {
            h * dv + (1.0 - h * h * du * du).max(0.0).sqrt()
        };
        len / boundary
    }
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
