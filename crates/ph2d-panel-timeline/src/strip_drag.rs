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
use ph2d_tokens::ROW_H_PX;

use crate::state::{self, StripDrag, TimelinePanelState};

/// Interpret one strip gesture. `edge`: `0`/`1` = trim start/end (red bottom corners),
/// `2` = body (slide), `3`/`4` = fade-in/out grips, `5`/`6` = stretch start/end (green
/// top corners).
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
                start_lane: lane,
                start_y: g.y,
                id: StripId(strip),
                edge,
                start_x: g.x,
                start_span: (s.t_start, s.t_end),
                // The fade the panel DREW at this edge, as a SIGNED offset from the start
                // corner: `+blend_in` when the fade-in reaches INWARD, `-lead_in` when it
                // reaches OUTWARD into the gap (the two are exclusive). Reading the drawn
                // numbers (not the strip's raw `ease_*`) is what keeps the handle and the
                // wedge the same object: the grip starts where the wedge tip is, so it
                // cannot jump on the first pixel of the drag.
                start_ease: match edge {
                    EASE_IN => s.blend_in - s.lead_in,
                    EASE_OUT => s.blend_out - s.lead_out,
                    _ => 0.0,
                },
            });
        }
        GesturePhase::Update | GesturePhase::End => {
            if let Some(d) = state.strip_drag {
                emit(state, px_per_s, snap, d, g.x, g.y);
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
    state: &mut TimelinePanelState,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    d: StripDrag,
    x: f32,
    y: f32,
) {
    let dt = f64::from(x - d.start_x) / px_per_s;
    let (a0, b0) = d.start_span;
    let intent = match d.edge {
        // **The RED bottom corners TRIM** (Enio, 2026-07-16): the span's edge and the
        // source slice's edge travel together, so the frames that stay visible stay put
        // (`TrimStrip`). Trim and stretch used to share an edge behind a Cmd modifier;
        // now each is its own corner — the gizmo you grab IS the operation, no modifier
        // to remember and no NLE-specific convention to learn.
        0 | 1 => {
            let (from, t) = edge_move(state, snap, d, dt, d.edge);
            TimelineIntent::TrimStrip {
                lane: d.lane,
                id: d.id,
                edge: d.edge, // 0 = start, 1 = end — the document's edge vocabulary
                t,
                from,
            }
        }
        // **The GREEN top corners STRETCH** — retime: the slice is pinned and the rate
        // falls out of the new span (`StretchStrip`). The strip does not change how it
        // LOOKS; the label carries the new speed factor.
        STRETCH_START | STRETCH_END => {
            let edge = u8::from(d.edge == STRETCH_END); // 0 = start, 1 = end
            let (from, t) = edge_move(state, snap, d, dt, edge);
            TimelineIntent::StretchStrip {
                lane: d.lane,
                id: d.id,
                edge,
                t,
                from,
            }
        }
        // **The fade grips** (B4). The handle rides the tip of the wedge, so the fade
        // grows as the tip travels INWARD: right at the start, left at the end. The
        // number authored is a LENGTH, but what gets snapped is the tip's TIME — snap
        // the length and a fade would land on the frame grid only when the strip
        // happened to start on it.
        //
        // The panel never offers these grips where a neighbour defines the window
        // (`ease_locked_*`), so a drag that reaches here is always about a fade the
        // strip owns. The apply clamps to `[0, span]` anyway: the document is not a
        // place to trust the caller.
        // Both fade grips PIVOT on their edge (Enio: start 2026-07-16, end 2026-07-19). The
        // tip inside the box grows the inward ease (blend against the clip PLAYING); the tip
        // dragged OUT into the gap grows the outward lead (blend against the clip's FROZEN
        // edge frame — the travel across the gap). One handle, two intents, the edge is the
        // pivot; `SetStripEase`/`SetStripLead` carry the same `edge` (0 = start, 1 = end).
        EASE_OUT => {
            let tip = snapped(state, snap, b0 - (d.start_ease - dt));
            let extent = b0 - tip; // inward is positive (tip left of the end edge)
            if extent >= 0.0 {
                TimelineIntent::SetStripEase {
                    lane: d.lane,
                    id: d.id,
                    edge: 1, // the document's edge vocabulary — same 0/1 as Trim and Stretch
                    seconds: extent,
                }
            } else {
                TimelineIntent::SetStripLead {
                    lane: d.lane,
                    id: d.id,
                    edge: 1,
                    seconds: -extent, // the tip dragged into the gap AFTER: the outward length
                }
            }
        }
        EASE_IN => {
            let tip = snapped(state, snap, a0 + (d.start_ease + dt));
            let extent = tip - a0;
            if extent >= 0.0 {
                TimelineIntent::SetStripEase {
                    lane: d.lane,
                    id: d.id,
                    edge: 0,
                    seconds: extent,
                }
            } else {
                TimelineIntent::SetStripLead {
                    lane: d.lane,
                    id: d.id,
                    edge: 0,
                    seconds: -extent, // the tip dragged into the gap: the outward length
                }
            }
        }
        // **The body SLIDES in BOTH axes**: rigidly in time, and across lanes.
        //
        // Vertically it was pinned, which meant the only way a strip could reach another
        // lane was to be deleted and re-added — and a container, which could not be placed
        // at all, had nowhere to be dragged to (Enio, 2026-07-21). Clamped at zero in time so
        // a strip cannot be dragged off the front of the timeline and become unreachable, and
        // clamped to the lanes that EXIST vertically for the same reason.
        _ => {
            let to_lane = lane_at(d, y, snap.lanes.len());
            let intent = TimelineIntent::MoveStrip {
                lane: d.lane,
                to_lane,
                id: d.id,
                t_start: snapped(state, snap, (a0 + dt).max(0.0)),
            };
            // The drag now holds the lane the strip is ON, not the one it came from: the
            // NEXT frame's intent has to name where the document has already put it.
            if let Some(cur) = state.strip_drag.as_mut() {
                cur.lane = to_lane;
            }
            intent
        }
    };
    state::push_intent(intent);
}

/// **Which lane the pointer has carried the strip to** — a whole number of rows from where the
/// drag began.
///
/// Every lane row is exactly one [`ROW_H_PX`] tall (`geom::stack_bands` lays them out from
/// that constant), so crossing rows is arithmetic on the pointer's travel and needs no
/// geometry threaded down here. Rounding means the strip changes lane at the halfway point,
/// which is where the pointer visually enters the next row.
///
/// Measured from `start_lane`/`start_y` — the ABSOLUTE pair — never accumulated per frame: a
/// drag that reads back its own output drifts (`arch_no_absolute_drag_pattern`).
fn lane_at(d: StripDrag, y: f32, lanes: usize) -> usize {
    if lanes == 0 {
        return d.lane;
    }
    let rows = ((y - d.start_y) / ROW_H_PX).round();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "rows is a small round number of lane heights"
    )]
    let rows = rows as i64;
    let target = i64::try_from(d.start_lane).unwrap_or(0) + rows;
    let last = i64::try_from(lanes - 1).unwrap_or(0);
    usize::try_from(target.clamp(0, last)).unwrap_or(0) // CLAMP-OK: measured bounds, min<=max
}

/// The panel's grip code for a strip's fade-in (mirrors `stack_lane_paint`).
const EASE_IN: u8 = 3;
/// …and for its fade-out.
const EASE_OUT: u8 = 4;
/// The GREEN top-left corner — stretch the start edge (retime, no cut).
const STRETCH_START: u8 = 5;
/// The GREEN top-right corner — stretch the end edge.
const STRETCH_END: u8 = 6;

/// Where this edge started the gesture, and where the pointer has taken it —
/// the pair every edge-moving intent carries.
///
/// They come from the SAME captured span, which is what makes the change bar honest:
/// the anchor is the position the drag began at, not the position of the previous
/// frame, so every frame of the drag reports the same total travel.
fn edge_move(
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
    d: StripDrag,
    dt: f64,
    edge: u8,
) -> (f64, f64) {
    let (a0, b0) = d.start_span;
    let from = if edge == 0 { a0 } else { b0 };
    (from, snapped(state, snap, from + dt))
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
