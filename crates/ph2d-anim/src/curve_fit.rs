//! **F-curve simplification** — fit dense recorded `(time, value)` samples (one
//! key per frame, from record-during-play) to a MINIMAL chain of cubic-Bézier
//! keyframes: a key at each PEAK and VALLEY and (almost) nothing between, the way
//! an animator draws by hand.
//!
//! The building block is a **Schneider** least-squares cubic fit ("An Algorithm
//! for Automatically Fitting Digitized Curves", Graphics Gems I 1990), but the
//! classic recursive split-on-error is NOT the top-level strategy: on a
//! multi-turn wave it over-subdivides badly, scattering keys along the slopes (a
//! 6-extremum sine → ~40 keys). Instead:
//!
//! 1. **Anchor at the extrema.** Detect the prominent peaks/valleys of the value
//!    axis ([`anchor_indices`], a prominence / swinging-door detector), then fit
//!    ONE least-squares cubic per monotone run between consecutive extrema
//!    ([`fit_run`] → [`one_cubic`]). A run's ends are turns, where the gesture's
//!    tangent is ~flat, so a single cubic tracks a wave's half-cycle closely.
//!    Result: a key per turn (a 6-extremum sine → ~9 keys). A genuinely complex
//!    run splits, but only to a bounded depth ([`RUN_SPLIT_DEPTH`]), never the
//!    Schneider blow-up. Fidelity is therefore APPROXIMATE (a few percent of the
//!    range) — the deliberate trade the user asked for: few clean keys over
//!    sub-percent precision.
//! 2. **Axis normalisation.** Time (seconds) and value (metres, radians, a 0..1
//!    opacity…) live on wildly different scales; a raw 2D fit would be dominated
//!    by whichever axis is larger. We fit in a space where both axes are scaled
//!    to `[0, 1]` by the session's extent, so the tolerance is a clean *fraction
//!    of the value range* and the least-squares stays balanced.
//! 3. **Error measured in VALUE at the correct time.** The classic Schneider
//!    error is the 2D distance from the sample to the nearest curve point — which
//!    charges for being early/late in *time*, meaningless for an F-curve (the
//!    sample IS at its time). We instead solve the curve's time axis for `s` such
//!    that `T(s) = tᵢ` and compare `V(s)` to `vᵢ`: the true vertical error.
//!
//! The output is [`Interp::BezierW`] keys — a weighted tangent pair IS exactly a
//! cubic Bézier in the `(u, value)` plane, so the conversion is exact; the handle
//! x-coordinates are clamped to `[0, 1]` so the result stays a valid function of
//! time (a handle may not run backward in time — Blender's `correct_bezpart`,
//! expressed here in normalised segment coordinates). Transcendental-free (HR-5).
//!
//! What a channel means beyond its numbers — an angle that wraps, a hard bound the
//! curve may not leave — is prepared by the sibling [`crate::curve_prep`]. The
//! bound reaches the fit here and clamps the fitted control points; by the
//! convex-hull property that holds the whole curve inside it.
//!
//! References cross-checked against the canonical `FitCurves.c` (Graphics Gems),
//! Inkscape/2geom `bezier-utils.cpp` (the hardened Newton guards + the Wu–Barsky
//! `α < ε·chord` fallback used below, not the book's `α < 0`), and Blender's
//! F-curve tools.

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
/// `bounds` are the channel's hard limits, which the fitted curve may not leave
/// (`None` = unbounded) — see [`crate::curve_prep::FitChannel`].
///
/// Returns the samples' endpoints unchanged (a fit always pins the ends) and as
/// few interior keys as the tolerance allows. Fewer than 3 distinct-time samples
/// pass through as-is (nothing to simplify). A dead-flat run collapses to its two
/// endpoints. Deterministic; runs once, off the hot path (end of a record drag).
#[must_use]
pub fn fit_fcurve(samples: &[(f64, f64)], tol: f64, bounds: Option<(f64, f64)>) -> Vec<FitKey> {
    let Some(prep) = Prep::new(samples) else {
        return trivial_keys(samples);
    };
    let tol_n = (tol / prep.v_span).max(1e-9);
    // Key at every prominent peak/valley (+ the two ends) and NOTHING between —
    // see the module docs. `min_prom` is the least value swing (normalised) for a
    // turn to count as a real extremum.
    let min_prom = (4.0 * tol_n).max(0.02);
    let anchors = anchor_indices(&prep.norm, min_prom);
    prep.build_keys(&anchors, tol_n, RUN_SPLIT_DEPTH, bounds)
}

/// Fit `samples` with the key times FORCED to `times` — the **aligned-columns**
/// path. Every track of one recording session is re-fitted at the SAME set of
/// times, so the dope sheet reads as clean columns (an animator can then grab a
/// whole column and re-time every channel together, which is why hand-keyed
/// animation is column-aligned in the first place).
///
/// Each run between consecutive forced times gets ONE least-squares cubic and is
/// never split — a split would insert a key OFF the shared column grid and break
/// the alignment. Times outside the samples' range are ignored; the two
/// endpoints are always pinned. `times` need not be sorted or unique.
///
/// `bounds` are as in [`fit_fcurve`].
#[must_use]
pub fn fit_fcurve_at(
    samples: &[(f64, f64)],
    times: &[f64],
    bounds: Option<(f64, f64)>,
) -> Vec<FitKey> {
    let Some(prep) = Prep::new(samples) else {
        return trivial_keys(samples);
    };
    let anchors = prep.indices_for_times(times);
    // No split (`depth = 0`): the keys land exactly on the shared columns.
    prep.build_keys(&anchors, f64::INFINITY, 0, bounds)
}

/// Fewer than 3 distinct-time samples (or a degenerate span): pass them through.
fn trivial_keys(samples: &[(f64, f64)]) -> Vec<FitKey> {
    let mut pts: Vec<(f64, f64)> = samples.to_vec();
    pts.sort_by(|a, b| a.0.total_cmp(&b.0));
    pts.dedup_by(|a, b| a.0 == b.0);
    if pts.len() > 2 {
        // A dead-flat / zero-span run: just its two ends.
        pts = vec![pts[0], pts[pts.len() - 1]];
    }
    pts.iter()
        .map(|&(t, v)| FitKey {
            t,
            v,
            interp: Interp::Linear,
        })
        .collect()
}

/// The sorted, de-duplicated samples plus the normalisation both fit paths share.
struct Prep {
    pts: Vec<(f64, f64)>,
    norm: Vec<P>,
    t0: f64,
    t_span: f64,
    v_min: f64,
    v_span: f64,
}

impl Prep {
    /// `None` when there is nothing to fit (fewer than 3 distinct times, no time
    /// extent, or a dead-flat value run) — the caller emits [`trivial_keys`].
    fn new(samples: &[(f64, f64)]) -> Option<Self> {
        // Sort by time and drop duplicate-time samples (a parked cursor records
        // the same frame twice); a fit needs strictly increasing time.
        let mut pts: Vec<(f64, f64)> = samples.to_vec();
        pts.sort_by(|a, b| a.0.total_cmp(&b.0));
        pts.dedup_by(|a, b| a.0 == b.0);
        let n = pts.len();
        if n < 3 {
            return None;
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
            return None;
        }
        // Normalise both axes to [0, 1]. A handle's x-ratio is scale-invariant,
        // so `x1/x2` come straight from normalised coordinates; `dy` denormalises
        // by `v_span`. The tolerance becomes a fraction of the value range.
        let norm: Vec<P> = pts
            .iter()
            .map(|&(t, v)| [(t - t0) / t_span, (v - v_min) / v_span])
            .collect();
        Some(Self {
            pts,
            norm,
            t0,
            t_span,
            v_min,
            v_span,
        })
    }

    /// Sample indices closest to each of `times` (plus both endpoints), sorted and
    /// de-duplicated — the forced anchors of [`fit_fcurve_at`].
    fn indices_for_times(&self, times: &[f64]) -> Vec<usize> {
        let n = self.pts.len();
        let mut idx: Vec<usize> = vec![0, n - 1];
        for &t in times {
            // Nearest sample by time (the samples are dense — one per frame — so
            // a shared column lands within half a frame of a real sample).
            let i = self
                .pts
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| (a.0 - t).abs().total_cmp(&(b.0 - t).abs()))
                .map(|(i, _)| i)
                .unwrap_or(0);
            idx.push(i);
        }
        idx.sort_unstable();
        idx.dedup();
        idx
    }

    /// Fit each run between consecutive `anchors` and convert to [`FitKey`]s.
    /// `bounds` (channel units) clamp the fitted curve's value range.
    fn build_keys(
        &self,
        anchors: &[usize],
        tol_n: f64,
        depth: u32,
        bounds: Option<(f64, f64)>,
    ) -> Vec<FitKey> {
        let mut segs: Vec<[P; 4]> = Vec::new();
        for w in anchors.windows(2) {
            fit_run(&self.norm[w[0]..=w[1]], tol_n, depth, &mut segs);
        }
        // Each fitted cubic → a BezierW key. `denorm` maps a normalised coordinate
        // back to (seconds, value); the handle x is the time ratio within the
        // segment (clamped so the curve stays a function of time).
        let denorm_t = |tn: f64| self.t0 + tn * self.t_span;
        let denorm_v = |vn: f64| self.v_min + vn * self.v_span;
        // Bounds in the normalised value axis. A cubic Bézier lies inside the convex
        // hull of its control points, so clamping ALL FOUR of a segment's value
        // coordinates to the bound keeps the whole curve inside it — exactly, not
        // approximately. Least-squares alone happily overshoots a channel that
        // settles ON its bound (an opacity fading to 1.0 reconstructs to 1.003),
        // which the graph editor draws even where the runtime clamps the display.
        //
        // The KEYS are clamped too, not just the handles: a recording carries
        // tremor, so a fade that rests at 1.0 has samples at 1.004 — the endpoints
        // are NOT inside the bound by construction, and a key at 1.004 opacity is
        // not a thing that exists.
        let bounds_n = bounds.map(|(lo, hi)| {
            (
                (lo - self.v_min) / self.v_span,
                (hi - self.v_min) / self.v_span,
            )
        });
        let clamp_v = |vn: f64| match bounds_n {
            Some((lo, hi)) => vn.clamp(lo, hi),
            None => vn,
        };
        let mut out = Vec::with_capacity(segs.len() + 1);
        for seg in &segs {
            let [p0, c0, c1, p1] = *seg;
            let dt = p1[0] - p0[0];
            let (x1, x2) = if dt.abs() > 1e-12 {
                (((c0[0] - p0[0]) / dt), ((c1[0] - p0[0]) / dt))
            } else {
                (1.0 / 3.0, 2.0 / 3.0)
            };
            let (p0v, p1v) = (clamp_v(p0[1]), clamp_v(p1[1]));
            let dy1 = (clamp_v(c0[1]) - p0v) * self.v_span;
            let dy2 = (clamp_v(c1[1]) - p1v) * self.v_span;
            out.push(FitKey {
                t: denorm_t(p0[0]),
                v: denorm_v(p0v),
                interp: Interp::bezier_w(x1, dy1, x2, dy2),
            });
        }
        // The trailing anchor (last segment's P1): pins the end, no outgoing segment.
        if let Some(last) = segs.last() {
            out.push(FitKey {
                t: denorm_t(last[3][0]),
                v: denorm_v(clamp_v(last[3][1])),
                interp: Interp::Linear,
            });
        }
        out
    }
}

/// Indices of the prominent local extrema (peaks + valleys) of the value axis,
/// always including both endpoints — the anchors the fit pins so a wave keys at
/// each turn. `min_prom` is the least value swing for a reversal to count as a
/// real extremum, which rejects the residual jitter the low-pass left behind (a
/// swinging-door / prominence peak detector). Returns strictly increasing
/// indices; a monotone run yields just `[0, n-1]`.
fn anchor_indices(pts: &[P], min_prom: f64) -> Vec<usize> {
    let n = pts.len();
    if n < 3 {
        return (0..n).collect();
    }
    let mut anchors = vec![0usize];
    let start_v = pts[0][1];
    let mut ext_i = 0usize; // running extremum candidate
    let mut ext_v = start_v;
    let mut trend = 0i32; // +1 rising, -1 falling, 0 not yet moved
    for (i, p) in pts.iter().enumerate().skip(1) {
        let v = p[1];
        match trend {
            0 => {
                if v > start_v + min_prom {
                    trend = 1;
                    ext_i = i;
                    ext_v = v;
                } else if v < start_v - min_prom {
                    trend = -1;
                    ext_i = i;
                    ext_v = v;
                }
            }
            1 => {
                if v >= ext_v {
                    ext_v = v;
                    ext_i = i;
                } else if v < ext_v - min_prom {
                    anchors.push(ext_i);
                    trend = -1;
                    ext_v = v;
                    ext_i = i;
                }
            }
            _ => {
                if v <= ext_v {
                    ext_v = v;
                    ext_i = i;
                } else if v > ext_v + min_prom {
                    anchors.push(ext_i);
                    trend = 1;
                    ext_v = v;
                    ext_i = i;
                }
            }
        }
    }
    if *anchors.last().unwrap() != n - 1 {
        anchors.push(n - 1);
    }
    anchors
}

/// Fit one monotone run (between two extrema) with ONE least-squares cubic —
/// its ends are peaks/valleys where the gesture's tangent is ~flat, so a single
/// cubic tracks a wave's half-cycle closely. A run a single cubic misses by more
/// than `SPLIT_SLACK × tol` (a genuinely complex run, not a wave) splits at its
/// worst point and recurses, but only to `depth` — so the key-per-extremum
/// result holds for waves while a complex run still gets bounded fidelity help
/// (at most `2^depth` cubics), never the Schneider blow-up.
fn fit_run(run: &[P], tol: f64, depth: u32, out: &mut Vec<[P; 4]>) {
    let m = run.len();
    if m < 2 {
        return;
    }
    let bez = one_cubic(run);
    let (err, worst) = max_value_error(run, &bez);
    if depth == 0 || m <= 3 || err <= SPLIT_SLACK * tol {
        out.push(bez);
        return;
    }
    let split = worst.clamp(1, m - 2);
    fit_run(&run[..=split], tol, depth - 1, out);
    fit_run(&run[split..], tol, depth - 1, out);
}

/// One least-squares cubic through `run` with tangents estimated from its ends
/// (a few Newton reparameterisation passes, no subdivision).
fn one_cubic(run: &[P]) -> [P; 4] {
    let m = run.len();
    if m == 2 {
        let third = scale(sub(run[1], run[0]), 1.0 / 3.0);
        return [run[0], add(run[0], third), sub(run[1], third), run[1]];
    }
    let lt = unit(sub(run[1], run[0]));
    let rt = unit(sub(run[m - 2], run[m - 1]));
    let mut u = chord_length_param(run);
    let mut bez = generate_bezier(run, &u, lt, rt);
    for _ in 0..4 {
        u = reparameterize(run, &u, &bez);
        bez = generate_bezier(run, &u, lt, rt);
    }
    bez
}

/// How far a single per-run cubic may miss (as a multiple of `tol`) before the
/// run is split further. Generous, so a wave keeps its one key per extremum;
/// only a genuinely complex run between two extrema gains extra keys.
const SPLIT_SLACK: f64 = 4.0;
/// Max recursive splits per run — bounds a complex run to `2^depth` cubics so it
/// gains fidelity without the Schneider blow-up (a wave never splits at all).
const RUN_SPLIT_DEPTH: u32 = 2;

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
