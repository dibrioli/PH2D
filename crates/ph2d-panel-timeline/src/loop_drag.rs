//! Loop-brace drag on the ruler (W4.T3) — resize or move the playback loop
//! range by dragging its two braces (or the band between them).
//!
//! The time axis is fully known here (`time_x` + `px_per_s` + `view_start`), so —
//! unlike the graph handle/anchor drags — there is no two-phase resolve: the
//! gesture maps the pointer straight to a time and streams a `SetLoop`. That
//! intent drives the `Playhead`, not the document, so it is not undoable and
//! needs no bracket.

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture};
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};

use crate::state::{self, LoopDrag, TimelinePanelState};

/// Interpret one loop-brace gesture. `edge` is `0` = start, `1` = end, `2` =
/// body (move both). `time_x` is the ruler's left edge (where `view_start` maps).
pub(crate) fn apply(
    state: &mut TimelinePanelState,
    time_x: f32,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    edge: u8,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => {
            state.loop_drag = Some(LoopDrag {
                edge,
                start_x: g.x,
                // No range yet? Start one at the playhead so the first drag has
                // something to pull — the brace is only ever drawn (and thus
                // grabbable) when a range exists, so in practice this is Some.
                start_range: snap
                    .loop_range
                    .unwrap_or((snap.time_seconds, snap.time_seconds)),
            });
        }
        GesturePhase::Update | GesturePhase::End => {
            if let Some(d) = state.loop_drag {
                let range = resolve(&d, state.view_start_s, time_x, px_per_s, g.x, snap);
                state::push_intent(TimelineIntent::SetLoop(Some(range)));
            }
            if matches!(g.phase, GesturePhase::End) {
                state.loop_drag = None;
            }
        }
        GesturePhase::Click | GesturePhase::DoubleClick => {
            // A plain click on the BODY band scrubs the playhead to that time (the
            // range only slides on an actual drag), so clicking inside the loop
            // behaves like clicking the ruler. A click on a brace handle does
            // nothing — it is a grab target, not a seek.
            if state.loop_drag.is_some_and(|d| d.edge == 2) {
                let t = time_at(state.view_start_s, time_x, px_per_s, g.x, snap);
                state::push_intent(TimelineIntent::Scrub(t));
            }
            state.loop_drag = None;
        }
    }
}

/// The `(start, end)` the drag produces, keeping `start + one frame <= end` and
/// `start >= 0`. `edge` 0 moves the start, 1 the end, 2 both rigidly.
fn resolve(
    d: &LoopDrag,
    view_start: f64,
    time_x: f32,
    px_per_s: f64,
    x: f32,
    snap: &TimelineViewSnapshot,
) -> (f64, f64) {
    let (a0, b0) = d.start_range;
    let min_len = min_len(snap);
    let at = |px: f32| time_at(view_start, time_x, px_per_s, px, snap);
    match d.edge {
        // Start handle: cannot cross the end (leave at least one frame), nor go
        // below zero.
        0 => (at(x).clamp(0.0, b0 - min_len), b0),
        // End handle: cannot cross the start.
        1 => (a0, at(x).max(a0 + min_len)),
        // Body: rigid translation, clamped so the start stays at or after zero
        // (the length is preserved).
        _ => {
            let raw = at(x) - at(d.start_x);
            let dt = raw.max(-a0);
            (a0 + dt, b0 + dt)
        }
    }
}

/// One display frame, the minimum loop length (falls back to a hair when fps is
/// unusable so the range never inverts).
fn min_len(snap: &TimelineViewSnapshot) -> f64 {
    if snap.fps > 0.0 {
        1.0 / snap.fps
    } else {
        f64::EPSILON
    }
}

/// The time under pointer x, frame-snapped when the snapshot asks for it, never
/// negative.
fn time_at(
    view_start: f64,
    time_x: f32,
    px_per_s: f64,
    x: f32,
    snap: &TimelineViewSnapshot,
) -> f64 {
    if px_per_s <= 0.0 {
        return view_start.max(0.0);
    }
    let t = view_start + f64::from(x - time_x) / px_per_s;
    let snapped = if snap.frame_snap && snap.fps > 0.0 {
        (t * snap.fps).round() / snap.fps
    } else {
        t
    };
    snapped.max(0.0)
}

#[cfg(test)]
#[path = "loop_drag_tests.rs"]
mod tests;
