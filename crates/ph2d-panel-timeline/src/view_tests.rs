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
            buffer_ghost: None,
            pre: ph2d_timeline::Extrap::Hold,
            post: ph2d_timeline::Extrap::Hold,
            expr: None,
            keys: times
                .iter()
                .enumerate()
                .map(|(i, &t)| KeyView {
                    id: KeyId::new(i as u64),
                    t_seconds: t,
                    value: 0.0,
                    interp: Interp::Linear,
                    selected: false,
                    roving: false,
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

/// A stack: `lanes` strips, given as `(t_start, t_end, lead_in)`. The document
/// also carries keys at 0..2 — the ACTIVE CLIP's own duration, which is what the
/// fit used to frame in Arrange and is exactly the thing these gates must see it
/// stop framing.
fn snap_with_strips(strips: &[(f64, f64, f64)]) -> TimelineViewSnapshot {
    use ph2d_timeline::{LaneMode, LaneView, StripId, StripLoop, StripView};
    TimelineViewSnapshot {
        lanes: vec![LaneView {
            name: "L".into(),
            muted: false,
            weight: 1.0,
            mode: LaneMode::Override,
            strips: strips
                .iter()
                .enumerate()
                .map(|(i, &(t_start, t_end, lead_in))| StripView {
                    id: StripId(i as u64),
                    clip_name: "C".into(),
                    container: None,
                    t_start,
                    t_end,
                    blend_in: 0.0,
                    blend_out: 0.0,
                    lead_in,
                    lead_out: 0.0,
                    volta: 0.0,
                    ease_locked_in: false,
                    ease_locked_out: false,
                    curve_in: None,
                    curve_out: None,
                    seam: None,
                    loop_mode: StripLoop::Once,
                    speed: 1.0,
                    marks: [0.0; 4],
                })
                .collect(),
        }],
        ..snap_with_keys(&[0.0, 2.0])
    }
}

fn arranged() -> TimelinePanelState {
    TimelinePanelState {
        tab: crate::tab::Tab::Arrange,
        ..TimelinePanelState::default()
    }
}

/// **`F` in Arrange frames the whole set, not the first strip** (Enio,
/// 2026-07-16). The fixture is the trap itself: the active clip runs 0..2 and the
/// first strip plays it 1:1 from the top, so framing the KEYS produces a view
/// that looks perfectly deliberate — and leaves the strip at 8..10 off screen.
#[test]
fn fit_in_arrange_frames_every_strip_not_just_the_first() {
    let mut st = arranged();
    apply_fit(
        &mut st,
        800.0,
        &snap_with_strips(&[(0.0, 2.0), (8.0, 10.0)].map(|(a, b)| (a, b, 0.0))),
    );
    assert!(
        view_end(&st, 800.0) > 10.0,
        "the last strip must be in view, not just the first: end={}",
        view_end(&st, 800.0)
    );
    assert!(st.view_start_s <= 0.0, "and the first one still is");
}

/// The mirror, and it is what makes the gate above mean anything: the Keys tab
/// keeps framing the KEYS even with a stack on the document. Without this pair,
/// "always fit the strips" would pass.
#[test]
fn fit_in_keys_still_frames_the_keys_even_when_a_stack_exists() {
    let mut st = TimelinePanelState::default(); // Tab::Keys
    apply_fit(&mut st, 800.0, &snap_with_strips(&[(8.0, 10.0, 0.0)]));
    assert!(
        view_end(&st, 800.0) < 8.0,
        "the far strip is NOT what the Keys tab frames: end={}",
        view_end(&st, 800.0)
    );
}

/// **The outward lead-in is content.** It is lane time the entry reaches back
/// over, so a fit that started at `t_start` would crop the travel the artist
/// authored at the exact moment they asked to see everything.
#[test]
fn fit_in_arrange_includes_a_strips_outward_lead() {
    let mut st = arranged();
    apply_fit(&mut st, 800.0, &snap_with_strips(&[(6.0, 8.0, 2.0)]));
    assert!(
        st.view_start_s < 4.0,
        "the lead-in reaches back to t=4 and must be framed: start={}",
        st.view_start_s
    );
}

/// Arrange with nothing arranged still frames something — falling back to the
/// keys beats snapping to the default zoom, which reads as "F did nothing".
///
/// Asserted on the ZOOM, not on "are the keys visible": the default view happens
/// to span 6.7 s at 800 px, so a fit that quietly gave up would still show keys at
/// 1 and 5 and the gate would pass having proved nothing. The fitted zoom
/// (`800 / 4.4`) is a number only a real fit produces.
#[test]
fn fit_in_arrange_falls_back_to_the_keys_when_no_lane_holds_a_strip() {
    let mut st = arranged();
    apply_fit(&mut st, 800.0, &snap_with_keys(&[1.0, 5.0]));
    assert!(
        (st.px_per_s - 800.0 / 4.4).abs() < 1e-6,
        "the keys were FITTED, not merely left in a default view: {}",
        st.px_per_s
    );
    assert!(st.view_start_s < 1.0 && view_end(&st, 800.0) > 5.0);
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
