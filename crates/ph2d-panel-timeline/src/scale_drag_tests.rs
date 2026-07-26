//! Gates for the time-scale grip (crown-jewels §4). The engine (`scale_keys` /
//! `ScaleSelectedKeys`) is tested in `ph2d-timeline`/`ph2d-anim`; here we pin the
//! GESTURE: it streams `ScaleSelectedKeys` about the OPPOSITE edge, never a
//! `StretchStrip`, and the factor matches the drag geometry.

use super::*;
use ph2d_editor_core::interaction::{GestureMods, GesturePhase, TimelineGesture, TimelineHitKind};
use ph2d_host::PointerButton;
use ph2d_timeline::{AnimTarget, Interp, KeyId, KeyView, PropKind, TrackView};

/// A snapshot with one track carrying keys at `times`, all marked selected.
/// `frame_snap` off so `time_at` maps x→t linearly (the geometry is the point).
fn snap_selected(times: &[f64]) -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 60.0,
        frame_snap: false,
        tracks: vec![TrackView {
            target: AnimTarget::new(7),
            prop: PropKind::TranslationX,
            entity: 1,
            missing: false,
            buffer_ghost: None,
            pre: ph2d_timeline::Extrap::Hold,
            post: ph2d_timeline::Extrap::Hold,
            keys: times
                .iter()
                .enumerate()
                .map(|(i, &t)| KeyView {
                    id: KeyId::new(i as u64),
                    t_seconds: t,
                    value: 0.0,
                    interp: Interp::Linear,
                    selected: true,
                    roving: false,
                })
                .collect(),
        }],
        ..TimelineViewSnapshot::default()
    }
}

/// time_x = 100, px_per_s = 100 (1 s = 100 px), view_start = 0: x = 100 + t·100.
fn state_at_origin() -> TimelinePanelState {
    // Default already parks the view at t=0; named for the test's intent.
    TimelinePanelState::default()
}

fn grip(right: bool, phase: GesturePhase, x: f32) -> TimelineGesture {
    TimelineGesture {
        surface: ph2d_a11y::NodeId(0),
        kind: TimelineHitKind::SelectionTimeHandle { right },
        phase,
        x,
        y: 0.0,
        button: PointerButton::Primary,
        mods: GestureMods::default(),
    }
}

#[test]
fn selection_extent_needs_two_distinct_times() {
    assert_eq!(super::selection_extent(&snap_selected(&[])), None);
    assert_eq!(super::selection_extent(&snap_selected(&[1.0])), None);
    assert_eq!(
        super::selection_extent(&snap_selected(&[0.0, 1.0, 0.5])),
        Some((0.0, 1.0)),
        "the extent spans min..max of the selected times"
    );
}

#[test]
fn dragging_the_right_grip_to_double_the_span_scales_by_two_about_the_left_edge() {
    let mut st = state_at_origin();
    let s = snap_selected(&[0.0, 1.0]); // extent [0,1], pivot (right grip) = 0.
    let _ = state::drain_intents();
    // Right grip begins near the right edge (x for t=1 is 200) and drags to x=300
    // (t = 2): the span goes 1 -> 2, so factor 2 about the LEFT edge (t=0).
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::Begin, 209.0),
    );
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::Update, 300.0),
    );
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::End, 300.0),
    );
    let got = state::drain_intents();
    assert_eq!(got[0], TimelineIntent::BeginEdit);
    assert_eq!(*got.last().unwrap(), TimelineIntent::EndEdit);
    let scales: Vec<_> = got
        .iter()
        .filter_map(|i| match i {
            TimelineIntent::ScaleSelectedKeys {
                pivot_seconds,
                factor,
            } => Some((*pivot_seconds, *factor)),
            _ => None,
        })
        .collect();
    // The composed factor is the product of the streamed increments = 2, about 0.
    let composed: f64 = scales.iter().map(|(_, f)| f).product();
    assert!(
        scales.iter().all(|(p, _)| *p == 0.0),
        "pivot is the left edge"
    );
    assert!((composed - 2.0).abs() < 1e-9, "composed factor {composed}");
}

#[test]
fn dragging_the_left_grip_scales_about_the_right_edge() {
    let mut st = state_at_origin();
    let s = snap_selected(&[0.0, 2.0]); // pivot (left grip) = the RIGHT edge = 2.
    let _ = state::drain_intents();
    // Left grip drags the moving edge from t=0 (x=100) INWARD to t=1 (x=200):
    // span 2 -> 1 about the pivot at t=2, so factor 0.5. (Growing it would need a
    // negative time, which `time_at` floors at 0 — so the gate shrinks instead.)
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        false,
        grip(false, GesturePhase::Begin, 91.0),
    );
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        false,
        grip(false, GesturePhase::Update, 200.0),
    );
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        false,
        grip(false, GesturePhase::End, 200.0),
    );
    let scales: Vec<_> = state::drain_intents()
        .into_iter()
        .filter_map(|i| match i {
            TimelineIntent::ScaleSelectedKeys {
                pivot_seconds,
                factor,
            } => Some((pivot_seconds, factor)),
            _ => None,
        })
        .collect();
    let composed: f64 = scales.iter().map(|(_, f)| f).product();
    assert!(
        scales.iter().all(|(p, _)| *p == 2.0),
        "pivot is the right edge"
    );
    assert!((composed - 0.5).abs() < 1e-9, "composed factor {composed}");
}

#[test]
fn the_grip_streams_scale_never_a_strip_stretch() {
    // The whole point of the surface split: this gesture must NEVER emit a strip
    // edit, which would reach the precious fade. (Mutation: route this hit to
    // strip_drag in `interact` -> a StretchStrip appears here -> RED.)
    let mut st = state_at_origin();
    let s = snap_selected(&[0.0, 2.0]);
    let _ = state::drain_intents();
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::Begin, 300.0),
    );
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::Update, 500.0),
    );
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::End, 500.0),
    );
    let got = state::drain_intents();
    assert!(
        got.iter()
            .any(|i| matches!(i, TimelineIntent::ScaleSelectedKeys { .. })),
        "the grip scales the keys"
    );
    assert!(
        !got.iter()
            .any(|i| matches!(i, TimelineIntent::StretchStrip { .. })),
        "a key-scale grip must never emit a strip stretch (that is the fade's surface)"
    );
}

#[test]
fn two_updates_stream_incremental_factors_that_compose_to_the_target() {
    // The streamed factors must be INCREMENTAL (each frame's `want / applied`), so
    // their product equals the drag's final target — emitting the absolute `want`
    // each frame would compound (2 then 3 -> a 6x scale, not 3x). Two distinct
    // Update positions are what separates the two. (Mutation: emit `want` -> RED.)
    let mut st = state_at_origin();
    let s = snap_selected(&[0.0, 1.0]); // extent 1, pivot (right grip) = 0.
    let _ = state::drain_intents();
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::Begin, 200.0),
    );
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::Update, 300.0),
    ); // t=2
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::Update, 400.0),
    ); // t=3
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::End, 400.0),
    );
    let composed: f64 = state::drain_intents()
        .iter()
        .filter_map(|i| match i {
            TimelineIntent::ScaleSelectedKeys { factor, .. } => Some(*factor),
            _ => None,
        })
        .product();
    assert!(
        (composed - 3.0).abs() < 1e-9,
        "the incremental factors compose to the final target 3x, got {composed}"
    );
}

#[test]
fn the_box_carries_the_markers_in_its_span_and_leaves_the_others() {
    // The time-scale box retimes its whole SPAN, so a marker inside [lo, hi] scales
    // with the keys (same pivot, same factor, same bracket) and one past `hi` is
    // left alone. (Mutation: capture ALL markers / drop the filter -> index 2
    // appears -> RED; capture NONE -> no ScaleMarkers -> RED.)
    let mut st = state_at_origin();
    let mut s = snap_selected(&[0.0, 2.0]); // box span [0,2], right grip pivot = 0.
    s.markers = vec![
        (0.5, "a".into(), None),
        (1.5, "b".into(), None),
        (3.0, "c".into(), None), // OUTSIDE the box
    ];
    let _ = state::drain_intents();
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::Begin, 300.0),
    );
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::Update, 400.0),
    );
    let got = state::drain_intents();
    let keys = got.iter().find_map(|i| match i {
        TimelineIntent::ScaleSelectedKeys {
            pivot_seconds,
            factor,
        } => Some((*pivot_seconds, *factor)),
        _ => None,
    });
    let markers = got.iter().find_map(|i| match i {
        TimelineIntent::ScaleMarkers {
            indices,
            pivot_seconds,
            factor,
        } => Some((indices.clone(), *pivot_seconds, *factor)),
        _ => None,
    });
    let (kp, kf) = keys.expect("the keys scale");
    let (idx, mp, mf) = markers.expect("the markers in the span scale with the keys");
    assert_eq!(
        idx,
        vec![0, 1],
        "only markers 0 and 1 (inside [0,2]) — not 2 at t=3"
    );
    assert_eq!(mp, kp, "markers scale about the same pivot as the keys");
    assert!(
        (mf - kf).abs() < 1e-9,
        "markers scale by the same factor as the keys"
    );
}

#[test]
fn a_selection_with_no_markers_in_its_span_emits_no_marker_scale() {
    let mut st = state_at_origin();
    let mut s = snap_selected(&[0.0, 1.0]);
    s.markers = vec![(5.0, "far".into(), None)]; // outside [0,1]
    let _ = state::drain_intents();
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::Begin, 200.0),
    );
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::Update, 300.0),
    );
    assert!(
        !state::drain_intents()
            .iter()
            .any(|i| matches!(i, TimelineIntent::ScaleMarkers { .. })),
        "no marker in the span -> no ScaleMarkers"
    );
}

#[test]
fn a_plain_click_on_a_grip_does_nothing_but_close_the_bracket() {
    let mut st = state_at_origin();
    let s = snap_selected(&[0.0, 1.0]);
    let _ = state::drain_intents();
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::Begin, 200.0),
    );
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::Click, 200.0),
    );
    assert_eq!(
        state::drain_intents(),
        vec![TimelineIntent::BeginEdit, TimelineIntent::EndEdit],
        "a grip tap is a no-op inside an empty bracket that commits no step"
    );
    assert!(st.scale_drag.is_none());
}

#[test]
fn the_left_grip_stays_clear_of_the_label_splitter() {
    // A selection starting at Frame 0 (x_lo == time_x) must NOT put its left grip
    // on the divider at time_x, which is itself a drag target (panel resize) — the
    // grip's whole hit rect stays right of the splitter's grab strip. (Mutation:
    // drop the clamp in `grip_bar_x` -> the grip lands left of time_x -> RED.)
    let time_x = 100.0;
    let right_edge = 400.0;
    // x_lo == time_x (Frame 0), a wide selection so the right grip has room.
    let left = super::grip_bar_x(false, time_x, 300.0, time_x, right_edge);
    let hit_left = left - super::HIT_PAD;
    assert!(
        hit_left >= time_x + super::geom::SPLIT_GRIP,
        "left grip hit ({hit_left}) must clear the splitter grab strip at {}",
        time_x + super::geom::SPLIT_GRIP
    );
    // The right grip of a selection near the far edge stays inside the area.
    let r = super::grip_bar_x(true, time_x, right_edge, time_x, right_edge);
    assert!(
        r + super::HANDLE_W + super::HIT_PAD <= right_edge,
        "right grip must stay inside the time area"
    );
    // A mid-timeline selection is unclamped (grips sit OUTSIDE the extent).
    let mid_l = super::grip_bar_x(false, 180.0, 260.0, time_x, right_edge);
    assert!(
        mid_l < 180.0,
        "a mid selection's left grip sits outside-left"
    );
}

#[test]
fn a_frame_that_did_not_move_the_span_emits_nothing() {
    let mut st = state_at_origin();
    let s = snap_selected(&[0.0, 1.0]);
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::Begin, 200.0),
    );
    let _ = state::drain_intents();
    // The cursor lands exactly on the current edge (t=1, x=200): factor 1, no emit.
    super::apply(
        &mut st,
        100.0,
        100.0,
        &s,
        true,
        grip(true, GesturePhase::Update, 200.0),
    );
    assert_eq!(state::drain_intents(), vec![]);
}
