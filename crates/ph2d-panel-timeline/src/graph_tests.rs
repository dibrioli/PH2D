//! Unit tests for [`super`] (`graph.rs`) — extracted to a sibling module
//! (`#[path]`) so the band/handle source stays under the 600-LOC panel cap.
//! Pure relocation of the `#[cfg(test)] mod tests` block — no test changed.
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

use ph2d_editor_core::interaction::{GestureMods, GesturePhase, TimelineGesture, TimelineHitKind};
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
    // (hx, hy) = (0.25, 0.5); the IN handle keeps the TANGENT it was drawn
    // at (flat, for a Hold), not the linear default.
    let want = Interp::bezier(0.25, 0.5, 2.0 / 3.0, 0.0);
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

/// Drive a full out-handle drag in SPEED mode to pointer `y`, returning intents.
/// The `x` is irrelevant to a speed edit (only `y = velocity` moves the tangent);
/// `band` maps velocity ↔ pixels.
fn drag_out_handle_speed(interp: Interp, band: &Band, y: f32) -> Vec<TimelineIntent> {
    let tr = track(interp);
    let mut st = TimelinePanelState {
        speed_view: true,
        ..TimelinePanelState::default()
    };
    apply_handle_gesture(&mut st, 9, 1, 0, gesture(GesturePhase::Begin, 0.0, 0.0));
    apply_handle_gesture(&mut st, 9, 1, 0, gesture(GesturePhase::Update, 50.0, y));
    resolve_drag(&mut st, band, VIEW, &tr);
    apply_handle_gesture(&mut st, 9, 1, 0, gesture(GesturePhase::End, 50.0, y));
    resolve_drag(&mut st, band, VIEW, &tr);
    assert!(st.handle_drag.is_none(), "the drag closed itself");
    state::drain_intents()
}

#[test]
fn dragging_a_speed_handle_retunes_the_tangent_to_that_velocity() {
    // The speed-graph edit, end to end. Track 0→10 over 0..1 s ⇒ value rate = 10.
    // A band mapping velocity 0..40 over the 100 px row puts y = 50 at velocity 20
    // (44 − 0.5·48, with the 10 % pad). The resulting bézier's START slope must be
    // y1/x1 = velocity/rate = 2, and the influence x1 kept at the Linear handle's
    // 1/3 (speed edits change only the slope, not the timing).
    let band = Band::fit(ROW, Some((0.0, 40.0)));
    let got = drag_out_handle_speed(Interp::Linear, &band, 50.0);
    let Some(TimelineIntent::SetInterp {
        interp: Interp::Bezier { x1, y1, .. },
        ..
    }) = got
        .iter()
        .rev()
        .find(|i| matches!(i, TimelineIntent::SetInterp { .. }))
    else {
        panic!("no SetInterp in {got:?}")
    };
    let start_slope = y1 / x1;
    assert!(
        (start_slope - 2.0).abs() < 1e-9,
        "start slope for v=20 at rate 10 must be 2: {start_slope}"
    );
    assert!(
        (x1 - 1.0 / 3.0).abs() < 1e-9,
        "the influence x is kept, not moved: {x1}"
    );
}

#[test]
fn a_speed_drag_on_a_flat_segment_keeps_the_handle() {
    // v0 == v1: there is no velocity to scale, so the inverse declines and the
    // segment stays where it was — no spurious tangent change.
    let mut tr = track(Interp::bezier(0.3, 0.7, 0.6, 0.2));
    for k in &mut tr.keys {
        k.value = 5.0; // flatten: dv = 0
    }
    let band = Band::fit(ROW, Some((-10.0, 10.0)));
    let mut st = TimelinePanelState {
        speed_view: true,
        ..TimelinePanelState::default()
    };
    apply_handle_gesture(&mut st, 9, 1, 0, gesture(GesturePhase::Begin, 0.0, 0.0));
    apply_handle_gesture(&mut st, 9, 1, 0, gesture(GesturePhase::Update, 50.0, 10.0));
    resolve_drag(&mut st, &band, VIEW, &tr);
    let got = state::drain_intents();
    assert!(
        got.iter().any(|i| matches!(
            i,
            TimelineIntent::SetInterp { interp: Interp::Bezier { y1, .. }, .. } if (*y1 - 0.7).abs() < 1e-9
        )),
        "a flat segment's out handle y stays 0.7: {got:?}"
    );
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
fn handles_sit_on_the_curve_not_on_the_chord() {
    // A Hold segment is flat, so its handles are flat. Drawing the LINEAR
    // handles here put the dots on the straight chord, visibly off the curve
    // until the first drag rebuilt them.
    assert_eq!(
        handle_pair(Interp::Hold),
        [(1.0 / 3.0, 0.0), (2.0 / 3.0, 0.0)]
    );
    assert_ne!(handle_pair(Interp::Hold), handle_pair(Interp::Linear));
}

/// Cubic InOut — the interpolation a fresh key gets.
fn cubic_in_out() -> Interp {
    Interp::Eased(ph2d_timeline::Easing::new(
        ph2d_timeline::EasingFamily::Cubic,
        ph2d_timeline::EasingMode::InOut,
    ))
}

#[test]
fn an_eased_segment_draws_its_handles_flat_like_its_ends() {
    let [(x1, y1), (x2, y2)] = handle_pair(cubic_in_out());
    assert!(y1.abs() < 1e-3, "leaves flat, y1 = {y1}");
    assert!((1.0 - y2).abs() < 1e-3, "arrives flat, y2 = {y2}");
    assert!((x1 - 1.0 / 3.0).abs() < 1e-9 && (x2 - 2.0 / 3.0).abs() < 1e-9);
}

#[test]
fn a_bezier_reports_its_own_control_points() {
    assert_eq!(
        handle_pair(Interp::bezier(0.1, 0.2, 0.3, 0.4)),
        [(0.1, 0.2), (0.3, 0.4)]
    );
}

// ── band_for: the freeze that keeps a drag under the cursor ──────────

#[test]
fn an_idle_row_refits_its_band_to_what_it_draws() {
    let mut st = TimelinePanelState::default();
    let a = band_for(&mut st, ROW, 9, Some((0.0, 10.0)));
    assert_eq!(a.range(), (-1.0, 11.0));
    // Values changed (a key was edited elsewhere): the fit follows them.
    let b = band_for(&mut st, ROW, 9, Some((0.0, 100.0)));
    assert_eq!(b.range(), (-10.0, 110.0));
}

#[test]
fn an_anchor_drag_freezes_the_band_against_the_values_it_is_changing() {
    // THE feedback loop: the drag raises the key values, `drawn_extent` grows,
    // and an unfrozen band would remap the pointer's y to a smaller value each
    // frame — the anchor would crawl out from under the cursor. Freeze on the
    // first paint; every later fit must be ignored.
    let mut st = TimelinePanelState {
        anchor_drag: Some(crate::state::AnchorDrag {
            target: 9,
            start: (0.0, 50.0),
            cur: (0.0, 50.0),
            base: vec![(1, 0.0)],
            applied_s: 0.0,
            applied_v: None,
            collapse_to: None,
            range: None,
            ending: false,
        }),
        ..TimelinePanelState::default()
    };
    let first = band_for(&mut st, ROW, 9, Some((0.0, 10.0)));
    assert_eq!(first.range(), (-1.0, 11.0), "the first paint still fits");
    // The drag pushed the top key to 100; the band must NOT follow.
    let later = band_for(&mut st, ROW, 9, Some((0.0, 100.0)));
    assert_eq!(later.range(), (-1.0, 11.0), "the band is frozen");
    assert_eq!(
        later.value(50.0),
        first.value(50.0),
        "y still means what it did"
    );
    // A different track's band is untouched by this drag.
    let other = band_for(&mut st, ROW, 77, Some((0.0, 100.0)));
    assert_eq!(other.range(), (-10.0, 110.0));
}

#[test]
fn a_handle_drag_freezes_its_own_bands_range_too() {
    let mut st = TimelinePanelState {
        handle_drag: Some(HandleDrag {
            target: 9,
            key: 1,
            which: 0,
            x: 0.0,
            y: 0.0,
            range: None,
            ending: false,
        }),
        ..TimelinePanelState::default()
    };
    let first = band_for(&mut st, ROW, 9, Some((0.0, 10.0)));
    assert_eq!(st.handle_drag.expect("armed").range, Some(first.range()));
    let later = band_for(&mut st, ROW, 9, Some((-50.0, 50.0)));
    assert_eq!(later.range(), first.range());
}
