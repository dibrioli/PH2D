//! **Duration handle drag on the ruler** (Enio, 2026-07-23) — resize the view's
//! composition duration by dragging the grip at the veil's left edge.
//!
//! The grip sits ON the authored-duration end. A drag maps the pointer straight to
//! a ruler time (the same snapped conversion the loop brace and the playhead use —
//! [`crate::loop_drag::time_at`]) and streams a scope-appropriate length intent:
//! the SAME `length_scope` the Dur(s) chip's router picks (clip on Keys, scene on
//! Arrange, the open container inside one), so typing the box and dragging the veil
//! edit the exact same number.
//!
//! Unlike the loop brace — which drives the `Playhead`, not the document — a
//! length is a DOCUMENT edit ([`crate::intent_loop_sync::apply_length`] runs
//! through `edit`, one undo step per call). So the whole press-to-release sits
//! inside ONE `BeginEdit`/`EndEdit` bracket (the anchor-drag pattern), or a slow
//! drag would leave a Ctrl+Z trail a frame long.

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture};
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};

use crate::state::{self, TimelinePanelState};
use crate::transport::{LengthScope, length_scope};

/// Interpret one duration-handle gesture. `time_x` is the ruler's left edge (where
/// `view_start` maps); the pointer x becomes the new authored end.
pub(crate) fn apply(
    state: &mut TimelinePanelState,
    time_x: f32,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    g: TimelineGesture,
) {
    match g.phase {
        // Open ONE undo step for the whole drag (a length edit is a document edit).
        GesturePhase::Begin => state::push_intent(TimelineIntent::BeginEdit),
        GesturePhase::Update => emit_length(state, time_x, px_per_s, snap, g.x),
        GesturePhase::End => {
            emit_length(state, time_x, px_per_s, snap, g.x);
            state::push_intent(TimelineIntent::EndEdit);
        }
        // A bare grab (no drag): nothing moved, just close the bracket the Begin
        // opened (its `commit_if_changed` records no step). The handle is a grip,
        // not a seek — a click on it does NOT scrub.
        GesturePhase::Click | GesturePhase::DoubleClick => {
            state::push_intent(TimelineIntent::EndEdit);
        }
    }
}

/// Push the scope-appropriate `Set*Length` for the snapped time under pointer `x`,
/// never shorter than one frame so the drag can never zero the duration and pull
/// the veil (and its own handle) out from under the pointer — the box's `0`
/// clears it deliberately; a drag resizes.
fn emit_length(
    state: &TimelinePanelState,
    time_x: f32,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    x: f32,
) {
    let t =
        crate::loop_drag::time_at(state.view_start_s, time_x, px_per_s, x, snap).max(min_len(snap));
    let len = Some(t);
    // The SAME door the Dur(s) chip router uses (`event.rs`), so dragging the veil
    // and typing the box author the identical scope.
    let intent = match length_scope(snap.container_open, state.tab) {
        LengthScope::Container(c) => TimelineIntent::SetContainerLength { container: c, len },
        LengthScope::Clip => TimelineIntent::SetClipLength { len },
        LengthScope::Scene => TimelineIntent::SetSceneLength { len },
    };
    state::push_intent(intent);
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
