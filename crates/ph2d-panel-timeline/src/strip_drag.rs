//! Dragging a clip strip: slide it, or trim an edge (ADR-0115 B3).
//!
//! Shaped like `loop_drag` — a span with two grabbable ends and a body — but it
//! **mutates the document**, so it carries `key_drag`'s undo bracket: one gesture
//! is one `Ctrl+Z`, however many frames of `Update` it spans.
//!
//! **The overlap needs no code.** Nothing here knows what a crossfade is. Two
//! strips whose spans intersect *are* a crossfade, because `ClipLane::blend_in`
//! and `blend_out` derive the blend window from the overlap and the evaluator
//! weights by it. So "drag one strip onto another and the crossfade appears" is
//! not a feature that was implemented — it is a feature that could not be avoided.
//! That is the whole reason the model was chosen (the ADR's §1.3).
//!
//! The deltas apply to the span captured at `Begin`, never to the live one: a
//! drag that reads back its own output drifts, and the arch gate
//! `arch_no_absolute_drag_pattern` exists because this project already paid for it.

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture};
use ph2d_timeline::{StripId, TimelineIntent, TimelineViewSnapshot};

use crate::state::{self, StripDrag, TimelinePanelState};

/// Interpret one strip gesture. `edge`: `0` = start, `1` = end, `2` = body.
pub(crate) fn apply(
    state: &mut TimelinePanelState,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    lane: usize,
    strip: u64,
    edge: u8,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => {
            let Some(s) = find(snap, lane, strip) else {
                return;
            };
            // The bracket opens FIRST: every `Update` below mutates the document,
            // and without it each frame of the drag would be its own undo step.
            state::push_intent(TimelineIntent::BeginEdit);
            state.strip_drag = Some(StripDrag {
                lane,
                id: StripId(strip),
                edge,
                start_x: g.x,
                start_span: (s.t_start, s.t_end),
            });
        }
        GesturePhase::Update | GesturePhase::End => {
            if let Some(d) = state.strip_drag {
                emit(state, px_per_s, snap, d, g.x);
            }
            if matches!(g.phase, GesturePhase::End) {
                state.strip_drag = None;
                state::push_intent(TimelineIntent::EndEdit);
            }
        }
        GesturePhase::Click | GesturePhase::DoubleClick => {
            // A press that never moved. The bracket still closes — having changed
            // nothing, it commits no step (`history::commit_if_changed`).
            if state.strip_drag.take().is_some() {
                state::push_intent(TimelineIntent::EndEdit);
            }
        }
    }
}

/// Turn the cursor's x into the intent this drag means.
fn emit(
    state: &TimelinePanelState,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    d: StripDrag,
    x: f32,
) {
    let dt = f64::from(x - d.start_x) / px_per_s;
    let (a0, b0) = d.start_span;
    let intent = match d.edge {
        // The two edges TRIM: the span's edge and the source slice's edge travel
        // together, so the frames that stay visible stay put (`TrimStrip`). This
        // is not a stretch — stretching is `speed`, and it is a different gesture
        // on purpose.
        0 => TimelineIntent::TrimStrip {
            lane: d.lane,
            id: d.id,
            edge: 0,
            t: snapped(state, snap, a0 + dt),
        },
        1 => TimelineIntent::TrimStrip {
            lane: d.lane,
            id: d.id,
            edge: 1,
            t: snapped(state, snap, b0 + dt),
        },
        // The body SLIDES, rigidly. Clamped at zero so a strip cannot be dragged
        // off the front of the timeline and become unreachable.
        _ => TimelineIntent::MoveStrip {
            lane: d.lane,
            id: d.id,
            t_start: snapped(state, snap, (a0 + dt).max(0.0)),
        },
    };
    let _ = b0;
    state::push_intent(intent);
}

/// Frame-snap, when the panel is snapping. Never negative.
fn snapped(state: &TimelinePanelState, snap: &TimelineViewSnapshot, t: f64) -> f64 {
    let _ = state;
    let t = t.max(0.0);
    if snap.frame_snap && snap.fps > 0.0 {
        (t * snap.fps).round() / snap.fps
    } else {
        t
    }
}

/// The strip this gesture is about, as the snapshot last saw it.
fn find(snap: &TimelineViewSnapshot, lane: usize, strip: u64) -> Option<&ph2d_timeline::StripView> {
    snap.lanes
        .get(lane)?
        .strips
        .iter()
        .find(|s| s.id == StripId(strip))
}

#[cfg(test)]
#[path = "strip_drag_tests.rs"]
mod tests;
