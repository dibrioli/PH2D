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

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture, TimelineHitKind};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerButton;
use ph2d_timeline::TimelineViewSnapshot;

use crate::anchor_drag;
use crate::box_select;
use crate::ids;
use crate::key_drag;
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
            PointerButton::Primary => match g.kind {
                TimelineHitKind::Key { target, key } => {
                    key_drag::apply_key(
                        state,
                        px_per_s,
                        snap,
                        ph2d_timeline::SelectedKey::new(target, key),
                        g,
                    );
                }
                TimelineHitKind::SummaryKey { t_bits } => {
                    summary::apply_gesture(state, px_per_s, snap, t_bits, g);
                }
                TimelineHitKind::Twirl { target } => apply_twirl(state, target, g),
                TimelineHitKind::LabelSplitter => resize::apply_label_drag(state, g),
                TimelineHitKind::GraphResize => resize::apply_graph_resize(state, g),
                TimelineHitKind::CurveAnchor { target, key } => {
                    anchor_drag::apply_gesture(state, px_per_s, snap, target, key, g);
                }
                TimelineHitKind::CurveHandle { target, key, which } => {
                    crate::graph::apply_handle_gesture(state, target, key, which, g);
                }
                TimelineHitKind::Lane => box_select::apply_lane(state, g),
                TimelineHitKind::ResizeEdge { .. } => unreachable!("handled above"),
            },
            // Secondary is reserved (future context menu).
            PointerButton::Secondary => {}
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
mod tests {
    use super::*;
    use crate::state;
    use ph2d_editor_core::interaction::GestureMods;
    use ph2d_host::PointerButton;
    use ph2d_timeline::TimelineIntent;

    const SURFACE: ph2d_a11y::NodeId = ph2d_a11y::NodeId(0);

    fn gesture(kind: TimelineHitKind, phase: GesturePhase, x: f32, shift: bool) -> TimelineGesture {
        gesture_at(kind, phase, x, 0.0, shift)
    }

    fn gesture_at(
        kind: TimelineHitKind,
        phase: GesturePhase,
        x: f32,
        y: f32,
        shift: bool,
    ) -> TimelineGesture {
        TimelineGesture {
            surface: SURFACE,
            kind,
            phase,
            x,
            y,
            button: PointerButton::Primary,
            mods: GestureMods {
                shift,
                cmd: false,
                alt: false,
            },
        }
    }

    /// 60 fps, frame-snap on, 120 px/s — one frame = 2 px.
    fn snap() -> TimelineViewSnapshot {
        TimelineViewSnapshot {
            fps: 60.0,
            frame_snap: true,
            ..TimelineViewSnapshot::default()
        }
    }

    // Run the machine directly (bypassing the store drain, which needs a host).
    fn feed(
        state: &mut TimelinePanelState,
        g: TimelineGesture,
        px_per_s: f64,
        snap: &TimelineViewSnapshot,
    ) {
        match g.kind {
            TimelineHitKind::Key { target, key } => {
                key_drag::apply_key(
                    state,
                    px_per_s,
                    snap,
                    ph2d_timeline::SelectedKey::new(target, key),
                    g,
                );
            }
            TimelineHitKind::Lane => box_select::apply_lane(state, g),
            TimelineHitKind::Twirl { target } => apply_twirl(state, target, g),
            TimelineHitKind::SummaryKey { t_bits } => {
                summary::apply_gesture(state, px_per_s, snap, t_bits, g);
            }
            TimelineHitKind::LabelSplitter
            | TimelineHitKind::GraphResize
            | TimelineHitKind::CurveAnchor { .. }
            | TimelineHitKind::CurveHandle { .. }
            | TimelineHitKind::ResizeEdge { .. } => unreachable!("not fed here"),
        }
    }

    #[test]
    fn a_twirl_click_toggles_the_tracks_graph_editor() {
        let mut st = TimelinePanelState::default();
        let twirl = TimelineHitKind::Twirl { target: 3 };
        assert!(!st.is_expanded(3));
        feed(
            &mut st,
            gesture(twirl, GesturePhase::Click, 0.0, false),
            120.0,
            &snap(),
        );
        assert!(st.is_expanded(3), "the row opened");
        feed(
            &mut st,
            gesture(twirl, GesturePhase::Click, 0.0, false),
            120.0,
            &snap(),
        );
        assert!(!st.is_expanded(3), "and closed again");
        assert_eq!(
            state::drain_intents(),
            vec![],
            "expansion is view state, not an edit"
        );
    }

    #[test]
    fn pressing_a_twirl_without_releasing_leaves_the_row_alone() {
        // Only a Click counts: an accidental drag from the twirl must not toggle.
        let mut st = TimelinePanelState::default();
        let twirl = TimelineHitKind::Twirl { target: 3 };
        feed(
            &mut st,
            gesture(twirl, GesturePhase::Begin, 0.0, false),
            120.0,
            &snap(),
        );
        feed(
            &mut st,
            gesture(twirl, GesturePhase::End, 40.0, false),
            120.0,
            &snap(),
        );
        assert!(!st.is_expanded(3));
    }

    #[test]
    fn collapsing_a_row_mid_drag_closes_the_handles_undo_bracket() {
        // The band is about to stop existing, so `resolve_drag` will never fire
        // again — leaving the bracket open would swallow the next atomic edit.
        let mut st = TimelinePanelState::default();
        st.toggle_expanded(3);
        st.handle_drag = Some(crate::state::HandleDrag {
            target: 3,
            key: 1,
            which: 0,
            x: 0.0,
            y: 0.0,
            range: None,
            ending: false,
        });
        let _ = state::drain_intents();
        st.toggle_expanded(3);
        assert!(st.handle_drag.is_none());
        assert_eq!(state::drain_intents(), vec![TimelineIntent::EndEdit]);
    }
}
