//! View gestures of the dope sheet: what moves the *camera*, not the document.
//!
//! Sibling of `interact` (which owns the key/selection gestures), split out
//! under the HR-18 panel LOC cap. Three behaviours, all Blender-shaped:
//!
//! - **wheel** — plain = anchored zoom of the time axis (the time under the
//!   cursor stays put), Ctrl = horizontal pan, Shift = vertical row scroll.
//! - **middle-drag** — grab-and-slide both axes at once.
//! - **F / transport jump** — fit to the keys, or pan the playhead into view.
//!
//! Chrome resizes (panel edges, label column, graph height) live in `resize`.
//! None of these raise intents: the view is panel-local and never undoable.

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture, TimelineWheel};
use ph2d_timeline::TimelineViewSnapshot;

use crate::state::{DEFAULT_PX_PER_S, MAX_PX_PER_S, MIN_PX_PER_S, TimelinePanelState};

/// Wheel **pixels** per e-fold of zoom. The shell delivers line-deltas already
/// scaled to logical px (16 px per notch), so one notch is ~7% zoom here — the
/// same sensitivity the motion graph uses.
const ZOOM_WHEEL_DIV: f64 = 240.0; // LITERAL-PX-OK: wheel px → zoom-factor sensitivity divisor

/// Fraction of the fitted span left as breathing room on each side, so the first
/// and last diamonds never sit exactly on the panel border.
const FIT_PAD: f64 = 0.05; // LITERAL-PX-OK: fit margin as a fraction of the span
/// The span a fit falls back to when every key sits at the same instant (or the
/// clip has a single key): one second centred on it, so the zoom stays sane.
const FIT_DEGENERATE_SPAN_S: f64 = 1.0; // LITERAL-PX-OK: fallback span for a zero-width key extent
/// Breathing room kept between a revealed playhead and the edge it came in from,
/// as a fraction of the visible span.
const REVEAL_MARGIN: f64 = 0.1; // LITERAL-PX-OK: reveal margin as a fraction of the span

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

/// Fit the time axis to the extent of every key in the document (`F`, Blender's
/// per-area focus), with a small margin. An empty document — or a zero-width
/// `time_area` — resets to the default zoom at `t = 0`; a single key (or several
/// stacked at one instant) gets a one-second window around it rather than an
/// infinite zoom.
pub(crate) fn apply_fit(
    state: &mut TimelinePanelState,
    time_area_w: f32,
    snap: &TimelineViewSnapshot,
) {
    let w = f64::from(time_area_w);
    let extent = key_extent(snap);
    let Some((first, last)) = extent.filter(|_| w > 0.0) else {
        state.view_start_s = 0.0;
        state.px_per_s = DEFAULT_PX_PER_S;
        return;
    };
    let (first, last) = if last - first > 0.0 {
        let pad = (last - first) * FIT_PAD;
        (first - pad, last + pad)
    } else {
        let half = FIT_DEGENERATE_SPAN_S * 0.5;
        (first - half, first + half)
    };
    state.px_per_s = (w / (last - first)).clamp(MIN_PX_PER_S, MAX_PX_PER_S); // CLAMP-OK: const bounds, min<max, non-NaN
    // Re-derive the start from the CLAMPED zoom, so a clamp keeps the extent
    // centred instead of pinning it to the left edge.
    let visible = w / state.px_per_s;
    state.view_start_s = ((first + last) * 0.5 - visible * 0.5).max(0.0);
}

/// Pan the time axis until `t` is visible, keeping [`REVEAL_MARGIN`] of the span
/// between it and the edge it was revealed against. A no-op when `t` is already
/// comfortably in view, so a frame-step in the middle of the strip never jerks
/// the view; `view_start_s` never goes negative, so go-to-start lands flush on
/// `t = 0`.
pub(crate) fn reveal_time(state: &mut TimelinePanelState, time_area_w: f32, t: f64) {
    let w = f64::from(time_area_w);
    if w <= 0.0 || state.px_per_s <= 0.0 {
        return;
    }
    let span = w / state.px_per_s;
    let margin = span * REVEAL_MARGIN;
    let start = state.view_start_s;
    if t < start + margin {
        state.view_start_s = (t - margin).max(0.0);
    } else if t > start + span - margin {
        state.view_start_s = (t - span + margin).max(0.0);
    }
}

/// The `(earliest, latest)` key time across every track, or `None` if the
/// document has no keys at all.
fn key_extent(snap: &TimelineViewSnapshot) -> Option<(f64, f64)> {
    let mut times = snap
        .tracks
        .iter()
        .flat_map(|t| &t.keys)
        .map(|k| k.t_seconds);
    let first = times.next()?;
    Some(times.fold((first, first), |(lo, hi), t| (lo.min(t), hi.max(t))))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::DEFAULT_PX_PER_S;
    use ph2d_editor_core::interaction::{GestureMods, TimelineHitKind};
    use ph2d_host::PointerButton;

    const SURFACE: ph2d_a11y::NodeId = ph2d_a11y::NodeId(0);

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
            "the time under the cursor must not move: {before} -> {after}"
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

    // ── Reveal (transport jump) ──────────────────────────────────────────

    #[test]
    fn reveal_pans_flush_to_zero_for_go_to_start() {
        // Panned far right; go-to-start scrubs to t = 0 and the view must follow.
        let mut st = TimelinePanelState {
            view_start_s: 30.0,
            ..TimelinePanelState::default()
        };
        reveal_time(&mut st, 800.0, 0.0);
        assert_eq!(
            st.view_start_s, 0.0,
            "no negative pan, t=0 sits on the edge"
        );
    }

    #[test]
    fn reveal_pans_right_until_the_time_is_in_view() {
        // 800 px at 120 px/s = 6.67 s visible from t = 0.
        let mut st = TimelinePanelState::default();
        reveal_time(&mut st, 800.0, 20.0);
        let span = 800.0 / DEFAULT_PX_PER_S;
        assert!(st.view_start_s > 0.0);
        assert!(
            20.0 > st.view_start_s && 20.0 < st.view_start_s + span,
            "the playhead is inside the span"
        );
        // ...and a tenth of the span from the right edge, not glued to it.
        assert!((st.view_start_s + span - 20.0 - span * 0.1).abs() < 1e-9);
    }

    #[test]
    fn reveal_leaves_a_time_that_is_already_comfortably_in_view() {
        // A frame step in the middle of the strip must not jerk the view.
        let mut st = TimelinePanelState::default();
        reveal_time(&mut st, 800.0, 3.0);
        assert_eq!(st.view_start_s, 0.0);
    }

    #[test]
    fn reveal_with_no_room_to_paint_is_a_no_op() {
        let mut st = TimelinePanelState {
            view_start_s: 5.0,
            ..TimelinePanelState::default()
        };
        reveal_time(&mut st, 0.0, 99.0);
        assert_eq!(st.view_start_s, 5.0);
    }

    // ── Fit (F) ──────────────────────────────────────────────────────────

    /// A snapshot whose single track carries a key at each of `times`.
    fn snap_with_keys(times: &[f64]) -> TimelineViewSnapshot {
        use ph2d_timeline::{AnimTarget, Interp, KeyId, KeyView, TrackView};
        TimelineViewSnapshot {
            tracks: vec![TrackView {
                target: AnimTarget::new(1),
                prop: ph2d_timeline::PropKind::TranslationX,
                entity: 1,
                missing: false,
                keys: times
                    .iter()
                    .enumerate()
                    .map(|(i, &t)| KeyView {
                        id: KeyId::new(i as u64),
                        t_seconds: t,
                        value: 0.0,
                        interp: Interp::Linear,
                        selected: false,
                    })
                    .collect(),
            }],
            ..TimelineViewSnapshot::default()
        }
    }

    /// The time at the right edge of a `w`-wide time area.
    fn view_end(st: &TimelinePanelState, w: f32) -> f64 {
        st.view_start_s + f64::from(w) / st.px_per_s
    }

    #[test]
    fn fit_frames_every_key_with_a_margin() {
        let mut st = TimelinePanelState::default();
        apply_fit(&mut st, 800.0, &snap_with_keys(&[1.0, 5.0]));
        assert!(st.view_start_s < 1.0, "the first key is not on the edge");
        assert!(
            view_end(&st, 800.0) > 5.0,
            "the last key is not on the edge"
        );
        // The 4 s extent + 5% each side fills the 800 px area.
        assert!((st.px_per_s - 800.0 / 4.4).abs() < 1e-6, "{}", st.px_per_s);
    }

    #[test]
    fn fit_on_an_empty_document_resets_to_the_default_view() {
        let mut st = TimelinePanelState {
            view_start_s: 12.0,
            px_per_s: 900.0,
            ..TimelinePanelState::default()
        };
        apply_fit(&mut st, 800.0, &TimelineViewSnapshot::default());
        assert_eq!((st.view_start_s, st.px_per_s), (0.0, DEFAULT_PX_PER_S));
    }

    #[test]
    fn fit_on_a_single_key_opens_a_one_second_window_around_it() {
        // A zero-width extent would divide by zero and slam px_per_s to the cap.
        let mut st = TimelinePanelState::default();
        apply_fit(&mut st, 800.0, &snap_with_keys(&[4.0]));
        assert!((st.px_per_s - 800.0).abs() < 1e-6, "800 px over 1 s");
        assert!((st.view_start_s - 3.5).abs() < 1e-6, "centred on the key");
    }

    #[test]
    fn fit_never_pans_before_zero_and_keeps_the_extent_centred_when_clamped() {
        // Two keys 4 ms apart: the exact fit wants px_per_s far past the ceiling.
        let mut st = TimelinePanelState::default();
        apply_fit(&mut st, 800.0, &snap_with_keys(&[10.0, 10.004]));
        assert_eq!(st.px_per_s, MAX_PX_PER_S, "zoom clamped");
        let centre = st.view_start_s + f64::from(800.0_f32) / st.px_per_s * 0.5;
        assert!((centre - 10.002).abs() < 1e-6, "extent stayed centred");

        // A key at t=0 with the zoom clamped the other way must not pan negative.
        let mut st = TimelinePanelState::default();
        apply_fit(&mut st, 800.0, &snap_with_keys(&[0.0]));
        assert!(st.view_start_s >= 0.0, "{}", st.view_start_s);
    }

    #[test]
    fn fit_with_no_room_to_paint_falls_back_to_the_default_view() {
        let mut st = TimelinePanelState::default();
        apply_fit(&mut st, 0.0, &snap_with_keys(&[1.0, 5.0]));
        assert_eq!((st.view_start_s, st.px_per_s), (0.0, DEFAULT_PX_PER_S));
    }
}
