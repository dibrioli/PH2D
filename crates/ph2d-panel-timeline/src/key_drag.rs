//! The key-diamond gesture (W2.E5b) — select, group-drag, collapse.
//!
//! Split from `interact` (the gesture router) under the HR-18 panel LOC cap.
//!
//! A drag streams one frame-snapped `MoveSelectedKeys` per frame, so the
//! diamonds, the curves and the SCENE all track the cursor instead of jumping on
//! release. Each frame emits only what accrued since the last one — emitting the
//! running total would move the keys twice — and the whole press-to-release sits
//! in one `BeginEdit`/`EndEdit` bracket, so it undoes in a single Ctrl+Z.

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture};
use ph2d_timeline::{SelectedKey, TimelineIntent, TimelineViewSnapshot};

use crate::state::{self, KeyDrag, TimelinePanelState};

/// Key-diamond gesture machine: Begin selects (or preserves a group) + arms a
/// drag; every Update emits the frame-snapped move that has accrued since the
/// last one, so the diamonds, the curves and the SCENE all follow the cursor; a
/// plain Click collapses a preserved group to the pressed key.
///
/// The whole press-to-release lives inside one `BeginEdit`/`EndEdit` bracket, so
/// streaming a move per frame still undoes in a single Ctrl+Z.
pub(crate) fn apply_key(
    state: &mut TimelinePanelState,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    sel: SelectedKey,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => {
            // Open the undo bracket first: every frame's move joins this step.
            state::push_intent(TimelineIntent::BeginEdit);
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
                applied_s: 0.0,
            });
        }
        GesturePhase::Update => {
            if let Some(d) = state.key_drag.as_mut() {
                d.cur_x = g.x;
            }
            emit_move(state, px_per_s, snap);
        }
        GesturePhase::End => {
            // End only fires after movement (dispatch chose it over Click), so a
            // drag happened: land the last delta, never collapse.
            if let Some(d) = state.key_drag.as_mut() {
                d.cur_x = g.x;
            }
            emit_move(state, px_per_s, snap);
            state.key_drag = None;
            state::push_intent(TimelineIntent::EndEdit);
        }
        GesturePhase::Click | GesturePhase::DoubleClick => {
            // A plain click on a preserved group collapses it to the pressed key.
            // The bracket closes having changed nothing, so it commits no step.
            if let Some(d) = state.key_drag.take()
                && let Some(one) = d.collapse_to
            {
                state::push_intent(TimelineIntent::SelectSingle(one));
            }
            state::push_intent(TimelineIntent::EndEdit);
        }
    }
}

/// Emit whatever frame-snapped move has accrued since the last emit, so the
/// document tracks the pointer instead of jumping on release.
///
/// The intents raised this frame land in the shell NEXT frame, so the snapshot
/// the diamonds are painted from still lags by exactly what was emitted here —
/// `pending_move_dx` carries that offset for one frame and is cleared at the top
/// of the next `process`.
pub(crate) fn emit_move(
    state: &mut TimelinePanelState,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
) {
    let Some(d) = state.key_drag else {
        return;
    };
    let want = drag_delta_seconds(d.start_x, d.cur_x, px_per_s, snap);
    let delta = want - d.applied_s;
    if delta == 0.0 {
        return;
    }
    if let Some(d) = state.key_drag.as_mut() {
        d.applied_s = want;
    }
    state::push_intent(TimelineIntent::MoveSelectedKeys {
        delta_seconds: delta,
    });
    let dx = (delta * px_per_s) as f32;
    state.pending_move_dx = Some(state.pending_move_dx.unwrap_or(0.0) + dx);
}

/// A drag's TOTAL frame-snapped time delta so far (`0.0` when it rounds to
/// nothing). Snaps to whole display frames when the snapshot says frame-snap is
/// on, matching the transport's scrub/AddKey snapping. Shared with the graph
/// editor's anchor drag, which rides the same time axis.
pub(crate) fn drag_delta_seconds(
    start_x: f32,
    cur_x: f32,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
) -> f64 {
    if px_per_s <= 0.0 {
        return 0.0;
    }
    let raw = f64::from(cur_x - start_x) / px_per_s;
    if snap.frame_snap && snap.fps > 0.0 {
        (raw * snap.fps).round() / snap.fps
    } else {
        raw
    }
}

/// The offset (px) the selected diamonds ride for one frame: exactly what was
/// emitted this frame and has not reached the published snapshot yet. Zero the
/// rest of the time — the document itself now holds the move.
pub(crate) fn preview_dx(state: &TimelinePanelState) -> f32 {
    state.pending_move_dx.unwrap_or(0.0)
}

/// Whether `sel` is currently selected in the published snapshot. Shared with
/// the stagger gesture, which uses the same "preserve a selection, select a
/// fresh key" disambiguation at Begin.
pub(crate) fn is_selected(snap: &TimelineViewSnapshot, sel: SelectedKey) -> bool {
    snap.tracks
        .iter()
        .filter(|t| t.target == sel.target)
        .flat_map(|t| &t.keys)
        .any(|k| k.id == sel.key && k.selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::interaction::{GestureMods, TimelineHitKind};
    use ph2d_host::PointerButton;

    fn gesture(kind: TimelineHitKind, phase: GesturePhase, x: f32, shift: bool) -> TimelineGesture {
        TimelineGesture {
            surface: ph2d_a11y::NodeId(0),
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
                    value: 0.0,
                    interp: Interp::Linear,
                    selected: true,
                    roving: false,
                }],
            }],
            ..snap()
        }
    }

    fn feed(
        state: &mut TimelinePanelState,
        g: TimelineGesture,
        px_per_s: f64,
        snap: &TimelineViewSnapshot,
    ) {
        let TimelineHitKind::Key { target, key } = g.kind else {
            unreachable!("only key gestures are fed here")
        };
        apply_key(state, px_per_s, snap, SelectedKey::new(target, key), g);
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
            vec![
                TimelineIntent::BeginEdit,
                TimelineIntent::SelectSingle(SelectedKey::new(9, 2)),
                TimelineIntent::EndEdit,
            ]
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
            vec![
                TimelineIntent::BeginEdit,
                TimelineIntent::ToggleSelect(SelectedKey::new(5, 1)),
            ]
        );
    }

    #[test]
    fn a_key_drag_streams_the_move_and_brackets_it_as_one_undo_step() {
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
        assert_eq!(
            state::drain_intents(),
            vec![
                TimelineIntent::BeginEdit,
                TimelineIntent::SelectSingle(SelectedKey::new(1, 0)),
                TimelineIntent::MoveSelectedKeys {
                    delta_seconds: 0.25
                },
                TimelineIntent::EndEdit,
            ],
            "the move is emitted on the Update, not held back to the release"
        );
        assert!(st.key_drag.is_none(), "the drag ended");
        assert_eq!(st.pending_move_dx, Some(30.0), "0.25 s × 120 px·s⁻¹");
        assert_eq!(preview_dx(&st), 30.0);
    }

    #[test]
    fn a_continuing_drag_emits_only_what_accrued_since_the_last_frame() {
        // The scene must follow the cursor, and the streamed deltas must SUM to
        // the drag: emitting the running total each frame would move it twice.
        let mut st = TimelinePanelState::default();
        let key = TimelineHitKind::Key { target: 1, key: 0 };
        let s = snap();
        feed(
            &mut st,
            gesture(key, GesturePhase::Begin, 100.0, false),
            120.0,
            &s,
        );
        let _ = state::drain_intents();

        feed(
            &mut st,
            gesture(key, GesturePhase::Update, 130.0, false),
            120.0,
            &s,
        );
        assert_eq!(
            state::drain_intents(),
            vec![TimelineIntent::MoveSelectedKeys {
                delta_seconds: 0.25
            }]
        );
        st.pending_move_dx = None; // the next `process` clears it

        // Another 12 px: 0.1 s from the START, so only +0.1 is still owed.
        feed(
            &mut st,
            gesture(key, GesturePhase::Update, 142.0, false),
            120.0,
            &s,
        );
        let got = state::drain_intents();
        assert_eq!(got.len(), 1);
        let TimelineIntent::MoveSelectedKeys { delta_seconds } = got[0] else {
            panic!("{got:?}")
        };
        assert!((delta_seconds - 0.1).abs() < 1e-9, "{delta_seconds}");
        assert!((st.key_drag.unwrap().applied_s - 0.35).abs() < 1e-9);
        assert_eq!(st.pending_move_dx, Some(12.0));
    }

    #[test]
    fn a_frame_where_the_pointer_did_not_move_emits_nothing() {
        let mut st = TimelinePanelState::default();
        let key = TimelineHitKind::Key { target: 1, key: 0 };
        let s = snap();
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
        let _ = state::drain_intents();
        // Sub-frame jitter: the snapped total is unchanged, so nothing is owed.
        feed(
            &mut st,
            gesture(key, GesturePhase::Update, 130.5, false),
            120.0,
            &s,
        );
        assert_eq!(state::drain_intents(), vec![]);
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
            vec![
                TimelineIntent::BeginEdit,
                TimelineIntent::SelectSingle(SelectedKey::new(1, 0)),
                TimelineIntent::EndEdit,
            ],
            "no MoveSelectedKeys when the snapped delta is zero; the empty \
             bracket commits no undo step"
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
            vec![
                TimelineIntent::BeginEdit,
                TimelineIntent::MoveSelectedKeys {
                    delta_seconds: 0.25
                },
                TimelineIntent::EndEdit,
            ],
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
            vec![
                TimelineIntent::BeginEdit,
                TimelineIntent::SelectSingle(SelectedKey::new(7, 4)),
                TimelineIntent::EndEdit,
            ],
            "a no-drag click collapses the group to the pressed key"
        );
    }
}
