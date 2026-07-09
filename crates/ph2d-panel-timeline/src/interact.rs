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
use ph2d_timeline::{SelectedKey, TimelineIntent, TimelineViewSnapshot};

use crate::box_select;
use crate::ids;
use crate::state::{self, KeyDrag, TimelinePanelState};
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
            view::apply_resize(state, rect, viewport, edges, g);
            continue;
        }
        match g.button {
            // Middle-drag pans both axes, anywhere in the dope sheet (Blender).
            PointerButton::Middle => view::apply_pan_drag(state, px_per_s, g),
            PointerButton::Primary => match g.kind {
                TimelineHitKind::Key { target, key } => {
                    apply_key(state, px_per_s, snap, SelectedKey::new(target, key), g);
                }
                TimelineHitKind::Twirl { target } => apply_twirl(state, target, g),
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

/// Key-diamond gesture machine: Begin selects (or preserves a group) + arms a
/// drag; Update tracks the pointer; End commits a frame-snapped move; a plain
/// Click collapses a preserved group to the pressed key.
fn apply_key(
    state: &mut TimelinePanelState,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    sel: SelectedKey,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => {
            // Shift toggles; a press on an already-selected key keeps the whole
            // selection (so a drag moves the group) and only collapses to this
            // key on a no-drag click; a press on an unselected key selects it.
            let collapse_to = if g.mods.shift {
                state::push_intent(TimelineIntent::ToggleSelect(sel));
                None
            } else if is_selected(snap, sel) {
                Some(sel)
            } else {
                state::push_intent(TimelineIntent::SelectSingle(sel));
                None
            };
            state.key_drag = Some(KeyDrag {
                start_x: g.x,
                cur_x: g.x,
                collapse_to,
            });
        }
        GesturePhase::Update => {
            if let Some(d) = state.key_drag.as_mut() {
                d.cur_x = g.x;
            }
        }
        GesturePhase::End => {
            // End only fires after movement (dispatch chose it over Click), so a
            // drag happened: move the group, never collapse.
            if let Some(d) = state.key_drag.take()
                && let Some(delta) = drag_delta_seconds(&d, px_per_s, snap)
            {
                state::push_intent(TimelineIntent::MoveSelectedKeys {
                    delta_seconds: delta,
                });
                // Hold the offset for this frame so the diamonds stay put while
                // the move round-trips through the shell (avoids a 1-frame snap
                // back to the old position). Cleared next `process`.
                state.pending_move_dx = Some((delta * px_per_s) as f32);
            }
        }
        GesturePhase::Click | GesturePhase::DoubleClick => {
            // A plain click on a preserved group collapses it to the pressed key.
            if let Some(d) = state.key_drag.take()
                && let Some(one) = d.collapse_to
            {
                state::push_intent(TimelineIntent::SelectSingle(one));
            }
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

/// Whether `sel` is currently selected in the published snapshot.
fn is_selected(snap: &TimelineViewSnapshot, sel: SelectedKey) -> bool {
    snap.tracks
        .iter()
        .filter(|t| t.target == sel.target)
        .flat_map(|t| &t.keys)
        .any(|k| k.id == sel.key && k.selected)
}

/// The frame-snapped time delta of a finished key drag, or `None` if it rounds to
/// zero (no move → no undo step). Snaps to whole display frames when the snapshot
/// says frame-snap is on, matching the transport's scrub/AddKey snapping.
fn drag_delta_seconds(d: &KeyDrag, px_per_s: f64, snap: &TimelineViewSnapshot) -> Option<f64> {
    if px_per_s <= 0.0 {
        return None;
    }
    let raw = f64::from(d.cur_x - d.start_x) / px_per_s;
    let delta = if snap.frame_snap && snap.fps > 0.0 {
        (raw * snap.fps).round() / snap.fps
    } else {
        raw
    };
    (delta != 0.0).then_some(delta)
}

/// The live preview offset (px) the selected diamonds ride during a drag —
/// frame-snapped so they visually snap to the grid, matching the commit.
pub(crate) fn preview_dx(
    state: &TimelinePanelState,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
) -> f32 {
    // A live drag wins; otherwise a just-committed move holds its offset for the
    // round-trip frame (see `pending_move_dx`).
    let Some(d) = state.key_drag.as_ref() else {
        return state.pending_move_dx.unwrap_or(0.0);
    };
    match drag_delta_seconds(d, px_per_s, snap) {
        Some(delta) => (delta * px_per_s) as f32,
        None => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::interaction::GestureMods;
    use ph2d_host::PointerButton;

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

    /// Like [`snap`] but with track `target` carrying key `key`, marked selected.
    fn snap_with_selected(target: u64, key: u64) -> TimelineViewSnapshot {
        use ph2d_timeline::{AnimTarget, Interp, KeyId, KeyView, TrackView};
        TimelineViewSnapshot {
            tracks: vec![TrackView {
                target: AnimTarget::new(target),
                prop: ph2d_timeline::PropKind::TranslationX,
                entity: 1,
                missing: false,
                keys: vec![KeyView {
                    id: KeyId::new(key),
                    t_seconds: 0.0,
                    value: 0.0,
                    interp: Interp::Linear,
                    selected: true,
                }],
            }],
            ..snap()
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
                apply_key(state, px_per_s, snap, SelectedKey::new(target, key), g);
            }
            TimelineHitKind::Lane => box_select::apply_lane(state, g),
            TimelineHitKind::Twirl { target } => apply_twirl(state, target, g),
            TimelineHitKind::CurveHandle { .. } | TimelineHitKind::ResizeEdge { .. } => {
                unreachable!("not fed here")
            }
        }
    }

    #[test]
    fn click_a_key_selects_it() {
        let mut st = TimelinePanelState::default();
        let key = TimelineHitKind::Key { target: 9, key: 2 };
        feed(
            &mut st,
            gesture(key, GesturePhase::Begin, 100.0, false),
            120.0,
            &snap(),
        );
        feed(
            &mut st,
            gesture(key, GesturePhase::Click, 100.0, false),
            120.0,
            &snap(),
        );
        assert_eq!(
            state::drain_intents(),
            vec![TimelineIntent::SelectSingle(SelectedKey::new(9, 2))]
        );
        assert!(st.key_drag.is_none(), "a tap leaves no drag armed");
    }

    #[test]
    fn shift_click_toggles_into_the_selection() {
        let mut st = TimelinePanelState::default();
        let key = TimelineHitKind::Key { target: 5, key: 1 };
        feed(
            &mut st,
            gesture(key, GesturePhase::Begin, 40.0, true),
            120.0,
            &snap(),
        );
        assert_eq!(
            state::drain_intents(),
            vec![TimelineIntent::ToggleSelect(SelectedKey::new(5, 1))]
        );
    }

    #[test]
    fn drag_a_key_commits_one_frame_snapped_move() {
        let mut st = TimelinePanelState::default();
        let key = TimelineHitKind::Key { target: 1, key: 0 };
        let s = snap();
        // Begin at x=100 (selects), drag to x=130, release. 30 px / 120 px·s⁻¹ =
        // 0.25 s → 15 frames / 60 fps = 0.25 s (already frame-aligned).
        feed(
            &mut st,
            gesture(key, GesturePhase::Begin, 100.0, false),
            120.0,
            &s,
        );
        feed(
            &mut st,
            gesture(key, GesturePhase::Update, 130.0, false),
            120.0,
            &s,
        );
        feed(
            &mut st,
            gesture(key, GesturePhase::End, 130.0, false),
            120.0,
            &s,
        );
        let got = state::drain_intents();
        assert_eq!(got[0], TimelineIntent::SelectSingle(SelectedKey::new(1, 0)));
        assert_eq!(
            got[1],
            TimelineIntent::MoveSelectedKeys {
                delta_seconds: 0.25
            }
        );
        assert!(st.key_drag.is_none(), "the drag ended");
        // The committed offset is held for the round-trip frame so the diamonds
        // don't snap back to the old position before the move lands.
        assert_eq!(st.pending_move_dx, Some(30.0), "0.25 s × 120 px·s⁻¹");
        assert_eq!(preview_dx(&st, 120.0, &s), 30.0);
    }

    #[test]
    fn a_zero_move_drag_emits_no_move() {
        let mut st = TimelinePanelState::default();
        let key = TimelineHitKind::Key { target: 1, key: 0 };
        let s = snap();
        feed(
            &mut st,
            gesture(key, GesturePhase::Begin, 100.0, false),
            120.0,
            &s,
        );
        // Sub-frame jitter (< 1 px) rounds to zero frames.
        feed(
            &mut st,
            gesture(key, GesturePhase::Update, 100.5, false),
            120.0,
            &s,
        );
        feed(
            &mut st,
            gesture(key, GesturePhase::End, 100.5, false),
            120.0,
            &s,
        );
        assert_eq!(
            state::drain_intents(),
            vec![TimelineIntent::SelectSingle(SelectedKey::new(1, 0))],
            "no MoveSelectedKeys when the snapped delta is zero"
        );
    }

    #[test]
    fn pressing_a_selected_key_preserves_the_group_and_drags_it() {
        // A press on an already-selected key must NOT collapse the selection —
        // dragging moves the whole group (no SelectSingle, just the move).
        let mut st = TimelinePanelState::default();
        let key = TimelineHitKind::Key { target: 7, key: 4 };
        let s = snap_with_selected(7, 4);
        feed(
            &mut st,
            gesture(key, GesturePhase::Begin, 100.0, false),
            120.0,
            &s,
        );
        feed(
            &mut st,
            gesture(key, GesturePhase::Update, 130.0, false),
            120.0,
            &s,
        );
        feed(
            &mut st,
            gesture(key, GesturePhase::End, 130.0, false),
            120.0,
            &s,
        );
        assert_eq!(
            state::drain_intents(),
            vec![TimelineIntent::MoveSelectedKeys {
                delta_seconds: 0.25
            }],
            "no SelectSingle — the multi-selection is preserved and moved"
        );
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

    #[test]
    fn clicking_a_selected_key_without_dragging_collapses_to_it() {
        // A plain click (no drag) on a selected key collapses to just that key.
        let mut st = TimelinePanelState::default();
        let key = TimelineHitKind::Key { target: 7, key: 4 };
        let s = snap_with_selected(7, 4);
        feed(
            &mut st,
            gesture(key, GesturePhase::Begin, 100.0, false),
            120.0,
            &s,
        );
        feed(
            &mut st,
            gesture(key, GesturePhase::Click, 100.0, false),
            120.0,
            &s,
        );
        assert_eq!(
            state::drain_intents(),
            vec![TimelineIntent::SelectSingle(SelectedKey::new(7, 4))],
            "a no-drag click collapses the group to the pressed key"
        );
    }
}
