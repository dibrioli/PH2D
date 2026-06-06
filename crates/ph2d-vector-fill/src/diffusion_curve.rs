//! Diffusion-curve **authoring model** (ADR-0060 §2.5, Inovação #2 — the mesh
//! gradient that replaces Illustrator's hand-authored mesh patches).
//!
//! A [`DiffusionCurve`] is a flattened polyline carrying a **colour on each
//! side**; a [`DiffusionCurveSet`] is the full authored input that the Poisson
//! solver ([`crate::poisson_cpu`]) diffuses into a smooth colour field. This is
//! the classic Orzan 2008 formulation: each curve injects its left/right colour
//! as a Dirichlet boundary condition, and the field everywhere else is the
//! harmonic (Laplace) interpolation of those sources.
//!
//! ## Why the side colours live in **OKLab**, not OKLCH
//!
//! The solver diffuses each colour *channel independently*. Diffusing OKLCH's
//! polar hue `h` directly is wrong — interpolating red (`h≈30°`) to blue
//! (`h≈260°`) along the short arc sweeps through magenta, and the wrap at 360°
//! is a discontinuity the Laplace solver cannot see. Diffusing the **Cartesian**
//! OKLab `(L, a, b)` interpolates straight through the gamut (red→…→blue passes
//! near grey), which is what a mesh gradient should do. So every authored
//! [`OklchColor`] is resolved to [`OklabColor`] *here*, at the authoring edge,
//! and the solver only ever sees `(L, a, b)`.
//!
//! Geometry is in **normalized fill space** `[0,1]²` (the region's UV box); the
//! solver rasterizes it onto whatever tier resolution it is handed.

use glam::Vec2;
use ph2d_color::{OklabColor, OklchColor};
use smallvec::SmallVec;

/// Inline capacity for a flattened curve's vertices before spilling to the heap.
/// A cubic Bézier flattens to a handful of segments at screen tolerance; longer
/// authored paths spill, which is fine (authoring is not the hot path).
pub const CURVE_INLINE_POINTS: usize = 8;
/// Inline capacity for per-side colour stops. One stop = a constant side colour
/// (the common case); more stops let the colour vary along the curve.
pub const SIDE_INLINE_STOPS: usize = 4;

/// One colour sample at arc-length parameter `t ∈ [0,1]` along a curve side.
/// Multiple stops (sorted by `t`) make the side colour vary along the curve;
/// a single stop is a constant side colour.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ColorStop {
    /// Arc-length parameter along the curve, `0` = first point, `1` = last.
    pub t: f32,
    /// Authored colour (OKLCH); resolved to OKLab when the solver samples it.
    pub color: OklchColor,
}

impl ColorStop {
    #[inline]
    pub fn new(t: f32, color: OklchColor) -> Self {
        Self { t, color }
    }
}

/// A single diffusion curve: a flattened polyline plus a colour profile on each
/// side. "Left" is the `+normal` side, where `normal = rotate90(tangent)` and
/// `tangent` points from `points[i]` to `points[i+1]` (so left/right is a
/// consistent, orientation-defined choice, not a screen-handedness one).
#[derive(Clone, Debug, PartialEq)]
pub struct DiffusionCurve {
    /// Flattened polyline in normalized `[0,1]²` fill space. Must hold ≥ 2
    /// points; degenerate (< 2) curves are skipped by the solver.
    pub points: SmallVec<[Vec2; CURVE_INLINE_POINTS]>,
    /// Colour stops on the `+normal` ("left") side, sorted by `t`.
    pub left: SmallVec<[ColorStop; SIDE_INLINE_STOPS]>,
    /// Colour stops on the `-normal` ("right") side, sorted by `t`.
    pub right: SmallVec<[ColorStop; SIDE_INLINE_STOPS]>,
}

impl DiffusionCurve {
    /// A straight two-point curve from `a` to `b` with a **constant** colour on
    /// each side — the canonical small validation case (a red-left / blue-right
    /// segment diffuses to two near-flat halves with a smooth seam).
    pub fn straight(a: Vec2, b: Vec2, left: OklchColor, right: OklchColor) -> Self {
        let mut points = SmallVec::new();
        points.push(a);
        points.push(b);
        let mut left_stops = SmallVec::new();
        left_stops.push(ColorStop::new(0.0, left));
        let mut right_stops = SmallVec::new();
        right_stops.push(ColorStop::new(0.0, right));
        Self {
            points,
            left: left_stops,
            right: right_stops,
        }
    }

    /// `true` once the curve has enough geometry and at least one colour per
    /// side to act as a boundary condition.
    pub fn is_valid(&self) -> bool {
        self.points.len() >= 2 && !self.left.is_empty() && !self.right.is_empty()
    }

    /// The total Euclidean length of the polyline (fill-space units). Used to
    /// arc-length–parameterize the colour stops during rasterization.
    pub fn arc_length(&self) -> f32 {
        self.points.windows(2).map(|w| (w[1] - w[0]).length()).sum()
    }

    /// The `+normal`-side colour at arc-length parameter `t ∈ [0,1]`, resolved
    /// to OKLab (the space the solver diffuses in).
    #[inline]
    pub fn left_color_at(&self, t: f32) -> OklabColor {
        side_color_at(&self.left, t)
    }

    /// The `-normal`-side colour at arc-length parameter `t ∈ [0,1]`, in OKLab.
    #[inline]
    pub fn right_color_at(&self, t: f32) -> OklabColor {
        side_color_at(&self.right, t)
    }
}

/// The full authored input to a single mesh-gradient solve: a set of diffusion
/// curves sharing one `[0,1]²` fill space. Referenced from a
/// [`crate::FillNode::MeshGradient`] by id; the id→set resolution is wired by
/// the Coordenador (this crate owns the *model* and the *solver*, not the doc
/// store).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiffusionCurveSet {
    pub curves: Vec<DiffusionCurve>,
}

impl DiffusionCurveSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from an iterator of curves (skips none — the solver skips invalid
    /// ones at rasterization time so callers keep a stable index).
    pub fn from_curves(curves: impl IntoIterator<Item = DiffusionCurve>) -> Self {
        Self {
            curves: curves.into_iter().collect(),
        }
    }

    pub fn push(&mut self, curve: DiffusionCurve) {
        self.curves.push(curve);
    }

    /// `true` when no curve carries usable geometry — the solver short-circuits
    /// to a transparent field instead of running a pointless V-cycle.
    pub fn is_empty(&self) -> bool {
        !self.curves.iter().any(DiffusionCurve::is_valid)
    }
}

/// Convert an authored OKLCH colour to Cartesian OKLab. Mirrors
/// [`OklchColor::to_linear`]'s first step exactly (`a = C·cos h`, `b = C·sin h`,
/// `h` in **degrees**), so the diffused field and a directly-authored OKLCH stop
/// agree at the curve.
#[inline]
pub(crate) fn oklch_to_oklab(c: OklchColor) -> OklabColor {
    let h_rad = c.h.to_radians();
    OklabColor::new(c.l, c.c * h_rad.cos(), c.c * h_rad.sin(), c.a)
}

/// Sample a sorted (by `t`) stop list at `t`, lerping in OKLab. Empty → opaque
/// black (callers gate on [`DiffusionCurve::is_valid`] so this is defensive).
fn side_color_at(stops: &[ColorStop], t: f32) -> OklabColor {
    match stops {
        [] => OklabColor::black(),
        [only] => oklch_to_oklab(only.color),
        _ => {
            let t = t.clamp(0.0, 1.0);
            // Below the first / above the last stop → clamp to the endpoint.
            if t <= stops[0].t {
                return oklch_to_oklab(stops[0].color);
            }
            let last = &stops[stops.len() - 1];
            if t >= last.t {
                return oklch_to_oklab(last.color);
            }
            // Find the bracketing pair and lerp each OKLab channel.
            for pair in stops.windows(2) {
                let (lo, hi) = (&pair[0], &pair[1]);
                if t >= lo.t && t <= hi.t {
                    let span = (hi.t - lo.t).max(f32::EPSILON);
                    let f = (t - lo.t) / span;
                    let a = oklch_to_oklab(lo.color);
                    let b = oklch_to_oklab(hi.color);
                    return OklabColor::new(
                        lerp(a.l, b.l, f),
                        lerp(a.a, b.a, f),
                        lerp(a.b, b.b, f),
                        lerp(a.alpha, b.alpha, f),
                    );
                }
            }
            oklch_to_oklab(last.color)
        }
    }
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
