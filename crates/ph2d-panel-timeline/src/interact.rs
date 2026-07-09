//! Dope-sheet gesture interpretation (W2.E5b). Drains the `TimelineSurface`
//! channel the pointer dispatch fills (Begin/Update/End/Click over key diamonds
//! and empty lanes) and turns it into ephemeral drag state plus the
//! [`TimelineIntent`]s the shell applies (selection + key moves).
//!
//! Coverage: click a diamond → select (Shift = toggle into a multi-selection);
//! click empty lane → clear; drag a diamond → live preview, one
//! `MoveSelectedKeys` (frame-snapped) committed at End (a single undo step).
//! Pressing an already-selected key keeps the whole selection so a drag moves
//! the **group** — it only collapses to that key on a plain click (no drag),
//! the standard dope-sheet disambiguation. Delete is handled shell-side against
//! the panel selection (no key channel here). Right/middle buttons are ignored
//! (reserved for future menu/pan).

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture, TimelineHitKind, TimelineZoom};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_host::PointerButton;
use ph2d_timeline::{SelectedKey, TimelineIntent, TimelineViewSnapshot};

use crate::ids;
use crate::state::{self, KeyDrag, MAX_PX_PER_S, MIN_PX_PER_S, TimelinePanelState};

/// Wheel **pixels** per e-fold of zoom. The shell delivers line-deltas already
/// scaled to logical px (16 px per notch), so one notch is ~7% zoom here — the
/// same sensitivity the motion graph uses.
const ZOOM_WHEEL_DIV: f64 = 240.0; // LITERAL-PX-OK: wheel px → zoom-factor sensitivity divisor

/// Drain this frame's dope-sheet wheel + gestures and raise the resulting
/// intents. Call from `paint` BEFORE the view is resolved, so a zoom/pan lands
/// on the same frame's ruler + diamonds (not one frame late). `time_x` is the
/// left edge of the time area (where `view_start_s` maps to).
pub(crate) fn process(
    state: &mut TimelinePanelState,
    ctx: &mut PaintCtx,
    time_x: f32,
    snap: &TimelineViewSnapshot,
) {
    // Drop last frame's committed-move preview: by now the shell has applied the
    // move and re-published the snapshot, so the diamonds' base positions already
    // include it (keeping the offset would double it).
    state.pending_move_dx = None;
    if let Some(z) = ctx.host.store_mut().take_timeline_zoom(ids::TIMELINE_PANEL) {
        apply_wheel(state, time_x, z);
    }
    // Read the scale AFTER the wheel landed, so a same-frame zoom and the key
    // drag agree on px-per-second.
    let px_per_s = state.px_per_s;
    let gestures: Vec<TimelineGesture> = ctx.host.store_mut().drain_timeline_gestures().collect();
    for g in gestures {
        // Primary button only; Secondary/Middle are reserved (context menu / pan).
        if g.button != PointerButton::Primary {
            continue;
        }
        match g.kind {
            TimelineHitKind::Key { target, key } => {
                apply_key(state, px_per_s, snap, SelectedKey::new(target, key), g);
            }
            TimelineHitKind::Lane => apply_lane(state, g),
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

/// Apply one frame's accumulated wheel to the time-axis view: pan slides
/// `view_start_s`; zoom scales `px_per_s` about the cursor, holding the time
/// under `anchor_x` fixed. `view_start_s` never goes negative (t=0 is the left
/// bound of the clip).
fn apply_wheel(state: &mut TimelinePanelState, time_x: f32, z: TimelineZoom) {
    // Pan first, in the pre-zoom scale (what the user saw when they scrolled).
    // `pan_delta` is already in logical px; a positive delta scrolls the content
    // right ⇒ the view moves EARLIER, the same sign convention as the
    // panel-scroll path (`panel_scroll - delta_y`).
    if z.pan_delta != 0.0 {
        state.view_start_s -= f64::from(z.pan_delta) / state.px_per_s;
    }
    if z.zoom_delta != 0.0 {
        let old = state.px_per_s;
        let new = (old * (f64::from(z.zoom_delta) / ZOOM_WHEEL_DIV).exp())
            .clamp(MIN_PX_PER_S, MAX_PX_PER_S); // CLAMP-OK: const bounds, min<max, non-NaN
        // Hold the time under the cursor: t = start + (anchor - time_x)/px_per_s.
        let off_px = f64::from(z.anchor_x - time_x);
        let t_anchor = state.view_start_s + off_px / old;
        state.view_start_s = t_anchor - off_px / new;
        state.px_per_s = new;
    }
    state.view_start_s = state.view_start_s.max(0.0);
}

/// Whether `sel` is currently selected in the published snapshot.
fn is_selected(snap: &TimelineViewSnapshot, sel: SelectedKey) -> bool {
    snap.tracks
        .iter()
        .filter(|t| t.target == sel.target)
        .flat_map(|t| &t.keys)
        .any(|k| k.id == sel.key && k.selected)
}

/// Empty-lane gesture: a click (tap) clears the selection.
fn apply_lane(state: &mut TimelinePanelState, g: TimelineGesture) {
    match g.phase {
        GesturePhase::Begin | GesturePhase::Update => state.key_drag = None,
        GesturePhase::Click => state::push_intent(TimelineIntent::ClearSelection),
        GesturePhase::End | GesturePhase::DoubleClick => {}
    }
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
        TimelineGesture {
            surface: SURFACE,
            kind,
            phase,
            x,
            y: 0.0,
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
            TimelineHitKind::Lane => apply_lane(state, g),
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

    // ── Wheel: anchored zoom + pan (W2.E6) ───────────────────────────────

    /// The time under the cursor, given the view.
    fn time_at(st: &TimelinePanelState, time_x: f32, x: f32) -> f64 {
        st.view_start_s + f64::from(x - time_x) / st.px_per_s
    }

    #[test]
    fn zoom_holds_the_time_under_the_cursor() {
        let mut st = TimelinePanelState::default(); // 120 px/s, view_start 0
        let (time_x, anchor_x) = (100.0_f32, 340.0_f32); // cursor sits at t = 2 s
        let before = time_at(&st, time_x, anchor_x);
        assert!((before - 2.0).abs() < 1e-9);

        apply_wheel(
            &mut st,
            time_x,
            TimelineZoom {
                zoom_delta: 240.0, // one e-fold in (wheel px, not notches)
                pan_delta: 0.0,
                anchor_x,
            },
        );
        assert!(st.px_per_s > 120.0, "zoomed in");
        let after = time_at(&st, time_x, anchor_x);
        assert!(
            (after - before).abs() < 1e-9,
            "the time under the cursor must not move: {before} → {after}"
        );
    }

    #[test]
    fn zoom_clamps_to_the_bounds() {
        let mut st = TimelinePanelState::default();
        apply_wheel(
            &mut st,
            0.0,
            TimelineZoom {
                zoom_delta: 1e4,
                pan_delta: 0.0,
                anchor_x: 0.0,
            },
        );
        assert_eq!(st.px_per_s, MAX_PX_PER_S);
        apply_wheel(
            &mut st,
            0.0,
            TimelineZoom {
                zoom_delta: -1e4,
                pan_delta: 0.0,
                anchor_x: 0.0,
            },
        );
        assert_eq!(st.px_per_s, MIN_PX_PER_S);
    }

    #[test]
    fn pan_slides_the_view_and_never_goes_negative() {
        let mut st = TimelinePanelState {
            view_start_s: 1.0,
            ..TimelinePanelState::default()
        }; // 120 px/s → 48 wheel px = 0.4 s
        // A NEGATIVE delta scrolls content left ⇒ the view moves later.
        apply_wheel(
            &mut st,
            0.0,
            TimelineZoom {
                zoom_delta: 0.0,
                pan_delta: -48.0,
                anchor_x: 0.0,
            },
        );
        assert!(
            (st.view_start_s - 1.4).abs() < 1e-9,
            "panned later by 0.4 s"
        );
        // A positive delta moves the view earlier.
        apply_wheel(
            &mut st,
            0.0,
            TimelineZoom {
                zoom_delta: 0.0,
                pan_delta: 48.0,
                anchor_x: 0.0,
            },
        );
        assert!(
            (st.view_start_s - 1.0).abs() < 1e-9,
            "panned earlier by 0.4 s"
        );

        apply_wheel(
            &mut st,
            0.0,
            TimelineZoom {
                zoom_delta: 0.0,
                pan_delta: 5_000.0,
                anchor_x: 0.0,
            },
        );
        assert_eq!(
            st.view_start_s, 0.0,
            "t=0 is the left bound; never negative"
        );
    }

    #[test]
    fn zoom_does_not_disturb_the_selection_or_drag() {
        let mut st = TimelinePanelState::default();
        apply_wheel(
            &mut st,
            0.0,
            TimelineZoom {
                zoom_delta: 3.0,
                pan_delta: 0.0,
                anchor_x: 50.0,
            },
        );
        assert!(st.key_drag.is_none());
        assert!(
            state::drain_intents().is_empty(),
            "view changes raise no intents"
        );
    }

    #[test]
    fn lane_click_clears_the_selection() {
        let mut st = TimelinePanelState::default();
        feed(
            &mut st,
            gesture(TimelineHitKind::Lane, GesturePhase::Click, 200.0, false),
            120.0,
            &snap(),
        );
        assert_eq!(state::drain_intents(), vec![TimelineIntent::ClearSelection]);
    }
}
