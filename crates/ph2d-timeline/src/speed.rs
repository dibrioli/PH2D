//! Speed-graph math (W5) — the VELOCITY view of a track's curve.
//!
//! The speed graph plots `d(value)/dt`: how fast the animated value changes over
//! time (After Effects' speed graph, Cavalry's, Blender's). It is derived from
//! the SAME easing the runtime plays ([`ph2d_anim::Interp::remap`]), so the speed
//! you see is the speed that plays (B0.P4).
//!
//! For a segment `k0 → k1` the value is `v0 + dv · P(u)`, `u = (t - t0) / span`
//! and `P = interp.remap`. So the velocity is `d(value)/dt = dv · P'(u) / span`,
//! where `P'` is [`ph2d_anim::Interp::slope`] — the ANALYTIC derivative of the
//! remap the runtime plays (parametric chain rule on the same bézier solve;
//! ported from the CSS reference, Chromium `cubic_bezier.cc`). Per-segment, so
//! a `Hold` reads as a clean zero, never a differenced jump spike. A vertical
//! tangent is `±∞` — display code skips non-finite samples.
//!
//! Editing the speed at a segment endpoint retunes that side's tangent while
//! keeping its influence — the inverse lives in
//! [`crate::graph::weighted_with_endpoint_speed`], which produces a
//! value-space [`ph2d_anim::Interp::BezierW`] (so it works on a flat segment
//! too: `dy` is absolute, `velocity(start) = dy1 / (x1·span)`).

use crate::snapshot::KeyView;

/// The curve's instantaneous velocity `d(value)/dt` at `t` seconds — the speed
/// graph's y. Zero outside the key range (the value is flat-clamped there), for a
/// single key, and on a zero-span segment. `None` for a track with no keys.
/// `±∞` at a vertical tangent — callers skip non-finite values.
#[must_use]
pub fn sample_speed(keys: &[KeyView], t: f64) -> Option<f32> {
    let (first, last) = (keys.first()?, keys.last()?);
    if keys.len() == 1 || t < first.t_seconds || t > last.t_seconds {
        return Some(0.0);
    }
    // The segment containing `t`. `t ∈ [first, last]`; clamp the index to the
    // final segment so `t == last` reads that segment's end, not past it.
    let idx = keys
        .partition_point(|k| k.t_seconds <= t)
        .saturating_sub(1)
        .min(keys.len() - 2);
    let (k0, k1) = (&keys[idx], &keys[idx + 1]);
    let span = k1.t_seconds - k0.t_seconds;
    if span <= 0.0 {
        return Some(0.0);
    }
    let u = ((t - k0.t_seconds) / span).clamp(0.0, 1.0); // CLAMP-OK: normalized u
    Some(
        (k0.interp
            .value_slope(f64::from(k0.value), f64::from(k1.value), u)
            / span) as f32,
    )
}

/// The velocity at one end of a segment — where the speed graph pins that
/// side's editable dot (AE attaches speed handles AT the keyframes). `which`
/// matches the value-view handles: `0` = the segment's START (out side of
/// `k0`), `1` = its END (in side of `k1`).
///
/// `dv/span · slope(0|1)` on the segment's own interp — NOT [`sample_speed`]
/// at the key's time, which reads the NEXT segment there (velocity is
/// discontinuous at keys). A `Hold` reads `0` at both ends (it is flat, then
/// steps); `±∞` at a vertical tangent — callers skip non-finite.
#[must_use]
pub fn segment_endpoint_speed(k0: &KeyView, k1: &KeyView, which: u8) -> f64 {
    let span = k1.t_seconds - k0.t_seconds;
    if span <= 0.0 {
        return 0.0;
    }
    let u = if which == 0 { 0.0 } else { 1.0 };
    k0.interp
        .value_slope(f64::from(k0.value), f64::from(k1.value), u)
        / span
}

/// The `(min, max)` velocity the speed graph draws across `[t0, t1]`: the
/// sampled curve plus the endpoint dots of segments touching a **selected** key
/// (the pixel grid can straddle a steep endpoint the dot pins exactly — the
/// same reason [`crate::drawn_extent`] folds the value handles in). Always
/// widened to include `0` so the zero-velocity reference line stays on-screen
/// and the sign of the motion is readable. `None` for an empty track.
#[must_use]
pub fn speed_extent(keys: &[KeyView], t0: f64, t1: f64, samples: usize) -> Option<(f32, f32)> {
    if keys.is_empty() {
        return None;
    }
    let (mut lo, mut hi) = (0.0f32, 0.0f32); // the zero reference line is always in
    let mut include = |v: f32| {
        if v.is_finite() {
            lo = lo.min(v);
            hi = hi.max(v);
        }
    };
    let n = samples.max(1);
    for i in 0..=n {
        let t = t0 + (t1 - t0) * (i as f64 / n as f64);
        if let Some(s) = sample_speed(keys, t) {
            include(s);
        }
    }
    for w in keys.windows(2) {
        let (k0, k1) = (&w[0], &w[1]);
        // Only the dots that are actually painted, and only if the segment
        // overlaps the window (mirror of `drawn_extent`).
        if !(k0.selected || k1.selected) || k1.t_seconds < t0 || k0.t_seconds > t1 {
            continue;
        }
        for which in [0u8, 1u8] {
            include(segment_endpoint_speed(k0, k1, which) as f32);
        }
    }
    Some((lo, hi))
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
            roving: false,
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
    fn the_weighted_inverse_hits_the_target_endpoint_speed_exactly() {
        // Round-trip through the producer: ask an endpoint for a velocity,
        // read it back through the ANALYTIC endpoint speed. Both ends, on a
        // sloped segment AND on a flat one — the flat case is the capability
        // weighted tangents add (the old normalized inverse had no value
        // change to scale and declined).
        use crate::graph::{speed_handle_tip, weighted_with_speed_handle};
        for (v0, v1) in [(0.0f32, 10.0f32), (5.0, 5.0)] {
            let k0 = key(0.0, v0, Interp::Linear);
            let k1 = key(2.0, v1, Interp::Linear);
            for which in [0u8, 1u8] {
                let before_other = segment_endpoint_speed(&k0, &k1, 1 - which);
                // A vertical-only drag: the pointer stays at the tip's t (the
                // influence holds) and asks for velocity 15.
                let (tip_t, _) = speed_handle_tip(&k0, &k1, which).unwrap();
                let mut k0w = k0.clone();
                k0w.interp = weighted_with_speed_handle(&k0, &k1, which, tip_t, 15.0);
                let got = segment_endpoint_speed(&k0w, &k1, which);
                assert!(
                    (got - 15.0).abs() < 1e-9,
                    "v0={v0} which={which}: asked 15, read {got}"
                );
                // The OTHER endpoint keeps the speed it had — retuning one
                // side must not silently re-ease the far end.
                let other = segment_endpoint_speed(&k0w, &k1, 1 - which);
                assert!(
                    (other - before_other).abs() < 1e-9,
                    "v0={v0} which={which}: far end moved {before_other} -> {other}"
                );
                // ...and the tip itself round-trips: same influence, new speed.
                let (t_after, s_after) = speed_handle_tip(&k0w, &k1, which).unwrap();
                assert!((t_after - tip_t).abs() < 1e-9, "influence held: {t_after}");
                assert!((s_after - 15.0).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn a_holds_endpoint_dots_read_zero_speed() {
        // THE display bug the audit caught: deriving the dot from the value
        // view's tangent handles read a Hold's IN side as 3× the linear rate
        // (the chord from the flat handle up to the anchor). A Hold is flat,
        // then steps — its velocity is 0 at BOTH ends.
        let k0 = key(0.0, 0.0, Interp::Hold);
        let k1 = key(1.0, 10.0, Interp::Linear);
        assert_eq!(segment_endpoint_speed(&k0, &k1, 0), 0.0, "out side");
        assert_eq!(segment_endpoint_speed(&k0, &k1, 1), 0.0, "in side");
        // And the reference cases stay right: Linear reads its rate at both
        // ends; an eased segment reads the analytic endpoint slopes.
        let k0 = key(0.0, 0.0, Interp::Linear);
        assert_eq!(segment_endpoint_speed(&k0, &k1, 0), 10.0);
        assert_eq!(segment_endpoint_speed(&k0, &k1, 1), 10.0);
        let k0 = key(0.0, 0.0, Interp::bezier(0.25, 0.1, 0.25, 1.0));
        let start = segment_endpoint_speed(&k0, &k1, 0);
        assert!((start - 4.0).abs() < 1e-9, "rate 10 · slope 0.4: {start}");
    }

    #[test]
    fn a_vertical_tangent_is_infinite_and_never_poisons_the_extent() {
        // P1 = (0, 1): the value moves instantly at the start — true velocity
        // +∞. The dot/sample is non-finite (display skips it) and the fitted
        // extent stays finite from the interior samples.
        let k0 = {
            let mut k = key(0.0, 0.0, Interp::bezier(0.0, 1.0, 0.58, 1.0));
            k.selected = true;
            k
        };
        let k1 = key(1.0, 10.0, Interp::Linear);
        assert!(segment_endpoint_speed(&k0, &k1, 0).is_infinite());
        let (lo, hi) = speed_extent(&[k0, k1], 0.0, 1.0, 64).unwrap();
        assert!(lo.is_finite() && hi.is_finite(), "extent stays finite");
    }

    #[test]
    fn the_extent_covers_a_steep_selected_endpoint_the_pixel_grid_misses() {
        // Start slope 1.0/0.05 = 20 ⇒ start speed 200. A panned window puts the
        // key BETWEEN grid samples (16 over [-0.3, 1.0] land at ±0.05 around
        // it), where the curve has already slowed far below 200. Selected: the
        // dot pins the true 200 into the fit. Unselected: the fit only sees
        // the samples (mirror of drawn_extent's handle rule).
        let mut k0 = key(0.0, 0.0, Interp::bezier(0.05, 1.0, 0.66, 1.0));
        let k1 = key(1.0, 10.0, Interp::Linear);
        let unselected = speed_extent(&[k0.clone(), k1.clone()], -0.3, 1.0, 16)
            .unwrap()
            .1;
        k0.selected = true;
        let selected = speed_extent(&[k0, k1], -0.3, 1.0, 16).unwrap().1;
        assert!(
            (f64::from(selected) - 200.0).abs() < 1e-3,
            "selected fit pins the endpoint speed: {selected}"
        );
        assert!(selected > unselected, "the dot widened the fit");
    }

    #[test]
    fn a_flat_weighted_segment_shows_the_speed_its_tangents_author() {
        // The capability weighted tangents add to the SPEED graph: a flat
        // segment (dv = 0) whose handles are lifted +2 in value has real,
        // non-zero velocity — it accelerates up, crests, and dives back.
        let k0 = key(0.0, 5.0, Interp::bezier_w(1.0 / 3.0, 2.0, 2.0 / 3.0, 2.0));
        let keys = [k0, key(1.0, 5.0, Interp::Linear)];
        let start = sample_speed(&keys, 0.0).unwrap();
        let mid = sample_speed(&keys, 0.5).unwrap();
        assert!(start > 0.0, "rising off the first key: {start}");
        assert!(mid.abs() < 1e-6, "flat at the symmetric crest: {mid}");
        // The dive back is the start's mirror.
        let end = sample_speed(&keys, 1.0).unwrap();
        assert!((end + start).abs() < 1e-3, "symmetric dive: {end}");
    }

    #[test]
    fn setting_the_out_handle_from_a_speed_moves_the_sampled_start_speed_that_way() {
        // End to end against the sampler: a segment eased slow-in has a low
        // start speed; retuning the endpoint to a HIGHER speed through the
        // weighted producer and reading the sampler back lands on the target.
        use crate::graph::{speed_handle_tip, weighted_with_speed_handle};
        let k0 = key(0.0, 0.0, Interp::bezier(1.0 / 3.0, 0.05, 0.66, 1.0)); // slow in
        let k1 = key(2.0, 10.0, Interp::Linear);
        let s0 = sample_speed(&[k0.clone(), k1.clone()], 0.02).unwrap();
        let (tip_t, _) = speed_handle_tip(&k0, &k1, 0).unwrap();
        let mut k0w = k0.clone();
        k0w.interp = weighted_with_speed_handle(&k0, &k1, 0, tip_t, 12.0);
        let s1 = sample_speed(&[k0w, k1], 0.02).unwrap();
        assert!(s1 > s0, "start speed rose: {s0} -> {s1}");
        // ...and lands near the 12 asked for (sampled just off the endpoint).
        assert!((s1 - 12.0).abs() < 0.5, "near the target 12: {s1}");
    }
}
