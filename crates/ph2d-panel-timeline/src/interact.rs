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
        TimelineHitKind::Marker { index } => {
            marker_drag::apply(state, time_x, px_per_s, snap, index, g);
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
        // Column lock (default): a press on a track key is a press on its whole
        // time column, so grabbing one key grabs the vertical group. Unlocked, it
        // moves alone. Either way, the Summary diamond itself always moves the
        // column (`SummaryKey`, below).
        TimelineHitKind::Key { target, key } => {
            if let Some(t_bits) = state
                .column_lock
                .then(|| summary::key_t_bits(snap, target, key))
                .flatten()
            {
                summary::apply_gesture(state, px_per_s, snap, t_bits, g);
            } else {
                key_drag::apply_key(
                    state,
                    px_per_s,
                    snap,
                    ph2d_timeline::SelectedKey::new(target, key),
                    g,
                );
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

    // Run the real Primary router (bypassing the store drain, which needs a host)
    // so these exercise the same dispatch the panel does, lock routing and all.
    fn feed(
        state: &mut TimelinePanelState,
        g: TimelineGesture,
        px_per_s: f64,
        snap: &TimelineViewSnapshot,
    ) {
        dispatch_primary(state, 0.0, px_per_s, snap, g);
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

    // ── column lock: press a key, grab its whole column (or not) ─────────────

    /// Two tracks, each with a key at t = 0, so pressing either key's diamond
    /// finds a two-key column. Track 0 key 1; track 5 key 7.
    fn two_key_column() -> TimelineViewSnapshot {
        use ph2d_timeline::{AnimTarget, Interp, KeyId, KeyView, PropKind, TrackView};
        let k = |id: u64| KeyView {
            id: KeyId::new(id),
            t_seconds: 0.0,
            value: 0.0,
            interp: Interp::Linear,
            selected: false,
            roving: false,
        };
        TimelineViewSnapshot {
            fps: 60.0,
            frame_snap: true,
            tracks: vec![
                TrackView {
                    target: AnimTarget::new(0),
                    prop: PropKind::TranslationX,
                    entity: 1,
                    missing: false,
                    keys: vec![k(1)],
                },
                TrackView {
                    target: AnimTarget::new(5),
                    prop: PropKind::Opacity,
                    entity: 1,
                    missing: false,
                    keys: vec![k(7)],
                },
            ],
            ..snap()
        }
    }

    #[test]
    fn locked_pressing_a_key_grabs_its_whole_column() {
        // With the padlock CLOSED, pressing one key at t = 0 selects EVERY key
        // at t = 0, so a drag moves the vertical group — routed through the
        // Summary machine. (Open is the default — see the test below.)
        let mut st = TimelinePanelState {
            column_lock: true,
            ..TimelinePanelState::default()
        };
        let key = TimelineHitKind::Key { target: 0, key: 1 };
        feed(
            &mut st,
            gesture(key, GesturePhase::Begin, 10.0, false),
            120.0,
            &two_key_column(),
        );
        assert_eq!(
            state::drain_intents(),
            vec![
                TimelineIntent::BeginEdit,
                TimelineIntent::ClearSelection,
                TimelineIntent::AddToSelection(ph2d_timeline::SelectedKey::new(0, 1)),
                TimelineIntent::AddToSelection(ph2d_timeline::SelectedKey::new(5, 7)),
            ],
            "locked: the whole column, not just the pressed key"
        );
    }

    #[test]
    fn unlocked_pressing_a_key_grabs_only_that_key() {
        // THE default (Enio 2026-07-11): a key click selects just that key —
        // it must never silently grab the whole Summary column.
        let mut st = TimelinePanelState::default();
        assert!(!st.column_lock, "open by default");
        let key = TimelineHitKind::Key { target: 0, key: 1 };
        feed(
            &mut st,
            gesture(key, GesturePhase::Begin, 10.0, false),
            120.0,
            &two_key_column(),
        );
        assert_eq!(
            state::drain_intents(),
            vec![
                TimelineIntent::BeginEdit,
                TimelineIntent::SelectSingle(ph2d_timeline::SelectedKey::new(0, 1)),
            ],
            "unlocked: just the pressed key — the other track stays put"
        );
    }

    #[test]
    fn a_locked_press_on_a_key_with_no_column_falls_back_to_the_key_itself() {
        // Snapshot lag: the pressed key isn't in the published snapshot. Rather
        // than doing nothing, treat it as a plain key press.
        let mut st = TimelinePanelState {
            column_lock: true,
            ..TimelinePanelState::default()
        };
        let key = TimelineHitKind::Key { target: 9, key: 9 };
        feed(
            &mut st,
            gesture(key, GesturePhase::Begin, 10.0, false),
            120.0,
            &two_key_column(),
        );
        assert_eq!(
            state::drain_intents(),
            vec![
                TimelineIntent::BeginEdit,
                TimelineIntent::SelectSingle(ph2d_timeline::SelectedKey::new(9, 9)),
            ],
        );
    }

    #[test]
    fn clicking_the_padlock_toggles_the_column_lock() {
        let mut st = TimelinePanelState::default();
        assert!(!st.column_lock, "open by default");
        feed(
            &mut st,
            gesture(
                TimelineHitKind::SummaryLock,
                GesturePhase::Click,
                0.0,
                false,
            ),
            120.0,
            &snap(),
        );
        assert!(st.column_lock, "one click closes it");
        feed(
            &mut st,
            gesture(
                TimelineHitKind::SummaryLock,
                GesturePhase::Click,
                0.0,
                false,
            ),
            120.0,
            &snap(),
        );
        assert!(!st.column_lock, "another opens it");
        assert_eq!(
            state::drain_intents(),
            vec![],
            "the lock is view state, not an edit"
        );
    }

    #[test]
    fn pressing_the_padlock_without_releasing_does_not_toggle() {
        // Only a Click counts, so a stray drag from the padlock leaves it alone.
        let mut st = TimelinePanelState::default();
        feed(
            &mut st,
            gesture(
                TimelineHitKind::SummaryLock,
                GesturePhase::Begin,
                0.0,
                false,
            ),
            120.0,
            &snap(),
        );
        assert!(!st.column_lock, "a press is not a toggle (stays open)");
    }
}
