//! Dope-sheet gesture interpretation (W2.E5b). Drains the `TimelineSurface`
//! channel the pointer dispatch fills (Begin/Update/End/Click over key diamonds
//! and empty lanes) and turns it into ephemeral drag state plus the
//! [`TimelineIntent`]s the shell applies (selection + key moves).
//!
//! Coverage: click a diamond → select (Shift = toggle into a multi-selection);
//! click empty lane → clear; **drag empty lane → box-select** (Shift = add to
//! the selection); drag a diamond → live preview, one
//! `MoveSelectedKeys` (frame-snapped) committed at End (a single undo step).
//! Pressing an already-selected key keeps the whole selection so a drag moves
//! the **group** — it only collapses to that key on a plain click (no drag),
//! the standard dope-sheet disambiguation. Delete is handled shell-side against
//! the panel selection (no key channel here).
//!
//! View gestures (E6+): plain wheel = anchored time zoom, Ctrl+wheel = time pan,
//! Shift+wheel = row scroll, **middle-drag = pan both axes** (Blender), and
//! dragging any panel edge/corner resizes it. Right button is reserved.

use ph2d_editor_core::interaction::{BufferAction, GesturePhase, TimelineGesture, TimelineHitKind};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerButton;
use ph2d_timeline::{AnimTarget, TimelineIntent, TimelineViewSnapshot};

use crate::anchor_drag;
use crate::box_select;
use crate::ids;
use crate::key_drag;
use crate::loop_drag;
use crate::marker_drag;
use crate::resize;
use crate::state::TimelinePanelState;
use crate::summary;
use crate::view;

/// Drain this frame's dope-sheet wheel + gestures and raise the resulting
/// intents. Call from `paint` BEFORE the view is resolved, so a zoom/pan/resize
/// lands on the same frame's ruler + diamonds (not one frame late). `rect` is the
/// panel's current rect and `time_x` the left edge of the time area (where
/// `view_start_s` maps to).
pub(crate) fn process(
    state: &mut TimelinePanelState,
    ctx: &mut PaintCtx,
    rect: Rect,
    time_x: f32,
    viewport: Rect,
    snap: &TimelineViewSnapshot,
) {
    // Drop last frame's committed-move preview: by now the shell has applied the
    // move and re-published the snapshot, so the diamonds' base positions already
    // include it (keeping the offset would double it).
    state.pending_move_dx = None;
    if let Some(w) = ctx
        .host
        .store_mut()
        .take_timeline_wheel(ids::TIMELINE_PANEL)
    {
        view::apply_wheel(state, time_x, w);
    }
    // Read the scale AFTER the wheel landed, so a same-frame zoom and the key
    // drag agree on px-per-second.
    let px_per_s = state.px_per_s;
    let gestures: Vec<TimelineGesture> = ctx.host.store_mut().drain_timeline_gestures().collect();
    for g in gestures {
        // Resize grippers own the gesture whatever the button.
        if let TimelineHitKind::ResizeEdge { edges } = g.kind {
            resize::apply_resize(state, rect, viewport, edges, g);
            continue;
        }
        match g.button {
            // Middle-drag pans both axes, anywhere in the dope sheet (Blender).
            PointerButton::Middle => view::apply_pan_drag(state, px_per_s, g),
            PointerButton::Primary => dispatch_primary(state, time_x, px_per_s, snap, g),
            // Secondary is reserved (future context menu).
            PointerButton::Secondary => {}
        }
    }
}

/// Route one Primary-button gesture to the machine that owns its hit kind.
/// Shared with the tests so the lock routing below is exercised by the same code
/// the panel runs.
pub(crate) fn dispatch_primary(
    state: &mut TimelinePanelState,
    time_x: f32,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    g: TimelineGesture,
) {
    match g.kind {
        TimelineHitKind::LoopBrace { edge } => {
            loop_drag::apply(state, time_x, px_per_s, snap, edge, g);
        }
        // The grip at the veil's left edge: drag it to resize the composition
        // duration (scope-appropriate `Set*Length`, snapped like the playhead).
        TimelineHitKind::DurationHandle => {
            crate::duration_drag::apply(state, time_x, px_per_s, snap, g);
        }
        TimelineHitKind::Marker { index } => {
            marker_drag::apply(state, time_x, px_per_s, snap, index, g);
        }
        // A time-scale grip on the key selection's box: scale the selected keys'
        // TIME about the opposite edge. A KEY edit (`ScaleSelectedKeys`) — never a
        // strip stretch, which is a different verb on the stack lane (`Strip`).
        TimelineHitKind::SelectionTimeHandle { right } => {
            crate::scale_drag::apply(state, time_x, px_per_s, snap, right, g);
        }
        // A clip strip: the body slides, the two edges trim. Overlapping two of
        // them IS the crossfade — no code here knows that, and none needs to.
        TimelineHitKind::Strip { lane, strip, edge } => {
            crate::strip_drag::apply(state, px_per_s, snap, lane, strip, edge, g);
        }
        // A lane's LABEL. Deliberately inert on the left button: it exists so a
        // right-click has a surface to open the lane menu on, and a header that
        // also dragged would be a header that swallows a press meant for a strip
        // scrolled underneath it.
        TimelineHitKind::LaneHeader { .. } => {}
        // A container's bar in the Containers list: a double-click walks in, and every other
        // phase is deliberately inert (`container_list::apply`).
        TimelineHitKind::ContainerRow { index } => {
            crate::container_list::apply(state, index, g);
        }
        // Column lock (default): a press on a track key is a press on its whole
        // time column, so grabbing one key grabs the vertical group. Unlocked, it
        // moves alone. Either way, the Summary diamond itself always moves the
        // column (`SummaryKey`, below).
        TimelineHitKind::Key { target, key } => {
            let sel = ph2d_timeline::SelectedKey::new(target, key);
            // Alt-drag OR Ctrl-drag is the Quick-Offset stagger (§3): it cascades
            // the selection instead of moving it rigidly, and bypasses column-lock
            // (staggering a whole aligned column is not the gesture). Plain drag =
            // key move. Ctrl (`cmd`) is here because a Linux compositor (KDE) grabs
            // Alt+left-drag for its window-move gesture, so the app never sees the
            // drag — Ctrl is the WM-safe path and is otherwise free on a diamond.
            if g.mods.alt || g.mods.cmd {
                crate::stagger_drag::apply(state, px_per_s, snap, sel, g);
            } else if let Some(t_bits) = state
                .column_lock
                .then(|| summary::key_t_bits(snap, target, key))
                .flatten()
            {
                summary::apply_gesture(state, px_per_s, snap, t_bits, g);
            } else {
                key_drag::apply_key(state, px_per_s, snap, sel, g);
            }
        }
        TimelineHitKind::SummaryKey { t_bits } => {
            summary::apply_gesture(state, px_per_s, snap, t_bits, g);
        }
        TimelineHitKind::SummaryLock => {
            if matches!(g.phase, GesturePhase::Click) {
                state.column_lock = !state.column_lock;
            }
        }
        TimelineHitKind::Twirl { target } => apply_twirl(state, target, g),
        // A per-band Buffer-Curves chip (§5): a Click stores the track's curve into
        // the A/B buffer, or swaps live <-> buffered. Only a Click acts (a chip has
        // no drag), the twirl's precedent — and it emits the SAME intents the plan's
        // engine already applies, never a strip/lane edit (the fade stays untouched).
        TimelineHitKind::GraphBufferButton { target, action } => {
            if matches!(g.phase, GesturePhase::Click) {
                let target = AnimTarget::new(target);
                crate::state::push_intent(match action {
                    BufferAction::Store => TimelineIntent::StoreTrackBuffer { target },
                    BufferAction::Swap => TimelineIntent::SwapTrackBuffer { target },
                });
            }
        }
        // The row label is a right-click surface only (the track menu opens on
        // the Secondary Down, in dispatch, before any gesture streams). Primary
        // click/drag on a name is deliberately inert — the twirl toggles, the
        // diamonds select; the label itself has no primary affordance.
        TimelineHitKind::Row { .. } => {}
        TimelineHitKind::LabelSplitter => resize::apply_label_drag(state, g),
        TimelineHitKind::GraphResize => resize::apply_graph_resize(state, g),
        TimelineHitKind::CurveAnchor { target, key } => {
            anchor_drag::apply_gesture(state, px_per_s, snap, target, key, g);
        }
        TimelineHitKind::CurveHandle { target, key, which } => {
            crate::graph::apply_handle_gesture(state, target, key, which, g);
        }
        TimelineHitKind::Lane => box_select::apply_lane(state, g),
        TimelineHitKind::ResizeEdge { .. } => {
            unreachable!("resize edges are handled before the button match")
        }
    }
}

/// Twirl gesture: a tap opens/closes that track's graph editor. Only a Click
/// counts, so a drag begun on the twirl by accident leaves the row alone.
fn apply_twirl(state: &mut TimelinePanelState, target: u64, g: TimelineGesture) {
    if matches!(g.phase, GesturePhase::Click) {
        state.toggle_expanded(target);
    }
}

#[cfg(test)]
#[path = "interact_tests.rs"]
mod tests;
