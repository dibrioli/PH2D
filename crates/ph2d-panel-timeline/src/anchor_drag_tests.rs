//! Unit tests for [`super`] (`anchor_drag.rs`) — extracted to a sibling module
//! (`#[path]`) so the gesture source stays under the 600-LOC panel cap. Pure
//! relocation of the `#[cfg(test)] mod tests` block — no test changed.
use super::*;
use ph2d_editor_core::interaction::{GestureMods, TimelineHitKind};
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerButton;
use ph2d_timeline::{AnimTarget, Interp, KeyId, KeyView, PropKind, TrackView};

const TARGET: u64 = 9;
/// A 100 px band; `Band::fit` pads 0..10 to -1..11, so 12 value units span
/// 100 px — one pixel is 0.12.
const ROW: Rect = Rect::new(0.0, 0.0, 400.0, 100.0);

fn band() -> Band {
    Band::fit(ROW, Some((0.0, 10.0)))
}

/// Two keys at `t = 0, v = 0` and `t = 1, v = 10`; `selected` per argument.
fn snap(sel0: bool, sel1: bool) -> TimelineViewSnapshot {
    let mk = |id: u64, t: f64, v: f32, selected: bool| KeyView {
        id: KeyId::new(id),
        t_seconds: t,
        value: v,
        interp: Interp::Linear,
        selected,
        roving: false,
    };
    TimelineViewSnapshot {
        fps: 60.0,
        frame_snap: true,
        tracks: vec![TrackView {
            target: AnimTarget::new(TARGET),
            prop: PropKind::TranslationX,
            entity: 1,
            missing: false,
            keys: vec![mk(1, 0.0, 0.0, sel0), mk(2, 1.0, 10.0, sel1)],
        }],
        ..TimelineViewSnapshot::default()
    }
}

fn track_of(snap: &TimelineViewSnapshot) -> TrackView {
    snap.tracks[0].clone()
}

fn gesture(phase: GesturePhase, x: f32, y: f32, shift: bool) -> TimelineGesture {
    TimelineGesture {
        surface: ph2d_a11y::NodeId(0),
        kind: TimelineHitKind::CurveAnchor {
            target: TARGET,
            key: 1,
        },
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

/// Every `SetKeyValue` in `got`, as `(key, value)`.
fn values(got: &[TimelineIntent]) -> Vec<(u64, f32)> {
    got.iter()
        .filter_map(|i| match i {
            TimelineIntent::SetKeyValue {
                key,
                value: AnimValue::Float(v),
                ..
            } => Some((key.get(), *v)),
            _ => None,
        })
        .collect()
}

#[test]
fn dragging_an_anchor_up_raises_its_value_and_brackets_the_drag() {
    // 25 px up over a band where 100 px spans 12 units ⇒ +3.0.
    let s = snap(false, false);
    let tr = track_of(&s);
    let mut st = TimelinePanelState::default();
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Begin, 0.0, 50.0, false),
    );
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Update, 0.0, 25.0, false),
    );
    resolve_drag(&mut st, &band(), &tr);
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::End, 0.0, 25.0, false),
    );
    resolve_drag(&mut st, &band(), &tr);

    let got = state::drain_intents();
    assert_eq!(got.first(), Some(&TimelineIntent::BeginEdit));
    assert_eq!(got.last(), Some(&TimelineIntent::EndEdit));
    let vals = values(&got);
    assert_eq!(vals.len(), 1, "only the pressed key: {got:?}");
    assert_eq!(vals[0].0, 1);
    assert!((vals[0].1 - 3.0).abs() < 1e-4, "{vals:?}");
    assert!(st.anchor_drag.is_none(), "the drag closed itself");
}

#[test]
fn a_frame_that_did_not_move_vertically_emits_no_value() {
    let s = snap(false, false);
    let tr = track_of(&s);
    let mut st = TimelinePanelState::default();
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Begin, 0.0, 50.0, false),
    );
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Update, 0.0, 25.0, false),
    );
    resolve_drag(&mut st, &band(), &tr);
    let _ = state::drain_intents();
    // Same y, new x: the value is already where it should be.
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Update, 30.0, 25.0, false),
    );
    resolve_drag(&mut st, &band(), &tr);
    let got = state::drain_intents();
    assert!(values(&got).is_empty(), "no repeat SetKeyValue: {got:?}");
}

#[test]
fn the_value_is_rebuilt_from_the_base_so_a_slow_drag_cannot_drift() {
    // Land on the same y in one jump and in twenty small steps: identical.
    let s = snap(false, false);
    let tr = track_of(&s);
    let one_shot = {
        let mut st = TimelinePanelState::default();
        apply_gesture(
            &mut st,
            100.0,
            &s,
            TARGET,
            1,
            gesture(GesturePhase::Begin, 0.0, 90.0, false),
        );
        apply_gesture(
            &mut st,
            100.0,
            &s,
            TARGET,
            1,
            gesture(GesturePhase::Update, 0.0, 10.0, false),
        );
        resolve_drag(&mut st, &band(), &tr);
        values(&state::drain_intents())
    };
    let crawled = {
        let mut st = TimelinePanelState::default();
        apply_gesture(
            &mut st,
            100.0,
            &s,
            TARGET,
            1,
            gesture(GesturePhase::Begin, 0.0, 90.0, false),
        );
        for i in 1..=20 {
            let y = 90.0 - 4.0 * i as f32;
            apply_gesture(
                &mut st,
                100.0,
                &s,
                TARGET,
                1,
                gesture(GesturePhase::Update, 0.0, y, false),
            );
            resolve_drag(&mut st, &band(), &tr);
        }
        let got = state::drain_intents();
        vec![*values(&got).last().expect("a value landed")]
    };
    assert_eq!(one_shot, crawled, "an accumulated f32 would have drifted");
}

#[test]
fn dragging_a_selected_anchor_retunes_the_whole_group_on_that_track() {
    let s = snap(true, true);
    let tr = track_of(&s);
    let mut st = TimelinePanelState::default();
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Begin, 0.0, 50.0, false),
    );
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Update, 0.0, 25.0, false),
    );
    resolve_drag(&mut st, &band(), &tr);
    let got = state::drain_intents();
    assert!(
        !got.iter()
            .any(|i| matches!(i, TimelineIntent::SelectSingle(_))),
        "a press on a selected anchor preserves the group: {got:?}"
    );
    let vals = values(&got);
    assert_eq!(vals.len(), 2);
    // Both rise by the same +3.0, each from its OWN base.
    assert!(
        (vals[0].1 - 3.0).abs() < 1e-4 && (vals[1].1 - 13.0).abs() < 1e-4,
        "{vals:?}"
    );
}

#[test]
fn dragging_sideways_moves_the_selection_in_time() {
    let s = snap(false, false);
    let mut st = TimelinePanelState::default();
    // 100 px/s, 60 fps: 30 px = 0.3 s = 18 frames exactly.
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Begin, 0.0, 50.0, false),
    );
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Update, 30.0, 50.0, false),
    );
    let got = state::drain_intents();
    let moves: Vec<f64> = got
        .iter()
        .filter_map(|i| match i {
            TimelineIntent::MoveSelectedKeys { delta_seconds } => Some(*delta_seconds),
            _ => None,
        })
        .collect();
    assert_eq!(moves.len(), 1);
    assert!((moves[0] - 0.3).abs() < 1e-9, "{moves:?}");
    assert_eq!(
        st.pending_move_dx, None,
        "the band lags one frame as a whole; no diamond preview here"
    );
}

#[test]
fn a_continuing_sideways_drag_emits_only_what_accrued_since_the_last_frame() {
    // The streamed deltas must SUM to the drag: emitting the running total
    // each frame would move the keys twice.
    let s = snap(false, false);
    let mut st = TimelinePanelState::default();
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Begin, 0.0, 50.0, false),
    );
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Update, 30.0, 50.0, false),
    );
    let _ = state::drain_intents();

    // 20 px further: 0.5 s from the START, so only +0.2 is still owed.
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Update, 50.0, 50.0, false),
    );
    let got = state::drain_intents();
    let TimelineIntent::MoveSelectedKeys { delta_seconds } = got[0] else {
        panic!("{got:?}")
    };
    assert_eq!(got.len(), 1);
    assert!((delta_seconds - 0.2).abs() < 1e-9, "{delta_seconds}");
    let applied = st.anchor_drag.as_ref().expect("armed").applied_s;
    assert!((applied - 0.5).abs() < 1e-9, "{applied}");
}

#[test]
fn a_tap_on_an_anchor_closes_the_bracket_and_edits_nothing() {
    let s = snap(false, false);
    let mut st = TimelinePanelState::default();
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Begin, 5.0, 5.0, false),
    );
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Click, 5.0, 5.0, false),
    );
    assert!(st.anchor_drag.is_none());
    assert_eq!(
        state::drain_intents(),
        vec![
            TimelineIntent::BeginEdit,
            TimelineIntent::SelectSingle(SelectedKey::new(TARGET, 1)),
            TimelineIntent::EndEdit,
        ],
        "an empty bracket commits no undo step"
    );
}

#[test]
fn clicking_a_selected_anchor_collapses_the_group_to_it() {
    let s = snap(true, true);
    let mut st = TimelinePanelState::default();
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Begin, 5.0, 5.0, false),
    );
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Click, 5.0, 5.0, false),
    );
    assert_eq!(
        state::drain_intents(),
        vec![
            TimelineIntent::BeginEdit,
            TimelineIntent::SelectSingle(SelectedKey::new(TARGET, 1)),
            TimelineIntent::EndEdit,
        ]
    );
}

#[test]
fn shift_pressing_an_unselected_anchor_grows_the_group_it_drags() {
    // Key 2 selected, shift-press key 1: the toggle ADDS it, so both retune.
    let s = snap(false, true);
    let tr = track_of(&s);
    let mut st = TimelinePanelState::default();
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Begin, 0.0, 50.0, true),
    );
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Update, 0.0, 25.0, true),
    );
    resolve_drag(&mut st, &band(), &tr);
    let got = state::drain_intents();
    assert!(got.contains(&TimelineIntent::ToggleSelect(SelectedKey::new(TARGET, 1))));
    assert_eq!(
        values(&got).len(),
        2,
        "the pressed key joined the group: {got:?}"
    );
}

#[test]
fn shift_pressing_a_selected_anchor_deselects_it_and_leaves_it_behind() {
    // Both selected; shift-press key 1 removes it. A drag then retunes only
    // what is still selected — key 2.
    let s = snap(true, true);
    let tr = track_of(&s);
    let mut st = TimelinePanelState::default();
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Begin, 0.0, 50.0, true),
    );
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Update, 0.0, 25.0, true),
    );
    resolve_drag(&mut st, &band(), &tr);
    let vals = values(&state::drain_intents());
    assert_eq!(vals.len(), 1);
    assert_eq!(vals[0].0, 2, "the deselected key is not retuned");
}

#[test]
fn a_drag_on_another_track_is_ignored_by_this_band() {
    let s = snap(false, false);
    let tr = track_of(&s);
    let mut st = TimelinePanelState::default();
    apply_gesture(
        &mut st,
        100.0,
        &s,
        77,
        1,
        gesture(GesturePhase::Begin, 0.0, 50.0, false),
    );
    let _ = state::drain_intents();
    apply_gesture(
        &mut st,
        100.0,
        &s,
        77,
        1,
        gesture(GesturePhase::Update, 0.0, 25.0, false),
    );
    resolve_drag(&mut st, &band(), &tr);
    let got = state::drain_intents();
    assert!(values(&got).is_empty(), "track 77 owns this drag: {got:?}");
    assert!(st.anchor_drag.is_some());
}

#[test]
fn collapsing_the_row_mid_drag_closes_the_undo_bracket() {
    // The band stops existing, so `resolve_drag` will never fire again.
    let s = snap(false, false);
    let mut st = TimelinePanelState::default();
    st.toggle_expanded(TARGET);
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Begin, 0.0, 50.0, false),
    );
    let _ = state::drain_intents();
    st.toggle_expanded(TARGET);
    assert!(st.anchor_drag.is_none());
    assert_eq!(state::drain_intents(), vec![TimelineIntent::EndEdit]);
}

#[test]
fn the_frozen_range_is_captured_once_and_then_held() {
    let mut st = TimelinePanelState::default();
    let s = snap(false, false);
    apply_gesture(
        &mut st,
        100.0,
        &s,
        TARGET,
        1,
        gesture(GesturePhase::Begin, 0.0, 50.0, false),
    );
    assert_eq!(frozen_range(&st, TARGET), None);
    freeze_range(&mut st, TARGET, (-1.0, 11.0));
    assert_eq!(frozen_range(&st, TARGET), Some((-1.0, 11.0)));
    // A refit band must not overwrite it — that feedback loop is the bug.
    freeze_range(&mut st, TARGET, (-5.0, 50.0));
    assert_eq!(frozen_range(&st, TARGET), Some((-1.0, 11.0)));
    assert_eq!(frozen_range(&st, 77), None, "another track's band is free");
    let _ = state::drain_intents();
}
