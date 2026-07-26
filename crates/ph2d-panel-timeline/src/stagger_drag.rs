//! The Quick-Offset stagger gesture (§3 crown jewel) — **Alt-drag OR Ctrl-drag**
//! a selected key to CASCADE the whole selection: each track is shifted by
//! `rank · step` (Ctrl is the WM-safe trigger — a Linux compositor like KDE grabs
//! Alt+left-drag for window-move, so the app never sees an Alt key-drag; routed in
//! `interact`), where `step` is the drag distance and `rank` is the track's stable
//! position (the apply resolves rank; see [`TimelineIntent::StaggerSelectedKeys`]).
//! The first track stays; each later one offsets more — the motion-graphics cascade.
//!
//! It rides on a KEY hit, distinguished from the plain key-move purely by the
//! Alt/Ctrl modifier (routed in `interact`). Like the key drag and the time-scale, it
//! streams one incremental [`StaggerSelectedKeys`] per frame inside a single
//! `BeginEdit`/`EndEdit` bracket, so the cascade tracks the cursor and undoes in
//! one step. A constant rank makes the per-frame steps compose to the total.
//!
//! ⚠️ This is a KEY edit and ONLY a key edit — it emits `StaggerSelectedKeys`
//! (which moves keys via `move_keys`), NEVER a strip/lane/container intent, so it
//! can never touch the precious Clips/Strips/Fade system.
//!
//! There is no diamond preview offset (unlike the key drag's `pending_move_dx`):
//! the cascade moves each track by a DIFFERENT amount, which one uniform offset
//! cannot represent, so the diamonds catch up on the next frame's snapshot.

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture};
use ph2d_timeline::{SelectedKey, TimelineIntent, TimelineViewSnapshot};

use crate::key_drag::{drag_delta_seconds, is_selected};
use crate::state::{self, StaggerDrag, TimelinePanelState};

/// Quick-Offset gesture machine: Begin preserves an existing selection (so the
/// cascade spans it) or selects the pressed key, and arms the drag; every Update
/// emits the per-rank step that accrued since the last frame; a plain Click
/// collapses a preserved group to the pressed key.
pub(crate) fn apply(
    state: &mut TimelinePanelState,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    sel: SelectedKey,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => {
            state::push_intent(TimelineIntent::BeginEdit);
            // Same select disambiguation as the key drag: preserve a multi-selection
            // (so the cascade has more than one track to span) and collapse to the
            // pressed key only on a no-drag click; select a fresh key.
            let collapse_to = if g.mods.shift {
                state::push_intent(TimelineIntent::ToggleSelect(sel));
                None
            } else if is_selected(snap, sel) {
                Some(sel)
            } else {
                state::push_intent(TimelineIntent::SelectSingle(sel));
                None
            };
            state.stagger_drag = Some(StaggerDrag {
                start_x: g.x,
                cur_x: g.x,
                collapse_to,
                applied_step_s: 0.0,
            });
        }
        GesturePhase::Update => {
            if let Some(d) = state.stagger_drag.as_mut() {
                d.cur_x = g.x;
            }
            emit_stagger(state, px_per_s, snap);
        }
        GesturePhase::End => {
            if let Some(d) = state.stagger_drag.as_mut() {
                d.cur_x = g.x;
            }
            emit_stagger(state, px_per_s, snap);
            state.stagger_drag = None;
            state::push_intent(TimelineIntent::EndEdit);
        }
        GesturePhase::Click | GesturePhase::DoubleClick => {
            // A plain Alt-click (no drag) collapses a preserved group to the
            // pressed key; the bracket closes having changed nothing.
            if let Some(d) = state.stagger_drag.take()
                && let Some(one) = d.collapse_to
            {
                state::push_intent(TimelineIntent::SelectSingle(one));
            }
            state::push_intent(TimelineIntent::EndEdit);
        }
    }
}

/// Emit the per-rank step that accrued since the last frame, so the cascade
/// tracks the pointer instead of jumping on release. The step is the same
/// frame-snapped drag delta the key move uses — track `k` then shifts by
/// `k · step` in the apply, and successive increments compose (rank constant).
fn emit_stagger(state: &mut TimelinePanelState, px_per_s: f64, snap: &TimelineViewSnapshot) {
    let Some(d) = state.stagger_drag else {
        return;
    };
    let want = drag_delta_seconds(d.start_x, d.cur_x, px_per_s, snap);
    let step = want - d.applied_step_s;
    if step == 0.0 {
        return;
    }
    if let Some(d) = state.stagger_drag.as_mut() {
        d.applied_step_s = want;
    }
    state::push_intent(TimelineIntent::StaggerSelectedKeys { step_seconds: step });
}

#[cfg(test)]
#[path = "stagger_drag_tests.rs"]
mod tests;
