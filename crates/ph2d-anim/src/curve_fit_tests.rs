//! Tests for [`super`] (`curve_fit.rs`) — the Schneider F-curve fit. The two
//! properties that matter: the reconstructed curve stays within tolerance of
//! every dense sample (FIDELITY), and it uses far fewer keys than the input
//! (REDUCTION). Both are checked by REBUILDING the curve from the fitted
//! `Interp::BezierW` keys and sampling it back — the exact path the runtime
//! evaluates, so a fit that looks right but converts wrong is caught.

use super::*;
use crate::curve::Interp;

/// Evaluate the fitted keyframe curve at absolute time `t` (flat-clamped ends),
/// exactly as the runtime does: find the bracketing segment, map `t` to the
/// segment fraction `u`, and read `Interp::value`.
fn eval(keys: &[FitKey], t: f64) -> f64 {
    if keys.is_empty() {
        return 0.0;
    }
    if t <= keys[0].t {
        return keys[0].v;
    }
    let last = keys.len() - 1;
    if t >= keys[last].t {
        return keys[last].v;
    }
    let i = keys.windows(2).position(|w| t < w[1].t).unwrap_or(last - 1);
    let (a, b) = (keys[i], keys[i + 1]);
    let span = b.t - a.t;
    let u = if span > 0.0 { (t - a.t) / span } else { 0.0 };
    a.interp.value(a.v, b.v, u)
}

/// Worst reconstruction error over every input sample.
fn max_err(keys: &[FitKey], samples: &[(f64, f64)]) -> f64 {
    samples
        .iter()
        .map(|&(t, v)| (eval(keys, t) - v).abs())
        .fold(0.0f64, f64::max)
}

/// Densely sample a closed-form curve `f` over `[0, dur]` at `n` frames — a
/// stand-in for a recorded gesture.
fn record(f: impl Fn(f64) -> f64, dur: f64, n: usize) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| {
            let t = dur * i as f64 / (n - 1) as f64;
            (t, f(t))
        })
        .collect()
}

#[test]
fn a_smooth_arc_fits_within_tolerance_with_a_handful_of_keys() {
    // 240 recorded frames of a smooth sine bump (up to 100 and back). The fit
    // must reproduce it within tolerance using a tiny fraction of the keys.
    let samples = record(|t| 100.0 * (t * std::f64::consts::PI / 2.0).sin(), 2.0, 240);
    let tol = 0.5; // 0.5 units of value = 0.5% of the 100-unit range
    let keys = fit_fcurve(&samples, tol);
    assert!(
        max_err(&keys, &samples) <= tol,
        "fidelity: worst error {} exceeds tol {tol}",
        max_err(&keys, &samples)
    );
    assert!(
        keys.len() <= samples.len() / 20,
        "reduction: 240 samples → {} keys (expected a dramatic cut)",
        keys.len()
    );
    assert!(keys.len() >= 2, "at least the two endpoints");
}

#[test]
fn a_looser_tolerance_reaches_the_minimal_key_count() {
    // The same bump at a relaxed tolerance collapses to the minimal shape: one
    // cubic up, one down — 3 keys — proving the fit is not over-keying.
    let samples = record(|t| 100.0 * (t * std::f64::consts::PI / 2.0).sin(), 2.0, 240);
    let keys = fit_fcurve(&samples, 2.0);
    assert_eq!(keys.len(), 3, "a bump is up + down = 3 keys at 2% tol");
    assert!(max_err(&keys, &samples) <= 2.0);
}

#[test]
fn tighter_tolerance_keeps_more_keys_but_never_worse_fidelity() {
    let samples = record(
        |t| 50.0 * (t * 3.0).sin() + 20.0 * (t * 7.0).cos(),
        3.0,
        360,
    );
    let coarse = fit_fcurve(&samples, 2.0);
    let fine = fit_fcurve(&samples, 0.2);
    assert!(max_err(&coarse, &samples) <= 2.0, "coarse within its tol");
    assert!(max_err(&fine, &samples) <= 0.2, "fine within its tol");
    assert!(
        fine.len() >= coarse.len(),
        "a tighter tolerance never uses fewer keys ({} fine vs {} coarse)",
        fine.len(),
        coarse.len()
    );
}

#[test]
fn the_endpoints_are_pinned_exactly() {
    let samples = record(|t| (t * 5.0).sin() * 10.0 + 3.0, 4.0, 200);
    let keys = fit_fcurve(&samples, 0.5);
    let first = keys.first().unwrap();
    let last = keys.last().unwrap();
    assert!((first.t - samples[0].0).abs() < 1e-9);
    assert!((first.v - samples[0].1).abs() < 1e-9, "start value exact");
    assert!((last.t - samples[samples.len() - 1].0).abs() < 1e-9);
    assert!(
        (last.v - samples[samples.len() - 1].1).abs() < 1e-9,
        "end value exact"
    );
}

#[test]
fn a_straight_ramp_collapses_to_two_keys() {
    // A perfectly linear recording needs exactly two keys — no interior at all.
    let samples = record(|t| 4.0 * t - 1.0, 5.0, 300);
    let keys = fit_fcurve(&samples, 0.1);
    assert_eq!(keys.len(), 2, "a line is two keys: {}", keys.len());
    assert!(max_err(&keys, &samples) < 1e-6, "and it is exact");
}

#[test]
fn a_dead_flat_run_is_two_keys() {
    let samples = record(|_| 7.5, 3.0, 180);
    let keys = fit_fcurve(&samples, 0.1);
    assert_eq!(keys.len(), 2);
    assert!((keys[0].v - 7.5).abs() < 1e-9 && (keys[1].v - 7.5).abs() < 1e-9);
}

#[test]
fn a_sharp_corner_is_preserved_by_splitting() {
    // A tent (up then down) has a cusp at the apex. The fit cannot smooth it
    // away within a tight tolerance — it must split there, keeping the peak.
    let apex = 2.0;
    let samples = record(
        |t| if t < apex { t } else { 2.0 * apex - t },
        2.0 * apex,
        200,
    );
    let keys = fit_fcurve(&samples, 0.05);
    assert!(max_err(&keys, &samples) <= 0.05, "the cusp is honoured");
    // A key lands near the apex time (2.0), where the peak value ~2.0 is.
    let near_apex = keys
        .iter()
        .any(|k| (k.t - apex).abs() < 0.1 && (k.v - apex).abs() < 0.1);
    assert!(near_apex, "a key pins the corner: {keys:?}");
}

#[test]
fn tiny_inputs_pass_through_unchanged() {
    assert!(fit_fcurve(&[], 0.5).is_empty());
    let one = fit_fcurve(&[(1.0, 2.0)], 0.5);
    assert_eq!(one.len(), 1);
    let two = fit_fcurve(&[(0.0, 0.0), (1.0, 3.0)], 0.5);
    assert_eq!(two.len(), 2);
}

#[test]
fn duplicate_time_samples_are_collapsed() {
    // A parked cursor records the same frame twice; the fit dedups by time
    // (strictly increasing time is required) rather than producing a zero-span
    // segment.
    let samples = vec![(0.0, 0.0), (0.0, 0.0), (1.0, 5.0), (1.0, 5.0), (2.0, 0.0)];
    let keys = fit_fcurve(&samples, 0.5);
    assert!(keys.len() >= 2 && keys.len() <= 3);
    assert!(max_err(&keys, &samples) <= 0.5 + 1e-9);
}

#[test]
fn the_fit_is_a_valid_function_of_time_handles_never_run_backward() {
    // Every fitted handle x must stay in [0, 1] — otherwise the curve would fold
    // back in time (not a function). A fast-then-slow ease stresses the timing.
    let samples = record(|t| 100.0 * (1.0 - (-3.0 * t).exp()), 2.0, 240);
    let keys = fit_fcurve(&samples, 0.5);
    for k in &keys {
        if let Interp::BezierW { x1, x2, .. } = k.interp {
            assert!((0.0..=1.0).contains(&x1), "x1 out of range: {x1}");
            assert!((0.0..=1.0).contains(&x2), "x2 out of range: {x2}");
        }
    }
}

/// Deterministic pseudo-noise in `[-1, 1]` from an integer (splitmix64-ish, no
/// transcendentals) — stands in for hand/mouse tremor.
fn noise(i: usize) -> f64 {
    let mut z = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z as f64 / u64::MAX as f64) * 2.0 - 1.0
}

#[test]
fn low_pass_lets_a_noisy_recording_reduce_far_more() {
    // A smooth bump buried in tremor — the real mocap case. Without smoothing
    // the fit chases every noise bump (barely reduces); a few low-pass passes
    // strip the jitter so it collapses to a clean handful, still tracking the
    // underlying signal.
    let n = 300;
    let signal = |t: f64| 100.0 * (t * std::f64::consts::PI / 2.0).sin();
    let noisy: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            let t = 2.0 * i as f64 / (n - 1) as f64;
            (t, signal(t) + 1.5 * noise(i)) // ~1.5 units of tremor
        })
        .collect();
    let tol = 0.5; // 0.5% of the 100 range — tight

    let raw = fit_fcurve(&noisy, tol);
    let mut smoothed = noisy.clone();
    smooth_values(&mut smoothed, 6);
    let clean = fit_fcurve(&smoothed, tol);

    assert!(
        clean.len() * 2 < raw.len(),
        "low-pass more than halves the keys: {} raw vs {} smoothed",
        raw.len(),
        clean.len()
    );
    // And it still follows the underlying SIGNAL (not the noise) — within a
    // couple of tremor amplitudes.
    for i in 0..n {
        let t = 2.0 * i as f64 / (n - 1) as f64;
        assert!(
            (eval(&clean, t) - signal(t)).abs() < 4.0,
            "tracks the signal at t={t}"
        );
    }
}

#[test]
fn smooth_values_pins_the_endpoints_and_is_a_noop_when_disabled() {
    let mut s = vec![(0.0, 0.0), (1.0, 10.0), (2.0, -5.0), (3.0, 7.0)];
    let orig = s.clone();
    smooth_values(&mut s, 0);
    assert_eq!(s, orig, "0 passes changes nothing");
    smooth_values(&mut s, 4);
    assert_eq!(s[0], orig[0], "first endpoint pinned");
    assert_eq!(s[3], orig[3], "last endpoint pinned");
    // Times never move.
    for (a, b) in s.iter().zip(&orig) {
        assert_eq!(a.0, b.0, "times untouched");
    }
}

#[test]
fn value_scale_is_handled_by_normalisation() {
    // The SAME shape at two wildly different value scales (radians ~π vs pixels
    // ~1000) fits to the same key count for a proportional tolerance — proof the
    // axis normalisation removes the scale dependence.
    let shape = |t: f64, amp: f64| amp * (t * std::f64::consts::PI).sin();
    let small_amp = 3.2; // radians-ish
    let small = record(|t| shape(t, small_amp), 1.0, 200);
    let large = record(|t| shape(t, 1000.0), 1.0, 200);
    let ks = fit_fcurve(&small, small_amp * 0.005); // 0.5% of range
    let kl = fit_fcurve(&large, 1000.0 * 0.005);
    assert_eq!(
        ks.len(),
        kl.len(),
        "normalisation makes the fit scale-independent: {} vs {}",
        ks.len(),
        kl.len()
    );
}
