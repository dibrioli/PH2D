//! Marker gestures on the ruler (W4.T3) — a plain click seeks the playhead to
//! the marker, Alt+click removes it, and a drag moves it.
//!
//! Move streams an absolute `MoveMarker` (idempotent — it sets the marker's time,
//! it does not accumulate a delta) inside a `BeginEdit`/`EndEdit` bracket, so the
//! whole drag is one undo step. `apply_intent` frame-snaps the time, so this only
//! maps the pointer to raw seconds. A seek is a `Scrub` (transport, not undoable),
//! so it rides outside the bracket.

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture};
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};

use crate::state::{self, TimelinePanelState};

/// Interpret one marker gesture at storage `index`. `time_x` is the ruler's left
/// edge (where `view_start` maps).
pub(crate) fn apply(
    state: &mut TimelinePanelState,
    time_x: f32,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    index: usize,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => {
            // Open the undo bracket now; a drag's streamed moves fold into it, and
            // a plain click closes it having changed nothing.
            state::push_intent(TimelineIntent::BeginEdit);
            state.marker_drag = Some(index);
        }
        GesturePhase::Update => emit_move(state, time_x, px_per_s, g.x),
        GesturePhase::End => {
            emit_move(state, time_x, px_per_s, g.x);
            state.marker_drag = None;
            state::push_intent(TimelineIntent::EndEdit);
        }
        GesturePhase::Click => {
            state.marker_drag = None;
            if g.mods.alt {
                // Alt+click removes — folds into the open bracket as one step.
                state::push_intent(TimelineIntent::RemoveMarker { index });
                state::push_intent(TimelineIntent::EndEdit);
            } else {
                // A plain click seeks. Close the (empty) bracket first — a seek is
                // a transport intent, not a document edit.
                state::push_intent(TimelineIntent::EndEdit);
                if let Some(&(t, _, _)) = snap.markers.get(index) {
                    state::push_intent(TimelineIntent::Scrub(t));
                }
            }
        }
        GesturePhase::DoubleClick => {
            // Unreachable by contract: a marker no longer opts into double-click
            // (`TimelineHitKind::wants_double_click`, 2026-07-25) — its Rename / Set
            // Signal / Delete moved to the right-click menu (`marker_menu`), so a
            // second tap arrives as a plain Click (it re-seeks). Kept exhaustive and
            // defensive: close the empty bracket the preceding Begin opened, and arm
            // nothing — if the double-click list ever re-adds the marker, this must
            // not resurrect the old inline authoring.
            let _ = index;
            state.marker_drag = None;
            state::push_intent(TimelineIntent::EndEdit);
        }
    }
}

/// Stream the marker to the pointer's time (raw seconds; `apply_intent` snaps).
fn emit_move(state: &mut TimelinePanelState, time_x: f32, px_per_s: f64, x: f32) {
    let Some(index) = state.marker_drag else {
        return;
    };
    if px_per_s <= 0.0 {
        return;
    }
    let t = (state.view_start_s + f64::from(x - time_x) / px_per_s).max(0.0);
    state::push_intent(TimelineIntent::MoveMarker {
        index,
        t_seconds: t,
    });
}

#[cfg(test)]
#[path = "marker_drag_tests.rs"]
mod tests;
