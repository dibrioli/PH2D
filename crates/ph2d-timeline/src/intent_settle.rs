//! [`settle`] — the post-edit invariant restoration for [`crate::intent_apply`].
//!
//! Split from `intent_apply.rs` under the LOC cap, and a unit in its own right: it is the
//! ONE place every invariant an authoring edit can break is re-derived — roving times,
//! strip order, and the motion path's 1:1 pairing with its Position track.

use crate::doc::TimelineDoc;

/// Restore every invariant an edit may have broken. Idempotent, cheap, and the
/// SINGLE place either one is re-derived — which is the whole point: an invariant
/// that any authoring path can break needs exactly one place that fixes it, or it
/// is really N places and one of them will be forgotten.
///
/// 1. **Motion path**: a dope-sheet edit (move/merge/delete/duplicate a Position key)
///    touches the track alone, so the anchor count drifts from the key count — the
///    *"more points on the curve than keys → the curve breaks on drag"* the smoke
///    reported (ADR-0141). A no-op for an in-sync path (no undo churn).
/// 2. **Roving keys**: their time is a function of their neighbours' values, so it
///    is re-derived after every add/move/scale/paste/value/interp/rove.
/// 3. **Strip order**: a lane's strips are sorted by start time, and that is what
///    `ClipLane::weight_at` reads to find the neighbour a crossfade blends with.
///    Drag a strip past its neighbour and the order is stale — the drawn crossfade
///    and the evaluated one would disagree, which is exactly the bug class that
///    "one place" exists to prevent.
pub(crate) fn settle(doc: &mut TimelineDoc) {
    doc.reconcile_position_paths();
    doc.active_clip_mut().resolve_roving();
    for lane in doc.stack_mut() {
        lane.resort();
    }
    // **Every stack, not just the document's** (ADR-0133). "Strips are sorted by start time"
    // is the invariant the evaluator's neighbour logic rests on — `hold_at`, the crossfade,
    // `gap_before` all read a lane in order. A strip dragged inside a CONTAINER would leave
    // that lane unsorted, and the damage would show up as a crossfade against the wrong
    // neighbour, one level down from where anyone is looking.
    for i in 0..doc.containers().len() {
        if let Some(lanes) = doc.container_stack_mut(i) {
            for lane in lanes {
                lane.resort();
            }
        }
    }
}
