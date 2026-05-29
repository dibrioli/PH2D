//! Levien single-cubic Bézier fitting — W1.T1.4 (real implementation).
//!
//! Per [ADR-0056 §2.4](../../../../docs/architecture/decisions/0056-vector-network-data-model.md)
//! (canonical Spiro/Hyperbézier → cubic Bézier conversion) + plan task
//! T1.4 (Levien algorithm, max error `< 0.5 px` on the canonical fixtures).
//!
//! ## Method (Raph Levien, 2021)
//!
//! <https://raphlinus.github.io/curves/2021/03/11/bezier-fitting.html>
//!
//! Instead of a least-squares minimization, the fit *matches integral
//! moments* of the source curve. Working in a normalized frame where the
//! chord runs from `(0, 0)` to `(1, 0)`, a cubic with endpoint tangent
//! angles `θ₀`, `θ₁` and control-arm lengths `δ₀`, `δ₁` has control points
//! `(0,0)`, `(δ₀cosθ₀, δ₀sinθ₀)`, `(1−δ₁cosθ₁, δ₁sinθ₁)`, `(1,0)`. Its
//! signed area and first x-moment (relative to the chord) are closed forms:
//!
//! ```text
//! area   = (3/20)·(2δ₀s₀ + 2δ₁s₁ − δ₀δ₁·sin(θ₀+θ₁))
//! moment = (1/280)·(34δ₀s₀ + 50δ₁s₁ + 15δ₀²s₀c₀ − 15δ₁²s₁c₁
//!                   − δ₀δ₁(33s₀c₁ + 9c₀s₁)
//!                   − 9δ₀²δ₁·sin(θ₀+θ₁)·c₀ + 9δ₀δ₁²·sin(θ₀+θ₁)·c₁)
//! ```
//!
//! (`sₖ = sinθₖ`, `cₖ = cosθₖ`). We measure the source curve's `area` and
//! `moment` from its samples, then solve the two equations for `(δ₀, δ₁)`:
//! the area equation is linear in `δ₁`, giving `δ₁(δ₀)`; substituting into
//! the moment equation leaves a 1-D root in `δ₀`, which we bracket-and-
//! bisect deterministically. When several roots are physically valid the
//! one with the smallest sampled deviation wins.
//!
//! ## Domain
//!
//! A *single* cubic matches a smooth arc with at most one inflection. A
//! 90° circular arc fits to `~0.135 px`; a half-circle (180°) cannot and
//! is out of domain — callers split such inputs into sub-segments
//! upstream (curve subdivision lands in W2). The fixtures in
//! `tests/cubic_fit_levien.rs` are chosen within this domain.
//!
//! ## Determinism (HR-5)
//!
//! The math uses only `+ − × ÷` and `f64::sqrt` — all IEEE-754
//! correctly-rounded and bit-identical across targets. Endpoint tangent
//! angles enter as `sinθ`/`cosθ` computed directly from unit-vector
//! components (dot / cross), so **no transcendental functions are called**
//! and no `libm` dependency is needed. The HR-5 guarantee covers the
//! *computation*; see the caller contract below for the *output boundary*.
//!
//! ## Caller contract
//!
//! - **Determinism at the boundary:** the returned control points are raw
//!   `f64 → f32` casts, **not** snapped to the Q16.16 grid. When the result
//!   is written into a [`crate::VectorNetwork`] with `deterministic == true`
//!   (whose stored coordinates must be cross-target bit-identical), the
//!   caller MUST route the four points through [`crate::deterministic::snap`]
//!   first — this primitive is network-agnostic and cannot see that flag.
//! - **Allocation (HR-3):** each call allocates two short-lived `Vec`s
//!   (`candidates`, and one of `samples.len()` points). That is fine on the
//!   editor write path / export / canonicalization — where this belongs —
//!   but it is **not** a per-frame-safe routine; do not call it inside the
//!   render tick without hoisting those buffers to caller-owned scratch.

use glam::Vec2;

/// Maximum control-arm length, in chord-length units. A best-fit cubic
/// whose handle exceeds 10× the chord is degenerate (a loop / overshoot);
/// such roots are rejected so a spurious bracket near a pole of `δ₁(δ₀)`
/// cannot win.
const MAX_ARM: f64 = 10.0;

/// Number of scan steps used to bracket sign changes of the moment
/// residual over `δ₀ ∈ (0, MAX_ARM]`. The residual is smooth with at most
/// a couple of roots in-domain; 512 steps bracket them with wide margin.
/// Fixed (not adaptive) so the result is identical on every target.
const SCAN_STEPS: usize = 512;

/// Bisection refinement iterations per bracket. ~50 halvings drive an
/// f64 interval to machine epsilon.
const BISECT_ITERS: usize = 50;

/// Samples used to score a candidate cubic against the source (max
/// nearest-point deviation). Cheap and deterministic.
const SCORE_SAMPLES: usize = 64;

/// Convert a sequence of points sampled along a curve into a single
/// best-fit cubic Bézier per Levien (2021).
///
/// Returns `(start, ctrl_a, ctrl_b, end)` — the four cubic control points.
/// `start`/`end` are always the first/last sample. Degenerate inputs
/// (`< 2` points, near-zero chord, collinear samples) fall back to a
/// straight cubic; the function never panics and never returns NaN for
/// finite input.
#[must_use]
pub fn fit_cubic_levien(samples: &[Vec2]) -> (Vec2, Vec2, Vec2, Vec2) {
    let n = samples.len();
    if n == 0 {
        return (Vec2::ZERO, Vec2::ZERO, Vec2::ZERO, Vec2::ZERO);
    }
    if n == 1 {
        let p = samples[0];
        return (p, p, p, p);
    }

    // HR-4 defense: a single non-finite endpoint would poison `chord` →
    // `d` → the whole frame into NaN, and NaN bypasses every `<`/`>` guard
    // below (all comparisons with NaN are false). The endpoints anchor the
    // result, so if either is non-finite there is nothing sane to fit —
    // emit the zero cubic. (A non-finite *interior* sample instead poisons
    // only `area`/`moment`, which the empty-root fallback below turns into
    // a finite straight cubic, so it needs no separate guard.)
    if !(samples[0].is_finite() && samples[n - 1].is_finite()) {
        return (Vec2::ZERO, Vec2::ZERO, Vec2::ZERO, Vec2::ZERO);
    }

    let p0 = dvec(samples[0]);
    let p3 = dvec(samples[n - 1]);
    let chord = (p3.0 - p0.0, p3.1 - p0.1);
    let d = hypot(chord.0, chord.1);

    // Endpoint tangent estimates: 2nd-order one-sided differences when we
    // have ≥ 3 points, else the chord direction.
    let (t0, t1) = endpoint_tangents(samples);

    // Near-coincident endpoints: no meaningful chord frame. The threshold
    // is scale-relative (to the largest sample magnitude) so a legitimate
    // sub-pixel curve isn't collapsed, nor a huge one mis-classified. Emit
    // a short straight cubic along the start tangent (bounded, never NaN).
    let scale = samples
        .iter()
        .map(|p| p.x.abs().max(p.y.abs()))
        .fold(0.0_f32, f32::max) as f64;
    let degen_eps = 1e-9 * scale.max(1.0);
    if d < degen_eps {
        let a = (p0.0 + t0.0 * d / 3.0, p0.1 + t0.1 * d / 3.0);
        let b = (p3.0 - t1.0 * d / 3.0, p3.1 - t1.1 * d / 3.0);
        return (fvec(p0), fvec(a), fvec(b), fvec(p3));
    }

    // Chord unit vector. `ch` doubles as cos/sin of the chord angle, so
    // the normalized-frame rotation needs no atan2.
    let ch = (chord.0 / d, chord.1 / d);

    // sin/cos of the endpoint tangent angles relative to the chord, taken
    // straight from unit-vector dot (cos) and cross (sin). `θ₁` is the
    // angle of the *incoming* end tangent measured from the chord, so its
    // sine is negated (the cubic's end handle points back toward the chord).
    let c0 = t0.0 * ch.0 + t0.1 * ch.1;
    let s0 = ch.0 * t0.1 - ch.1 * t0.0;
    let c1 = t1.0 * ch.0 + t1.1 * ch.1;
    let s1 = -(ch.0 * t1.1 - ch.1 * t1.0);
    let s01 = s0 * c1 + c0 * s1; // sin(θ₀ + θ₁)

    // Source area & first moment in the normalized frame (chord on +x).
    let (area, moment) = source_area_moment(samples, p0, ch, d);

    // δ₁ as a function of δ₀ from the (linear-in-δ₁) area equation.
    let d1_of = |d0: f64| -> Option<f64> {
        let den = 2.0 * s1 - d0 * s01;
        if den.abs() < 1e-9 {
            None
        } else {
            Some((20.0 * area / 3.0 - 2.0 * d0 * s0) / den)
        }
    };
    // Moment residual: 0 at a valid fit.
    let moment_of = |d0: f64, d1: f64| -> f64 {
        (1.0 / 280.0)
            * (34.0 * d0 * s0 + 50.0 * d1 * s1 + 15.0 * d0 * d0 * s0 * c0
                - 15.0 * d1 * d1 * s1 * c1
                - d0 * d1 * (33.0 * s0 * c1 + 9.0 * c0 * s1)
                - 9.0 * d0 * d0 * d1 * s01 * c0
                + 9.0 * d0 * d1 * d1 * s01 * c1)
    };
    let resid = |d0: f64| -> Option<f64> { d1_of(d0).map(|d1| moment_of(d0, d1) - moment) };

    // `d1_of` has a pole where its denominator `2·s1 − δ₀·sin(θ₀+θ₁)`
    // vanishes; `resid` flips sign across that asymptote without crossing
    // zero. Locate it (it is linear in δ₀, so at most one) and skip any
    // bracket straddling it, so the sign-change scan only ever brackets
    // genuine roots and `bisect`'s "sign at `a` is fixed" invariant holds.
    let pole = if s01.abs() > 1e-12 {
        Some(2.0 * s1 / s01)
    } else {
        None
    };
    let brackets_pole = |a: f64, b: f64| pole.is_some_and(|p| a < p && p <= b);

    // Bracket-and-bisect every sign change of the residual over the domain.
    let mut candidates: Vec<(f64, f64)> = Vec::new();
    let lo = 1e-4;
    let hi = MAX_ARM;
    let mut prev = resid(lo);
    let mut prev_x = lo;
    for k in 1..=SCAN_STEPS {
        let x = lo + (hi - lo) * (k as f64) / (SCAN_STEPS as f64);
        let r = resid(x);
        if let (Some(rp), Some(rc)) = (prev, r)
            && (rp <= 0.0) != (rc <= 0.0)
            && !brackets_pole(prev_x, x)
            && let Some(root) = bisect(&resid, prev_x, x)
            && let Some(d1) = d1_of(root)
            && root > 0.0
            && root <= MAX_ARM
            && d1 > 0.0
            && d1 <= MAX_ARM
        {
            candidates.push((root, d1));
        }
        // Always advance the reference point so a bracket never spans a gap
        // where `resid` was undefined (which would also straddle a pole).
        prev = r;
        prev_x = x;
    }

    // No physical root (straight line, or all roots out of bounds): the
    // chord-thirds cubic is the correct degenerate answer.
    if candidates.is_empty() {
        candidates.push((1.0 / 3.0, 1.0 / 3.0));
    }

    // Pick the candidate with the smallest sampled deviation from source.
    let npts: Vec<(f64, f64)> = samples
        .iter()
        .map(|&p| to_norm(dvec(p), p0, ch, d))
        .collect();
    let best = candidates
        .iter()
        .copied()
        .map(|(d0, d1)| (d0, d1, normalized_fit_error(d0, d1, c0, s0, c1, s1, &npts)))
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(d0, d1, _)| (d0, d1))
        .unwrap_or((1.0 / 3.0, 1.0 / 3.0));
    let (d0, d1) = best;

    // Control points back in world coordinates.
    let to_world = |nx: f64, ny: f64| -> (f64, f64) {
        let x = nx * d;
        let y = ny * d;
        (x * ch.0 - y * ch.1 + p0.0, x * ch.1 + y * ch.0 + p0.1)
    };
    let ctrl_a = to_world(d0 * c0, d0 * s0);
    let ctrl_b = to_world(1.0 - d1 * c1, d1 * s1);
    (fvec(p0), fvec(ctrl_a), fvec(ctrl_b), fvec(p3))
}

/// 2nd-order one-sided tangent estimates at the first/last sample, as unit
/// vectors. Falls back to the chord direction for `< 3` points.
fn endpoint_tangents(samples: &[Vec2]) -> ((f64, f64), (f64, f64)) {
    let n = samples.len();
    if n >= 3 {
        let p = |i: usize| dvec(samples[i]);
        let (a, b, c) = (p(0), p(1), p(2));
        let t0 = normalize((-3.0 * a.0 + 4.0 * b.0 - c.0, -3.0 * a.1 + 4.0 * b.1 - c.1));
        let (z, y, x) = (p(n - 1), p(n - 2), p(n - 3));
        let t1 = normalize((3.0 * z.0 - 4.0 * y.0 + x.0, 3.0 * z.1 - 4.0 * y.1 + x.1));
        (t0, t1)
    } else {
        let a = dvec(samples[0]);
        let b = dvec(samples[1]);
        let t = normalize((b.0 - a.0, b.1 - a.1));
        (t, t)
    }
}

/// Signed area and first x-moment of the sampled polyline relative to the
/// chord, in the normalized frame. Sign convention matches the cubic
/// closed forms in the module docs.
fn source_area_moment(samples: &[Vec2], p0: (f64, f64), ch: (f64, f64), d: f64) -> (f64, f64) {
    let mut area = 0.0;
    let mut moment = 0.0;
    let mut prev = to_norm(dvec(samples[0]), p0, ch, d);
    for &p in &samples[1..] {
        let cur = to_norm(dvec(p), p0, ch, d);
        let (x0, y0) = prev;
        let (x1, y1) = cur;
        area += 0.5 * (0.5 * (y0 + y1) * (x1 - x0) - 0.5 * (x0 + x1) * (y1 - y0));
        moment += -0.5 * ((x0 * x0 + x0 * x1 + x1 * x1) / 3.0) * (y1 - y0);
        prev = cur;
    }
    (area, moment)
}

/// Symmetric (two-sided Hausdorff) deviation between a normalized-frame
/// cubic and the source points, in chord-length units. Measuring only
/// source→curve would be blind to a candidate that loops or overshoots
/// *between* two adjacent samples (every sample stays near the curve while
/// the curve wanders far); the reverse curve→source term penalizes exactly
/// that, so the candidate scorer never prefers a wild fit.
fn normalized_fit_error(
    d0: f64,
    d1: f64,
    c0: f64,
    s0: f64,
    c1: f64,
    s1: f64,
    npts: &[(f64, f64)],
) -> f64 {
    let p1 = (d0 * c0, d0 * s0);
    let p2 = (1.0 - d1 * c1, d1 * s1);
    let p3 = (1.0, 0.0);

    // source → nearest point on the curve.
    let mut worst = 0.0_f64;
    for &q in npts {
        let mut nearest = f64::INFINITY;
        for j in 0..=SCORE_SAMPLES {
            let t = j as f64 / SCORE_SAMPLES as f64;
            let pt = eval_cubic((0.0, 0.0), p1, p2, p3, t);
            let dist = hypot(pt.0 - q.0, pt.1 - q.1);
            if dist < nearest {
                nearest = dist;
            }
        }
        worst = worst.max(nearest);
    }

    // curve → nearest point on the source *polyline* (segments, not just
    // vertices — measuring to vertices alone would inflate the distance by
    // ~half the sample spacing even for a perfect fit).
    for j in 0..=SCORE_SAMPLES {
        let t = j as f64 / SCORE_SAMPLES as f64;
        let pt = eval_cubic((0.0, 0.0), p1, p2, p3, t);
        let mut nearest = f64::INFINITY;
        for w in npts.windows(2) {
            let dist = point_to_segment(pt, w[0], w[1]);
            if dist < nearest {
                nearest = dist;
            }
        }
        worst = worst.max(nearest);
    }
    worst
}

/// Distance from point `p` to the line segment `a`–`b`.
fn point_to_segment(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (abx, aby) = (b.0 - a.0, b.1 - a.1);
    let len2 = abx * abx + aby * aby;
    let t = if len2 > 0.0 {
        (((p.0 - a.0) * abx + (p.1 - a.1) * aby) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    hypot(p.0 - (a.0 + t * abx), p.1 - (a.1 + t * aby))
}

/// Evaluate a cubic Bézier at `t`.
#[inline]
fn eval_cubic(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    t: f64,
) -> (f64, f64) {
    let mt = 1.0 - t;
    let a = mt * mt * mt;
    let b = 3.0 * mt * mt * t;
    let c = 3.0 * mt * t * t;
    let e = t * t * t;
    (
        a * p0.0 + b * p1.0 + c * p2.0 + e * p3.0,
        a * p0.1 + b * p1.1 + c * p2.1 + e * p3.1,
    )
}

/// Bisect a sign change of `f` bracketed by `[a, b]`. Returns the midpoint
/// of the final interval, or `None` if an endpoint becomes undefined.
fn bisect(f: &impl Fn(f64) -> Option<f64>, mut a: f64, mut b: f64) -> Option<f64> {
    // Sign at `a` is fixed by construction (the caller bracketed a change),
    // so we only need to compare each midpoint against it.
    let a_neg = f(a)? <= 0.0;
    for _ in 0..BISECT_ITERS {
        let m = 0.5 * (a + b);
        if (f(m)? <= 0.0) == a_neg {
            a = m;
        } else {
            b = m;
        }
    }
    Some(0.5 * (a + b))
}

/// Transform a world point into the normalized chord frame
/// (`p0 → (0,0)`, `p3 → (1,0)`).
#[inline]
fn to_norm(p: (f64, f64), p0: (f64, f64), ch: (f64, f64), d: f64) -> (f64, f64) {
    let dx = p.0 - p0.0;
    let dy = p.1 - p0.1;
    // Rotate by −chord_angle (cos = ch.0, sin = ch.1) then scale by 1/d.
    ((dx * ch.0 + dy * ch.1) / d, (-dx * ch.1 + dy * ch.0) / d)
}

#[inline]
fn normalize(v: (f64, f64)) -> (f64, f64) {
    let m = hypot(v.0, v.1);
    if m > 1e-12 {
        (v.0 / m, v.1 / m)
    } else {
        (0.0, 0.0)
    }
}

#[inline]
fn hypot(x: f64, y: f64) -> f64 {
    // Correctly-rounded sqrt is bit-identical across targets (HR-5); we
    // avoid `f64::hypot` (libm-backed, not guaranteed correctly-rounded)
    // on purpose.
    (x * x + y * y).sqrt()
}

#[inline]
fn dvec(v: Vec2) -> (f64, f64) {
    (f64::from(v.x), f64::from(v.y))
}

#[inline]
fn fvec(p: (f64, f64)) -> Vec2 {
    Vec2::new(p.0 as f32, p.1 as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample a cubic at `n` evenly-spaced parameter values.
    fn sample_cubic(p: [Vec2; 4], n: usize) -> Vec<Vec2> {
        (0..n)
            .map(|i| {
                let t = i as f64 / (n - 1) as f64;
                let v = eval_cubic(dvec(p[0]), dvec(p[1]), dvec(p[2]), dvec(p[3]), t);
                fvec(v)
            })
            .collect()
    }

    /// Max nearest-point deviation (world units) of the fitted cubic from
    /// the source samples — the DoD metric.
    fn max_error(samples: &[Vec2], fit: (Vec2, Vec2, Vec2, Vec2)) -> f64 {
        let (a, b, c, d) = fit;
        let (a, b, c, d) = (dvec(a), dvec(b), dvec(c), dvec(d));
        samples
            .iter()
            .map(|&q| {
                let q = dvec(q);
                (0..=600)
                    .map(|j| {
                        let t = j as f64 / 600.0;
                        let pt = eval_cubic(a, b, c, d, t);
                        hypot(pt.0 - q.0, pt.1 - q.1)
                    })
                    .fold(f64::INFINITY, f64::min)
            })
            .fold(0.0_f64, f64::max)
    }

    #[test]
    fn empty_and_single_are_degenerate() {
        assert_eq!(
            fit_cubic_levien(&[]),
            (Vec2::ZERO, Vec2::ZERO, Vec2::ZERO, Vec2::ZERO)
        );
        let p = Vec2::new(5.0, 7.0);
        assert_eq!(fit_cubic_levien(&[p]), (p, p, p, p));
    }

    #[test]
    fn two_points_make_a_straight_cubic() {
        let s = [Vec2::new(0.0, 0.0), Vec2::new(9.0, 0.0)];
        let (p0, a, b, p3) = fit_cubic_levien(&s);
        assert_eq!(p0, s[0]);
        assert_eq!(p3, s[1]);
        // Handles on the chord at the thirds.
        assert!((a.x - 3.0).abs() < 1e-3 && a.y.abs() < 1e-3, "ctrl_a={a:?}");
        assert!((b.x - 6.0).abs() < 1e-3 && b.y.abs() < 1e-3, "ctrl_b={b:?}");
    }

    #[test]
    fn collinear_samples_stay_on_the_line() {
        let s: Vec<Vec2> = (0..6).map(|i| Vec2::new(i as f32 * 2.0, 0.0)).collect();
        let fit = fit_cubic_levien(&s);
        assert!(max_error(&s, fit) < 1e-4);
        // No vertical excursion anywhere.
        for &c in &[fit.0, fit.1, fit.2, fit.3] {
            assert!(c.y.abs() < 1e-3, "off-line control point {c:?}");
        }
    }

    #[test]
    fn endpoints_are_always_preserved() {
        let s = [
            Vec2::new(2.0, 3.0),
            Vec2::new(4.0, 9.0),
            Vec2::new(8.0, 9.0),
            Vec2::new(11.0, 4.0),
        ];
        let (p0, _, _, p3) = fit_cubic_levien(&s);
        assert_eq!(p0, s[0]);
        assert_eq!(p3, s[3]);
    }

    #[test]
    fn refits_an_exact_cubic() {
        let ctrl = [
            Vec2::new(0.0, 0.0),
            Vec2::new(30.0, 80.0),
            Vec2::new(120.0, 90.0),
            Vec2::new(150.0, 10.0),
        ];
        let s = sample_cubic(ctrl, 20);
        let fit = fit_cubic_levien(&s);
        assert!(max_error(&s, fit) < 0.5, "err={}", max_error(&s, fit));
    }

    /// `(min, max)` corner of the axis-aligned bounding box of `pts`.
    fn bbox(pts: &[Vec2]) -> (Vec2, Vec2) {
        let mut lo = Vec2::splat(f32::INFINITY);
        let mut hi = Vec2::splat(f32::NEG_INFINITY);
        for &p in pts {
            lo = lo.min(p);
            hi = hi.max(p);
        }
        (lo, hi)
    }

    #[test]
    fn cusp_stays_pinned_and_bounded() {
        // Sharp reversal: the tangent estimate is ill-conditioned. We can't
        // demand a low error (a cusp is out of single-cubic domain), but the
        // fit must keep endpoints pinned and must NOT blow up to MAX_ARM·chord
        // — control points stay within the sample bbox grown by the chord.
        let s = [
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, -3.0),
            Vec2::new(4.0, -5.0),
            Vec2::new(5.0, -5.5),
            Vec2::new(4.0, -5.0),
            Vec2::new(2.0, -3.0),
            Vec2::new(8.0, 0.02),
        ];
        let (p0, a, b, p3) = fit_cubic_levien(&s);
        assert_eq!(p0, s[0]);
        assert_eq!(p3, s[s.len() - 1]);
        let (lo, hi) = bbox(&s);
        let chord = (s[s.len() - 1] - s[0]).length();
        let pad = Vec2::splat(chord);
        for c in [a, b] {
            assert!(c.x.is_finite() && c.y.is_finite(), "non-finite {c:?}");
            assert!(
                c.cmpge(lo - pad).all() && c.cmple(hi + pad).all(),
                "control {c:?} escaped bbox [{lo:?}..{hi:?}] + chord {chord}"
            );
        }
    }

    #[test]
    fn non_finite_endpoint_yields_finite_zero_cubic() {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            // Bad first endpoint.
            let s = [
                Vec2::new(bad, 0.0),
                Vec2::new(5.0, 0.0),
                Vec2::new(9.0, 1.0),
            ];
            let fit = fit_cubic_levien(&s);
            for c in [fit.0, fit.1, fit.2, fit.3] {
                assert!(c.x.is_finite() && c.y.is_finite(), "leaked {bad} via start");
            }
            // Bad last endpoint.
            let s = [
                Vec2::new(0.0, 0.0),
                Vec2::new(5.0, 0.0),
                Vec2::new(bad, 1.0),
            ];
            let fit = fit_cubic_levien(&s);
            for c in [fit.0, fit.1, fit.2, fit.3] {
                assert!(c.x.is_finite() && c.y.is_finite(), "leaked {bad} via end");
            }
        }
    }

    #[test]
    fn non_finite_interior_sample_degrades_to_finite_fit() {
        // Interior NaN poisons area/moment only; the empty-root fallback must
        // still produce a finite (straight) cubic with endpoints pinned.
        let s = [
            Vec2::new(0.0, 0.0),
            Vec2::new(3.0, f32::NAN),
            Vec2::new(9.0, 0.0),
        ];
        let (p0, a, b, p3) = fit_cubic_levien(&s);
        assert_eq!(p0, s[0]);
        assert_eq!(p3, s[2]);
        for c in [a, b] {
            assert!(
                c.x.is_finite() && c.y.is_finite(),
                "interior NaN leaked: {c:?}"
            );
        }
    }

    #[test]
    fn duplicate_leading_samples_do_not_break_the_fit() {
        // A Pen-tool double-tap repeats the first point. n ≥ 3 keeps the
        // tangent well-defined (uses sample[2]); result must stay finite and
        // pinned, near a straight line.
        let s = [
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(10.0, 0.0),
        ];
        let fit = fit_cubic_levien(&s);
        assert_eq!(fit.0, s[0]);
        assert_eq!(fit.3, s[3]);
        for c in [fit.0, fit.1, fit.2, fit.3] {
            assert!(
                c.x.is_finite() && c.y.is_finite() && c.y.abs() < 1e-2,
                "{c:?}"
            );
        }
    }

    #[test]
    fn deterministic_across_calls() {
        let s = [
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 40.0),
            Vec2::new(60.0, 55.0),
            Vec2::new(100.0, 20.0),
        ];
        let a = fit_cubic_levien(&s);
        let b = fit_cubic_levien(&s);
        assert_eq!(a, b);
    }
}
