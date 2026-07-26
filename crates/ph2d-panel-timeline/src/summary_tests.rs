//! Unit tests for [`super`] (`summary.rs`) — extracted to a sibling module
//! (`#[path]`) so the gesture source stays under the 600-LOC panel cap.

use super::*;
use ph2d_editor_core::interaction::{GestureMods, TimelineHitKind};
use ph2d_host::PointerButton;
use ph2d_timeline::{AnimTarget, Interp, KeyId, KeyView, PropKind, TrackView};

/// Two tracks. `tx` has keys at t = 0 and t = 1; `opacity` has one at t = 0.
/// So the column at 0 spans both tracks, and the column at 1 is a lone key.
fn snap(sel: &[(u64, u64)]) -> TimelineViewSnapshot {
    let mk = |target: u64, id: u64, t: f64| KeyView {
        id: KeyId::new(id),
        t_seconds: t,
        value: 0.0,
        interp: Interp::Linear,
        selected: sel.contains(&(target, id)),
        roving: false,
    };
    let track = |target: u64, prop, keys| TrackView {
        target: AnimTarget::new(target),
        prop,
        entity: 1,
        missing: false,
        buffer_ghost: None,
        pre: ph2d_timeline::Extrap::Hold,
        post: ph2d_timeline::Extrap::Hold,
        keys,
    };
    TimelineViewSnapshot {
        fps: 60.0,
        frame_snap: true,
        tracks: vec![
            track(
                0,
                PropKind::TranslationX,
                vec![mk(0, 1, 0.0), mk(0, 2, 1.0)],
            ),
            track(5, PropKind::Opacity, vec![mk(5, 7, 0.0)]),
        ],
        ..TimelineViewSnapshot::default()
    }
}

fn gesture(phase: GesturePhase, x: f32, shift: bool) -> TimelineGesture {
    TimelineGesture {
        surface: ph2d_a11y::NodeId(0),
        kind: TimelineHitKind::SummaryKey {
            t_bits: 0.0_f64.to_bits(),
        },
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

const T0: u64 = 0;
fn t0() -> u64 {
    0.0_f64.to_bits()
}
fn t1() -> u64 {
    1.0_f64.to_bits()
}

// ── columns: the aggregation itself ──────────────────────────────────────

#[test]
fn a_column_gathers_every_track_key_that_shares_a_time() {
    let cols = columns(&snap(&[]));
    assert_eq!(cols.len(), 2, "two distinct times");
    assert_eq!(cols[0].t_seconds, 0.0);
    assert_eq!(
        cols[0].keys,
        vec![SelectedKey::new(0, 1), SelectedKey::new(5, 7)],
        "the column at t=0 reaches across both tracks"
    );
    assert_eq!(cols[1].keys, vec![SelectedKey::new(0, 2)]);
}

#[test]
fn columns_come_out_in_time_order_whatever_order_the_keys_arrive_in() {
    let cols = columns(&snap(&[]));
    assert!(cols[0].t_seconds < cols[1].t_seconds);
}

#[test]
fn a_column_reads_as_selected_only_when_all_of_its_keys_are() {
    // Half a column selected must not look grabbed — that is the state where a
    // press has to (re)select, not preserve.
    let half = columns(&snap(&[(0, 1)]));
    assert!(
        !half[0].all_selected,
        "only one of the two keys is selected"
    );
    let whole = columns(&snap(&[(0, 1), (5, 7)]));
    assert!(whole[0].all_selected);
    assert!(!whole[1].all_selected, "the other column is untouched");
}

#[test]
fn an_empty_timeline_has_no_columns() {
    assert!(columns(&TimelineViewSnapshot::default()).is_empty());
}

#[test]
fn column_at_finds_a_column_by_its_opaque_time_handle() {
    let s = snap(&[]);
    assert_eq!(column_at(&s, t1()).map(|c| c.keys.len()), Some(1));
    assert_eq!(column_at(&s, 12345).map(|c| c.keys.len()), None);
}

// ── the gesture ──────────────────────────────────────────────────────────

#[test]
fn pressing_an_unselected_column_selects_every_key_in_it() {
    let mut st = TimelinePanelState::default();
    let s = snap(&[]);
    apply_gesture(
        &mut st,
        120.0,
        &s,
        t0(),
        gesture(GesturePhase::Begin, 10.0, false),
    );
    assert_eq!(
        state::drain_intents(),
        vec![
            TimelineIntent::BeginEdit,
            TimelineIntent::ClearSelection,
            TimelineIntent::AddToSelection(SelectedKey::new(0, 1)),
            TimelineIntent::AddToSelection(SelectedKey::new(5, 7)),
        ],
        "the whole column, and the Clear lands before the Adds"
    );
    assert!(st.key_drag.is_some(), "and the drag is armed");
}

#[test]
fn shift_pressing_a_column_adds_it_to_the_selection_instead_of_replacing_it() {
    let mut st = TimelinePanelState::default();
    let s = snap(&[(0, 2)]); // the OTHER column is already selected
    apply_gesture(
        &mut st,
        120.0,
        &s,
        t0(),
        gesture(GesturePhase::Begin, 10.0, true),
    );
    let got = state::drain_intents();
    assert!(
        !got.contains(&TimelineIntent::ClearSelection),
        "Shift must not wipe the other column: {got:?}"
    );
    assert!(got.contains(&TimelineIntent::AddToSelection(SelectedKey::new(0, 1))));
}

#[test]
fn pressing_an_already_selected_column_preserves_the_wider_selection() {
    // Both keys of column 0 selected, AND a key of column 1: dragging must move
    // all three, so the press may not re-select just the column under it.
    let mut st = TimelinePanelState::default();
    let s = snap(&[(0, 1), (5, 7), (0, 2)]);
    apply_gesture(
        &mut st,
        120.0,
        &s,
        t0(),
        gesture(GesturePhase::Begin, 10.0, false),
    );
    assert_eq!(
        state::drain_intents(),
        vec![TimelineIntent::BeginEdit],
        "no selection change — the group travels together"
    );
    assert_eq!(
        st.summary_press,
        Some(SummaryPress {
            t_bits: t0(),
            was_selected: true
        })
    );
}

#[test]
fn clicking_an_already_selected_column_collapses_the_selection_to_it() {
    let mut st = TimelinePanelState::default();
    let s = snap(&[(0, 1), (5, 7), (0, 2)]);
    apply_gesture(
        &mut st,
        120.0,
        &s,
        t0(),
        gesture(GesturePhase::Begin, 10.0, false),
    );
    let _ = state::drain_intents();
    apply_gesture(
        &mut st,
        120.0,
        &s,
        t0(),
        gesture(GesturePhase::Click, 10.0, false),
    );
    assert_eq!(
        state::drain_intents(),
        vec![
            TimelineIntent::ClearSelection,
            TimelineIntent::AddToSelection(SelectedKey::new(0, 1)),
            TimelineIntent::AddToSelection(SelectedKey::new(5, 7)),
            TimelineIntent::EndEdit,
        ]
    );
    assert!(st.summary_press.is_none());
}

#[test]
fn dragging_a_column_streams_the_move_and_brackets_it_as_one_undo_step() {
    // This is the row's whole purpose: one grab, every key at that time travels.
    // The move itself is the ordinary selection move — no new document op.
    let mut st = TimelinePanelState::default();
    let s = snap(&[]);
    apply_gesture(
        &mut st,
        120.0,
        &s,
        t0(),
        gesture(GesturePhase::Begin, 100.0, false),
    );
    apply_gesture(
        &mut st,
        120.0,
        &s,
        t0(),
        gesture(GesturePhase::Update, 130.0, false),
    );
    apply_gesture(
        &mut st,
        120.0,
        &s,
        t0(),
        gesture(GesturePhase::End, 130.0, false),
    );
    let got = state::drain_intents();
    assert_eq!(got.first(), Some(&TimelineIntent::BeginEdit));
    assert_eq!(got.last(), Some(&TimelineIntent::EndEdit));
    assert_eq!(
        got.iter()
            .filter(|i| matches!(i, TimelineIntent::MoveSelectedKeys { .. }))
            .count(),
        1
    );
    assert!(got.contains(&TimelineIntent::MoveSelectedKeys {
        delta_seconds: 0.25
    }));
    assert!(st.key_drag.is_none() && st.summary_press.is_none());
}

#[test]
fn a_continuing_column_drag_emits_only_what_accrued_since_the_last_frame() {
    let mut st = TimelinePanelState::default();
    let s = snap(&[]);
    apply_gesture(
        &mut st,
        120.0,
        &s,
        t0(),
        gesture(GesturePhase::Begin, 100.0, false),
    );
    apply_gesture(
        &mut st,
        120.0,
        &s,
        t0(),
        gesture(GesturePhase::Update, 130.0, false),
    );
    let _ = state::drain_intents();
    apply_gesture(
        &mut st,
        120.0,
        &s,
        t0(),
        gesture(GesturePhase::Update, 142.0, false),
    );
    let got = state::drain_intents();
    let TimelineIntent::MoveSelectedKeys { delta_seconds } = got[0] else {
        panic!("{got:?}")
    };
    assert_eq!(got.len(), 1);
    assert!((delta_seconds - 0.1).abs() < 1e-9, "{delta_seconds}");
}

#[test]
fn pressing_a_column_that_vanished_closes_the_bracket_it_opened() {
    // The snapshot can change between the paint that registered the diamond and
    // the press that lands on it. An unmatched BeginEdit would swallow the next
    // atomic edit into a step the user never made.
    let mut st = TimelinePanelState::default();
    let s = snap(&[]);
    apply_gesture(
        &mut st,
        120.0,
        &s,
        999,
        gesture(GesturePhase::Begin, 10.0, false),
    );
    assert_eq!(
        state::drain_intents(),
        vec![TimelineIntent::BeginEdit, TimelineIntent::EndEdit]
    );
    assert!(st.key_drag.is_none(), "no drag armed on a dead column");
    assert_eq!(T0, 0, "the summary row is row zero");
}
