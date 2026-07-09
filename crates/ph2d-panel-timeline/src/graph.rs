//! Graph editor (W3) — an expanded track row's curve, anchors and bézier
//! handles, painted in the band under its dope-sheet strip (theatre.js's
//! expand-per-track model, not a hidden global mode: B0.P1).
//!
//! The curve itself comes from [`ph2d_timeline::sample_keys`], which is pinned
//! bit-for-bit to the real `Track::sample` by a golden test — the polyline you
//! drag IS the animation that plays (B0.P4).
//!
//! **Handles** are the two control points of the segment's
//! [`Interp::Bezier`](ph2d_timeline::Interp) in normalized `[0, 1]²` timing
//! space, drawn in the `(time, value)` plane. A `Hold`/`Eased` segment has no
//! two-handle form, so its handles paint at the *linear* positions: dragging one
//! converts the segment to `Bezier` (the sanctioned upgrade path). Handles show
//! only for segments touching a **selected** key, so a dense track stays legible.
//!
//! A drag streams one `SetInterp` per frame inside a `BeginEdit`/`EndEdit`
//! bracket, so the scene follows the cursor live and one Ctrl+Z undoes the whole
//! gesture.

use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{Interp, TimelineIntent, TrackView, handle_coords};

use crate::state::{self, HandleDrag, TimelinePanelState};

/// Default height of the curve band added below an expanded row's dope-sheet
/// strip. The live height is panel state (draggable) — see `state.graph_h`.
pub(crate) const GRAPH_H_DEFAULT: f32 = 132.0; // LITERAL-PX-OK: default graph band height
/// Shortest a graph band may be dragged (a curve still readable).
const GRAPH_H_MIN: f32 = 64.0; // LITERAL-PX-OK: min graph band height
/// Tallest a graph band may be dragged.
const GRAPH_H_MAX: f32 = 640.0; // LITERAL-PX-OK: max graph band height

/// Hold a dragged graph height inside its bounds.
pub(crate) fn clamp_graph_h(h: f32) -> f32 {
    h.clamp(GRAPH_H_MIN, GRAPH_H_MAX) // CLAMP-OK: const bounds, min<max, non-NaN
}
/// Breathing room above and below the fitted range, as a fraction of it.
const V_PAD_FRAC: f64 = 0.1; // LITERAL-PX-OK: vertical fit margin, fraction of the drawn range
/// Half-height of the fitted range when everything drawn sits at one value (a
/// flat track has no range to take a fraction of).
const V_PAD_FLAT: f64 = 0.5; // LITERAL-PX-OK: fallback half-range for a flat track

/// The time axis, shared with the ruler and the dope-sheet lanes.
#[derive(Clone, Copy)]
pub(crate) struct TimeView {
    /// Screen x where `view_start` maps.
    pub time_x: f32,
    /// Right edge of the time area.
    pub right: f32,
    /// Seconds at `time_x`.
    pub view_start: f64,
    /// Zoom.
    pub px_per_s: f64,
}

impl TimeView {
    pub(crate) fn x(self, t: f64) -> f32 {
        self.time_x + ((t - self.view_start) * self.px_per_s) as f32
    }
    pub(crate) fn t(self, x: f32) -> f64 {
        self.view_start + f64::from(x - self.time_x) / self.px_per_s
    }
}

/// One expanded row's value↔pixel mapping: the fitted (and padded) value range
/// projected onto the band's rect, `v_max` at the top.
#[derive(Clone, Copy)]
pub(crate) struct Band {
    pub rect: Rect,
    pub v_min: f64,
    pub v_max: f64,
}

impl Band {
    /// Fit `extent` (what the row actually DRAWS — see
    /// [`ph2d_timeline::drawn_extent`]) into `rect`, with a margin. `None` (an
    /// empty track) gets a symmetric unit window so the mapping stays finite.
    pub(crate) fn fit(rect: Rect, extent: Option<(f32, f32)>) -> Self {
        let (lo, hi) = extent.map_or((0.0, 0.0), |(a, b)| (f64::from(a), f64::from(b)));
        let range = hi - lo;
        let pad = if range > 0.0 {
            range * V_PAD_FRAC
        } else {
            V_PAD_FLAT
        };
        Self::from_range(rect, lo - pad, hi + pad)
    }

    /// A band over an explicit value range — used to hold the mapping still for
    /// the length of a handle drag (see `graph_paint::paint_track`).
    pub(crate) fn from_range(rect: Rect, v_min: f64, v_max: f64) -> Self {
        Self { rect, v_min, v_max }
    }

    /// The value range this band maps, for freezing it across frames.
    pub(crate) fn range(&self) -> (f64, f64) {
        (self.v_min, self.v_max)
    }

    pub(crate) fn y(&self, v: f64) -> f32 {
        let span = self.v_max - self.v_min;
        let f = if span > 0.0 {
            (self.v_max - v) / span
        } else {
            0.5
        };
        self.rect.y + (f as f32) * self.rect.h
    }

    pub(crate) fn value(&self, y: f32) -> f64 {
        if self.rect.h <= 0.0 {
            return self.v_min;
        }
        let f = f64::from((y - self.rect.y) / self.rect.h);
        self.v_max - f * (self.v_max - self.v_min)
    }
}

/// Interpret one gesture on a bézier handle. Geometry-free: the pointer position
/// is only *recorded* here (the band's value↔pixel mapping lives in `paint`), and
/// [`resolve_drag`] turns it into a `SetInterp` once the band is resolved.
pub(crate) fn apply_handle_gesture(
    state: &mut TimelinePanelState,
    target: u64,
    key: u64,
    which: u8,
    g: ph2d_editor_core::interaction::TimelineGesture,
) {
    use ph2d_editor_core::interaction::GesturePhase as P;
    match g.phase {
        P::Begin => {
            // Open the undo bracket now: every frame's SetInterp joins this step.
            state::push_intent(TimelineIntent::BeginEdit);
            state.handle_drag = Some(HandleDrag {
                target,
                key,
                which,
                x: g.x,
                y: g.y,
                range: None,
                ending: false,
            });
        }
        P::Update => {
            if let Some(d) = state.handle_drag.as_mut() {
                (d.x, d.y) = (g.x, g.y);
            }
        }
        P::End => {
            if let Some(d) = state.handle_drag.as_mut() {
                (d.x, d.y, d.ending) = (g.x, g.y, true);
            }
        }
        // A tap on a handle moved nothing: close the bracket (it commits no step
        // because the document did not change) and drop the drag.
        P::Click | P::DoubleClick => {
            state.handle_drag = None;
            state::push_intent(TimelineIntent::EndEdit);
        }
    }
}

/// The segment's two handles in normalized timing space. `Hold`/`Eased` have no
/// two-handle form, so they show the LINEAR positions — dragging one is what
/// converts the segment to `Bezier`.
pub(crate) fn handle_pair(interp: Interp) -> [(f64, f64); 2] {
    let ((a, b), (c, d)) = interp.handles().unwrap_or(Interp::LINEAR_HANDLES);
    [(a, b), (c, d)]
}

/// Turn this frame of an in-flight handle drag into a `SetInterp`, now that the
/// band's value↔pixel mapping is known. Closes the undo bracket on the last one.
pub(crate) fn resolve_drag(
    state: &mut TimelinePanelState,
    band: &Band,
    view: TimeView,
    track: &TrackView,
) {
    let Some(d) = state.handle_drag else {
        return;
    };
    if d.target != track.target.get() {
        return;
    }
    let Some(i) = track.keys.iter().position(|k| k.id.get() == d.key) else {
        return;
    };
    let Some(k1) = track.keys.get(i + 1) else {
        return; // the last key owns no segment
    };
    let k0 = &track.keys[i];
    let (t, v) = (view.t(d.x), band.value(d.y));
    let (hx, hy) = handle_coords(
        k0.t_seconds,
        f64::from(k0.value),
        k1.t_seconds,
        f64::from(k1.value),
        t,
        v,
    );
    // A flat segment has no representable handle `y` — keep the one it had
    // rather than snapping the handle onto the line (see `graph::handle_coords`).
    let old = handle_pair(k0.interp);
    let hy = hy.unwrap_or(old[d.which as usize].1);
    let interp = if d.which == 0 {
        k0.interp.with_out_handle(hx, hy)
    } else {
        k0.interp.with_in_handle(hx, hy)
    };
    state::push_intent(TimelineIntent::SetInterp {
        target: track.target,
        key: k0.id,
        interp,
    });
    if d.ending {
        state::push_intent(TimelineIntent::EndEdit);
        state.handle_drag = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BAND: Rect = Rect::new(100.0, 50.0, 400.0, 100.0);

    fn key(t: f64, v: f32) -> ph2d_timeline::KeyView {
        use ph2d_timeline::KeyView;
        KeyView {
            id: ph2d_timeline::KeyId::new(0),
            t_seconds: t,
            value: v,
            interp: Interp::Linear,
            selected: false,
        }
    }

    #[test]
    fn the_band_puts_the_highest_value_at_the_top() {
        let b = Band::fit(BAND, Some((0.0, 10.0)));
        assert!(b.y(10.0) < b.y(0.0), "bigger value = smaller y");
        // 25% padding each side of the 0..10 range.
        // 10% padding each side of the 0..10 range.
        assert_eq!((b.v_min, b.v_max), (-1.0, 11.0));
        assert!((b.y(11.0) - BAND.y).abs() < 1e-4);
        assert!((b.y(-1.0) - (BAND.y + BAND.h)).abs() < 1e-4);
    }

    #[test]
    fn a_flat_track_gets_a_finite_band_instead_of_dividing_by_zero() {
        let b = Band::fit(BAND, Some((4.0, 4.0)));
        assert_eq!((b.v_min, b.v_max), (3.5, 4.5));
        assert!(b.y(4.0).is_finite());
        assert!((b.y(4.0) - (BAND.y + BAND.h * 0.5)).abs() < 1e-4);
    }

    #[test]
    fn an_empty_track_still_maps_finitely() {
        let b = Band::fit(BAND, None);
        assert_eq!((b.v_min, b.v_max), (-0.5, 0.5));
        assert!(b.value(BAND.y).is_finite());
    }

    #[test]
    fn pixels_round_trip_back_to_values() {
        let b = Band::fit(BAND, Some((-3.0, 7.0)));
        for v in [-3.0, 0.0, 2.5, 7.0] {
            assert!((b.value(b.y(v)) - v).abs() < 1e-4, "{v}");
        }
    }

    #[test]
    fn a_zero_height_band_never_divides_by_zero() {
        let b = Band::fit(Rect::new(0.0, 0.0, 100.0, 0.0), Some((1.0, 1.0)));
        assert!(b.value(0.0).is_finite());
    }

    #[test]
    fn time_maps_both_ways() {
        let view = TimeView {
            time_x: 200.0,
            right: 800.0,
            view_start: 1.0,
            px_per_s: 120.0,
        };
        assert_eq!(view.x(1.0), 200.0);
        assert_eq!(view.x(2.0), 320.0);
        assert!((view.t(320.0) - 2.0).abs() < 1e-9);
    }

    // ── The handle drag, end to end ──────────────────────────────────────

    use ph2d_editor_core::interaction::{
        GestureMods, GesturePhase, TimelineGesture, TimelineHitKind,
    };
    use ph2d_host::PointerButton;

    /// A track from `t = 0, v = 0` to `t = 1, v = 10`, both keys selected.
    fn track(interp: Interp) -> TrackView {
        let mut k0 = key(0.0, 0.0);
        k0.interp = interp;
        k0.id = ph2d_timeline::KeyId::new(1);
        let mut k1 = key(1.0, 10.0);
        k1.id = ph2d_timeline::KeyId::new(2);
        k0.selected = true;
        k1.selected = true;
        TrackView {
            target: ph2d_timeline::AnimTarget::new(9),
            prop: ph2d_timeline::PropKind::TranslationX,
            entity: 1,
            missing: false,
            keys: vec![k0, k1],
        }
    }

    /// 100 px/s from x = 0, and a 100 px band starting at y = 0.
    const VIEW: TimeView = TimeView {
        time_x: 0.0,
        right: 400.0,
        view_start: 0.0,
        px_per_s: 100.0,
    };
    const ROW: Rect = Rect::new(0.0, 0.0, 400.0, 100.0);

    fn gesture(phase: GesturePhase, x: f32, y: f32) -> TimelineGesture {
        TimelineGesture {
            surface: ph2d_a11y::NodeId(0),
            kind: TimelineHitKind::CurveHandle {
                target: 9,
                key: 1,
                which: 0,
            },
            phase,
            x,
            y,
            button: PointerButton::Primary,
            mods: GestureMods::default(),
        }
    }

    /// Drive one full drag of the out handle to `(x, y)` and return the intents.
    fn drag_out_handle(interp: Interp, x: f32, y: f32) -> Vec<TimelineIntent> {
        let tr = track(interp);
        let band = Band::fit(ROW, Some((0.0, 10.0)));
        let mut st = TimelinePanelState::default();
        apply_handle_gesture(&mut st, 9, 1, 0, gesture(GesturePhase::Begin, 0.0, 0.0));
        apply_handle_gesture(&mut st, 9, 1, 0, gesture(GesturePhase::Update, x, y));
        resolve_drag(&mut st, &band, VIEW, &tr);
        apply_handle_gesture(&mut st, 9, 1, 0, gesture(GesturePhase::End, x, y));
        resolve_drag(&mut st, &band, VIEW, &tr);
        assert!(st.handle_drag.is_none(), "the drag closed itself");
        state::drain_intents()
    }

    #[test]
    fn dragging_an_out_handle_converts_a_hold_segment_to_bezier() {
        // Band fits 0..10 with 10% padding, so the midpoint v = 5 lands at
        // y = 50 either way. x = 25 px is t = 0.25 s.
        let got = drag_out_handle(Interp::Hold, 25.0, 50.0);
        // (hx, hy) = (0.25, 0.5); the IN handle keeps its linear default.
        let want = Interp::bezier(0.25, 0.5, 2.0 / 3.0, 2.0 / 3.0);
        assert!(
            got.iter().any(|i| matches!(
                i,
                TimelineIntent::SetInterp { interp, .. } if *interp == want
            )),
            "a Hold segment must upgrade to Bezier on the first drag: {got:?}"
        );
    }

    #[test]
    fn the_whole_drag_is_one_undo_bracket() {
        let got = drag_out_handle(Interp::Linear, 25.0, 50.0);
        assert_eq!(got.first(), Some(&TimelineIntent::BeginEdit));
        assert_eq!(got.last(), Some(&TimelineIntent::EndEdit));
        let edits = got
            .iter()
            .filter(|i| matches!(i, TimelineIntent::SetInterp { .. }))
            .count();
        assert_eq!(edits, 2, "one SetInterp per frame, both inside the bracket");
    }

    #[test]
    fn dragging_past_the_next_key_pins_the_handle_at_the_segment_end() {
        // x = 900 px is t = 9 s, far past k1 at t = 1. A non-monotone timing
        // function has no single solution, so x clamps.
        let got = drag_out_handle(Interp::Linear, 900.0, 50.0);
        let Some(TimelineIntent::SetInterp {
            interp: Interp::Bezier { x1, .. },
            ..
        }) = got
            .iter()
            .find(|i| matches!(i, TimelineIntent::SetInterp { .. }))
        else {
            panic!("no SetInterp in {got:?}")
        };
        assert_eq!(*x1, 1.0);
    }

    #[test]
    fn dragging_above_the_keys_overshoots_instead_of_clamping() {
        // y = 0 is v = 11, past the segment's end value of 10 ⇒ hy = 1.1.
        let got = drag_out_handle(Interp::Linear, 50.0, 0.0);
        let Some(TimelineIntent::SetInterp {
            interp: Interp::Bezier { y1, .. },
            ..
        }) = got
            .iter()
            .find(|i| matches!(i, TimelineIntent::SetInterp { .. }))
        else {
            panic!("no SetInterp in {got:?}")
        };
        assert!(*y1 > 1.0, "overshoot must survive: y1 = {y1}");
    }

    #[test]
    fn a_flat_segment_keeps_the_handle_y_it_already_had() {
        // v0 == v1: the normalized value axis is degenerate. Dragging must edit
        // the timing (x) without snapping y onto the line.
        let mut tr = track(Interp::bezier(0.4, 0.9, 0.6, 0.1));
        tr.keys[1].value = 0.0;
        let band = Band::fit(ROW, Some((0.0, 0.0)));
        let mut st = TimelinePanelState::default();
        apply_handle_gesture(&mut st, 9, 1, 0, gesture(GesturePhase::Begin, 0.0, 0.0));
        apply_handle_gesture(&mut st, 9, 1, 0, gesture(GesturePhase::End, 10.0, 90.0));
        resolve_drag(&mut st, &band, VIEW, &tr);
        let got = state::drain_intents();
        let Some(TimelineIntent::SetInterp {
            interp: Interp::Bezier { x1, y1, .. },
            ..
        }) = got
            .iter()
            .find(|i| matches!(i, TimelineIntent::SetInterp { .. }))
        else {
            panic!("no SetInterp in {got:?}")
        };
        assert!((*x1 - 0.1).abs() < 1e-9, "x still tracks the pointer");
        assert_eq!(*y1, 0.9, "y kept, not reset to 0");
    }

    #[test]
    fn the_last_key_owns_no_segment_and_cannot_be_dragged() {
        let tr = track(Interp::Linear);
        let band = Band::fit(ROW, Some((0.0, 10.0)));
        let mut st = TimelinePanelState::default();
        // key 2 is the final key.
        apply_handle_gesture(&mut st, 9, 2, 0, gesture(GesturePhase::Begin, 0.0, 0.0));
        apply_handle_gesture(&mut st, 9, 2, 0, gesture(GesturePhase::End, 25.0, 50.0));
        resolve_drag(&mut st, &band, VIEW, &tr);
        let got = state::drain_intents();
        assert!(
            !got.iter()
                .any(|i| matches!(i, TimelineIntent::SetInterp { .. })),
            "{got:?}"
        );
    }

    #[test]
    fn a_drag_on_another_track_is_ignored_by_this_band() {
        let tr = track(Interp::Linear);
        let band = Band::fit(ROW, Some((0.0, 10.0)));
        let mut st = TimelinePanelState::default();
        apply_handle_gesture(&mut st, 77, 1, 0, gesture(GesturePhase::Begin, 0.0, 0.0));
        resolve_drag(&mut st, &band, VIEW, &tr);
        assert!(st.handle_drag.is_some(), "the drag belongs to track 77");
        assert_eq!(state::drain_intents(), vec![TimelineIntent::BeginEdit]);
    }

    #[test]
    fn a_tap_on_a_handle_closes_the_bracket_and_edits_nothing() {
        let mut st = TimelinePanelState::default();
        apply_handle_gesture(&mut st, 9, 1, 0, gesture(GesturePhase::Begin, 5.0, 5.0));
        apply_handle_gesture(&mut st, 9, 1, 0, gesture(GesturePhase::Click, 5.0, 5.0));
        assert!(st.handle_drag.is_none());
        assert_eq!(
            state::drain_intents(),
            vec![TimelineIntent::BeginEdit, TimelineIntent::EndEdit],
            "an empty bracket commits no undo step"
        );
    }

    #[test]
    fn hold_and_eased_segments_show_the_linear_handles_to_drag_from() {
        // Nothing to grab otherwise — this is the documented upgrade path.
        assert_eq!(
            handle_pair(Interp::Hold),
            [(1.0 / 3.0, 1.0 / 3.0), (2.0 / 3.0, 2.0 / 3.0)]
        );
        assert_eq!(handle_pair(Interp::Linear), handle_pair(Interp::Hold));
        assert_eq!(
            handle_pair(Interp::bezier(0.1, 0.2, 0.3, 0.4)),
            [(0.1, 0.2), (0.3, 0.4)]
        );
    }
}
