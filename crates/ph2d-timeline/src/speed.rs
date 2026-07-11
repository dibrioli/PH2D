//! Speed-graph math (W5) — the VELOCITY view of a track's curve.
//!
//! The speed graph plots `d(value)/dt`: how fast the animated value changes over
//! time (After Effects' speed graph, Cavalry's, Blender's). It is derived from
//! the SAME easing the runtime plays ([`ph2d_anim::Interp::remap`]), so the speed
//! you see is the speed that plays (B0.P4).
//!
//! For a segment `k0 → k1` the value is `v0 + dv · P(u)`, `u = (t - t0) / span`
//! and `P = interp.remap`. So the velocity is
//! `d(value)/dt = dv · P'(u) / span`, where `P'` is differenced in normalized
//! `u`-space — never across a key, so a `Hold` reads as a clean zero, not a
//! differenced jump spike.
//!
//! Editing the speed at a segment endpoint maps back to that segment's bézier
//! tangent slope, keeping the handle's timing (influence) fixed. For a cubic
//! through `(0,0)/(1,1)` the endpoint slopes are `y1/x1` (start) and
//! `(1 - y2)/(1 - x2)` (end), so `velocity = (dv/span) · slope`. Inverting that
//! gives the handle `y` for a target velocity — see [`out_handle_y_for_speed`] /
//! [`in_handle_y_for_speed`].

use crate::snapshot::KeyView;

/// Central-difference half-window in normalized `u`-space. Small enough that the
/// O(h) error of differencing a smooth easing is sub-pixel, large enough to stay
/// clear of f64 cancellation.
const DIFF_U: f64 = 1.0e-3;

/// The curve's instantaneous velocity `d(value)/dt` at `t` seconds — the speed
/// graph's y. Zero outside the key range (the value is flat-clamped there), for a
/// single key, and on a zero-span segment. `None` for a track with no keys.
#[must_use]
pub fn sample_speed(keys: &[KeyView], t: f64) -> Option<f32> {
    let (first, last) = (keys.first()?, keys.last()?);
    if keys.len() == 1 || t < first.t_seconds || t > last.t_seconds {
        return Some(0.0);
    }
    // The segment containing `t`. `t ∈ [first, last]`; clamp the index to the
    // final segment so `t == last` differences that segment, not past it.
    let idx = keys
        .partition_point(|k| k.t_seconds <= t)
        .saturating_sub(1)
        .min(keys.len() - 2);
    let (k0, k1) = (&keys[idx], &keys[idx + 1]);
    let span = k1.t_seconds - k0.t_seconds;
    let dv = f64::from(k1.value) - f64::from(k0.value);
    if span <= 0.0 {
        return Some(0.0);
    }
    let u = ((t - k0.t_seconds) / span).clamp(0.0, 1.0); // CLAMP-OK: normalized u
    // dP/du of the pure easing, differenced in u-space (bounded [0,1], never
    // crossing a key). At the ends it degrades to a one-sided difference.
    let ua = (u - DIFF_U).max(0.0);
    let ub = (u + DIFF_U).min(1.0);
    let du = ub - ua;
    if du <= 0.0 {
        return Some(0.0);
    }
    let dpdu = (k0.interp.remap(ub) - k0.interp.remap(ua)) / du;
    Some((dv * dpdu / span) as f32)
}

/// The `(min, max)` velocity the speed graph draws across `[t0, t1]`, always
/// widened to include `0` so the zero-velocity reference line stays on-screen and
/// the sign of the motion is readable. `None` for an empty track.
#[must_use]
pub fn speed_extent(keys: &[KeyView], t0: f64, t1: f64, samples: usize) -> Option<(f32, f32)> {
    if keys.is_empty() {
        return None;
    }
    let (mut lo, mut hi) = (0.0f32, 0.0f32); // the zero reference line is always in
    let n = samples.max(1);
    for i in 0..=n {
        let t = t0 + (t1 - t0) * (i as f64 / n as f64);
        if let Some(s) = sample_speed(keys, t)
            && s.is_finite()
        {
            lo = lo.min(s);
            hi = hi.max(s);
        }
    }
    Some((lo, hi))
}

/// The normalized OUT-handle `y` (`P1.y`) that makes a segment START at velocity
/// `speed`, keeping the handle's `x` (influence) at `x1`. `None` on a degenerate
/// segment (no span or no value change — there is no velocity to scale), so the
/// caller keeps the handle's current `y`.
///
/// `velocity(start) = (dv/span) · (y1 / x1)`  ⇒  `y1 = x1 · speed / (dv/span)`.
#[must_use]
pub fn out_handle_y_for_speed(
    t0: f64,
    v0: f64,
    t1: f64,
    v1: f64,
    x1: f64,
    speed: f64,
) -> Option<f64> {
    let rate = value_rate(t0, v0, t1, v1)?;
    (x1 != 0.0).then(|| x1 * speed / rate)
}

/// The normalized IN-handle `y` (`P2.y`) that makes a segment END at velocity
/// `speed`, keeping the handle's `x` at `x2`. `None` on a degenerate segment.
///
/// `velocity(end) = (dv/span) · ((1 - y2) / (1 - x2))`  ⇒
/// `y2 = 1 - (1 - x2) · speed / (dv/span)`.
#[must_use]
pub fn in_handle_y_for_speed(
    t0: f64,
    v0: f64,
    t1: f64,
    v1: f64,
    x2: f64,
    speed: f64,
) -> Option<f64> {
    let rate = value_rate(t0, v0, t1, v1)?;
    (x2 != 1.0).then(|| 1.0 - (1.0 - x2) * speed / rate)
}

/// The segment's average value rate `dv / span`, or `None` when it is degenerate
/// (zero span or zero value change) — the reference the endpoint slope scales.
fn value_rate(t0: f64, v0: f64, t1: f64, v1: f64) -> Option<f64> {
    let (dt, dv) = (t1 - t0, v1 - v0);
    (dt != 0.0 && dv != 0.0).then_some(dv / dt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_anim::Interp;

    fn key(t: f64, v: f32, interp: Interp) -> KeyView {
        KeyView {
            id: ph2d_anim::KeyId::new(0),
            t_seconds: t,
            value: v,
            interp,
            selected: false,
        }
    }

    #[test]
    fn a_linear_segment_has_constant_speed_equal_to_its_rate() {
        // value 0 → 10 over 0..2 s ⇒ dv/span = 5 everywhere inside.
        let keys = [
            key(0.0, 0.0, Interp::Linear),
            key(2.0, 10.0, Interp::Linear),
        ];
        for t in [0.0, 0.1, 0.5, 1.0, 1.9, 2.0] {
            let s = sample_speed(&keys, t).unwrap();
            assert!((s - 5.0).abs() < 1e-3, "linear speed at {t} = {s}");
        }
    }

    #[test]
    fn speed_is_zero_outside_the_range_and_within_a_hold() {
        let keys = [key(1.0, 0.0, Interp::Hold), key(2.0, 10.0, Interp::Linear)];
        assert_eq!(sample_speed(&keys, 0.0), Some(0.0), "before the first key");
        assert_eq!(sample_speed(&keys, 5.0), Some(0.0), "after the last key");
        // The value is held constant across the whole Hold segment — including at
        // its far edge, where a cross-key difference would have spiked.
        assert_eq!(sample_speed(&keys, 1.5), Some(0.0), "hold interior");
        assert_eq!(
            sample_speed(&keys, 2.0),
            Some(0.0),
            "hold far edge, no spike"
        );
    }

    #[test]
    fn an_empty_or_single_key_track_has_no_or_zero_speed() {
        assert_eq!(sample_speed(&[], 0.5), None);
        assert_eq!(
            sample_speed(&[key(0.0, 3.0, Interp::Linear)], 0.5),
            Some(0.0)
        );
    }

    #[test]
    fn an_ease_in_out_starts_and_ends_slow_and_peaks_in_the_middle() {
        // The defining shape of the speed graph: a symmetric ease has ~0 speed at
        // both ends and its maximum in the middle.
        let keys = [
            key(0.0, 0.0, Interp::bezier(0.42, 0.0, 0.58, 1.0)), // CSS ease-in-out
            key(1.0, 10.0, Interp::Linear),
        ];
        let s_start = sample_speed(&keys, 0.0).unwrap();
        let s_mid = sample_speed(&keys, 0.5).unwrap();
        let s_end = sample_speed(&keys, 1.0).unwrap();
        assert!(s_start < 1.0, "slow start: {s_start}");
        assert!(s_end < 1.0, "slow end: {s_end}");
        assert!(s_mid > s_start && s_mid > s_end, "fast middle: {s_mid}");
    }

    #[test]
    fn speed_extent_always_includes_the_zero_line() {
        // An all-positive-speed segment still reports 0 as the lower bound, so the
        // zero reference is drawn.
        let keys = [key(0.0, 0.0, Interp::Linear), key(1.0, 4.0, Interp::Linear)];
        let (lo, hi) = speed_extent(&keys, 0.0, 1.0, 64).unwrap();
        assert_eq!(lo, 0.0, "zero line kept as the floor");
        assert!(hi >= 4.0, "the +4 rate is in range: {hi}");
        assert_eq!(speed_extent(&[], 0.0, 1.0, 8), None);
    }

    #[test]
    fn the_out_handle_inverse_hits_the_target_start_speed_exactly() {
        // dv/span = 5; ask for a 15 start speed (3× the linear rate), influence
        // x1 = 1/3. Solve y1, then the ANALYTIC start slope must reproduce 15.
        let x1 = 1.0 / 3.0;
        let y1 = out_handle_y_for_speed(0.0, 0.0, 2.0, 10.0, x1, 15.0).unwrap();
        let start_speed = (y1 / x1) * (10.0 / 2.0); // (dv/span) · (y1/x1)
        assert!(
            (start_speed - 15.0).abs() < 1e-9,
            "start speed = {start_speed}"
        );
    }

    #[test]
    fn the_in_handle_inverse_hits_the_target_end_speed_exactly() {
        let x2 = 2.0 / 3.0;
        let y2 = in_handle_y_for_speed(0.0, 0.0, 2.0, 10.0, x2, 15.0).unwrap();
        let end_speed = ((1.0 - y2) / (1.0 - x2)) * (10.0 / 2.0);
        assert!((end_speed - 15.0).abs() < 1e-9, "end speed = {end_speed}");
    }

    #[test]
    fn a_flat_segment_has_no_speed_to_scale() {
        // dv == 0: the endpoint slope is undefined, so the inverse declines and
        // the caller keeps the handle where it was.
        assert_eq!(out_handle_y_for_speed(0.0, 5.0, 1.0, 5.0, 0.33, 9.0), None);
        assert_eq!(in_handle_y_for_speed(0.0, 5.0, 1.0, 5.0, 0.66, 9.0), None);
    }

    #[test]
    fn setting_the_out_handle_from_a_speed_moves_the_sampled_start_speed_that_way() {
        // End to end against the numeric sampler: a segment eased slow-in has a
        // low start speed; solving the handle for a HIGHER start speed and reading
        // the sampler back shows the start speed rose toward the target.
        let x1 = 1.0 / 3.0;
        let before = [
            key(0.0, 0.0, Interp::bezier(x1, 0.05, 0.66, 1.0)), // slow in
            key(2.0, 10.0, Interp::Linear),
        ];
        let s0 = sample_speed(&before, 0.02).unwrap();
        let y1 = out_handle_y_for_speed(0.0, 0.0, 2.0, 10.0, x1, 12.0).unwrap();
        let after = [
            key(0.0, 0.0, Interp::bezier(x1, y1, 0.66, 1.0)),
            key(2.0, 10.0, Interp::Linear),
        ];
        let s1 = sample_speed(&after, 0.02).unwrap();
        assert!(s1 > s0, "start speed rose: {s0} -> {s1}");
        // ...and lands near the 12 asked for (forward-difference tolerance).
        assert!((s1 - 12.0).abs() < 0.5, "near the target 12: {s1}");
    }
}
