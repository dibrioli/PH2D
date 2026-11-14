//! The CSS/AE cubic-bézier timing solver behind [`crate::Interp::Bezier`] — split from `curve.rs`
//! (file cap) verbatim: one solve, shared by `remap`, `slope` and `curve_weighted`.

/// Evaluate one axis of the cubic bézier `B(s)` with `P0 = 0`, `P3 = 1` and
/// control coordinates `p1`, `p2`. Polynomial (transcendental-free).
pub(crate) fn bezier_axis(p1: f64, p2: f64, s: f64) -> f64 {
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
pub(crate) fn solve_cubic_bezier(x1: f64, y1: f64, x2: f64, y2: f64, x: f64) -> f64 {
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
pub(crate) fn bezier_slope(x1: f64, y1: f64, x2: f64, y2: f64, x: f64) -> f64 {
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
