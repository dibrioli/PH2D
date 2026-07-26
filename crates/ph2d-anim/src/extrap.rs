//! Per-track **EXTRAPOLATION** beyond the keyed range (crown-jewels plan §6) —
//! the `loopOut`/cycle/pingpong/continue of After Effects and Unreal's Pre/Post
//! Infinity. A track's `pre`/`post` say what it does *before the first key* and
//! *after the last*; the rule lives here, consulted by [`Track::sample`] at the
//! flat-clamp ends (`track.rs`).
//!
//! # The fade pin (why this is the delicate wave)
//!
//! The default [`Extrap::Hold`] is the flat-clamp this project shipped forever —
//! and a Hold/Hold track samples **BYTE-IDENTICALLY** to the pre-extrapolation
//! engine, so `Track::sample`'s Hold branch returns the boundary key value
//! DIRECTLY and never enters this module. That byte-identity is what keeps the
//! fade fingerprint (`ph2d-timeline/tests/fade_fingerprint.rs`) stable: the
//! crossfade's `hold_at` crosses to values read by this same sampler, so a
//! non-Hold mode changes them — but only when opt-in.
//!
//! # Time Remap is inert here BY CONSTRUCTION
//!
//! A `PropKind::TimeRemap` track is sampled through `clock::remap_through`, which
//! contours `Track::sample` outside the range with its own rule (slope-1 /
//! freeze). It never reaches this module, so extrapolation cannot touch it — and
//! the panel does not offer the control for a Time-Remap row (both sides of the
//! same fact).
//!
//! Transcendental-free (HR-5): Loop/PingPong map `t` back to the range with
//! `rem_euclid`, Continue is a linear extension along the boundary segment's
//! end-slope ([`Interp::value_slope`], the SAME analytic slope the speed graph
//! reads — no second slope implementation).

use ph2d_vector_traits::AnimValue;
use serde::{Deserialize, Serialize};

use super::Track;

/// How a [`Track`] behaves OUTSIDE its keyed range, on one side.
///
/// [`Extrap::Hold`] is the historical flat-clamp default: a Hold/Hold track is
/// sampled without ever entering the extrapolation path, so it is byte-identical
/// to the engine that predates this feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Extrap {
    /// Flat-clamp: hold the boundary key's value forever (the historical default).
    #[default]
    Hold,
    /// Cycle: repeat the whole keyed range end-to-end (AE `loopOut("cycle")`).
    Loop,
    /// Reflect: bounce back and forth across the range (AE `loopOut("pingpong")`).
    PingPong,
    /// Linear extension at the boundary segment's end-slope
    /// (AE `loopOut("continue")` / Unreal *Linear* infinity).
    Continue,
}

/// Which end of a track's keyed range: `Pre` before the first key, `Post` after
/// the last (the two extrapolate independently — the AE loopIn / loopOut). The
/// authoring layer names sides with this too (a `SetTrackExtrap` intent), so it
/// is one public enum, not a crate-private `Side` plus a mirror in the panel.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExtrapSide {
    /// Before the first key.
    Pre,
    /// After the last key.
    Post,
}

/// The value at an out-of-range `t`, given a non-Hold `mode`. `t0`/`tn` are the
/// first/last key times (seconds); the caller guarantees `t < t0` on `Pre` and
/// `t > tn` on `Post` (the exact boundaries return the boundary value in
/// [`Track::sample`]). Pure — reads only the track's keys.
pub(crate) fn extrapolate(
    track: &Track,
    side: ExtrapSide,
    mode: Extrap,
    t: f64,
    t0: f64,
    tn: f64,
) -> AnimValue {
    let period = tn - t0;
    // A zero-width range (every key at one instant) has nothing to repeat or
    // slope along; hold the boundary value.
    if period <= 0.0 {
        return boundary_value(track, side);
    }
    match mode {
        Extrap::Hold => boundary_value(track, side),
        // Cycle: `(t - t0) mod period` maps into `[0, period)`. `rem_euclid`
        // handles the Pre side (`t - t0 < 0`) too — just before the start reads
        // just before the end, which is what looping backward shows.
        Extrap::Loop => track.value_at_in_range(t0 + (t - t0).rem_euclid(period)),
        // Reflect: fold the phase over `[0, period]` — `q` in the second half of
        // the `2·period` cycle bounces back off the far end.
        Extrap::PingPong => {
            let q = (t - t0).rem_euclid(2.0 * period);
            let mapped = if q <= period { q } else { 2.0 * period - q };
            track.value_at_in_range(t0 + mapped)
        }
        Extrap::Continue => continue_value(track, side, t),
    }
}

/// The boundary key's value — the Hold answer, and the safe fallback for a
/// degenerate range or a non-scalar Continue.
fn boundary_value(track: &Track, side: ExtrapSide) -> AnimValue {
    let keys = track.keys();
    match side {
        ExtrapSide::Pre => keys[0].value,
        ExtrapSide::Post => keys[keys.len() - 1].value,
    }
}

/// Linear extension of the boundary segment: `v_edge + slope·(t − t_edge)`, where
/// `slope` is the segment's value-slope (per second) at the boundary — `u = 0` at
/// the first key (Pre), `u = 1` at the last (Post). Uses the SAME analytic
/// [`Interp::value_slope`] the speed graph reads.
///
/// Scalar-only: a non-`Float` boundary has no linear extension, so it holds. A
/// vertical tangent (slope `±∞`) would shoot the value to infinity, so it holds
/// too — Continue extends smoothly or not at all, it never explodes.
fn continue_value(track: &Track, side: ExtrapSide, t: f64) -> AnimValue {
    let keys = track.keys();
    let n = keys.len();
    // `edge` = the boundary key; `seg` = the segment touching it; `u` = the
    // parameter at the boundary within that segment.
    let (edge, seg, u) = match side {
        ExtrapSide::Pre => (0, 0, 0.0), // first key, first segment, its start
        ExtrapSide::Post => (n - 1, n - 2, 1.0), // last key, last segment, its end
    };
    let (AnimValue::Float(a), AnimValue::Float(b)) = (keys[seg].value, keys[seg + 1].value) else {
        return keys[edge].value;
    };
    let span = keys[seg + 1].t.to_seconds() - keys[seg].t.to_seconds();
    let slope = if span > 0.0 {
        keys[seg].interp.value_slope(f64::from(a), f64::from(b), u) / span
    } else {
        0.0
    };
    if !slope.is_finite() {
        return keys[edge].value;
    }
    let v_edge = if edge == seg { a } else { b };
    let t_edge = keys[edge].t.to_seconds();
    AnimValue::Float((f64::from(v_edge) + slope * (t - t_edge)) as f32)
}
