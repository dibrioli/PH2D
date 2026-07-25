//! **Duration handle drag on the ruler** (Enio, 2026-07-23) — resize the view's
//! composition duration by dragging the ↔ grip at the veil's left edge.
//!
//! The grip's ↔ glyph sits a little to the RIGHT of the authored-duration edge (so
//! it never collides with the loop braces), so the drag is **grab-relative**: the
//! Begin captures how far the pointer is from the edge, and the edge then tracks
//! the pointer minus that offset — grabbing the arrow does NOT jump the duration to
//! the arrow's position. The RESULT is frame-snapped the same way the playhead and
//! the loop brace are (`snap.frame_snap` / `snap.fps`).
//!
//! It streams the SAME scope-appropriate length intent the Dur(s) chip's router
//! picks (`length_scope`: clip on Keys, scene on Arrange, the open container inside
//! one), so typing the box and dragging the veil edit the exact same number.
//!
//! Unlike the loop brace — which drives the `Playhead`, not the document — a length
//! is a DOCUMENT edit ([`crate::intent_loop_sync::apply_length`] runs through
//! `edit`, one undo step per call). So the whole press-to-release sits inside ONE
//! `BeginEdit`/`EndEdit` bracket (the anchor-drag pattern), or a slow drag would
//! leave a Ctrl+Z trail a frame long.

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture};
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};

use crate::state::{self, TimelinePanelState};
use crate::transport::{LengthScope, length_scope};

/// Interpret one duration-handle gesture. `time_x` is the ruler's left edge (where
/// `view_start` maps); the pointer x, minus the grab offset, becomes the new end.
pub(crate) fn apply(
    state: &mut TimelinePanelState,
    time_x: f32,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => {
            // How far the grabbed pointer is from the veil edge (in seconds). The
            // ↔ sits to the RIGHT of the edge, so an absolute map would jump the
            // duration to the arrow on grab; this keeps the edge where it is.
            state.dur_drag =
                Some(raw_time(state, time_x, px_per_s, g.x) - snap.view_length_seconds);
            // Open ONE undo step for the whole drag (a length is a document edit).
            state::push_intent(TimelineIntent::BeginEdit);
        }
        GesturePhase::Update => emit_length(state, time_x, px_per_s, snap, g.x),
        GesturePhase::End => {
            emit_length(state, time_x, px_per_s, snap, g.x);
            state::push_intent(TimelineIntent::EndEdit);
            state.dur_drag = None;
        }
        // A bare grab (no drag): nothing moved, just close the bracket the Begin
        // opened (its `commit_if_changed` records no step). The handle is a grip,
        // not a seek — a click on it does NOT scrub.
        GesturePhase::Click | GesturePhase::DoubleClick => {
            state::push_intent(TimelineIntent::EndEdit);
            state.dur_drag = None;
        }
    }
}

/// Push the scope-appropriate `Set*Length` for the snapped time under pointer `x`
/// (minus the grab offset), never shorter than one frame so the drag can never
/// zero the duration and pull the veil (and its own handle) out from under the
/// pointer — the box's `0` clears it deliberately; a drag resizes.
fn emit_length(
    state: &TimelinePanelState,
    time_x: f32,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    x: f32,
) {
    let off = state.dur_drag.unwrap_or(0.0);
    let t = snap_frame(raw_time(state, time_x, px_per_s, x) - off, snap).max(min_len(snap));
    let len = Some(t);
    // The SAME door the Dur(s) chip router uses (`event.rs`), so dragging the veil
    // and typing the box author the identical scope.
    let intent = match length_scope(snap.container_open, snap.keys_mode) {
        LengthScope::Container(c) => TimelineIntent::SetContainerLength { container: c, len },
        LengthScope::Clip => TimelineIntent::SetClipLength { len },
        LengthScope::Scene => TimelineIntent::SetSceneLength { len },
    };
    state::push_intent(intent);
}

/// The RAW (un-snapped) ruler time under pointer `x`. Snapping happens once, on the
/// grab-corrected value, so the RESULT lands on a frame — not the raw grab.
fn raw_time(state: &TimelinePanelState, time_x: f32, px_per_s: f64, x: f32) -> f64 {
    if px_per_s <= 0.0 {
        return state.view_start_s;
    }
    state.view_start_s + f64::from(x - time_x) / px_per_s
}

/// Frame-snap `t` the same way the playhead and the loop brace do.
fn snap_frame(t: f64, snap: &TimelineViewSnapshot) -> f64 {
    if snap.frame_snap && snap.fps > 0.0 {
        (t * snap.fps).round() / snap.fps
    } else {
        t
    }
}

/// One display frame — the floor a dragged duration never goes below (a hair when
/// fps is unusable, so it can never hit zero).
fn min_len(snap: &TimelineViewSnapshot) -> f64 {
    if snap.fps > 0.0 {
        1.0 / snap.fps
    } else {
        f64::EPSILON
    }
}

#[cfg(test)]
#[path = "duration_drag_tests.rs"]
mod tests;
