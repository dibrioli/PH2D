//! Weighted (value-space) tangent evaluation — the engine of
//! [`Interp::BezierW`](crate::Interp). Sibling of `curve.rs` under the
//! workspace file cap: `curve.rs` owns the enum and dispatch, this module owns
//! the math.
//!
//! A weighted segment is a cubic bézier in the `(u, value)` plane:
//! `P0 = (0, v0)` · `P1 = (x1, v0 + dy1)` · `P2 = (x2, v1 + dy2)` · `P3 = (1, v1)`.
//! `x` is an influence fraction (CSS-clamped to `[0, 1]`, so the timing axis
//! stays monotone and the SAME Newton/bisection solve `remap` uses applies);
//! `dy` is an **absolute value offset** from its anchor — AE keyframe-velocity /
//! Blender F-curve semantics. That is what lets a flat segment (`v0 == v1`)
//! curve at all: the normalized [`Interp::Bezier`](crate::Interp) stores its
//! handle y as a *fraction of the value change*, and a zero change has nothing
//! to scale (the `handle_coords` gap this variant closes).
//!
//! All polynomial — deterministic (HR-5), zero-alloc.

use crate::curve::{bezier_axis_deriv, solve_bezier_param};

/// One axis of a cubic bézier with ARBITRARY endpoints `p0 → p3` and control
/// values `c1`, `c2` (the general form of `curve::bezier_axis`, which is the
/// `0 → 1` special case).
fn cubic(p0: f64, c1: f64, c2: f64, p3: f64, s: f64) -> f64 {
    let c = 3.0 * (c1 - p0);
    let b = 3.0 * (c2 - c1) - c;
    let a = (p3 - p0) - c - b;
    ((a * s + b) * s + c) * s + p0
}

/// Derivative of [`cubic`] w.r.t. `s`.
fn cubic_deriv(p0: f64, c1: f64, c2: f64, p3: f64, s: f64) -> f64 {
    let c = 3.0 * (c1 - p0);
    let b = 3.0 * (c2 - c1) - c;
    let a = (p3 - p0) - c - b;
    (3.0 * a * s + 2.0 * b) * s + c
}

/// The weighted segment's VALUE at `u` — solve the timing axis for the curve
/// parameter (the same solver `remap` uses), then evaluate the value axis at it.
pub(crate) fn value(v0: f64, v1: f64, x1: f64, dy1: f64, x2: f64, dy2: f64, u: f64) -> f64 {
    let s = solve_bezier_param(x1, x2, u);
    cubic(v0, v0 + dy1, v1 + dy2, v1, s)
}

/// `d(value)/du` of the weighted segment — the parametric chain rule
/// `value'(s) / x'(s)`, with the same degenerate handling as the normalized
/// [`curve::bezier_slope`](crate::curve) (ported from Chromium's
/// `cubic_bezier.cc`): a `0/0` endpoint falls through to the next control
/// point; an interior cusp is flat; a vertical tangent is the true `±∞`
/// (callers skip non-finite for display).
pub(crate) fn value_slope(v0: f64, v1: f64, x1: f64, dy1: f64, x2: f64, dy2: f64, u: f64) -> f64 {
    let s = solve_bezier_param(x1, x2, u);
    let dx = bezier_axis_deriv(x1, x2, s);
    let dy = cubic_deriv(v0, v0 + dy1, v1 + dy2, v1, s);
    if dx == 0.0 && dy == 0.0 {
        return if s <= 0.0 {
            start_gradient(v0, v1, x1, dy1, x2, dy2)
        } else if s >= 1.0 {
            end_gradient(v0, v1, x1, dy1, x2, dy2)
        } else {
            0.0
        };
    }
    dy / dx
}

/// The start tangent when the direct ratio is `0/0` (`P1` coincident with
/// `P0`): the tangent falls through to the line toward `P2`, then the chord —
/// the `InitGradients` cascade in value space.
fn start_gradient(v0: f64, v1: f64, x1: f64, dy1: f64, x2: f64, dy2: f64) -> f64 {
    if x1 > 0.0 {
        dy1 / x1
    } else if dy1 == 0.0 && x2 > 0.0 {
        ((v1 + dy2) - v0) / x2
    } else if dy1 == 0.0 && dy2 == 0.0 {
        v1 - v0
    } else {
        0.0
    }
}

/// The end tangent when the direct ratio is `0/0` (`P2` coincident with `P3`).
fn end_gradient(v0: f64, v1: f64, x1: f64, dy1: f64, x2: f64, dy2: f64) -> f64 {
    if x2 < 1.0 {
        -dy2 / (1.0 - x2)
    } else if dy2 == 0.0 && x1 < 1.0 {
        (v1 - (v0 + dy1)) / (1.0 - x1)
    } else if dy2 == 0.0 && dy1 == 0.0 {
        v1 - v0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use crate::Interp;

    #[test]
    fn a_flat_segment_can_finally_bulge() {
        // THE point of weighted tangents: v0 == v1 == 5, both handles lifted
        // +2 in absolute value → the curve rises above 5 mid-segment. The
        // normalized Bezier provably cannot draw this (dv = 0 scales any
        // handle y to nothing).
        let w = Interp::bezier_w(1.0 / 3.0, 2.0, 2.0 / 3.0, 2.0);
        let mid = w.value(5.0, 5.0, 0.5);
        assert!(mid > 6.0, "the flat segment bulges: {mid}");
        // The endpoints stay pinned to the keys.
        assert_eq!(w.value(5.0, 5.0, 0.0), 5.0);
        assert_eq!(w.value(5.0, 5.0, 1.0), 5.0);
        // Control: the normalized Bezier with any handle y stays dead flat.
        let n = Interp::bezier(1.0 / 3.0, 5.0, 2.0 / 3.0, 5.0);
        let flat = n.value(5.0, 5.0, 0.5);
        assert_eq!(flat, 5.0, "normalized bezier cannot bulge a flat segment");
    }

    #[test]
    fn a_weighted_segment_with_dy_from_the_normalized_form_draws_the_same_curve() {
        // Losslessness of the upgrade path: normalized (x, hy) and weighted
        // (x, dy = hy·dv) — with dy2 = (hy2 − 1)·dv — are the SAME curve, so
        // converting a legacy handle to W on drag never jumps.
        let (v0, v1) = (2.0, 12.0); // dv = 10
        let n = Interp::bezier(0.3, 0.8, 0.7, 1.4); // overshooting normalized
        let w = Interp::bezier_w(0.3, 0.8 * 10.0, 0.7, (1.4 - 1.0) * 10.0);
        for i in 0..=32 {
            let u = f64::from(i) / 32.0;
            let (a, b) = (n.value(v0, v1, u), w.value(v0, v1, u));
            assert!((a - b).abs() < 1e-9, "diverged at u = {u}: {a} vs {b}");
        }
    }

    #[test]
    fn the_value_slope_is_the_derivative_of_the_value() {
        let w = Interp::bezier_w(0.25, 3.0, 0.75, -2.0);
        let (v0, v1) = (5.0, 5.0); // flat keys, curved segment
        let h = 1e-5;
        for i in 1..40 {
            let u = f64::from(i) / 40.0;
            let fd = (w.value(v0, v1, u + h) - w.value(v0, v1, u - h)) / (2.0 * h);
            let got = w.value_slope(v0, v1, u);
            assert!(
                (got - fd).abs() < 1e-3 + fd.abs() * 1e-2,
                "u = {u}: slope {got} vs fd {fd}"
            );
        }
        // Endpoint slopes: dy1/x1 at the start, −dy2/(1−x2) at the end.
        assert!((w.value_slope(v0, v1, 0.0) - 3.0 / 0.25).abs() < 1e-9);
        assert!((w.value_slope(v0, v1, 1.0) - 2.0 / 0.25).abs() < 1e-9);
    }

    #[test]
    fn the_degenerate_cascade_holds_in_value_space() {
        // P1 on P0 (x1 = 0, dy1 = 0): tangent toward P2.
        let w = Interp::bezier_w(0.0, 0.0, 0.5, -1.0);
        let s = w.value_slope(0.0, 10.0, 0.0);
        assert!(
            ((10.0 - 1.0) / 0.5 - s).abs() < 1e-9,
            "toward P2 = (0.5, 9): {s}"
        );
        // A vertical start (x1 = 0 with dy1 ≠ 0) is the true ±∞.
        let w = Interp::bezier_w(0.0, 4.0, 0.5, 0.0);
        assert!(w.value_slope(0.0, 10.0, 0.0).is_infinite());
    }

    #[test]
    fn legacy_variants_route_value_and_value_slope_through_the_old_math() {
        // For the normalized variants, value()/value_slope() must agree with
        // v0 + dv·remap(u) and dv·slope(u) — the display/speed funnel and the
        // sampler stay two views of one curve.
        let cases = [
            Interp::Linear,
            Interp::Hold,
            Interp::bezier(0.42, 0.0, 0.58, 1.0),
        ];
        let (v0, v1) = (-3.0, 7.0);
        for interp in cases {
            for i in 0..=20 {
                let u = f64::from(i) / 20.0;
                let want = v0 + (v1 - v0) * interp.remap(u);
                assert!((interp.value(v0, v1, u) - want).abs() < 1e-12);
                let want = (v1 - v0) * interp.slope(u);
                assert!((interp.value_slope(v0, v1, u) - want).abs() < 1e-12);
            }
        }
    }
}
