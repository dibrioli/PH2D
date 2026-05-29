//! Golden fixtures for the Levien single-cubic fit (W1.T1.4 DoD).
//!
//! Each canonical fixture is a smooth curve **within the single-cubic
//! domain** (≤ 1 inflection); the fit must reproduce it to `< 0.5 px` max
//! deviation. A 180° arc is included as a *negative* control: it is out of
//! domain and must NOT spuriously pass (proving the metric has teeth).
//!
//! "px" here is literal: fixtures are built at a ~100 px scale.
//!
//! ## Scope vs the T1.4 plan fixtures
//!
//! The plan ([`docs/Vector Module/17_plano_de_implementacao.md`] T1.4) names
//! *triangle / full circle / spiral / hand-drawn pencil / complex polygon*.
//! Those are **multi-cubic** shapes: a single cubic cannot represent a full
//! circle (needs ≥ 4) or a spiral, so they require the curve-**subdivision**
//! layer that splits a long stroke into per-segment chords before fitting —
//! and subdivision is explicitly W2 (see the module doc's "Domain" section).
//! This file therefore covers the *single-cubic primitive* over each
//! single-cubic-domain segment such a subdivider would hand it (arcs ≤ 90°,
//! an exact cubic, a one-inflection S, a straight run). The named composite
//! fixtures land when the W2 subdivider that feeds them exists.

use glam::Vec2;
use ph2d_vector_doc::cubic_fit::fit_cubic_levien;

/// DoD ceiling.
const MAX_PX: f64 = 0.5;

fn evalc(p: [Vec2; 4], t: f64) -> (f64, f64) {
    let d = |v: Vec2| (f64::from(v.x), f64::from(v.y));
    let (p0, p1, p2, p3) = (d(p[0]), d(p[1]), d(p[2]), d(p[3]));
    let mt = 1.0 - t;
    let (a, b, c, e) = (mt * mt * mt, 3.0 * mt * mt * t, 3.0 * mt * t * t, t * t * t);
    (
        a * p0.0 + b * p1.0 + c * p2.0 + e * p3.0,
        a * p0.1 + b * p1.1 + c * p2.1 + e * p3.1,
    )
}

/// Symmetric (two-sided Hausdorff) deviation between the fitted cubic and
/// `samples`. The reverse term (curve → nearest sample) is what catches a
/// fit that loops or overshoots *between* two samples — a one-sided
/// sample → curve metric would read ~0 there and pass vacuously.
fn max_error(samples: &[Vec2]) -> f64 {
    let (a, b, c, d) = fit_cubic_levien(samples);
    let p = [a, b, c, d];
    let pts: Vec<(f64, f64)> = samples
        .iter()
        .map(|&q| (f64::from(q.x), f64::from(q.y)))
        .collect();
    let curve: Vec<(f64, f64)> = (0..=1000).map(|j| evalc(p, j as f64 / 1000.0)).collect();
    let dist = |a: (f64, f64), b: (f64, f64)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
    // distance from point `q` to the source polyline (segments).
    let to_polyline = |q: (f64, f64)| {
        pts.windows(2)
            .map(|w| {
                let (a, b) = (w[0], w[1]);
                let (abx, aby) = (b.0 - a.0, b.1 - a.1);
                let len2 = abx * abx + aby * aby;
                let t = if len2 > 0.0 {
                    (((q.0 - a.0) * abx + (q.1 - a.1) * aby) / len2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                dist(q, (a.0 + t * abx, a.1 + t * aby))
            })
            .fold(f64::INFINITY, f64::min)
    };
    // fwd: each sample to the nearest point on the (densely sampled) curve.
    let fwd = pts
        .iter()
        .map(|&q| {
            curve
                .iter()
                .map(|&c| dist(q, c))
                .fold(f64::INFINITY, f64::min)
        })
        .fold(0.0_f64, f64::max);
    // rev: each curve point to the nearest point on the source polyline.
    let rev = curve
        .iter()
        .map(|&q| to_polyline(q))
        .fold(0.0_f64, f64::max);
    fwd.max(rev)
}

/// Circular arc on radius 100, swept from `a0` to `a1`, `n` samples.
fn arc(a0: f64, a1: f64, n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let a = a0 + (a1 - a0) * (i as f64) / ((n - 1) as f64);
            Vec2::new((100.0 * a.cos()) as f32, (100.0 * a.sin()) as f32)
        })
        .collect()
}

fn sample_cubic(p: [Vec2; 4], n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let (x, y) = evalc(p, i as f64 / (n - 1) as f64);
            Vec2::new(x as f32, y as f32)
        })
        .collect()
}

// ── Canonical fixtures (must pass) ──────────────────────────────────────

#[test]
fn fixture_straight_line() {
    let s: Vec<Vec2> = (0..16).map(|i| Vec2::new(i as f32 * 8.0, 0.0)).collect();
    let e = max_error(&s);
    assert!(e < MAX_PX, "straight line err={e} px");
}

#[test]
fn fixture_quarter_circle_90deg() {
    let s = arc(std::f64::consts::FRAC_PI_2, 0.0, 20);
    let e = max_error(&s);
    assert!(e < MAX_PX, "90° arc err={e} px");
}

#[test]
fn fixture_arc_60deg() {
    let s = arc(0.0, std::f64::consts::FRAC_PI_3, 20);
    let e = max_error(&s);
    assert!(e < MAX_PX, "60° arc err={e} px");
}

#[test]
fn fixture_exact_cubic_roundtrip() {
    let ctrl = [
        Vec2::new(0.0, 0.0),
        Vec2::new(30.0, 80.0),
        Vec2::new(120.0, 90.0),
        Vec2::new(150.0, 10.0),
    ];
    let s = sample_cubic(ctrl, 20);
    let e = max_error(&s);
    // Tighter than the 0.5 DoD: a sampled exact cubic round-trips to ~0.19 px
    // (residual is tangent re-estimation error, not solver slop). Asserting
    // 0.25 turns this fixture into a real solver-regression guard rather than
    // sleeping under the loose ceiling.
    assert!(e < 0.25, "cubic round-trip err={e} px");
}

#[test]
fn fixture_single_inflection_s() {
    // A genuine S with exactly one inflection (still single-cubic territory).
    let ctrl = [
        Vec2::new(0.0, 0.0),
        Vec2::new(40.0, 90.0),
        Vec2::new(90.0, -30.0),
        Vec2::new(150.0, 40.0),
    ];
    let s = sample_cubic(ctrl, 20);
    let e = max_error(&s);
    assert!(e < MAX_PX, "single-inflection S err={e} px");
}

#[test]
fn fit_is_rotation_and_translation_equivariant() {
    // The fit is built in a normalized chord frame and rotated back, so a
    // rotated+translated input must yield the rotated+translated cubic to
    // ~f32 epsilon. This is the test that catches a sign error in the
    // to_norm/to_world rotation that every axis-aligned fixture would miss.
    let base = arc(0.0, std::f64::consts::FRAC_PI_2, 18);
    let (th, tx, ty) = (0.6435_f64, 12.5_f64, -7.25_f64); // arbitrary rigid motion
    let (ct, st) = (th.cos(), th.sin());
    let xform = |p: Vec2| {
        let (x, y) = (f64::from(p.x), f64::from(p.y));
        Vec2::new((x * ct - y * st + tx) as f32, (x * st + y * ct + ty) as f32)
    };
    let moved: Vec<Vec2> = base.iter().map(|&p| xform(p)).collect();

    let f0 = fit_cubic_levien(&base);
    let f1 = fit_cubic_levien(&moved);
    for (orig, got) in [(f0.0, f1.0), (f0.1, f1.1), (f0.2, f1.2), (f0.3, f1.3)] {
        let want = xform(orig);
        let drift = (want - got).length();
        assert!(
            drift < 1e-2,
            "rigid-motion drift {drift} (want {want:?}, got {got:?})"
        );
    }
}

#[test]
fn closed_input_first_equals_last_stays_bounded() {
    // A closed loop is out of single-cubic domain; callers split upstream.
    // The fit must not explode — it collapses to a bounded near-point at
    // the coincident endpoint, never NaN, never a giant overshoot.
    let mut s = arc(0.0, std::f64::consts::TAU * 0.9, 24);
    s.push(s[0]); // close it
    let (p0, a, b, p3) = fit_cubic_levien(&s);
    for c in [p0, a, b, p3] {
        assert!(c.x.is_finite() && c.y.is_finite(), "non-finite {c:?}");
        assert!(c.length() < 1000.0, "closed-input fit exploded: {c:?}");
    }
}

// ── Robustness guards (these are the two the implementation documents but
//    the happy-path fixtures don't exercise — a mutation that deletes either
//    guard passes every fixture above) ─────────────────────────────────────

#[test]
fn scorer_rejects_looping_candidate() {
    // A multi-candidate input. The one-sided source→curve score is small for
    // a fit that loops *between* the two coincident middle samples, so only
    // the reverse (curve→source) Hausdorff term steers selection to the sane
    // fit. Deleting that term lets a ~20 px looping cubic win here.
    let s = [
        Vec2::new(0.0, 0.0),
        Vec2::new(20.0, 15.0),
        Vec2::new(20.0, 15.0),
        Vec2::new(50.0, 1.0),
    ];
    let e = max_error(&s);
    assert!(e < 5.0, "scorer chose a looping fit: {e} px");
}

#[test]
fn overshoot_handle_is_rejected() {
    // Without the `d1 <= MAX_ARM` bound the solver can pick a root whose end
    // handle is ~12× the chord, flinging a control point far off the curve.
    let s = [
        Vec2::new(0.0, 0.0),
        Vec2::new(2.0, 40.0),
        Vec2::new(4.0, 40.0),
        Vec2::new(6.0, 0.2),
    ];
    let (p0, a, b, p3) = fit_cubic_levien(&s);
    let chord = (p3 - p0).length();
    assert!(chord > 0.0);
    assert!(
        (a - p0).length() / chord <= 10.5,
        "ctrl_a arm exceeds MAX_ARM"
    );
    assert!(
        (b - p3).length() / chord <= 10.5,
        "ctrl_b arm exceeds MAX_ARM"
    );
}

#[test]
fn output_is_a_stable_golden() {
    // Pins the exact fit for a fixed input. Any change to the scan/bisect
    // tuning constants (SCAN_STEPS, BISECT_ITERS) or the root-selection that
    // shifts which root wins moves these control points well beyond the
    // tolerance — so this is the executable guard on the determinism
    // contract that the per-call `deterministic_across_calls` unit test
    // (same-run repeatability) cannot give. Recorded on the reference build;
    // the no-transcendental math is bit-stable cross-target.
    let s: Vec<Vec2> = (0..12)
        .map(|i| {
            let a = (75.0_f64).to_radians() * (i as f64) / 11.0;
            Vec2::new((80.0 * a.cos()) as f32, (80.0 * a.sin()) as f32)
        })
        .collect();
    let got = fit_cubic_levien(&s);
    // 5 sig-figs (f32-clean); the 2e-3 tolerance is far below the px-scale
    // shift any constant/root-selection change would cause, yet immune to
    // sub-ULP cross-target noise.
    let want = (
        Vec2::new(80.0, 0.0),
        Vec2::new(79.985, 35.826),
        Vec2::new(55.307, 67.987),
        Vec2::new(20.706, 77.274),
    );
    for (g, w) in [
        (got.0, want.0),
        (got.1, want.1),
        (got.2, want.2),
        (got.3, want.3),
    ] {
        assert!(
            (g - w).length() < 2e-3,
            "golden drift: got {g:?}, want {w:?}"
        );
    }
}

#[test]
fn handles_a_dense_realistic_path() {
    // T1.4 perf criterion is "< 1 ms for a 100-segment path"; a wall-clock
    // assert would flake across CI machines, so instead we prove the O(N)
    // path stays correct and bounded at a realistic-large sample count (the
    // perf lens confirmed the cost is microseconds — linear in N). A 256-pt
    // ≤90° arc must still fit within the DoD.
    let s = arc(0.0, std::f64::consts::FRAC_PI_2, 256);
    let e = max_error(&s);
    assert!(e < MAX_PX, "dense 256-pt arc err={e} px");
}

// ── Negative control (must NOT pass — proves the metric is real) ─────────

#[test]
fn negative_control_half_circle_is_out_of_domain() {
    let s = arc(std::f64::consts::PI, 0.0, 28);
    let e = max_error(&s);
    assert!(
        e > MAX_PX,
        "a 180° arc fit to {e} px would mean the error metric is vacuous"
    );
}
