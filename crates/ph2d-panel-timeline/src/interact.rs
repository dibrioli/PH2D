//! Dope-sheet gesture interpretation (W2.E5b). Drains the `TimelineSurface`
//! channel the pointer dispatch fills (Begin/Update/End/Click over key diamonds
//! and empty lanes) and turns it into ephemeral drag state plus the
//! [`TimelineIntent`]s the shell applies (selection + key moves).
//!
//! Coverage: click a diamond → select (Shift = toggle into a multi-selection);
//! click empty lane → clear; drag a selected diamond → live preview, one
//! `MoveSelectedKeys` (frame-snapped) committed at End (a single undo step).
//! Delete is handled shell-side against the panel selection (no key channel
//! here). Right/middle buttons are ignored (reserved for future menu/pan).

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture, TimelineHitKind};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_host::PointerButton;
use ph2d_timeline::{SelectedKey, TimelineIntent, TimelineViewSnapshot};

use crate::state::{self, KeyDrag, TimelinePanelState};

/// Drain this frame's dope-sheet gestures and raise the resulting intents. Call
/// from `paint`, before drawing, so the preview offset (`state.key_drag`) is
/// up to date for the same frame's diamonds.
pub(crate) fn process(
    state: &mut TimelinePanelState,
    ctx: &mut PaintCtx,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
) {
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

/// Key-diamond gesture machine: Begin selects + arms a drag; Update tracks the
/// pointer; End commits a frame-snapped move; Click just ends the drag.
fn apply_key(
    state: &mut TimelinePanelState,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    sel: SelectedKey,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => {
            state::push_intent(if g.mods.shift {
                TimelineIntent::ToggleSelect(sel)
            } else {
                TimelineIntent::SelectSingle(sel)
            });
            state.key_drag = Some(KeyDrag {
                start_x: g.x,
                cur_x: g.x,
            });
        }
        GesturePhase::Update => {
            if let Some(d) = state.key_drag.as_mut() {
                d.cur_x = g.x;
            }
        }
        GesturePhase::End => {
            if let Some(d) = state.key_drag.take()
                && let Some(delta) = drag_delta_seconds(&d, px_per_s, snap)
            {
                state::push_intent(TimelineIntent::MoveSelectedKeys {
                    delta_seconds: delta,
                });
            }
        }
        GesturePhase::Click | GesturePhase::DoubleClick => state.key_drag = None,
    }
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
    let Some(d) = state.key_drag.as_ref() else {
        return 0.0;
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
