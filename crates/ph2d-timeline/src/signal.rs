//! Timeline signals — the crossing law (ADR-0143).
//!
//! A marker can carry a named signal ([`Marker::signal`](crate::doc::Marker)). During
//! FORWARD play the signal emits exactly once when the playhead's advance crosses the
//! marker. The emission is a function of the playhead's PATH across a dispatch — the
//! interval `(prev, now]` — **never** of frame equality (`marker.t == frame`). Frame
//! equality is the Godot Call Method Track bug: it misses a marker that falls between
//! two ticks (catch-up) and double-fires on a paused boundary. The path law is the one
//! this workspace has learned N times (relief, bite, smear, the protection gate): *the
//! law is a function of the path, never of how finely the engine sampled it.*
//!
//! This module only answers *"which signals did the play cross?"* The timeline EMITS
//! those names as a decoupled event (ADR-0075) and never calls a consumer — the bridge
//! turns the answer into `TimelineSignal` events and any system drains them.

use crate::doc::Marker;

/// A signal the timeline emitted because forward play crossed a marker carrying it
/// ([ADR-0143]). A **decoupled** event: the timeline fills these and never calls a
/// consumer (ADR-0075); any system drains them by matching `name`. The audio cue, a
/// gameplay reaction and a Luau callback are all just consumers of this — none of them
/// is `ph2d-timeline`'s concern.
///
/// [ADR-0143]: ../../../docs/architecture/decisions/0143-timeline-signals-a-marker-emits-a-decoupled-event-not-a-call.md
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSignal {
    /// The name the marker carried — the contract a consumer matches on.
    pub name: String,
    /// The scene time (seconds) the play was at when it crossed the marker.
    pub t: f64,
}

/// The signals crossed as forward play advanced from `prev` to `now` (scene seconds),
/// in the order the playhead reached them.
///
/// **Forward play only** — the caller (the bridge) invokes this only while playing
/// forward; scrub, reverse and pause never reach here (ADR-0143 §3). The interval is
/// **half-open `(prev, now]`**: a marker at `t` fires when `prev < t <= now`, so it
/// fires on the tick that reaches it and never again on the next tick (when `t` becomes
/// `<= prev`).
///
/// A **loop wrap** is signalled by `now < prev` together with an active
/// `loop_range = (lo, hi)`: the sweep went `prev -> hi` then `lo -> now`, so the crossed
/// set is `(prev, hi] ∪ [lo, now]` — the `lo` end is CLOSED so a marker exactly at the
/// loop start fires once each cycle, while `prev`/`now` stay open/closed so nothing
/// double-fires across ticks. `now < prev` with no loop is a backward jump, not a
/// forward crossing, and returns empty.
///
/// ⚠️ **Known limit (deliberate, not faked):** a single dispatch that advances *more*
/// than one whole loop length (a tiny loop, or a huge stall in one frame) under-fires
/// the lapped middle. From the post-wrap `(prev, now)` the true lap count is not
/// recoverable — the bridge would have to pass the raw advance. This is one frame
/// longer than the entire loop; rather than fake-handle it with a branch that can never
/// be reached from these two numbers, it is named here.
#[must_use]
pub fn signals_crossed<'a>(
    markers: &'a [Marker],
    prev: f64,
    now: f64,
    loop_range: Option<(f64, f64)>,
) -> Vec<&'a str> {
    if now >= prev {
        return crossed_in(markers, prev, now, false);
    }
    // now < prev: within forward play, only a loop wrap gets here.
    let Some((lo, hi)) = loop_range else {
        return Vec::new();
    };
    let mut out = crossed_in(markers, prev, hi, false); // (prev, hi]
    out.extend(crossed_in(markers, lo, now, true)); // [lo, now]
    out
}

/// Signalled markers with `t` in `(a, b]` (or `[a, b]` when `closed_low`), sorted
/// ascending by time. Markers are stored by insertion order; a signal fires in
/// chronological order, so we sort here.
fn crossed_in<'a>(markers: &'a [Marker], a: f64, b: f64, closed_low: bool) -> Vec<&'a str> {
    let mut hits: Vec<(f64, &'a str)> = markers
        .iter()
        .filter_map(|m| {
            let sig = m.signal.as_deref()?;
            let t = m.t.to_seconds();
            let in_low = if closed_low { t >= a } else { t > a };
            (in_low && t <= b).then_some((t, sig))
        })
        .collect();
    hits.sort_by(|x, y| x.0.total_cmp(&y.0));
    hits.into_iter().map(|(_, s)| s).collect()
}
