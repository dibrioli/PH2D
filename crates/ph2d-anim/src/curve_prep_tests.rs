//! Tests for [`super`] (`curve_prep.rs`) — the channel-aware preparation the fit
//! runs on dense recorded samples.
//!
//! Each property is asserted against what the ANIMATOR would see, not against the
//! routine that produces it: a spin that really turned twice must come out turning
//! twice.

use super::*;

/// Deterministic pseudo-noise in `[-1, 1]` (splitmix64-ish, no transcendentals) —
/// stands in for hand/mouse tremor.
fn noise(i: usize) -> f64 {
    let mut z = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z ^= z >> 30;
    z = z.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 27;
    ((z >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
}

/// A 60 fps recording of `f` over `[0, dur]`, with `tremor` of hand shake.
fn rec(f: impl Fn(f64) -> f64, dur: f64, tremor: f64) -> Vec<(f64, f64)> {
    let n = (dur * 60.0) as usize + 1;
    (0..n)
        .map(|i| {
            let t = dur * i as f64 / (n - 1) as f64;
            (t, f(t) + tremor * noise(i))
        })
        .collect()
}

/// Wrap an angle into `(−π, π]` — what the rotate gizmo's `atan2` pair writes into
/// `Transform.rotation`, and therefore what a recorded spin really contains.
fn wrap(a: f64) -> f64 {
    let mut a = a % TAU;
    if a > PI {
        a -= TAU;
    } else if a <= -PI {
        a += TAU;
    }
    a
}

// ── unwrap_angles ───────────────────────────────────────────────────────────

#[test]
fn a_two_turn_spin_recorded_wrapped_unwraps_to_the_spin_it_really_was() {
    // The gizmo records a continuous 2-turn spin as a ±2π sawtooth. Unwrapped, it
    // must be the original ramp back — monotone, spanning a full 4π.
    let truth = |t: f64| 2.0 * TAU * (t / 2.0); // 2 turns over 2 s
    let mut samples = rec(|t| wrap(truth(t)), 2.0, 0.0);

    // RED without the unwrap: the raw recording spans barely one turn.
    let hi = samples.iter().map(|&(_, v)| v).fold(f64::MIN, f64::max);
    let lo = samples.iter().map(|&(_, v)| v).fold(f64::MAX, f64::min);
    assert!(
        hi - lo < TAU + 0.01,
        "the wrapped recording cannot span more than one turn: {}",
        hi - lo
    );

    unwrap_angles(&mut samples);

    for w in samples.windows(2) {
        assert!(
            w[1].1 >= w[0].1 - 1e-9,
            "a forward spin never goes backward: {} -> {}",
            w[0].1,
            w[1].1
        );
    }
    for &(t, v) in &samples {
        assert!(
            (v - truth(t)).abs() < 1e-9,
            "unwrapped value at t={t} is {v}, the true angle is {}",
            truth(t)
        );
    }
}

#[test]
fn unwrapping_a_reversing_spin_follows_it_back() {
    // Turn one way, then back past the start: the unwrap must track the reversal,
    // not accumulate turns in one direction.
    let truth = |t: f64| if t < 1.0 { 6.0 * t } else { 6.0 * (2.0 - t) };
    let mut samples = rec(|t| wrap(truth(t)), 2.0, 0.0);
    unwrap_angles(&mut samples);
    for &(t, v) in &samples {
        assert!(
            (v - truth(t)).abs() < 1e-9,
            "t={t}: got {v}, want {}",
            truth(t)
        );
    }
}

#[test]
fn a_channel_that_never_wraps_is_untouched_by_the_unwrap() {
    let mut samples = rec(|t| 0.5 * t, 2.0, 0.0);
    let before = samples.clone();
    unwrap_angles(&mut samples);
    assert_eq!(samples, before, "no wrap to undo, nothing to change");
}

#[test]
fn the_unwrap_survives_tremor_on_the_wrap_itself() {
    // The sawtooth edge lands between two frames and both carry shake — the step
    // is still nowhere near π, so the unwrap reads it right.
    let truth = |t: f64| 3.0 * TAU * (t / 2.0);
    let mut samples = rec(|t| wrap(truth(t)), 2.0, 0.05);
    unwrap_angles(&mut samples);
    let span = samples[samples.len() - 1].1 - samples[0].1;
    assert!(
        (span - 3.0 * TAU).abs() < 0.2,
        "three turns is {:.2} rad; unwrapped span is {span:.2}",
        3.0 * TAU
    );
}

// ── prepare (the whole preparation, as `Track::range_samples` runs it) ───────

#[test]
fn prepare_unwraps_an_angle_channel_before_low_passing_it() {
    // Order matters: low-passing the WRAPPED signal would smear each 2π jump across
    // the frames around it, and there would be no clean sawtooth left to undo.
    let truth = |t: f64| 2.0 * TAU * (t / 2.0);
    let mut prepared = rec(|t| wrap(truth(t)), 2.0, 0.02);
    prepare(&mut prepared, FitChannel::ANGLE, 8);

    let span = prepared[prepared.len() - 1].1 - prepared[0].1;
    assert!(
        (span - 2.0 * TAU).abs() < 0.2,
        "the prepared spin still spans two turns: {span:.2}"
    );
    for w in prepared.windows(2) {
        assert!(
            w[1].1 >= w[0].1 - 1e-6,
            "and never doubles back: {} -> {}",
            w[0].1,
            w[1].1
        );
    }
}

#[test]
fn a_linear_channel_is_prepared_by_the_low_pass_alone() {
    let mut a = rec(|t| 3.0 * t, 2.0, 0.2);
    let mut b = a.clone();
    prepare(&mut a, FitChannel::LINEAR, 8);
    smooth_values(&mut b, 8);
    assert_eq!(a, b, "no angle to unwrap — the preparation IS the low-pass");
}

#[test]
fn preparing_a_bounded_channel_does_not_touch_the_values() {
    // The bound is enforced by the FIT (it clamps the fitted control points), not
    // by mangling the recorded samples.
    let mut a = rec(|t| 0.5 * t, 2.0, 0.01);
    let mut b = a.clone();
    prepare(&mut a, FitChannel::bounded(0.0, 1.0), 8);
    smooth_values(&mut b, 8);
    assert_eq!(a, b);
}

// ── FitChannel ──────────────────────────────────────────────────────────────

#[test]
fn the_channel_defaults_are_the_do_nothing_channel() {
    // `default()` must be the channel that changes nothing: a `FitChannel` field
    // added later then leaves every existing caller's behaviour byte-identical.
    assert_eq!(FitChannel::default(), FitChannel::LINEAR);
    assert_eq!(
        FitChannel::LINEAR,
        FitChannel {
            angular: false,
            bounds: None
        }
    );
    assert_eq!(
        FitChannel::ANGLE,
        FitChannel {
            angular: true,
            bounds: None
        }
    );
    assert_eq!(FitChannel::bounded(0.0, 1.0).bounds, Some((0.0, 1.0)));
}
