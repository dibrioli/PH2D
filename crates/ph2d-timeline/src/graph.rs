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
    // The segment's rhythm first (plano 18), exactly where `Track::segment_value` applies it.
    let u = ph2d_anim::retime::warp(k0.retime, u);
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
        // A Custom with points draws every node and handle (retimed or not, through the rhythm
        // map): each tip must be inside the band, or it can be neither seen nor grabbed (dono 15/11).
        if let Interp::Spline(_) = k0.interp {
            for v in crate::graph_pontos::custom_drawn_values(k0, k1) {
                include(v as f32);
            }
            continue;
        }
        // A retimed segment's handles are derived and already inside its played values.
        if k0.retime.is_some()
            || !matches!(k0.interp, Interp::Bezier { .. } | Interp::BezierW { .. })
        {
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
///
/// A RETIMED segment draws `E(W(u))`, so its handles are the tangents of what PLAYS: the played
/// endpoint slope `E'·W'`, with the easing's influence (`x`) shortened until the tip is inside the
/// values the segment plays — the same arm the speed view's Override shows, and the `BezierW` a
/// handle drag rebuilds the segment into. A vertical played tangent
/// (`W'` infinite at a box corner) keeps the easing's own point: there is no finite tip to draw.
#[must_use]
pub fn segment_handle_points(k0: &KeyView, k1: &KeyView) -> [(f64, f64); 2] {
    let (t0, v0) = (k0.t_seconds, f64::from(k0.value));
    let (t1, v1) = (k1.t_seconds, f64::from(k1.value));
    let mut pts = if let Interp::BezierW { x1, dy1, x2, dy2 } = k0.interp {
        let span = t1 - t0;
        [(t0 + x1 * span, v0 + dy1), (t0 + x2 * span, v1 + dy2)]
    } else {
        let (h_out, h_in) = k0.interp.tangent_handles();
        [
            handle_point(t0, v0, t1, v1, h_out),
            handle_point(t0, v0, t1, v1, h_in),
        ]
    };
    // A DERIVED arm (a preset's tangent: Elastic, Back, Bounce…; or anything retimed) has no
    // control point to honour: it is the direction of what plays, and its tip must stay where it
    // can be seen and grabbed (dono 15/11, «alças fora de vista»). Exact control points
    // (`Bezier`/`BezierW`/`Spline`) are what the band fits instead (`drawn_extent`).
    let exato = matches!(
        k0.interp,
        Interp::Bezier { .. } | Interp::BezierW { .. } | Interp::Spline(_)
    );
    if k0.retime.is_none() && !exato {
        let caixa = segment_value_range(k0, k1);
        let (len0, len1) = (pts[0].0 - t0, t1 - pts[1].0);
        if len0 > 0.0 {
            let s = (pts[0].1 - v0) / len0;
            let x = arm_inside(len0, s, v0, caixa);
            if x < len0 {
                pts[0] = (t0 + x, v0 + s * x);
            }
        }
        if len1 > 0.0 {
            let s = (v1 - pts[1].1) / len1;
            let x = arm_inside(len1, -s, v1, caixa);
            if x < len1 {
                pts[1] = (t1 - x, v1 - s * x);
            }
        }
    }
    if k0.retime.is_some() {
        let toca = [0u8, 1].map(|w| crate::speed::segment_endpoint_speed(k0, k1, w));
        let caixa = segment_value_range(k0, k1);
        let piso = MIN_INFLUENCE * (t1 - t0);
        if toca[0].is_finite() {
            let x = arm_inside(pts[0].0 - t0, toca[0], v0, caixa).max(piso);
            pts[0] = (t0 + x, v0 + toca[0] * x);
        }
        if toca[1].is_finite() {
            let x = arm_inside(t1 - pts[1].0, -toca[1], v1, caixa).max(piso);
            pts[1] = (t1 - x, v1 - toca[1] * x);
        }
    }
    pts
}

/// The `(min, max)` of the values a segment PLAYS (its two keys and 64 samples between).
fn segment_value_range(k0: &KeyView, k1: &KeyView) -> (f64, f64) {
    let keys = [k0.clone(), k1.clone()];
    let (t0, span) = (k0.t_seconds, k1.t_seconds - k0.t_seconds);
    (0..=64)
        .filter_map(|i| sample_keys(&keys, t0 + span * f64::from(i) / 64.0))
        .map(f64::from)
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
            (lo.min(v), hi.max(v))
        })
}

/// An arm's time length `len` from the key at value `v` along slope `s` (value per second),
/// shortened so its tip stays inside `[lo, hi]` — a steep played tangent would otherwise put the
/// tip far outside the band, where it can be neither seen nor grabbed.
fn arm_inside(len: f64, s: f64, v: f64, (lo, hi): (f64, f64)) -> f64 {
    let lim = if s > 0.0 {
        (hi - v) / s
    } else if s < 0.0 {
        (lo - v) / s
    } else {
        f64::INFINITY
    };
    len.min(lim)
}

/// Whether the value graph draws handle `which` (`0` = out, `1` = in): a retimed side whose played
/// tangent is VERTICAL (the time handle in a corner of the box) has no finite tip — hidden, as the
/// speed view hides it ([`speed_handle_tip`] → `None`), never drawn at the easing's own point.
#[must_use]
pub fn segment_handle_drawn(k0: &KeyView, k1: &KeyView, which: u8) -> bool {
    k0.retime.is_none()
        || crate::graph_pontos::end_handle_keeps_rhythm(k0, k1)
        || crate::speed::segment_endpoint_speed(k0, k1, which).is_finite()
}

/// A weighted tangent pair from one dragged handle: `which` (`0` = out, `1` =
/// in) moves to the absolute `(t, v)` while the OTHER handle keeps the exact
/// position it is drawn at (see [`segment_handle_points`]) — converting a
/// normalized segment to `BezierW` without the far end jumping, and editing a
/// flat segment's shape at all (the `handle_coords` gap this replaces: `dy` is
/// absolute, so `v0 == v1` no longer freezes the handle's value axis).
#[must_use]
pub fn weighted_with_handle(k0: &KeyView, k1: &KeyView, which: u8, t: f64, v: f64) -> Interp {
    // A Custom com PONTOS só troca a alça da ponta — virar `BezierW` apagaria os pontos. Parada
    // (`v0 == v1`) não tem forma normalizada: cai no `BezierW` como as outras.
    let (span, dv) = (k1.t_seconds - k0.t_seconds, f64::from(k1.value - k0.value));
    // Retimed WITH points: the end handle goes through the same rhythm map as the points
    // (`value_handle_points`) and the rhythm stays. Retimed without points: the handles drawn are
    // what PLAYS, and the drag rebuilds a `BezierW` (Override).
    if let Interp::Spline(_) = k0.interp
        && span > 0.0
        && dv.abs() > f64::EPSILON
    {
        let u = (t - k0.t_seconds) / span;
        let h = (
            ph2d_anim::retime::warp(k0.retime, u),
            (v - f64::from(k0.value)) / dv,
        );
        return if which == 0 {
            k0.interp.with_out_handle(h.0, h.1)
        } else {
            k0.interp.with_in_handle(h.0, h.1)
        };
    }
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
pub(crate) const MIN_INFLUENCE: f64 = 1.0e-3;

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
    // A mesma ponta que a vista de valor desenha (numa Custom com pontos retimada, pelo ritmo).
    let [out, inn] = crate::graph_pontos::value_handle_points(k0, k1);
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
    // Override REBUILDS (dono 15/11): a retimed segment comes out without its rhythm (the caller
    // drops it); `segment_handle_points` already gives the far side the speed that PLAYS there.
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
#[path = "graph_tests.rs"]
mod tests;
