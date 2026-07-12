//! **F-curve simplification** — fit dense recorded `(time, value)` samples (one
//! key per frame, from record-during-play) to a MINIMAL chain of cubic-Bézier
//! keyframes within a value tolerance. The industry-standard pipeline every
//! serious editor uses for this (Inkscape simplify, paper.js `path.simplify()`,
//! the brush tool's Free-Hand fit) is **Schneider**, "An Algorithm for
//! Automatically Fitting Digitized Curves", Graphics Gems I (1990): least-squares
//! cubic fit + Newton reparameterisation + adaptive splitting at the worst point.
//!
//! Two adaptations make it an *F-curve* fit rather than a generic 2D curve fit:
//!
//! 1. **Axis normalisation.** Time (seconds) and value (metres, radians, a 0..1
//!    opacity…) live on wildly different scales; a raw 2D fit would be dominated
//!    by whichever axis is larger. We fit in a space where both axes are scaled
//!    to `[0, 1]` by the session's extent, so the tolerance is a clean *fraction
//!    of the value range* and the least-squares stays balanced.
//! 2. **Error measured in VALUE at the correct time.** The classic Schneider
//!    error is the 2D distance from the sample to the nearest curve point — which
//!    charges for being early/late in *time*, meaningless for an F-curve (the
//!    sample IS at its time). We instead solve the curve's time axis for `s` such
//!    that `T(s) = tᵢ` and compare `V(s)` to `vᵢ`: the true vertical error. This
//!    is what makes the result "very precise in value with few keys".
//!
//! The output is [`Interp::BezierW`] keys — a weighted tangent pair IS exactly a
//! cubic Bézier in the `(u, value)` plane, so the conversion is exact; the handle
//! x-coordinates are clamped to `[0, 1]` so the result stays a valid function of
//! time (a handle may not run backward in time — Blender's `correct_bezpart`,
//! expressed here in normalised segment coordinates). Transcendental-free (HR-5).
//!
//! References cross-checked against the canonical `FitCurves.c` (Graphics Gems),
//! Inkscape/2geom `bezier-utils.cpp` (the hardened Newton guards + the Wu–Barsky
//! `α < ε·chord` fallback used below, not the book's `α < 0`), and Blender's
//! F-curve tools. **Deliberately deferred** (v1 fits smooth gizmo mocap well; add
//! when the input demands it): a corner pre-pass that keeps sharp cusps as BROKEN
//! (non-aligned) tangents rather than smoothing them under one shared centre
//! tangent; a value-overshoot clamp for bounded channels (opacity's runtime
//! `clamp(0,1)` already catches the display side); a rotation unwrap before
//! fitting multi-turn Euler spins; and a low-pass pre-filter for noisy input.

use crate::curve::Interp;

/// A point during the fit: `[time, value]`, in the NORMALISED space (both axes
/// scaled to `[0, 1]` by the session extent).
type P = [f64; 2];

/// One fitted keyframe: an absolute `(time, value)` plus the interpolation
/// leaving it toward the next. The caller turns these into its own key type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FitKey {
    /// Absolute time (seconds).
    pub t: f64,
    /// Value at the key.
    pub v: f64,
    /// Interpolation from this key toward the next (`Linear` on the last key,
    /// which has no following segment).
    pub interp: Interp,
}

/// Low-pass the VALUE axis of `samples` in place (times untouched) — a binomial
/// `[1, 2, 1] / 4` kernel applied `passes` times, endpoints pinned. This is the
/// mocap-standard pre-filter before a fit: hand/mouse input carries
/// high-frequency tremor, and without smoothing the recursive fitter
/// over-subdivides (every noise bump above tolerance spawns a keyframe — the
/// "reduziu um pouco" symptom). A few passes ≈ a 3–5 sample window: enough to
/// strip jitter, conservative enough to keep the gesture's real shape (the
/// literature's "filter conservatively — over-filtering makes motion floaty").
/// `passes == 0` or fewer than 3 samples is a no-op. Transcendental-free (HR-5).
pub fn smooth_values(samples: &mut [(f64, f64)], passes: usize) {
    let n = samples.len();
    if n < 3 || passes == 0 {
        return;
    }
    let mut src = vec![0.0f64; n];
    for _ in 0..passes {
        for (s, &(_, v)) in src.iter_mut().zip(samples.iter()) {
            *s = v;
        }
        for i in 1..n - 1 {
            samples[i].1 = 0.25 * src[i - 1] + 0.5 * src[i] + 0.25 * src[i + 1];
        }
    }
}

/// Fit dense `samples` (`(time, value)`, any order) to a minimal chain of
/// cubic-Bézier [`FitKey`]s whose reconstructed curve stays within `tol` — an
/// ABSOLUTE value tolerance in the samples' own units — of every sample.
///
/// Returns the samples' endpoints unchanged (a fit always pins the ends) and as
/// few interior keys as the tolerance allows. Fewer than 3 distinct-time samples
/// pass through as-is (nothing to simplify). A dead-flat run collapses to its two
/// endpoints. Deterministic; runs once, off the hot path (end of a record drag).
#[must_use]
pub fn fit_fcurve(samples: &[(f64, f64)], tol: f64) -> Vec<FitKey> {
    // Sort by time and drop duplicate-time samples (a parked cursor records the
    // same frame twice); a fit needs strictly increasing time.
    let mut pts: Vec<(f64, f64)> = samples.to_vec();
    pts.sort_by(|a, b| a.0.total_cmp(&b.0));
    pts.dedup_by(|a, b| a.0 == b.0);
    let n = pts.len();
    if n < 3 {
        return pts
            .iter()
            .map(|&(t, v)| FitKey {
                t,
                v,
                interp: Interp::Linear,
            })
            .collect();
    }

    let t0 = pts[0].0;
    let t_span = pts[n - 1].0 - t0;
    let (mut v_min, mut v_max) = (pts[0].1, pts[0].1);
    for &(_, v) in &pts {
        v_min = v_min.min(v);
        v_max = v_max.max(v);
    }
    let v_span = v_max - v_min;
    if t_span <= 0.0 || v_span <= 0.0 {
        // No time extent (impossible after dedup) or a dead-flat value run: two
        // endpoints, no curve to fit.
        return vec![
            FitKey {
                t: pts[0].0,
                v: pts[0].1,
                interp: Interp::Linear,
            },
            FitKey {
                t: pts[n - 1].0,
                v: pts[n - 1].1,
                interp: Interp::Linear,
            },
        ];
    }

    // Normalise both axes to [0, 1]. The x-ratio of a handle is scale-invariant,
    // so `x1/x2` come straight from normalised coordinates; `dy` denormalises by
    // `v_span`. The value tolerance becomes a fraction of the value range.
    let norm: Vec<P> = pts
        .iter()
        .map(|&(t, v)| [(t - t0) / t_span, (v - v_min) / v_span])
        .collect();
    let tol_n = (tol / v_span).max(1e-9);

    let left_t = unit(sub(norm[1], norm[0]));
    let right_t = unit(sub(norm[n - 2], norm[n - 1]));
    let mut segs: Vec<[P; 4]> = Vec::new();
    fit_cubic(&norm, left_t, right_t, tol_n, &mut segs);

    // Each fitted cubic → a BezierW key. `denorm` maps a normalised coordinate
    // back to (seconds, value); the handle x is the time ratio within the
    // segment (clamped so the curve stays a function of time).
    let denorm_t = |tn: f64| t0 + tn * t_span;
    let denorm_v = |vn: f64| v_min + vn * v_span;
    let mut out = Vec::with_capacity(segs.len() + 1);
    for seg in &segs {
        let [p0, c0, c1, p1] = *seg;
        let dt = p1[0] - p0[0];
        let (x1, x2) = if dt.abs() > 1e-12 {
            (((c0[0] - p0[0]) / dt), ((c1[0] - p0[0]) / dt))
        } else {
            (1.0 / 3.0, 2.0 / 3.0)
        };
        let dy1 = (c0[1] - p0[1]) * v_span;
        let dy2 = (c1[1] - p1[1]) * v_span;
        out.push(FitKey {
            t: denorm_t(p0[0]),
            v: denorm_v(p0[1]),
            interp: Interp::bezier_w(x1, dy1, x2, dy2),
        });
    }
    // The trailing anchor (last segment's P1): pins the end, no outgoing segment.
    if let Some(last) = segs.last() {
        out.push(FitKey {
            t: denorm_t(last[3][0]),
            v: denorm_v(last[3][1]),
            interp: Interp::Linear,
        });
    }
    out
}

/// Fit one run (`pts`, with end tangents) to cubics within value error `tol`,
/// appending each `[P0, C0, C1, P1]` to `out`. Recurses, splitting at the
/// worst-fit sample (Schneider). Error is the VALUE gap at the sample's time.
fn fit_cubic(pts: &[P], left_t: P, right_t: P, tol: f64, out: &mut Vec<[P; 4]>) {
    let n = pts.len();
    if n == 2 {
        // A straight cubic: handles at the 1/3 chord (so it flattens to the line).
        let third = scale(sub(pts[1], pts[0]), 1.0 / 3.0);
        out.push([pts[0], add(pts[0], third), sub(pts[1], third), pts[1]]);
        return;
    }
    let mut u = chord_length_param(pts);
    let mut bez = generate_bezier(pts, &u, left_t, right_t);
    let (mut max_e, mut split) = max_value_error(pts, &bez);
    if max_e < tol {
        out.push(bez);
        return;
    }
    // Close enough to try Newton reparameterisation a few times before splitting
    // (Schneider's `maxError < error·4` gate — here on the value error; the
    // paper's "four or five" iteration cap, `FitCurves.c` uses 4).
    if max_e < 4.0 * tol {
        for _ in 0..4 {
            u = reparameterize(pts, &u, &bez);
            bez = generate_bezier(pts, &u, left_t, right_t);
            let (e, s) = max_value_error(pts, &bez);
            max_e = e;
            split = s;
            if max_e < tol {
                out.push(bez);
                return;
            }
        }
    }
    let split = split.clamp(1, n - 2);
    let center_t = unit(sub(pts[split - 1], pts[split + 1]));
    fit_cubic(&pts[..=split], left_t, center_t, tol, out);
    fit_cubic(&pts[split..], scale(center_t, -1.0), right_t, tol, out);
}

/// Least-squares fit of the two interior control points, given the endpoints
/// (`pts` first/last) and the end tangents — the Bézier through `pts` at
/// parameters `u`. Falls back to the Wu–Barsky 1/3-chord heuristic when the
/// normal equations are degenerate or yield a non-positive tangent length.
fn generate_bezier(pts: &[P], u: &[f64], left_t: P, right_t: P) -> [P; 4] {
    let n = pts.len();
    let (first, last) = (pts[0], pts[n - 1]);
    let (mut c00, mut c01, mut c11) = (0.0f64, 0.0, 0.0);
    let (mut x0, mut x1) = (0.0f64, 0.0);
    for i in 0..n {
        let w = bernstein(u[i]);
        let a0 = scale(left_t, w[1]);
        let a1 = scale(right_t, w[2]);
        c00 += dot(a0, a0);
        c01 += dot(a0, a1);
        c11 += dot(a1, a1);
        let base = add(scale(first, w[0] + w[1]), scale(last, w[2] + w[3]));
        let tmp = sub(pts[i], base);
        x0 += dot(a0, tmp);
        x1 += dot(a1, tmp);
    }
    let det = c00 * c11 - c01 * c01;
    let chord = dist(first, last);
    let (mut al, mut ar) = if det.abs() > 1e-15 {
        ((x0 * c11 - c01 * x1) / det, (c00 * x1 - c01 * x0) / det)
    } else {
        (0.0, 0.0)
    };
    let eps = 1e-6 * chord;
    if al < eps || ar < eps {
        let third = chord / 3.0;
        al = third;
        ar = third;
    }
    [
        first,
        add(first, scale(left_t, al)),
        add(last, scale(right_t, ar)),
        last,
    ]
}

/// Worst VALUE error of `pts` from `bez` — for each sample, solve the curve's
/// time axis for the parameter at that sample's time, then take `|V − v|`.
/// Returns the error and the split index (where it occurs). Endpoints are exact
/// by construction, so the interior is searched.
fn max_value_error(pts: &[P], bez: &[P; 4]) -> (f64, usize) {
    let n = pts.len();
    let (mut max_e, mut split) = (0.0f64, n / 2);
    // Interior only — the endpoints are exact by construction.
    for (i, &p) in pts.iter().enumerate().take(n - 1).skip(1) {
        let s = solve_time(bez, p[0]);
        let v = bezier_at(bez, s)[1];
        let e = (v - p[1]).abs();
        if e >= max_e {
            max_e = e;
            split = i;
        }
    }
    (max_e, split)
}

/// The curve parameter `s ∈ [0, 1]` whose time-axis value is `t_target` — the
/// time axis is monotone for a well-formed F-curve segment, so a few Newton
/// steps (guarded by bisection bounds) converge. Used to read the value error at
/// the sample's exact time.
fn solve_time(bez: &[P; 4], t_target: f64) -> f64 {
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    let mut s = t_target.clamp(0.0, 1.0); // linear-time initial guess
    for _ in 0..24 {
        let t = bezier_at(bez, s)[0];
        let e = t - t_target;
        if e.abs() < 1e-10 {
            break;
        }
        if e > 0.0 {
            hi = s;
        } else {
            lo = s;
        }
        let dt = bezier_d1(bez, s)[0];
        let step = if dt.abs() > 1e-12 {
            s - e / dt
        } else {
            f64::NAN
        };
        s = if step.is_finite() && step > lo && step < hi {
            step
        } else {
            0.5 * (lo + hi)
        };
    }
    s
}

/// One Newton–Raphson step per sample toward its nearest point on `bez` (the 2D
/// projection — improves the chord-length guess so the next least-squares fit is
/// tighter; Schneider's reparameterisation).
fn reparameterize(pts: &[P], u: &[f64], bez: &[P; 4]) -> Vec<f64> {
    pts.iter()
        .zip(u)
        .map(|(&p, &ui)| {
            let q = bezier_at(bez, ui);
            let q1 = bezier_d1(bez, ui);
            let q2 = bezier_d2(bez, ui);
            let diff = sub(q, p);
            let den = dot(q1, q1) + dot(diff, q2);
            // `den <= 0` (not just ~0): a non-positive denominator would send
            // raw Newton toward a MAXIMUM, not the foot of perpendicular — hold
            // the parameter instead (Inkscape hardening of the Graphics Gems
            // routine, which had no such guard). Result clamped to `[0, 1]`.
            if den <= 1e-15 {
                ui
            } else {
                (ui - dot(diff, q1) / den).clamp(0.0, 1.0)
            }
        })
        .collect()
}

/// Cumulative chord-length parameterisation of `pts`, normalised to `[0, 1]`.
fn chord_length_param(pts: &[P]) -> Vec<f64> {
    let mut u = vec![0.0f64; pts.len()];
    for i in 1..pts.len() {
        u[i] = u[i - 1] + dist(pts[i], pts[i - 1]);
    }
    let total = u[pts.len() - 1];
    if total > 0.0 {
        for x in &mut u {
            *x /= total;
        }
    }
    u
}

// ── 2D vector + cubic-Bézier helpers (f64) ──────────────────────────────────

fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1]]
}
fn add(a: P, b: P) -> P {
    [a[0] + b[0], a[1] + b[1]]
}
fn scale(a: P, s: f64) -> P {
    [a[0] * s, a[1] * s]
}
fn dot(a: P, b: P) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn dist(a: P, b: P) -> f64 {
    let d = sub(a, b);
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}
fn unit(a: P) -> P {
    let m = (a[0] * a[0] + a[1] * a[1]).sqrt();
    if m > 1e-12 {
        [a[0] / m, a[1] / m]
    } else {
        [0.0, 0.0]
    }
}
fn bernstein(t: f64) -> [f64; 4] {
    let mt = 1.0 - t;
    [mt * mt * mt, 3.0 * mt * mt * t, 3.0 * mt * t * t, t * t * t]
}
fn bezier_at(b: &[P; 4], t: f64) -> P {
    let w = bernstein(t);
    add(
        add(scale(b[0], w[0]), scale(b[1], w[1])),
        add(scale(b[2], w[2]), scale(b[3], w[3])),
    )
}
fn bezier_d1(b: &[P; 4], t: f64) -> P {
    let mt = 1.0 - t;
    let a = scale(sub(b[1], b[0]), 3.0 * mt * mt);
    let c = scale(sub(b[2], b[1]), 6.0 * mt * t);
    let d = scale(sub(b[3], b[2]), 3.0 * t * t);
    add(add(a, c), d)
}
fn bezier_d2(b: &[P; 4], t: f64) -> P {
    let mt = 1.0 - t;
    let a = scale(add(sub(b[2], scale(b[1], 2.0)), b[0]), 6.0 * mt);
    let c = scale(add(sub(b[3], scale(b[2], 2.0)), b[1]), 6.0 * t);
    add(a, c)
}

#[cfg(test)]
#[path = "curve_fit_tests.rs"]
mod tests;
