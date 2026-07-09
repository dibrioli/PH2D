//! View gestures of the dope sheet: what moves the *camera*, not the document.
//!
//! Sibling of `interact` (which owns the key/selection gestures), split out
//! under the HR-18 panel LOC cap. Three behaviours, all Blender-shaped:
//!
//! - **wheel** — plain = anchored zoom of the time axis (the time under the
//!   cursor stays put), Ctrl = horizontal pan, Shift = vertical row scroll.
//! - **middle-drag** — grab-and-slide both axes at once.
//! - **edge/corner drag** — resize the panel on any of its four sides.
//!
//! None of these raise intents: the view is panel-local and never undoable.

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture, TimelineWheel};
use ph2d_editor_core::zones::Rect;

use crate::geom;
use crate::state::{MAX_PX_PER_S, MIN_PX_PER_S, ResizeDrag, TimelinePanelState};

/// Wheel **pixels** per e-fold of zoom. The shell delivers line-deltas already
/// scaled to logical px (16 px per notch), so one notch is ~7% zoom here — the
/// same sensitivity the motion graph uses.
const ZOOM_WHEEL_DIV: f64 = 240.0; // LITERAL-PX-OK: wheel px → zoom-factor sensitivity divisor

/// Apply one frame's accumulated wheel: `pan` slides `view_start_s`, `scroll`
/// slides the rows, `zoom` scales `px_per_s` about the cursor holding the time
/// under `anchor_x` fixed. `view_start_s` never goes negative (t=0 is the left
/// bound of the clip); `scroll_y` stays within the measured overflow.
pub(crate) fn apply_wheel(state: &mut TimelinePanelState, time_x: f32, w: TimelineWheel) {
    // Pan first, in the pre-zoom scale (what the user saw when they scrolled).
    // Deltas are already in logical px; a positive delta scrolls the content
    // right/down ⇒ the view moves EARLIER / up, the same sign convention as the
    // panel-scroll path (`panel_scroll - delta_y`).
    if w.pan_delta != 0.0 {
        state.view_start_s -= f64::from(w.pan_delta) / state.px_per_s;
    }
    if w.scroll_delta != 0.0 {
        state.scroll_y = (state.scroll_y - w.scroll_delta).clamp(0.0, state.scroll_max); // CLAMP-OK: measured bounds, min<=max
    }
    if w.zoom_delta != 0.0 {
        let old = state.px_per_s;
        let new = (old * (f64::from(w.zoom_delta) / ZOOM_WHEEL_DIV).exp())
            .clamp(MIN_PX_PER_S, MAX_PX_PER_S); // CLAMP-OK: const bounds, min<max, non-NaN
        // Hold the time under the cursor: t = start + (anchor - time_x)/px_per_s.
        let off_px = f64::from(w.anchor_x - time_x);
        let t_anchor = state.view_start_s + off_px / old;
        state.view_start_s = t_anchor - off_px / new;
        state.px_per_s = new;
    }
    state.view_start_s = state.view_start_s.max(0.0);
}

/// Middle-drag: grab-and-slide both axes. Dragging right moves the content right,
/// so the view moves EARLIER; dragging down reveals earlier rows.
pub(crate) fn apply_pan_drag(state: &mut TimelinePanelState, px_per_s: f64, g: TimelineGesture) {
    match g.phase {
        GesturePhase::Begin => state.pan_drag = Some((g.x, g.y)),
        GesturePhase::Update => {
            if let Some((ax, ay)) = state.pan_drag {
                state.view_start_s = (state.view_start_s - f64::from(g.x - ax) / px_per_s).max(0.0);
                state.scroll_y = (state.scroll_y - (g.y - ay)).clamp(0.0, state.scroll_max); // CLAMP-OK: measured bounds, min<=max
                state.pan_drag = Some((g.x, g.y));
            }
        }
        _ => state.pan_drag = None,
    }
}

/// Edge/corner drag: move the set edges by the pointer delta from Begin. Deltas
/// apply to the rect captured at Begin, so a slow drag never accumulates drift.
pub(crate) fn apply_resize(
    state: &mut TimelinePanelState,
    rect: Rect,
    viewport: Rect,
    edges: u8,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => {
            state.resize = Some(ResizeDrag {
                edges,
                start_rect: rect,
                start_pointer: (g.x, g.y),
            });
        }
        GesturePhase::Update => {
            if let Some(d) = state.resize {
                let (dx, dy) = (g.x - d.start_pointer.0, g.y - d.start_pointer.1);
                state.rect = Some(geom::resized(d.start_rect, d.edges, dx, dy, viewport));
            }
        }
        _ => state.resize = None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::DEFAULT_PX_PER_S;
    use ph2d_editor_core::interaction::{GestureMods, TimelineHitKind};
    use ph2d_host::PointerButton;

    const SURFACE: ph2d_a11y::NodeId = ph2d_a11y::NodeId(0);
    const VP: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);

    fn wheel(zoom: f32, pan: f32, scroll: f32, anchor_x: f32) -> TimelineWheel {
        TimelineWheel {
            zoom_delta: zoom,
            pan_delta: pan,
            scroll_delta: scroll,
            anchor_x,
        }
    }

    fn drag(button: PointerButton, phase: GesturePhase, x: f32, y: f32) -> TimelineGesture {
        TimelineGesture {
            surface: SURFACE,
            kind: TimelineHitKind::Lane,
            phase,
            x,
            y,
            button,
            mods: GestureMods::default(),
        }
    }

    fn mmb(phase: GesturePhase, x: f32, y: f32) -> TimelineGesture {
        drag(PointerButton::Middle, phase, x, y)
    }

    /// The time under `x`, given the view.
    fn time_at(st: &TimelinePanelState, time_x: f32, x: f32) -> f64 {
        st.view_start_s + f64::from(x - time_x) / st.px_per_s
    }

    // ── Wheel ────────────────────────────────────────────────────────────

    #[test]
    fn zoom_holds_the_time_under_the_cursor() {
        let mut st = TimelinePanelState::default(); // 120 px/s, view_start 0
        let (time_x, anchor_x) = (100.0_f32, 340.0_f32); // cursor sits at t = 2 s
        let before = time_at(&st, time_x, anchor_x);
        assert!((before - 2.0).abs() < 1e-9);

        apply_wheel(&mut st, time_x, wheel(240.0, 0.0, 0.0, anchor_x)); // one e-fold in
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
        apply_wheel(&mut st, 0.0, wheel(1e4, 0.0, 0.0, 0.0));
        assert_eq!(st.px_per_s, MAX_PX_PER_S);
        apply_wheel(&mut st, 0.0, wheel(-1e4, 0.0, 0.0, 0.0));
        assert_eq!(st.px_per_s, MIN_PX_PER_S);
    }

    #[test]
    fn pan_slides_the_view_and_never_goes_negative() {
        let mut st = TimelinePanelState {
            view_start_s: 1.0,
            ..TimelinePanelState::default()
        }; // 120 px/s → 48 wheel px = 0.4 s
        // A NEGATIVE delta scrolls content left ⇒ the view moves later.
        apply_wheel(&mut st, 0.0, wheel(0.0, -48.0, 0.0, 0.0));
        assert!(
            (st.view_start_s - 1.4).abs() < 1e-9,
            "panned later by 0.4 s"
        );
        // A positive delta moves the view earlier.
        apply_wheel(&mut st, 0.0, wheel(0.0, 48.0, 0.0, 0.0));
        assert!(
            (st.view_start_s - 1.0).abs() < 1e-9,
            "panned earlier by 0.4 s"
        );

        apply_wheel(&mut st, 0.0, wheel(0.0, 5_000.0, 0.0, 0.0));
        assert_eq!(
            st.view_start_s, 0.0,
            "t=0 is the left bound; never negative"
        );
    }

    #[test]
    fn shift_wheel_scrolls_the_rows_within_the_measured_range() {
        let mut st = TimelinePanelState {
            scroll_y: 50.0,
            scroll_max: 200.0,
            ..TimelinePanelState::default()
        };
        // A positive delta scrolls content down ⇒ earlier rows ⇒ scroll_y drops.
        apply_wheel(&mut st, 0.0, wheel(0.0, 0.0, 30.0, 0.0));
        assert_eq!(st.scroll_y, 20.0);
        assert_eq!(st.px_per_s, DEFAULT_PX_PER_S, "scrolling never zooms");
        // And it saturates at both ends.
        apply_wheel(&mut st, 0.0, wheel(0.0, 0.0, -10_000.0, 0.0));
        assert_eq!(st.scroll_y, 200.0, "clamped to scroll_max");
    }

    #[test]
    fn a_view_change_never_disturbs_the_key_drag() {
        let mut st = TimelinePanelState::default();
        apply_wheel(&mut st, 0.0, wheel(3.0, 0.0, 0.0, 50.0));
        assert!(st.key_drag.is_none());
    }

    // ── Middle-drag pan ──────────────────────────────────────────────────

    #[test]
    fn middle_drag_pans_time_and_rows_together() {
        let mut st = TimelinePanelState {
            view_start_s: 2.0,
            scroll_y: 100.0,
            scroll_max: 400.0,
            ..TimelinePanelState::default()
        }; // 120 px/s
        apply_pan_drag(&mut st, 120.0, mmb(GesturePhase::Begin, 500.0, 300.0));
        // Drag right + down: grab-and-slide, so the view goes EARLIER and up.
        apply_pan_drag(&mut st, 120.0, mmb(GesturePhase::Update, 560.0, 340.0));
        assert!(
            (st.view_start_s - 1.5).abs() < 1e-9,
            "60 px right = 0.5 s earlier"
        );
        assert_eq!(
            st.scroll_y, 60.0,
            "40 px down reveals 40 px of earlier rows"
        );
        assert!(st.pan_drag.is_some(), "still dragging");

        apply_pan_drag(&mut st, 120.0, mmb(GesturePhase::End, 560.0, 340.0));
        assert!(st.pan_drag.is_none());
    }

    #[test]
    fn middle_drag_pan_respects_the_t0_and_scroll_bounds() {
        let mut st = TimelinePanelState {
            view_start_s: 0.1,
            scroll_y: 10.0,
            scroll_max: 50.0,
            ..TimelinePanelState::default()
        };
        apply_pan_drag(&mut st, 120.0, mmb(GesturePhase::Begin, 0.0, 0.0));
        apply_pan_drag(&mut st, 120.0, mmb(GesturePhase::Update, 9_000.0, 9_000.0));
        assert_eq!(st.view_start_s, 0.0, "clamped at t=0");
        assert_eq!(st.scroll_y, 0.0, "clamped at the top of the list");
    }

    // ── Resize ───────────────────────────────────────────────────────────

    #[test]
    fn dragging_the_top_edge_resizes_from_the_rect_captured_at_begin() {
        let mut st = TimelinePanelState::default();
        let start = Rect::new(100.0, 600.0, 800.0, 240.0);
        let g = |phase, y| drag(PointerButton::Primary, phase, 400.0, y);

        apply_resize(
            &mut st,
            start,
            VP,
            geom::EDGE_T,
            g(GesturePhase::Begin, 600.0),
        );
        apply_resize(
            &mut st,
            start,
            VP,
            geom::EDGE_T,
            g(GesturePhase::Update, 550.0),
        );
        let r = st.rect.expect("resized");
        assert_eq!((r.y, r.h), (550.0, 290.0), "grew upward");

        // A second Update still measures from Begin, not from the live rect.
        apply_resize(
            &mut st,
            start,
            VP,
            geom::EDGE_T,
            g(GesturePhase::Update, 500.0),
        );
        let r = st.rect.expect("resized");
        assert_eq!((r.y, r.h), (500.0, 340.0), "no drift accumulation");

        apply_resize(
            &mut st,
            start,
            VP,
            geom::EDGE_T,
            g(GesturePhase::End, 500.0),
        );
        assert!(st.resize.is_none());
    }
}
