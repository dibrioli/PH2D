//! Per-segment interpolation ([`Interp`]) + the standalone editable curve
//! ([`AnimCurve`], implementing [`AnimationCurveSampler`]).
//!
//! [`Interp`] maps a normalized segment fraction `u ∈ [0, 1]` to an eased
//! fraction `v ∈ [0, 1]`; the actual value blend is always delegated to
//! [`LinearInterp::lerp`] (so OKLCH colours keep their short-arc hue behaviour).

use core::cmp::Ordering;

use ph2d_vector_traits::{AnimValue, AnimationCurveSampler, LinearInterp};
use serde::{Deserialize, Serialize};

use crate::easing::Easing;

/// How a keyframe segment is interpolated from its start value to its end value.
///
/// The variant belongs to the **outgoing** key of a segment (the interpolation
/// *leaving* that key toward the next one).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Interp {
    /// Stepped: hold the start value for the whole segment (no blend).
    Hold,
    /// Linear: the eased fraction equals `u`.
    Linear,
    /// A named easing preset.
    Eased(Easing),
    /// A CSS/After-Effects cubic-bézier timing curve with control points
    /// `P1 = (x1, y1)`, `P2 = (x2, y2)` between the fixed `P0 = (0, 0)` and
    /// `P3 = (1, 1)`. `Linear`/`Eased` are special cases kept as fast paths.
    Bezier {
        /// `P1.x`, kept in `[0, 1]` so the timing function stays monotone in `u`.
        x1: f64,
        /// `P1.y` (may leave `[0, 1]` to overshoot).
        y1: f64,
        /// `P2.x`, kept in `[0, 1]`.
        x2: f64,
        /// `P2.y` (may leave `[0, 1]` to overshoot).
        y2: f64,
    },
    /// A **weighted / value-space** tangent pair (W5): a cubic bézier in the
    /// `(u, value)` plane, `P1 = (x1, v0 + dy1)`, `P2 = (x2, v1 + dy2)` — AE
    /// keyframe-velocity / Blender F-curve semantics. `x` is an influence
    /// fraction (clamped to `[0, 1]` like [`Interp::Bezier`]); `dy` is an
    /// ABSOLUTE value offset from its anchor, which is what lets a flat
    /// segment (`v0 == v1`) curve — the normalized form scales its handle y
    /// by the value change and a zero change has nothing to scale.
    ///
    /// Scalar-exact through [`Interp::value`]; a non-scalar [`AnimValue`]
    /// blends by the unit-span remap instead (unreachable in the v1 timeline,
    /// whose properties are all `Float`). Appended LAST so postcard's variant
    /// indexes — and every saved document — stay stable.
    BezierW {
        /// `P1.x` — the out handle's influence, kept in `[0, 1]`.
        x1: f64,
        /// `P1.y − v0` — the out handle's absolute value offset from its key.
        dy1: f64,
        /// `P2.x` — the in handle's influence, kept in `[0, 1]`.
        x2: f64,
        /// `P2.y − v1` — the in handle's absolute value offset from its key.
        dy2: f64,
    },
}

impl Interp {
    /// Build a cubic-bézier timing curve, clamping the control-point *x*
    /// coordinates into `[0, 1]` (CSS validity — a non-monotone `x` has no
    /// single solution).
    #[must_use]
    pub fn bezier(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Interp::Bezier {
            x1: x1.clamp(0.0, 1.0),
            y1,
            x2: x2.clamp(0.0, 1.0),
            y2,
        }
    }

    /// Build a weighted (value-space) tangent pair, clamping the influence
    /// `x` coordinates into `[0, 1]` (monotone timing — same rule as
    /// [`Interp::bezier`]). The `dy`s are free: they are absolute value
    /// offsets, and overshoot is their whole point.
    #[must_use]
    pub fn bezier_w(x1: f64, dy1: f64, x2: f64, dy2: f64) -> Self {
        Interp::BezierW {
            x1: x1.clamp(0.0, 1.0),
            dy1,
            x2: x2.clamp(0.0, 1.0),
            dy2,
        }
    }

    /// Map a normalized segment fraction `u` to an eased fraction `v`.
    ///
    /// `Hold` returns `0.0` (so a downstream `lerp(start, end, 0)` holds the
    /// start value — `Hold` needs no special-casing at the call site).
    /// `BezierW` has no normalized form — it reads as its unit-span curve
    /// (`dy` treated as a fraction), the documented fallback for non-scalar
    /// blends; scalars evaluate exactly through [`Interp::value`].
    #[must_use]
    pub fn remap(self, u: f64) -> f64 {
        let u = u.clamp(0.0, 1.0);
        match self {
            Interp::Hold => 0.0,
            Interp::Linear => u,
            Interp::Eased(e) => e.eval(u),
            Interp::Bezier { x1, y1, x2, y2 } => solve_cubic_bezier(x1, y1, x2, y2, u),
            Interp::BezierW { x1, dy1, x2, dy2 } => {
                crate::curve_weighted::value(0.0, 1.0, x1, dy1, x2, dy2, u)
            }
        }
    }

    /// The segment's VALUE at `u`, given its endpoint values — the scalar
    /// funnel the graph editor and the speed graph read. The normalized
    /// variants blend `v0 + (v1 − v0) · remap(u)`; `BezierW` evaluates its
    /// value-axis bézier exactly (its `dy`s are absolute, so a flat segment
    /// can curve).
    #[must_use]
    pub fn value(self, v0: f64, v1: f64, u: f64) -> f64 {
        match self {
            Interp::BezierW { x1, dy1, x2, dy2 } => {
                crate::curve_weighted::value(v0, v1, x1, dy1, x2, dy2, u.clamp(0.0, 1.0))
            }
            _ => v0 + (v1 - v0) * self.remap(u),
        }
    }

    /// `d(value)/du` at `u`, given the endpoint values — what the speed graph
    /// plots (divide by the segment span for units-per-second). The normalized
    /// variants scale [`Interp::slope`] by the value change; `BezierW`
    /// differentiates its value axis directly (same parametric chain rule +
    /// degenerate cascade). May be `±∞` at a vertical tangent.
    #[must_use]
    pub fn value_slope(self, v0: f64, v1: f64, u: f64) -> f64 {
        match self {
            Interp::BezierW { x1, dy1, x2, dy2 } => {
                crate::curve_weighted::value_slope(v0, v1, x1, dy1, x2, dy2, u.clamp(0.0, 1.0))
            }
            _ => (v1 - v0) * self.slope(u),
        }
    }

    /// The timing function's slope `dP/du` at `u` — the derivative of
    /// [`Interp::remap`], for the speed graph.
    ///
    /// Analytic where a closed form exists: `Hold = 0`, `Linear = 1`, and
    /// `Bezier` by the parametric chain rule `y'(s) / x'(s)` on the SAME
    /// Newton/bisection solve `remap` uses (ported from the CSS reference,
    /// Chromium `cubic_bezier.cc` — see [`bezier_slope`] for the degenerate
    /// cascade). `Eased` has no closed form exposed, so it is a central
    /// difference of [`Easing::eval`] (unclamped, unlike the handle-placement
    /// [`endpoint_slope`]).
    ///
    /// May be `±∞` at a vertical tangent (`Bezier` with a control point on the
    /// segment edge) — display code skips non-finite values.
    #[must_use]
    pub fn slope(self, u: f64) -> f64 {
        match self {
            Interp::Hold => 0.0,
            Interp::Linear => 1.0,
            Interp::Eased(e) => eased_slope(e, u),
            Interp::Bezier { x1, y1, x2, y2 } => bezier_slope(x1, y1, x2, y2, u),
            // The unit-span slope (`dy` read as a fraction) — consistent with
            // what `remap` samples. Value-space callers use `value_slope`.
            Interp::BezierW { x1, dy1, x2, dy2 } => {
                crate::curve_weighted::value_slope(0.0, 1.0, x1, dy1, x2, dy2, u.clamp(0.0, 1.0))
            }
        }
    }

    /// `true` if this interpolation is transcendental-free.
    ///
    /// `Hold`, `Linear` and `Bezier` (solved by polynomial Newton/bisection)
    /// are always deterministic; `Eased` defers to [`Easing::is_deterministic`].
    #[must_use]
    pub fn is_deterministic(self) -> bool {
        match self {
            Interp::Eased(e) => e.is_deterministic(),
            _ => true,
        }
    }

    /// The bézier handles of a straight-line (`Linear`) segment — the control
    /// points of the cubic that reproduces `y = x`. Used as the starting point
    /// when the graph editor upgrades a non-bézier segment to `Bezier`.
    pub const LINEAR_HANDLES: ((f64, f64), (f64, f64)) =
        ((1.0 / 3.0, 1.0 / 3.0), (2.0 / 3.0, 2.0 / 3.0));

    /// The two editable bézier handles in normalized `[0, 1]²` timing space —
    /// `((out.x, out.y), (in.x, in.y))` = `(P1, P2)`, with the endpoints `P0 =
    /// (0, 0)` / `P3 = (1, 1)` fixed.
    ///
    /// `Bezier` returns its own control points; `Linear` returns
    /// [`Interp::LINEAR_HANDLES`]. `Hold`/`Eased` have no 2-handle representation
    /// (`None`) — the editor paints their curve read-only and offers to convert
    /// by dragging (which routes through [`Interp::with_out_handle`] /
    /// [`Interp::with_in_handle`], starting from the linear handles).
    #[must_use]
    pub fn handles(self) -> Option<((f64, f64), (f64, f64))> {
        match self {
            Interp::Bezier { x1, y1, x2, y2 } => Some(((x1, y1), (x2, y2))),
            Interp::Linear => Some(Self::LINEAR_HANDLES),
            // `BezierW` IS a two-handle form, but not a normalized one — its
            // `dy`s are value-space and need the endpoint values to place
            // (`ph2d-timeline`'s segment-handle helpers own that mapping).
            Interp::Hold | Interp::Eased(_) | Interp::BezierW { .. } => None,
        }
    }

    /// The two handles a graph editor should **draw**, for any interpolation.
    ///
    /// Unlike [`Interp::handles`] (which answers "is this exactly a two-handle
    /// bézier?"), this always answers "where does the curve point?". Each handle
    /// sits a third of the segment along `x`, on the curve's **tangent** at its
    /// anchor — so a `Cubic InOut` shows its flat ends instead of the straight
    /// chord [`Interp::LINEAR_HANDLES`] would draw across them.
    ///
    /// It is also the starting point for a drag: [`Interp::with_out_handle`]
    /// keeps the *other* handle from here, so converting an eased segment to
    /// `Bezier` leaves the untouched end's shape where it was.
    ///
    /// `Hold` has no two-handle form at all (it is flat, then jumps), so both of
    /// its handles land on the flat part — where the curve actually is.
    #[must_use]
    pub fn tangent_handles(self) -> ((f64, f64), (f64, f64)) {
        // The handles sit at these `x`, exactly where `LINEAR_HANDLES` puts them
        // (same literals, so `Linear` reaches the identical floats either way).
        // A cubic through `(0,0)`/`(1,1)` then has endpoint slopes `y1 / x1` and
        // `(1 - y2) / (1 - x2)`; invert that to place `y` from a measured slope.
        let (x1, x2) = (Self::LINEAR_HANDLES.0.0, Self::LINEAR_HANDLES.1.0);
        match self {
            Interp::Bezier { x1, y1, x2, y2 } => ((x1, y1), (x2, y2)),
            Interp::Linear => Self::LINEAR_HANDLES,
            Interp::Hold => ((x1, 0.0), (x2, 0.0)),
            Interp::Eased(e) => {
                let m0 = endpoint_slope(e, 0.0);
                let m1 = endpoint_slope(e, 1.0);
                ((x1, m0 * x1), (x2, 1.0 - m1 * (1.0 - x2)))
            }
            // No normalized form (value-space `dy` needs the endpoint values,
            // which this signature cannot see). Only the influence survives;
            // value-space callers use `ph2d-timeline`'s segment-handle helpers
            // — no production path reaches here with a `BezierW`.
            Interp::BezierW { x1, x2, .. } => ((x1, 0.0), (x2, 1.0)),
        }
    }

    /// Replace the **out** handle (`P1`) with `(x1, y1)`, upgrading to `Bezier`.
    /// The **in** handle is kept from [`Interp::tangent_handles`], so an eased
    /// segment converts without the far end jumping. `x1` is clamped to `[0, 1]`.
    #[must_use]
    pub fn with_out_handle(self, x1: f64, y1: f64) -> Interp {
        let (_, (x2, y2)) = self.tangent_handles();
        Interp::bezier(x1, y1, x2, y2)
    }

    /// Replace the **in** handle (`P2`) with `(x2, y2)`, upgrading to `Bezier`.
    /// The **out** handle is kept from [`Interp::tangent_handles`]. `x2` is
    /// clamped to `[0, 1]`.
    #[must_use]
    pub fn with_in_handle(self, x2: f64, y2: f64) -> Interp {
        let ((x1, y1), _) = self.tangent_handles();
        Interp::bezier(x1, y1, x2, y2)
    }

    /// Freeze this interpolation into the `Bezier` **drawn through the handles it
    /// already shows** — the graph editor's "Custom" preset, and the same curve a
    /// drag would land on if it grabbed a handle and put it back down.
    ///
    /// The result is not always the same *curve*: only the endpoint slopes are
    /// preserved, and `Hold`, `Bounce` and `Elastic` have no cubic form at all. It
    /// is the same **handles**, which is what "make this editable" has to mean —
    /// the alternative (silently keeping a shape no two handles can express) would
    /// leave the editor lying about what it draws.
    #[must_use]
    pub fn to_bezier(self) -> Interp {
        match self {
            // Both bézier forms ARE already the editable two-handle shape the
            // graph draws — "make this editable" has nothing left to do.
            Interp::Bezier { .. } | Interp::BezierW { .. } => self,
            _ => {
                let ((x1, y1), (x2, y2)) = self.tangent_handles();
                Interp::bezier(x1, y1, x2, y2)
            }
        }
    }
}

/// Step used to measure an easing's slope at an endpoint. Small enough to read
/// as a tangent, large enough that `f(u + h) - f(u)` keeps its significant bits.
const SLOPE_H: f64 = 1e-4;

/// Slopes are clamped here before becoming a handle: `Expo`/`Back` leave their
/// anchor almost vertically, and an unbounded handle would fly off the band and
/// drag the vertical fit with it. The tangent's *direction* survives; only an
/// absurd length is cut.
const MAX_SLOPE: f64 = 16.0;

/// `f'(u)` of an easing at an endpoint, by one-sided finite difference (the
/// interior side, so the sample never leaves `[0, 1]` where `eval` clamps).
fn endpoint_slope(e: Easing, u: f64) -> f64 {
    let (a, b) = if u <= 0.0 {
        (0.0, SLOPE_H)
    } else {
        (1.0 - SLOPE_H, 1.0)
    };
    ((e.eval(b) - e.eval(a)) / SLOPE_H).clamp(-MAX_SLOPE, MAX_SLOPE)
}

/// Blend two values across a segment: `lerp(v0, v1, interp.remap(u))`.
///
/// Shared by [`crate::Track`] and [`AnimCurve`] so both interpolate identically.
/// A weighted segment's `dy` is ABSOLUTE, so a scalar evaluates exactly through
/// [`Interp::value`] instead of the normalized lerp funnel; every other
/// `AnimValue` kind falls back to the unit-span remap blend (unreachable in the
/// v1 timeline, whose properties are all `Float`). The normalized variants keep
/// the original `lerp(remap)` float path bit-for-bit.
pub(crate) fn interpolate(v0: AnimValue, v1: AnimValue, interp: Interp, u: f64) -> AnimValue {
    if let (Interp::BezierW { .. }, AnimValue::Float(a), AnimValue::Float(b)) = (interp, v0, v1) {
        return AnimValue::Float(interp.value(f64::from(a), f64::from(b), u) as f32);
    }
    AnimValue::lerp(v0, v1, interp.remap(u))
}

/// One control point of an [`AnimCurve`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CurveKey {
    /// The curve parameter (usually in `[0, 1]`, but any finite value is valid).
    pub u: f64,
    /// The value at this point.
    #[serde(with = "crate::anim_value_serde")]
    pub value: AnimValue,
    /// The interpolation *leaving* this point toward the next.
    pub interp: Interp,
}

/// A first-class, editable animation curve — the object of the future graph
/// editor. Sampled by parameter via [`AnimationCurveSampler::at`].
///
/// Control points are kept sorted by `u`. Sampling outside the point range
/// clamps to the first/last value (flat ends).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "AnimCurveData", into = "AnimCurveData")]
pub struct AnimCurve {
    keys: Vec<CurveKey>,
    default: AnimValue,
}

/// Serde proxy for [`AnimCurve`] — round-trips through [`AnimCurve::new`] so the
/// sorted-by-`u` invariant is re-established on load.
#[derive(Serialize, Deserialize)]
struct AnimCurveData {
    keys: Vec<CurveKey>,
    #[serde(with = "crate::anim_value_serde")]
    default: AnimValue,
}

impl From<AnimCurve> for AnimCurveData {
    fn from(c: AnimCurve) -> Self {
        Self {
            keys: c.keys,
            default: c.default,
        }
    }
}

impl From<AnimCurveData> for AnimCurve {
    fn from(d: AnimCurveData) -> Self {
        let mut c = AnimCurve::new(d.keys);
        c.default = d.default;
        c
    }
}

impl AnimCurve {
    /// Build a curve from control points (sorted by `u` internally).
    ///
    /// An empty curve samples to `default` — chosen as the first point's value,
    /// or `AnimValue::Float(0.0)` when there are no points (documented explicit
    /// default; never a silent empty body).
    #[must_use]
    pub fn new(mut keys: Vec<CurveKey>) -> Self {
        keys.sort_by(|a, b| a.u.partial_cmp(&b.u).unwrap_or(Ordering::Equal));
        let default = keys.first().map_or(AnimValue::Float(0.0), |k| k.value);
        Self { keys, default }
    }

    /// A two-point curve blending `from → to` over `u ∈ [0, 1]` with the given
    /// easing. Behaviourally the real replacement for
    /// `ph2d_vector_traits::MockAnimationCurveSampler` when the mock's linear
    /// blend is generalised to any preset.
    #[must_use]
    pub fn ease(from: AnimValue, to: AnimValue, easing: Easing) -> Self {
        Self::new(vec![
            CurveKey {
                u: 0.0,
                value: from,
                interp: Interp::Eased(easing),
            },
            CurveKey {
                u: 1.0,
                value: to,
                interp: Interp::Hold,
            },
        ])
    }

    /// A two-point straight linear blend `from → to` over `u ∈ [0, 1]`.
    #[must_use]
    pub fn linear(from: AnimValue, to: AnimValue) -> Self {
        Self::new(vec![
            CurveKey {
                u: 0.0,
                value: from,
                interp: Interp::Linear,
            },
            CurveKey {
                u: 1.0,
                value: to,
                interp: Interp::Hold,
            },
        ])
    }

    /// A two-point cubic-bézier blend `from → to` over `u ∈ [0, 1]`.
    #[must_use]
    pub fn bezier(from: AnimValue, to: AnimValue, x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self::new(vec![
            CurveKey {
                u: 0.0,
                value: from,
                interp: Interp::bezier(x1, y1, x2, y2),
            },
            CurveKey {
                u: 1.0,
                value: to,
                interp: Interp::Hold,
            },
        ])
    }

    /// The control points, sorted by `u`.
    #[must_use]
    pub fn keys(&self) -> &[CurveKey] {
        &self.keys
    }

    /// `true` if the curve has no control points.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Number of control points.
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }
}

impl AnimationCurveSampler for AnimCurve {
    fn at(&self, t: f64) -> AnimValue {
        sample_sorted(&self.keys, self.default, t)
    }
}

/// Sample a sorted `[CurveKey]` slice at parameter `t` (flat-clamped ends).
/// Zero-allocation: binary search + arithmetic only.
fn sample_sorted(keys: &[CurveKey], default: AnimValue, t: f64) -> AnimValue {
    match keys {
        [] => default,
        [only] => only.value,
        _ => {
            let first = keys[0];
            let last = keys[keys.len() - 1];
            if t <= first.u {
                return first.value;
            }
            if t >= last.u {
                return last.value;
            }
            // Largest index `i` with `keys[i].u <= t`.
            let idx = keys
                .partition_point(|k| k.u <= t)
                .saturating_sub(1)
                .min(keys.len() - 2);
            let k0 = keys[idx];
            let k1 = keys[idx + 1];
            let span = k1.u - k0.u;
            let u = if span > 0.0 { (t - k0.u) / span } else { 0.0 };
            interpolate(k0.value, k1.value, k0.interp, u)
        }
    }
}

/// Evaluate one axis of the cubic bézier `B(s)` with `P0 = 0`, `P3 = 1` and
/// control coordinates `p1`, `p2`. Polynomial (transcendental-free).
fn bezier_axis(p1: f64, p2: f64, s: f64) -> f64 {
    let c = 3.0 * p1;
    let b = 3.0 * (p2 - p1) - c;
    let a = 1.0 - c - b;
    ((a * s + b) * s + c) * s
}

/// Derivative of [`bezier_axis`] w.r.t. `s`. `pub(crate)`: `curve_weighted`
/// differentiates its timing axis with the SAME polynomial.
pub(crate) fn bezier_axis_deriv(p1: f64, p2: f64, s: f64) -> f64 {
    let c = 3.0 * p1;
    let b = 3.0 * (p2 - p1) - c;
    let a = 1.0 - c - b;
    (3.0 * a * s + 2.0 * b) * s + c
}

/// Solve a CSS/AE cubic-bézier timing curve: given `x`, find the curve
/// parameter `s` with `bezier_x(s) == x`, then return `bezier_y(s)`.
///
/// Newton–Raphson from `s = x` with a bisection fallback — all polynomial, so
/// the result is **deterministic** (no transcendental ops).
fn solve_cubic_bezier(x1: f64, y1: f64, x2: f64, y2: f64, x: f64) -> f64 {
    bezier_axis(y1, y2, solve_bezier_param(x1, x2, x))
}

/// The parameter half of [`solve_cubic_bezier`]: the `s` with
/// `bezier_x(s) == x`. Split out so [`bezier_slope`] — and `curve_weighted`'s
/// value-space evaluation — differentiate the SAME solve `remap` samples; a
/// second solver would drift.
pub(crate) fn solve_bezier_param(x1: f64, x2: f64, x: f64) -> f64 {
    const EPS: f64 = 1e-7;
    let x = x.clamp(0.0, 1.0);

    // Newton–Raphson.
    let mut s = x;
    for _ in 0..8 {
        let err = bezier_axis(x1, x2, s) - x;
        if err.abs() < EPS {
            return s;
        }
        let d = bezier_axis_deriv(x1, x2, s);
        if d.abs() < EPS {
            break;
        }
        s -= err / d;
    }

    // Bisection fallback on `[0, 1]`.
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    let mut s = x.clamp(lo, hi);
    while hi - lo > EPS {
        let cx = bezier_axis(x1, x2, s);
        if (cx - x).abs() < EPS {
            break;
        }
        if x > cx {
            lo = s;
        } else {
            hi = s;
        }
        s = 0.5 * (lo + hi);
    }
    s
}

/// The slope `dy/dx` of a CSS cubic-bézier timing curve at input `x` — the
/// parametric chain rule `y'(s) / x'(s)` at the solved parameter, exactly as
/// the reference implementation computes it (Chromium
/// `ui/gfx/geometry/cubic_bezier.cc`, `SlopeWithEpsilon`).
///
/// Degenerate cases follow the reference too:
/// - `0/0` at an endpoint (control point coincident with it): the tangent is
///   derived from the next control point — the `InitGradients` cascade.
/// - `0/0` in the interior (a cusp): `0`, per `SlopeWithEpsilon`.
/// - `x' == 0` with `y' != 0` (vertical tangent): the true limit `±∞` — the
///   reference squeezes it through `ToFinite`; callers here guard/skip
///   non-finite for display instead.
fn bezier_slope(x1: f64, y1: f64, x2: f64, y2: f64, x: f64) -> f64 {
    let s = solve_bezier_param(x1, x2, x);
    let dx = bezier_axis_deriv(x1, x2, s);
    let dy = bezier_axis_deriv(y1, y2, s);
    if dx == 0.0 && dy == 0.0 {
        return if s <= 0.0 {
            bezier_start_gradient(x1, y1, x2, y2)
        } else if s >= 1.0 {
            bezier_end_gradient(x1, y1, x2, y2)
        } else {
            0.0
        };
    }
    dy / dx
}

/// The bézier's tangent slope at its START when the direct ratio is `0/0`
/// (`P1` coincident with `P0`) — ported verbatim from Chromium
/// `cubic_bezier.cc` `InitGradients`: the tangent falls through to the line
/// toward the next non-coincident control point.
fn bezier_start_gradient(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    if x1 > 0.0 {
        y1 / x1
    } else if y1 == 0.0 && x2 > 0.0 {
        y2 / x2
    } else if y1 == 0.0 && y2 == 0.0 {
        1.0
    } else {
        0.0
    }
}

/// The bézier's tangent slope at its END when the direct ratio is `0/0`
/// (`P2` coincident with `P3`) — the `InitGradients` end cascade.
fn bezier_end_gradient(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    if x2 < 1.0 {
        (y2 - 1.0) / (x2 - 1.0)
    } else if y2 == 1.0 && x1 < 1.0 {
        (y1 - 1.0) / (x1 - 1.0)
    } else if y2 == 1.0 && y1 == 1.0 {
        1.0
    } else {
        0.0
    }
}

/// `f'(u)` of an easing anywhere in `[0, 1]`, by central difference (one-sided
/// at the ends). Unlike [`endpoint_slope`] this is NOT clamped: it feeds the
/// speed graph, which must show the true rate, not a handle-length policy.
fn eased_slope(e: Easing, u: f64) -> f64 {
    let u = u.clamp(0.0, 1.0);
    let a = (u - SLOPE_H).max(0.0);
    let b = (u + SLOPE_H).min(1.0);
    (e.eval(b) - e.eval(a)) / (b - a)
}

#[cfg(test)]
#[path = "curve_slope_tests.rs"]
mod slope_tests;
