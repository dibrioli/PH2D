//! Graph-editor math (W3) — pure functions over a track's [`KeyView`]s.
//!
//! Everything the expanded-row curve needs, headless: sample the curve for the
//! polyline, find its vertical extent, and map a bézier handle between its
//! normalized `[0, 1]²` timing space and the `(time, value)` plane the panel
//! paints in.
//!
//! **Why not in the panel?** [`sample_keys`] must agree with
//! [`ph2d_anim::Track::sample`] *bit for bit* — the whole promise of the graph
//! editor is that the curve you drag is the curve that plays (B0.P4). A second
//! copy of the interpolation living in paint code would silently drift. Here it
//! is pinned by a golden test against the real sampler.

use ph2d_anim::{AnimValue, Interp, LinearInterp};

use crate::snapshot::KeyView;

/// Sample the curve at `t` seconds, exactly as [`ph2d_anim::Track::sample`]
/// would: flat-clamped outside the key range, otherwise the segment's `Interp`
/// remapping `u` before the lerp. `None` for an empty track.
#[must_use]
pub fn sample_keys(keys: &[KeyView], t: f64) -> Option<f32> {
    let (first, last) = (keys.first()?, keys.last()?);
    if keys.len() == 1 || t <= first.t_seconds {
        return Some(first.value);
    }
    if t >= last.t_seconds {
        return Some(last.value);
    }
    // The last key starting at or before `t` — `t` is strictly inside the range,
    // so this is never the final key and `idx + 1` always exists.
    let idx = keys.partition_point(|k| k.t_seconds <= t) - 1;
    let (k0, k1) = (&keys[idx], &keys[idx + 1]);
    let span = k1.t_seconds - k0.t_seconds;
    let u = if span > 0.0 {
        (t - k0.t_seconds) / span
    } else {
        0.0
    };
    // Lockstep with `ph2d_anim::curve::interpolate` (the golden below pins the
    // pair bit-for-bit): a weighted segment's `dy` is absolute, so the scalar
    // evaluates through `Interp::value`; the normalized variants keep the
    // original lerp float path.
    if let Interp::BezierW { .. } = k0.interp {
        return Some(k0.interp.value(f64::from(k0.value), f64::from(k1.value), u) as f32);
    }
    match AnimValue::lerp(
        AnimValue::Float(k0.value),
        AnimValue::Float(k1.value),
        k0.interp.remap(u),
    ) {
        AnimValue::Float(v) => Some(v),
        _ => Some(k0.value),
    }
}

/// The `(min, max)` of the keys' values, or `None` for an empty track. The
/// anchors only — see [`drawn_extent`] for what the band must actually fit.
#[must_use]
pub fn value_extent(keys: &[KeyView]) -> Option<(f32, f32)> {
    let first = keys.first()?.value;
    Some(
        keys.iter()
            .map(|k| k.value)
            .fold((first, first), |(lo, hi), v| (lo.min(v), hi.max(v))),
    )
}

/// The `(min, max)` of **everything the graph editor draws** over the visible
/// window `[t0, t1]`: the sampled curve (so an overshooting bézier is inside the
/// range, not off the top of the band), the key anchors, and the handles of any
/// segment touching a selected key. `None` when the track has no keys.
///
/// [`value_extent`] alone is not enough to fit the band: a segment whose handle
/// was dragged past its endpoint draws far outside its keys' range.
#[must_use]
pub fn drawn_extent(keys: &[KeyView], t0: f64, t1: f64, samples: usize) -> Option<(f32, f32)> {
    if keys.is_empty() {
        return None;
    }
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    let mut include = |v: f32| {
        if v.is_finite() {
            lo = lo.min(v);
            hi = hi.max(v);
        }
    };
    let n = samples.max(1);
    for i in 0..=n {
        let t = t0 + (t1 - t0) * (i as f64 / n as f64);
        if let Some(v) = sample_keys(keys, t) {
            include(v);
        }
    }
    for k in keys
        .iter()
        .filter(|k| k.t_seconds >= t0 && k.t_seconds <= t1)
    {
        include(k.value);
    }
    for w in keys.windows(2) {
        let (k0, k1) = (&w[0], &w[1]);
        // Only the handles that are actually painted, and only if the segment
        // overlaps the window.
        if !(k0.selected || k1.selected) || k1.t_seconds < t0 || k0.t_seconds > t1 {
            continue;
        }
        // Exact control points only (`Bezier`/`BezierW`). A `Hold`/`Eased`
        // segment's handles are TANGENTS the editor derives for display; a
        // steep preset (Expo, Back) would drag the whole band's fit out to
        // meet a hint, squashing the curve the author came to look at. Those
        // get clipped instead.
        if !matches!(k0.interp, Interp::Bezier { .. } | Interp::BezierW { .. }) {
            continue;
        }
        for (_, v) in segment_handle_points(k0, k1) {
            include(v as f32);
        }
    }
    (lo <= hi).then_some((lo, hi))
}

/// The absolute `(time, value)` positions of a segment's two handles — out
/// (`P1`, index 0) then in (`P2`, index 1) — for ANY interpolation: `BezierW`
/// places its `dy` offsets directly; every normalized variant maps its
/// [`Interp::tangent_handles`] through the segment's value change (the exact
/// positions the value graph has always drawn). This is the value-space
/// counterpart of [`handle_point`], and the one source the panel paints and
/// drags from.
#[must_use]
pub fn segment_handle_points(k0: &KeyView, k1: &KeyView) -> [(f64, f64); 2] {
    let (t0, v0) = (k0.t_seconds, f64::from(k0.value));
    let (t1, v1) = (k1.t_seconds, f64::from(k1.value));
    if let Interp::BezierW { x1, dy1, x2, dy2 } = k0.interp {
        let span = t1 - t0;
        return [(t0 + x1 * span, v0 + dy1), (t0 + x2 * span, v1 + dy2)];
    }
    let (h_out, h_in) = k0.interp.tangent_handles();
    [
        handle_point(t0, v0, t1, v1, h_out),
        handle_point(t0, v0, t1, v1, h_in),
    ]
}

/// A weighted tangent pair from one dragged handle: `which` (`0` = out, `1` =
/// in) moves to the absolute `(t, v)` while the OTHER handle keeps the exact
/// position it is drawn at (see [`segment_handle_points`]) — converting a
/// normalized segment to `BezierW` without the far end jumping, and editing a
/// flat segment's shape at all (the `handle_coords` gap this replaces: `dy` is
/// absolute, so `v0 == v1` no longer freezes the handle's value axis).
#[must_use]
pub fn weighted_with_handle(k0: &KeyView, k1: &KeyView, which: u8, t: f64, v: f64) -> Interp {
    let pts = segment_handle_points(k0, k1);
    let (out, inn) = if which == 0 {
        ((t, v), pts[1])
    } else {
        (pts[0], (t, v))
    };
    weighted_from_points(k0, k1, out, inn)
}

/// AE's minimum influence (0.1%): a zero-influence side has no tangent to
/// author a speed with (`speed = dy / (x·span)` degenerates), so the speed
/// graph's horizontal drag clamps just short of it.
const MIN_INFLUENCE: f64 = 1.0e-3;

/// The speed graph's editable handle TIP for one side of a segment (AE's
/// yellow arm): it extends horizontally from the key's speed point, and its
/// length IS the influence — the tip sits at `(t0 + x·span, endpoint speed)`.
/// `None` when the endpoint speed is not finite (a vertical tangent: the
/// curve spikes off-band and there is nothing to grab).
#[must_use]
pub fn speed_handle_tip(k0: &KeyView, k1: &KeyView, which: u8) -> Option<(f64, f64)> {
    let speed = crate::speed::segment_endpoint_speed(k0, k1, which);
    if !speed.is_finite() {
        return None;
    }
    let [out, inn] = segment_handle_points(k0, k1);
    let t = if which == 0 { out.0 } else { inn.0 };
    Some((t, speed))
}

/// A weighted tangent pair from one SPEED-graph handle drag — the AE gesture:
/// the pointer's `t` sets that side's INFLUENCE (the arm's length, clamped to
/// the segment and away from [`MIN_INFLUENCE`]'s degenerate zero) and `v` its
/// SPEED (value units per second): `dy1 = v·x1·span` / `dy2 = −v·(1−x2)·span`.
/// The other side keeps the position it is drawn at — a speed edit on one key
/// must not silently re-ease the far end of the segment.
#[must_use]
pub fn weighted_with_speed_handle(k0: &KeyView, k1: &KeyView, which: u8, t: f64, v: f64) -> Interp {
    let span = k1.t_seconds - k0.t_seconds;
    let (t0, v0) = (k0.t_seconds, f64::from(k0.value));
    let v1 = f64::from(k1.value);
    let [out, inn] = segment_handle_points(k0, k1);
    let mut x1 = if span > 0.0 { (out.0 - t0) / span } else { 0.0 };
    let mut x2 = if span > 0.0 { (inn.0 - t0) / span } else { 1.0 };
    let (mut dy1, mut dy2) = (out.1 - v0, inn.1 - v1);
    let u = if span > 0.0 { (t - t0) / span } else { 0.0 };
    if which == 0 {
        x1 = u.clamp(MIN_INFLUENCE, 1.0); // CLAMP-OK: const bounds, min<max
        dy1 = v * x1 * span;
    } else {
        x2 = u.clamp(0.0, 1.0 - MIN_INFLUENCE); // CLAMP-OK: const bounds, min<max
        dy2 = -v * (1.0 - x2) * span;
    }
    Interp::bezier_w(x1, dy1, x2, dy2)
}

/// The `BezierW` whose handles sit at the absolute points `out` / `inn`.
/// Influences clamp to the segment (monotone timing, as everywhere); a
/// zero-span segment pins them at its ends.
fn weighted_from_points(k0: &KeyView, k1: &KeyView, out: (f64, f64), inn: (f64, f64)) -> Interp {
    let (t0, v0) = (k0.t_seconds, f64::from(k0.value));
    let (t1, v1) = (k1.t_seconds, f64::from(k1.value));
    let span = t1 - t0;
    let (x1, x2) = if span > 0.0 {
        ((out.0 - t0) / span, (inn.0 - t0) / span)
    } else {
        (0.0, 1.0)
    };
    Interp::bezier_w(x1, out.1 - v0, x2, inn.1 - v1)
}

/// The `(time, value)` point of a segment handle whose normalized timing-space
/// coordinates are `h`. `P0 = (t0, v0)` and `P3 = (t1, v1)` are the segment's
/// endpoints; `h.1` outside `[0, 1]` is an overshoot.
#[must_use]
pub fn handle_point(t0: f64, v0: f64, t1: f64, v1: f64, h: (f64, f64)) -> (f64, f64) {
    (t0 + h.0 * (t1 - t0), v0 + h.1 * (v1 - v0))
}

/// Invert [`handle_point`]: the normalized coordinates of a handle dragged to
/// `(t, v)`. `hx` is clamped to `[0, 1]` (a non-monotone timing function has no
/// single solution — CSS validity, enforced again by [`ph2d_anim::Interp::bezier`]).
///
/// `hy` is `None` on a **flat** segment (`v0 == v1`): `Interp::Bezier` stores the
/// handle as a fraction of the value change, so a segment with no value change
/// has a degenerate value axis and no representable overshoot. The caller keeps
/// the handle's current `hy` rather than snapping it to zero. (Value-space
/// tangents, which would fix this, are W5 backlog.)
#[must_use]
pub fn handle_coords(t0: f64, v0: f64, t1: f64, v1: f64, t: f64, v: f64) -> (f64, Option<f64>) {
    let dt = t1 - t0;
    let dv = v1 - v0;
    let hx = if dt != 0.0 {
        ((t - t0) / dt).clamp(0.0, 1.0) // CLAMP-OK: CSS bezier x must stay in [0,1]
    } else {
        0.0
    };
    let hy = (dv != 0.0).then(|| (v - v0) / dv);
    (hx, hy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::TimelineViewSnapshot;
    use crate::state::TimelineState;
    use ph2d_anim::{AttributeEvaluator, Interp, RationalTime, Track};
    use ph2d_core::Playhead;

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

    /// THE golden assertion (plan W3.E6): the polyline the panel draws is the
    /// animation that plays. A second interpolation implementation is exactly the
    /// bug this test exists to prevent, so compare bit-for-bit, not approximately.
    #[test]
    fn sample_keys_matches_the_real_track_sampler_bit_for_bit() {
        use crate::prop::PropKind;
        let mut st = TimelineState::new();
        let mut ph = Playhead::new(1.0 / 60.0);
        let interps = [
            Interp::Linear,
            Interp::Hold,
            Interp::bezier(0.2, 1.4, 0.8, -0.3), // overshoot both ends
            Interp::bezier_w(0.3, 4.0, 0.7, -2.5), // weighted (value-space)
            Interp::Eased(ph2d_anim::Easing::new(
                ph2d_anim::EasingFamily::Cubic,
                ph2d_anim::EasingMode::InOut,
            )),
        ];
        for (i, interp) in interps.iter().enumerate() {
            crate::apply_intent(
                &mut st,
                &mut ph,
                crate::TimelineIntent::AddKey {
                    entity: 1,
                    prop: PropKind::TranslationX,
                    t: RationalTime::from_seconds(i as f64),
                    value: AnimValue::Float(i as f32 * 3.0 - 1.0),
                    interp: *interp,
                },
            );
        }
        let target = st
            .doc
            .binding_for(1, PropKind::TranslationX)
            .unwrap()
            .target;
        let mut snap = TimelineViewSnapshot::default();
        snap.rebuild(&mut st, &ph, false);
        let track: &Track = st.doc.active_clip().track(target).unwrap();
        let keys = &snap.tracks[0].keys;

        // Sweep past both ends so the flat clamps are covered too.
        for i in -20..=420 {
            let t = f64::from(i) / 100.0;
            let want = match track.sample(t) {
                AnimValue::Float(v) => v,
                other => panic!("expected Float, got {other:?}"),
            };
            assert_eq!(
                sample_keys(keys, t),
                Some(want),
                "graph polyline diverged from the sampler at t = {t}"
            );
        }
    }

    #[test]
    fn sampling_an_empty_track_has_no_value() {
        assert_eq!(sample_keys(&[], 0.5), None);
    }

    #[test]
    fn a_single_key_holds_its_value_everywhere() {
        let keys = [key(1.0, 7.0, Interp::Linear)];
        assert_eq!(sample_keys(&keys, -5.0), Some(7.0));
        assert_eq!(sample_keys(&keys, 100.0), Some(7.0));
    }

    #[test]
    fn coincident_keys_do_not_divide_by_zero() {
        // Two keys at the same instant: the segment has no span; `u` is 0, so the
        // sampler holds the first value rather than producing NaN.
        let keys = [
            key(1.0, 2.0, Interp::Linear),
            key(1.0, 9.0, Interp::Linear),
            key(2.0, 9.0, Interp::Linear),
        ];
        assert!(sample_keys(&keys, 1.0).unwrap().is_finite());
    }

    #[test]
    fn value_extent_spans_the_keys() {
        let keys = [
            key(0.0, 3.0, Interp::Linear),
            key(1.0, -2.0, Interp::Linear),
            key(2.0, 0.5, Interp::Linear),
        ];
        assert_eq!(value_extent(&keys), Some((-2.0, 3.0)));
        assert_eq!(value_extent(&[]), None);
    }

    #[test]
    fn drawn_extent_covers_an_overshoot_the_keys_do_not() {
        // The bug: fitting the band to `value_extent` (the keys) left an
        // overshooting curve drawn far outside the band, over the next row.
        let mut k0 = key(0.0, 0.0, Interp::bezier(0.2, 2.5, 0.8, -1.5));
        k0.selected = true;
        let keys = [k0, key(1.0, 10.0, Interp::Linear)];
        assert_eq!(value_extent(&keys), Some((0.0, 10.0)));
        let (lo, hi) = drawn_extent(&keys, 0.0, 1.0, 200).unwrap();
        assert!(hi > 10.0, "the curve rises past the last key: {hi}");
        assert!(lo < 0.0, "and dips below the first: {lo}");
    }

    #[test]
    fn drawn_extent_includes_the_handles_of_a_selected_segment() {
        // A handle can sit outside its own curve (a bezier stays inside the
        // convex hull of its control points), so fitting the curve alone would
        // still let the handle escape the band.
        let mut k0 = key(0.0, 0.0, Interp::bezier(0.5, 3.0, 0.5, 3.0));
        let keys_unselected = [k0.clone(), key(1.0, 1.0, Interp::Linear)];
        let curve_only = drawn_extent(&keys_unselected, 0.0, 1.0, 200).unwrap();
        k0.selected = true;
        let keys = [k0, key(1.0, 1.0, Interp::Linear)];
        let with_handles = drawn_extent(&keys, 0.0, 1.0, 200).unwrap();
        assert_eq!(with_handles.1, 3.0, "the handle at v = 3 is in range");
        assert!(with_handles.1 > curve_only.1);
    }

    #[test]
    fn drawn_extent_only_sees_the_visible_window() {
        let keys = [
            key(0.0, 0.0, Interp::Linear),
            key(1.0, 100.0, Interp::Linear),
        ];
        // A window entirely before the first key: the curve flat-clamps to it.
        assert_eq!(drawn_extent(&keys, -2.0, -1.0, 8), Some((0.0, 0.0)));
        assert_eq!(drawn_extent(&keys, 0.0, 1.0, 8), Some((0.0, 100.0)));
        assert_eq!(drawn_extent(&[], 0.0, 1.0, 8), None);
    }

    #[test]
    fn drawn_extent_survives_a_degenerate_sample_count() {
        let keys = [key(0.0, 2.0, Interp::Linear), key(1.0, 4.0, Interp::Linear)];
        let (lo, hi) = drawn_extent(&keys, 0.0, 1.0, 0).unwrap();
        assert!(lo.is_finite() && hi.is_finite() && lo <= hi);
    }

    #[test]
    fn a_handle_round_trips_through_the_time_value_plane() {
        let (t0, v0, t1, v1) = (1.0, 10.0, 3.0, 20.0);
        let h = (0.25, 1.4); // overshoot past the end value
        let (t, v) = handle_point(t0, v0, t1, v1, h);
        assert_eq!((t, v), (1.5, 24.0));
        let (hx, hy) = handle_coords(t0, v0, t1, v1, t, v);
        assert_eq!((hx, hy), (0.25, Some(1.4)));
    }

    #[test]
    fn handle_x_is_clamped_but_y_is_free_to_overshoot() {
        let (hx, hy) = handle_coords(0.0, 0.0, 1.0, 1.0, 5.0, 9.0);
        assert_eq!(hx, 1.0, "dragging past the next key pins x at the endpoint");
        assert_eq!(hy, Some(9.0), "y is unbounded — that is what overshoot is");
        let (hx, _) = handle_coords(0.0, 0.0, 1.0, 1.0, -5.0, 0.0);
        assert_eq!(hx, 0.0);
    }

    #[test]
    fn a_flat_segment_has_no_representable_handle_y() {
        // v0 == v1: `Interp::Bezier` stores y as a fraction of the value change,
        // so there is nothing to express. The caller must keep the old y.
        let (hx, hy) = handle_coords(0.0, 5.0, 2.0, 5.0, 1.0, 99.0);
        assert_eq!(hx, 0.5);
        assert_eq!(hy, None);
    }

    #[test]
    fn a_zero_length_segment_pins_the_handle_at_its_start() {
        let (hx, hy) = handle_coords(2.0, 0.0, 2.0, 1.0, 2.0, 0.5);
        assert_eq!((hx, hy), (0.0, Some(0.5)));
    }
}
