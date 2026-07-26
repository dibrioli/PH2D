//! Box-select (marquee) over the dope-sheet lanes — the mass-selection gesture
//! (`P5`). Sibling of `interact` (key diamonds) and `view` (camera), split out
//! under the HR-18 panel LOC cap.
//!
//! The gesture is recorded in two halves because the two facts it needs live in
//! different places:
//!
//! - [`apply_lane`] runs while draining gestures, BEFORE the panel geometry is
//!   resolved. It only knows pointer positions, so it just tracks the marquee
//!   and parks the finished one in `state.box_commit`.
//! - [`commit`] runs from `paint`, once the rows' `y` and the time scale are
//!   known, and resolves which diamonds the marquee caught.
//!
//! Both run in the same frame, so a release selects immediately.

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};

use crate::state::{self, BoxDrag, TimelinePanelState};

/// Empty-lane gesture: a tap clears the selection, a drag rubber-bands a
/// box-select (Shift = add to the selection rather than replace it).
pub(crate) fn apply_lane(state: &mut TimelinePanelState, g: TimelineGesture) {
    match g.phase {
        GesturePhase::Begin => {
            state.key_drag = None;
            state.box_drag = Some(BoxDrag {
                start: (g.x, g.y),
                cur: (g.x, g.y),
                additive: g.mods.shift,
            });
        }
        GesturePhase::Update => {
            if let Some(b) = state.box_drag.as_mut() {
                b.cur = (g.x, g.y);
            }
        }
        GesturePhase::End => state.box_commit = state.box_drag.take(),
        GesturePhase::Click => {
            state.box_drag = None;
            // Shift-clicking empty space keeps the selection (it is the additive
            // modifier); a plain click clears it.
            if !g.mods.shift {
                state::push_intent(TimelineIntent::ClearSelection);
            }
        }
        GesturePhase::DoubleClick => state.box_drag = None,
    }
}

/// Turn a finished marquee into selection intents, given the resolved row
/// geometry: `ClearSelection` first unless the drag was additive (Shift), then
/// one `AddToSelection` per key whose diamond centre lies inside the box.
/// A no-op when no marquee is pending.
pub(crate) fn commit(
    state: &mut TimelinePanelState,
    rows: Rect,
    view: crate::graph::TimeView,
    snap: &TimelineViewSnapshot,
) {
    let Some(b) = state.box_commit.take() else {
        return;
    };
    if !b.additive {
        state::push_intent(TimelineIntent::ClearSelection);
    }
    for sel in crate::tracks::keys_in_rect(rows, view, state, snap, b.rect()) {
        state::push_intent(TimelineIntent::AddToSelection(sel));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::interaction::{GestureMods, TimelineHitKind};
    use ph2d_host::PointerButton;
    use ph2d_timeline::SelectedKey;
    use ph2d_tokens::ROW_H_PX;

    fn gesture(phase: GesturePhase, x: f32, y: f32, shift: bool) -> TimelineGesture {
        TimelineGesture {
            surface: ph2d_a11y::NodeId(0),
            kind: TimelineHitKind::Lane,
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

    /// Two rows at 100 px/s from `time_x = 0`: row 0 has keys at x = 10 and
    /// x = 200, row 1 a key at x = 15.
    fn snap_two_rows() -> TimelineViewSnapshot {
        use ph2d_timeline::{AnimTarget, Interp, KeyId, KeyView, TrackView};
        let key = |id: u64, t: f64| KeyView {
            id: KeyId::new(id),
            t_seconds: t,
            value: 0.0,
            interp: Interp::Linear,
            selected: false,
            roving: false,
        };
        let row = |target: u64, keys: Vec<KeyView>| TrackView {
            target: AnimTarget::new(target),
            prop: ph2d_timeline::PropKind::TranslationX,
            entity: 1,
            missing: false,
            buffer_ghost: None,
            pre: ph2d_timeline::Extrap::Hold,
            post: ph2d_timeline::Extrap::Hold,
            keys,
        };
        TimelineViewSnapshot {
            tracks: vec![
                row(10, vec![key(0, 0.10), key(1, 2.00)]),
                row(20, vec![key(2, 0.15)]),
            ],
            ..TimelineViewSnapshot::default()
        }
    }

    /// Drag a marquee from `from` to `to` over the lanes and resolve it against
    /// `rows`, returning the intents raised.
    fn box_select(
        rows: Rect,
        scroll_y: f32,
        from: (f32, f32),
        to: (f32, f32),
        shift: bool,
    ) -> Vec<TimelineIntent> {
        let s = snap_two_rows();
        let mut st = TimelinePanelState {
            scroll_y,
            ..TimelinePanelState::default()
        };
        for (phase, (x, y)) in [
            (GesturePhase::Begin, from),
            (GesturePhase::Update, to),
            (GesturePhase::End, to),
        ] {
            apply_lane(&mut st, gesture(phase, x, y, shift));
        }
        assert!(st.box_commit.is_some(), "End parks the marquee for paint");
        let view = crate::graph::TimeView {
            time_x: 0.0,
            right: rows.x + rows.w,
            view_start: 0.0,
            px_per_s: 100.0,
        };
        commit(&mut st, rows, view, &s);
        assert!(st.box_commit.is_none(), "the marquee was consumed");
        state::drain_intents()
    }

    #[test]
    fn lane_click_clears_the_selection() {
        let mut st = TimelinePanelState::default();
        apply_lane(&mut st, gesture(GesturePhase::Click, 200.0, 0.0, false));
        assert_eq!(state::drain_intents(), vec![TimelineIntent::ClearSelection]);
    }

    #[test]
    fn shift_clicking_an_empty_lane_keeps_the_selection() {
        // Shift is the additive modifier: it must never wipe the selection the
        // user is building, even when they miss a diamond.
        let mut st = TimelinePanelState::default();
        apply_lane(&mut st, gesture(GesturePhase::Click, 200.0, 0.0, true));
        assert_eq!(state::drain_intents(), vec![], "no ClearSelection");
    }

    /// The Summary channel occupies the first row, so the track rows begin one
    /// row lower. It carries no keys of its own — a marquee over it catches
    /// nothing, which is why every span below starts at the very top and still
    /// only ever reports the two track keys.
    const SUMMARY: f32 = ROW_H_PX;

    #[test]
    fn dragging_an_empty_lane_box_selects_the_keys_inside_it() {
        let rows = Rect::new(0.0, 0.0, 400.0, SUMMARY + ROW_H_PX * 2.0);
        // x ∈ [5, 50] catches row 0's key at x=10 and row 1's at x=15, never the
        // one at x=200; y spans the Summary band and both track rows.
        let got = box_select(
            rows,
            0.0,
            (5.0, 0.0),
            (50.0, SUMMARY + ROW_H_PX * 2.0),
            false,
        );
        assert_eq!(
            got,
            vec![
                TimelineIntent::ClearSelection,
                TimelineIntent::AddToSelection(SelectedKey::new(10, 0)),
                TimelineIntent::AddToSelection(SelectedKey::new(20, 2)),
            ],
            "a plain marquee replaces the selection with what it caught"
        );
    }

    #[test]
    fn a_shift_marquee_adds_without_clearing() {
        let rows = Rect::new(0.0, 0.0, 400.0, SUMMARY + ROW_H_PX * 2.0);
        let got = box_select(
            rows,
            0.0,
            (5.0, 0.0),
            (50.0, SUMMARY + ROW_H_PX * 2.0),
            true,
        );
        assert_eq!(
            got,
            vec![
                TimelineIntent::AddToSelection(SelectedKey::new(10, 0)),
                TimelineIntent::AddToSelection(SelectedKey::new(20, 2)),
            ],
            "Shift keeps whatever was already selected"
        );
    }

    #[test]
    fn a_marquee_drawn_backwards_selects_the_same_keys() {
        let rows = Rect::new(0.0, 0.0, 400.0, SUMMARY + ROW_H_PX * 2.0);
        // Bottom-right → top-left: the rect normalizes, the result must not change.
        let got = box_select(
            rows,
            0.0,
            (50.0, SUMMARY + ROW_H_PX * 2.0),
            (5.0, 0.0),
            false,
        );
        assert_eq!(got.len(), 3, "ClearSelection + the same two keys");
        assert_eq!(
            got[1],
            TimelineIntent::AddToSelection(SelectedKey::new(10, 0))
        );
    }

    #[test]
    fn a_marquee_cannot_reach_rows_scrolled_out_of_the_band() {
        // A one-row-tall band scrolled past the Summary channel AND row 0: row 0
        // is clipped above it, row 1 is the visible one. A marquee spanning the
        // whole panel height must catch ONLY row 1's key — not the row hidden
        // under the ruler.
        let rows = Rect::new(0.0, 100.0, 400.0, ROW_H_PX);
        let got = box_select(rows, SUMMARY + ROW_H_PX, (5.0, 0.0), (50.0, 1000.0), false);
        assert_eq!(
            got,
            vec![
                TimelineIntent::ClearSelection,
                TimelineIntent::AddToSelection(SelectedKey::new(20, 2)),
            ]
        );
    }

    #[test]
    fn an_empty_marquee_still_clears_the_selection() {
        let rows = Rect::new(0.0, 0.0, 400.0, ROW_H_PX * 2.0);
        // x ∈ [300, 350] — past every key.
        let got = box_select(rows, 0.0, (300.0, 0.0), (350.0, ROW_H_PX * 2.0), false);
        assert_eq!(got, vec![TimelineIntent::ClearSelection]);
    }
}
