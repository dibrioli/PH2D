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

use ph2d_anim::{AnimValue, LinearInterp};

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
    match AnimValue::lerp(
        AnimValue::Float(k0.value),
        AnimValue::Float(k1.value),
        k0.interp.remap(u),
    ) {
        AnimValue::Float(v) => Some(v),
        _ => Some(k0.value),
    }
}

/// The `(min, max)` of the keys' values, or `None` for an empty track. This is
/// the vertical extent the expanded row fits to — the keys, not the curve, so a
/// handle drag never makes the view breathe under the cursor (an overshoot past
/// the padded range is clipped, which is honest and stable).
#[must_use]
pub fn value_extent(keys: &[KeyView]) -> Option<(f32, f32)> {
    let first = keys.first()?.value;
    Some(
        keys.iter()
            .map(|k| k.value)
            .fold((first, first), |(lo, hi), v| (lo.min(v), hi.max(v))),
    )
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
        let track: &Track = st.doc.active_clip().track(target).unwrap();
        let mut snap = TimelineViewSnapshot::default();
        snap.rebuild(&st, &ph);
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
