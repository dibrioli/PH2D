//! Hobby's algorithm — minimum-curvature-variation interpolating spline.
//!
//! Per plan task **T2.1** (`docs/Vector Module/17_plano_de_implementacao.md`
//! §5): the Pencil tool smooths a recorded freehand stroke into a cubic
//! Bézier spline that **passes through** a decimated set of knots while
//! minimizing curvature variation (the "most pleasing" curve). This is
//! John D. Hobby's algorithm (MetaPost's `..` path join), explicitly
//! **NOT** Schneider's least-squares fit (the Inkscape default the plan
//! flags as the anti-pattern).
//!
//! ## Relationship to [`crate::cubic_fit`]
//!
//! [`cubic_fit::fit_cubic_levien`](crate::cubic_fit::fit_cubic_levien)
//! *approximates* a single smooth arc with one cubic (it does not pass
//! through the interior samples). Hobby *interpolates*: the returned
//! spline passes through every knot exactly, with one cubic per knot
//! interval. The Pencil decimates its raw samples to knots upstream
//! (≈ 1 knot per 10 samples per the plan DoD) so the interpolation
//! smooths jitter without preserving every wobble.
//!
//! ## Method (Hobby 1986; Jackowski velocity)
//!
//! For an open chain of knots `z₀ … zₙ` (n segments) we solve for the
//! tangent **direction** at each knot such that *mock curvature* is
//! continuous across knots (G1 smoothness with a curvature-fairness
//! objective). The unknowns are the departure angles `αᵢ` measured from
//! each chord; the arrival angles follow as `βᵢ = −γᵢ₊₁ − αᵢ₊₁` where
//! `γ` are the polygon turning angles. The αᵢ satisfy a tridiagonal
//! linear system (tension = 1):
//!
//! ```text
//! row 0      :  (2+ω)·α₀ + (2ω+1)·α₁                      = −(2ω+1)·γ₁
//! row i (int):  α_{i-1}/d_{i-1} + (2/d_{i-1}+2/d_i)·αᵢ
//!                              + α_{i+1}/d_i              = −2γᵢ/d_{i-1} − γ_{i+1}/d_i
//! row n      :  (2ω+1)·α_{n-1} + (2+ω)·αₙ                 = 0
//! ```
//!
//! with `dᵢ` the chord lengths and `ω` the endpoint *curl* (0 = relaxed,
//! the default). The system is **strictly diagonally dominant**, so the
//! Thomas (tridiagonal) solver is stable without pivoting. Control-point
//! handle lengths come from the Jackowski "velocity" function
//! `ρ(α,β) = 2 / (1 + ⅔·cos β + ⅓·cos α)`, capped at 4 (MetaPost's bound)
//! so a near-cusp knot cannot produce a runaway loop.
//!
//! ## Output
//!
//! One [`TangentsCubic`] per segment (`knots.len() − 1` entries). Segment
//! `i` connects `knots[i] → knots[i+1]`; its tangents are stored as
//! offset **vectors** matching the renderer's control-point convention
//! (`c₀ = start + out_at_start`, `c₁ = end + in_at_end`), identical to
//! [`crate::Segment`].
//!
//! ## Determinism (HR-5)
//!
//! Unlike [`crate::cubic_fit`] (which deliberately avoids transcendentals
//! for bit-identical procedural output), Hobby's fit calls `sin`/`cos`/
//! `atan2` on `f64`. That is acceptable here: the Pencil runs on the
//! **editor write path** over **non-reproducible freehand input**, and
//! the committed network is `deterministic == false`. If a caller ever
//! feeds a deterministic network it MUST route the resulting control
//! points through [`crate::deterministic::snap`] (same contract as
//! `cubic_fit`). The math never panics and never returns NaN for finite
//! input.
//!
//! ## Allocation (HR-3)
//!
//! Each call allocates a handful of `Vec<f64>` of length `n+1` for the
//! solver. This is an editor-write-path routine (stroke commit), **not**
//! per-frame-safe — do not call inside the render tick.

use glam::Vec2;

use crate::cubic::TangentsCubic;

/// Default endpoint *curl* (`ω`). `0.0` = the relaxed end condition that
/// lets the spline straighten naturally toward the endpoints — the most
/// neutral choice for a freehand Pencil stroke.
pub const DEFAULT_CURL: f32 = 0.0;

/// Upper bound on the Jackowski velocity `ρ`. MetaPost caps handle
/// "velocity" at 4 so a near-180° turn at a knot cannot blow the control
/// arm up to a self-intersecting loop. A value of 4 already corresponds
/// to a handle `4·d/3` long, well past anything a smooth stroke needs.
const MAX_VELOCITY: f64 = 4.0;

/// Below this chord length (world px) a knot pair is treated as
/// coincident: the segment degenerates to a zero-length straight cubic
/// rather than dividing by zero. Callers (the Pencil decimator) should
/// dedupe coincident samples upstream; this is the defensive floor.
const MIN_CHORD: f64 = 1e-6;

/// Fit a smooth **open** cubic Bézier spline through `knots` with the
/// default endpoint curl ([`DEFAULT_CURL`]).
///
/// Returns one [`TangentsCubic`] per segment (`knots.len() − 1` entries),
/// or an empty `Vec` for `< 2` knots. See the module docs for the method
/// and the output convention.
#[must_use]
pub fn fit_hobby_open(knots: &[Vec2]) -> Vec<TangentsCubic> {
    fit_hobby_open_with_curl(knots, DEFAULT_CURL)
}

/// Fit a smooth **open** cubic Bézier spline through `knots` with an
/// explicit endpoint `curl` (`ω`; `0.0` = relaxed). Larger values bend
/// the spline more sharply toward the first/last chord direction.
///
/// See [`fit_hobby_open`] for the common case.
#[must_use]
pub fn fit_hobby_open_with_curl(knots: &[Vec2], curl: f32) -> Vec<TangentsCubic> {
    let count = knots.len();
    if count < 2 {
        return Vec::new();
    }
    // n = number of segments; knot indices run 0..=n.
    let n = count - 1;
    let omega = f64::from(curl);

    // Non-finite input poisons every downstream `<`/atan2; bail to a
    // straight-thirds chain (finite, endpoints honored) — mirror of
    // `cubic_fit`'s zero-cubic fallback for non-finite endpoints.
    if knots.iter().any(|k| !k.is_finite()) {
        return straight_thirds(knots);
    }

    // Chords + lengths (one per segment). A coincident pair floors to
    // MIN_CHORD for the solver weights so we never divide by zero.
    let chords: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            (
                f64::from(knots[i + 1].x) - f64::from(knots[i].x),
                f64::from(knots[i + 1].y) - f64::from(knots[i].y),
            )
        })
        .collect();
    let d: Vec<f64> = chords.iter().map(|c| hypot(c.0, c.1).max(MIN_CHORD)).collect();

    // Turning angles γ at every knot. γ₀ and γₙ are boundary sentinels
    // (0); interior γᵢ is the signed angle from chord i−1 to chord i.
    let mut gamma = vec![0.0_f64; n + 1];
    for i in 1..n {
        gamma[i] = turning_angle(chords[i - 1], chords[i]);
    }

    // Tridiagonal system for α₀..αₙ (sub A, diag B, super C, rhs D).
    let mut sub = vec![0.0_f64; n + 1];
    let mut diag = vec![0.0_f64; n + 1];
    let mut sup = vec![0.0_f64; n + 1];
    let mut rhs = vec![0.0_f64; n + 1];

    // Row 0 (start curl boundary).
    diag[0] = 2.0 + omega;
    sup[0] = 2.0 * omega + 1.0;
    rhs[0] = -sup[0] * gamma[1];

    // Interior rows 1..=n-1.
    for i in 1..n {
        sub[i] = 1.0 / d[i - 1];
        diag[i] = 2.0 / d[i - 1] + 2.0 / d[i];
        sup[i] = 1.0 / d[i];
        rhs[i] = -2.0 * gamma[i] / d[i - 1] - gamma[i + 1] / d[i];
    }

    // Row n (end curl boundary).
    sub[n] = 2.0 * omega + 1.0;
    diag[n] = 2.0 + omega;
    rhs[n] = 0.0;

    let alpha = thomas(&sub, &diag, &sup, &rhs);

    // Arrival angles β (one per segment) from the departure angles.
    let mut beta = vec![0.0_f64; n];
    for i in 0..n - 1 {
        beta[i] = -gamma[i + 1] - alpha[i + 1];
    }
    beta[n - 1] = -alpha[n];

    // Per-segment control-handle vectors.
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let a = velocity(alpha[i], beta[i]) * d[i] / 3.0;
        let b = velocity(beta[i], alpha[i]) * d[i] / 3.0;
        let out_dir = normalize(rotate(chords[i], alpha[i]));
        let in_dir = normalize(rotate(chords[i], -beta[i]));
        out.push(TangentsCubic {
            out_at_start: Vec2::new((out_dir.0 * a) as f32, (out_dir.1 * a) as f32),
            in_at_end: Vec2::new((-in_dir.0 * b) as f32, (-in_dir.1 * b) as f32),
        });
    }
    out
}

/// Straight-line cubic for every segment: handles sit at the chord
/// thirds. Used as the finite fallback for non-finite input.
fn straight_thirds(knots: &[Vec2]) -> Vec<TangentsCubic> {
    let n = knots.len().saturating_sub(1);
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let chord = knots[i + 1] - knots[i];
        let third = if chord.is_finite() {
            chord / 3.0
        } else {
            Vec2::ZERO
        };
        out.push(TangentsCubic {
            out_at_start: third,
            in_at_end: -third,
        });
    }
    out
}

/// Jackowski velocity `ρ(α, β) = 2 / (1 + ⅔·cos β + ⅓·cos α)`, capped at
/// [`MAX_VELOCITY`] and floored at 0. The denominator is clamped away
/// from zero so a near-cusp (`α ≈ β ≈ π`) yields a bounded handle
/// instead of `∞`.
#[inline]
fn velocity(alpha: f64, beta: f64) -> f64 {
    const C: f64 = 2.0 / 3.0;
    let denom = 1.0 + C * beta.cos() + (1.0 - C) * alpha.cos();
    // denom ∈ [0, 2]; clamp the small end so ρ stays finite.
    let denom = denom.max(1e-6);
    (2.0 / denom).clamp(0.0, MAX_VELOCITY)
}

/// Signed turning angle from `prev` to `cur` (CCW positive), in radians.
/// `atan2(cross, dot)` is numerically robust for any pair of non-zero
/// vectors and returns 0 for parallel chords.
#[inline]
fn turning_angle(prev: (f64, f64), cur: (f64, f64)) -> f64 {
    let cross = prev.0 * cur.1 - prev.1 * cur.0;
    let dot = prev.0 * cur.0 + prev.1 * cur.1;
    cross.atan2(dot)
}

/// Rotate a 2-vector by `ang` radians (CCW positive).
#[inline]
fn rotate(v: (f64, f64), ang: f64) -> (f64, f64) {
    let (s, c) = ang.sin_cos();
    (v.0 * c - v.1 * s, v.0 * s + v.1 * c)
}

/// Normalize a 2-vector; returns `(0, 0)` for a (near-)zero vector.
#[inline]
fn normalize(v: (f64, f64)) -> (f64, f64) {
    let m = hypot(v.0, v.1);
    if m > 1e-12 { (v.0 / m, v.1 / m) } else { (0.0, 0.0) }
}

#[inline]
fn hypot(x: f64, y: f64) -> f64 {
    // Correctly-rounded sqrt (HR-5); avoid libm-backed `f64::hypot`.
    (x * x + y * y).sqrt()
}

/// Thomas algorithm for a tridiagonal system (`sub`, `diag`, `sup`, `rhs`
/// all length `m`). `sub[0]` and `sup[m-1]` are ignored. Returns the
/// solution vector. Stable without pivoting for the diagonally dominant
/// Hobby matrix; a defensive epsilon keeps a degenerate pivot finite.
fn thomas(sub: &[f64], diag: &[f64], sup: &[f64], rhs: &[f64]) -> Vec<f64> {
    let m = diag.len();
    let mut c_prime = vec![0.0_f64; m];
    let mut d_prime = vec![0.0_f64; m];

    let mut denom = diag[0];
    if denom.abs() < 1e-12 {
        denom = denom.signum().max(1.0) * 1e-12;
    }
    c_prime[0] = sup[0] / denom;
    d_prime[0] = rhs[0] / denom;

    for i in 1..m {
        let mut den = diag[i] - sub[i] * c_prime[i - 1];
        if den.abs() < 1e-12 {
            den = if den < 0.0 { -1e-12 } else { 1e-12 };
        }
        c_prime[i] = sup[i] / den;
        d_prime[i] = (rhs[i] - sub[i] * d_prime[i - 1]) / den;
    }

    let mut x = vec![0.0_f64; m];
    x[m - 1] = d_prime[m - 1];
    for i in (0..m - 1).rev() {
        x[i] = d_prime[i] - c_prime[i] * x[i + 1];
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Evaluate the cubic of segment `i` (knots[i] → knots[i+1]) at `t`,
    /// reconstructing control points from the tangent offset vectors.
    fn eval_segment(knots: &[Vec2], tan: &[TangentsCubic], i: usize, t: f32) -> Vec2 {
        let p0 = knots[i];
        let p3 = knots[i + 1];
        let c1 = p0 + tan[i].out_at_start;
        let c2 = p3 + tan[i].in_at_end;
        let mt = 1.0 - t;
        p0 * (mt * mt * mt)
            + c1 * (3.0 * mt * mt * t)
            + c2 * (3.0 * mt * t * t)
            + p3 * (t * t * t)
    }

    /// Tangent direction of segment `i` at its start (= `out_at_start`)
    /// and at its end (the curve tangent entering the end vertex, which
    /// is `−in_at_end` pointing forward along the curve).
    fn start_dir(tan: &TangentsCubic) -> Vec2 {
        tan.out_at_start
    }
    fn end_dir(tan: &TangentsCubic) -> Vec2 {
        -tan.in_at_end
    }

    fn cross(a: Vec2, b: Vec2) -> f32 {
        a.x * b.y - a.y * b.x
    }

    #[test]
    fn fewer_than_two_knots_is_empty() {
        assert!(fit_hobby_open(&[]).is_empty());
        assert!(fit_hobby_open(&[Vec2::new(1.0, 2.0)]).is_empty());
    }

    #[test]
    fn count_is_one_tangent_per_segment() {
        let knots: Vec<Vec2> = (0..7).map(|i| Vec2::new(i as f32, (i % 2) as f32)).collect();
        assert_eq!(fit_hobby_open(&knots).len(), knots.len() - 1);
    }

    #[test]
    fn two_knots_make_a_straight_cubic_at_thirds() {
        let knots = [Vec2::new(0.0, 0.0), Vec2::new(9.0, 0.0)];
        let tan = fit_hobby_open(&knots);
        assert_eq!(tan.len(), 1);
        // Handles along the chord at the thirds: out = +chord/3, in = −chord/3.
        assert!((tan[0].out_at_start.x - 3.0).abs() < 1e-3, "{:?}", tan[0]);
        assert!(tan[0].out_at_start.y.abs() < 1e-3);
        assert!((tan[0].in_at_end.x + 3.0).abs() < 1e-3, "{:?}", tan[0]);
        assert!(tan[0].in_at_end.y.abs() < 1e-3);
    }

    #[test]
    fn collinear_knots_stay_on_the_line() {
        let knots: Vec<Vec2> = (0..6).map(|i| Vec2::new(i as f32 * 2.0, 0.0)).collect();
        let tan = fit_hobby_open(&knots);
        // Every handle is horizontal (no vertical excursion off the line).
        for (i, t) in tan.iter().enumerate() {
            assert!(t.out_at_start.y.abs() < 1e-3, "seg {i} out {:?}", t);
            assert!(t.in_at_end.y.abs() < 1e-3, "seg {i} in {:?}", t);
        }
        // And midpoints of each cubic stay on y = 0.
        let knots_slice = &knots[..];
        for i in 0..tan.len() {
            let mid = eval_segment(knots_slice, &tan, i, 0.5);
            assert!(mid.y.abs() < 1e-3, "seg {i} mid {:?}", mid);
        }
    }

    #[test]
    fn interpolates_every_knot_exactly() {
        // The spline must pass through every knot: segment i at t=0 is
        // knots[i] and at t=1 is knots[i+1] (control-point convention).
        let knots = [
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 30.0),
            Vec2::new(40.0, 35.0),
            Vec2::new(70.0, 5.0),
            Vec2::new(90.0, 40.0),
        ];
        let tan = fit_hobby_open(&knots);
        for i in 0..tan.len() {
            let p_start = eval_segment(&knots, &tan, i, 0.0);
            let p_end = eval_segment(&knots, &tan, i, 1.0);
            assert!((p_start - knots[i]).length() < 1e-3, "seg {i} start");
            assert!((p_end - knots[i + 1]).length() < 1e-3, "seg {i} end");
        }
    }

    #[test]
    fn g1_continuity_at_interior_knots() {
        // THE correctness property: at each interior knot the incoming
        // tangent direction (segment i−1's end) is colinear with the
        // outgoing tangent direction (segment i's start). Hobby gives
        // exact G1 smoothness.
        let knots = [
            Vec2::new(0.0, 0.0),
            Vec2::new(20.0, 50.0),
            Vec2::new(60.0, 40.0),
            Vec2::new(80.0, -10.0),
            Vec2::new(120.0, 20.0),
            Vec2::new(150.0, 70.0),
        ];
        let tan = fit_hobby_open(&knots);
        for i in 1..tan.len() {
            let incoming = end_dir(&tan[i - 1]); // forward dir into knot i
            let outgoing = start_dir(&tan[i]); // forward dir out of knot i
            // Both should be non-degenerate and parallel (cross ≈ 0).
            assert!(incoming.length() > 1e-4 && outgoing.length() > 1e-4);
            let c = cross(incoming.normalize(), outgoing.normalize());
            assert!(c.abs() < 1e-3, "G1 broken at knot {i}: cross={c}");
        }
    }

    #[test]
    fn symmetric_input_yields_symmetric_output() {
        // A symmetric arch about x = 0 should produce a mirror-symmetric
        // spline: the first and last segments mirror each other.
        let knots = [
            Vec2::new(-30.0, 0.0),
            Vec2::new(-15.0, 20.0),
            Vec2::new(0.0, 26.0),
            Vec2::new(15.0, 20.0),
            Vec2::new(30.0, 0.0),
        ];
        let tan = fit_hobby_open(&knots);
        let first = &tan[0];
        let last = &tan[tan.len() - 1];
        // By mirror symmetry, last.in_at_end mirrors first.out_at_start:
        // x flips sign, y matches.
        assert!(
            (last.in_at_end.x + first.out_at_start.x).abs() < 1e-2,
            "first.out={:?} last.in={:?}",
            first.out_at_start,
            last.in_at_end
        );
        assert!(
            (last.in_at_end.y - first.out_at_start.y).abs() < 1e-2,
            "first.out={:?} last.in={:?}",
            first.out_at_start,
            last.in_at_end
        );
    }

    #[test]
    fn non_finite_knot_degrades_to_finite_straight_chain() {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let knots = [
                Vec2::new(0.0, 0.0),
                Vec2::new(bad, 10.0),
                Vec2::new(20.0, 0.0),
            ];
            let tan = fit_hobby_open(&knots);
            assert_eq!(tan.len(), 2);
            for (i, t) in tan.iter().enumerate() {
                assert!(
                    t.out_at_start.is_finite() && t.in_at_end.is_finite(),
                    "seg {i} leaked {bad}: {t:?}"
                );
            }
        }
    }

    #[test]
    fn coincident_knots_do_not_divide_by_zero() {
        // A decimator bug could leave a duplicate knot; the fit must stay
        // finite (segment degenerates rather than producing NaN/Inf).
        let knots = [
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(20.0, 0.0),
        ];
        let tan = fit_hobby_open(&knots);
        for (i, t) in tan.iter().enumerate() {
            assert!(
                t.out_at_start.is_finite() && t.in_at_end.is_finite(),
                "seg {i} non-finite {t:?}"
            );
        }
    }

    #[test]
    fn sharp_turn_handles_stay_bounded() {
        // A near-180° reversal is out of the smooth domain; the velocity
        // cap must keep handles bounded (no self-intersecting blowup).
        let knots = [
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(0.5, 0.2), // doubles back almost on top of knot 0..1
            Vec2::new(10.0, 5.0),
        ];
        let tan = fit_hobby_open(&knots);
        for (i, t) in tan.iter().enumerate() {
            // Velocity cap is 4 → handle ≤ 4·d/3; the longest chord here
            // is ~10 px, so no handle should exceed ~14 px.
            assert!(
                t.out_at_start.length() < 14.0 && t.in_at_end.length() < 14.0,
                "seg {i} runaway handle {t:?}"
            );
            assert!(t.out_at_start.is_finite() && t.in_at_end.is_finite());
        }
    }

    #[test]
    fn deterministic_across_calls() {
        let knots = [
            Vec2::new(0.0, 0.0),
            Vec2::new(12.0, 40.0),
            Vec2::new(55.0, 33.0),
            Vec2::new(90.0, 18.0),
        ];
        let a = fit_hobby_open(&knots);
        let b = fit_hobby_open(&knots);
        assert_eq!(a, b);
    }

    #[test]
    fn curl_parameter_changes_endpoint_tangents() {
        let knots = [
            Vec2::new(0.0, 0.0),
            Vec2::new(20.0, 30.0),
            Vec2::new(50.0, 10.0),
        ];
        let relaxed = fit_hobby_open_with_curl(&knots, 0.0);
        let curled = fit_hobby_open_with_curl(&knots, 1.0);
        // The first segment's start tangent should differ between curl
        // settings (endpoint boundary condition changed).
        assert!(
            (relaxed[0].out_at_start - curled[0].out_at_start).length() > 1e-3,
            "curl had no effect: {:?} vs {:?}",
            relaxed[0].out_at_start,
            curled[0].out_at_start
        );
    }
}
