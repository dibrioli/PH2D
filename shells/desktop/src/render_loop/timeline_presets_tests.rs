//! Tests for [`super`] (`timeline_presets.rs`) — the pick→intents
//! resolution. Extracted to a sibling module (`#[path]`) under the HR-18
//! shell LOC cap. Pure relocation.
use super::*;
use crate::render_loop::timeline_bridge::default_interp;
use ph2d_anim::{AnimTarget, AnimValue, Easing, EasingFamily, EasingMode, Interp, KeyId};
use ph2d_core::Playhead;
use ph2d_editor::ids as c;
use ph2d_editor::interaction::{TL_NO_EASE_MODE, TimelineInterpPick, TimelineInterpScope};
use ph2d_timeline::{PropKind, SelectedKey, apply_intent};

fn add(st: &mut TimelineState, ph: &mut Playhead, prop: PropKind, t: f64) {
    apply_intent(
        st,
        ph,
        TimelineIntent::AddKey {
            entity: 1,
            prop,
            t: ph2d_anim::RationalTime::from_seconds(t),
            value: AnimValue::Float(0.0),
            interp: default_interp(),
        },
    );
}

/// One track (entity 1, TranslationX) with a single Cubic-InOut key at t=0.
/// `AddKey` leaves it selected — the bulk scope — so the single-key tests
/// clear the selection first.
fn doc_with_one_key() -> (TimelineState, AnimTarget, KeyId) {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    add(&mut st, &mut ph, PropKind::TranslationX, 0.0);
    let target = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .unwrap()
        .target;
    let key = st.doc.active_clip().track(target).unwrap().ids()[0];
    (st, target, key)
}

fn doc_unselected() -> (TimelineState, AnimTarget, KeyId) {
    let (mut st, target, key) = doc_with_one_key();
    st.selection.clear();
    (st, target, key)
}

/// TranslationX keyed at t = 0 and t = 1; Opacity keyed at t = 0. So the
/// column at 0 spans two tracks and the column at 1 holds a single key.
fn doc_two_tracks() -> TimelineState {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    add(&mut st, &mut ph, PropKind::TranslationX, 0.0);
    add(&mut st, &mut ph, PropKind::TranslationX, 1.0);
    add(&mut st, &mut ph, PropKind::Opacity, 0.0);
    st.selection.clear();
    st
}

fn pick(target: AnimTarget, key: KeyId, item: ph2d_editor::NodeId, mode: u8) -> TimelineInterpPick {
    TimelineInterpPick {
        scope: TimelineInterpScope::Key {
            target: target.get(),
            key: key.get(),
        },
        item,
        mode,
    }
}

fn column_pick(t: f64, item: ph2d_editor::NodeId) -> TimelineInterpPick {
    TimelineInterpPick {
        scope: TimelineInterpScope::Column {
            t_bits: t.to_bits(),
        },
        item,
        mode: TL_NO_EASE_MODE,
    }
}

// ── scope: one key, or the whole selection ───────────────────────────

#[test]
fn a_key_outside_the_selection_is_retuned_alone() {
    let (st, target, key) = doc_unselected();
    for (item, want) in [
        (c::CTX_MENU_TL_HOLD, Interp::Hold),
        (c::CTX_MENU_TL_LINEAR, Interp::Linear),
    ] {
        assert_eq!(
            intents_for_pick(&st, pick(target, key, item, TL_NO_EASE_MODE)),
            vec![TimelineIntent::SetInterp {
                target,
                key,
                interp: want
            }],
            "an unselected key must not drag the selection along"
        );
    }
}

#[test]
fn right_clicking_a_selected_key_retunes_every_selected_key() {
    let (st, target, key) = doc_with_one_key();
    assert!(st.selection.contains(SelectedKey { target, key }));
    assert_eq!(
        intents_for_pick(
            &st,
            pick(target, key, c::CTX_MENU_TL_FAM_BACK, c::TL_EASE_MODE_OUT)
        ),
        vec![TimelineIntent::SetSelectedInterp {
            interp: Interp::Eased(Easing::new(EasingFamily::Back, EasingMode::Out))
        }]
    );
}

#[test]
fn a_family_row_becomes_that_family_in_the_mode_its_cascade_carried() {
    let (st, target, key) = doc_unselected();
    assert_eq!(
        intents_for_pick(
            &st,
            pick(target, key, c::CTX_MENU_TL_FAM_BACK, c::TL_EASE_MODE_OUT)
        ),
        vec![TimelineIntent::SetInterp {
            target,
            key,
            interp: Interp::Eased(Easing::new(EasingFamily::Back, EasingMode::Out))
        }]
    );
}

// ── Custom: no single Interp to broadcast ────────────────────────────

#[test]
fn custom_freezes_the_handles_the_graph_is_already_drawing() {
    let (st, target, key) = doc_unselected();
    let current = default_interp();
    let got = intents_for_pick(
        &st,
        pick(target, key, c::CTX_MENU_TL_CUSTOM, TL_NO_EASE_MODE),
    );
    assert_eq!(
        got,
        vec![TimelineIntent::SetInterp {
            target,
            key,
            interp: current.to_bezier()
        }]
    );
    let [TimelineIntent::SetInterp { interp, .. }] = got[..] else {
        panic!("{got:?}")
    };
    assert_eq!(interp.handles(), Some(current.tangent_handles()));
}

#[test]
fn custom_over_a_selection_converts_each_key_from_its_own_curve() {
    let (st, target, key) = doc_with_one_key();
    assert_eq!(
        intents_for_pick(
            &st,
            pick(target, key, c::CTX_MENU_TL_CUSTOM, TL_NO_EASE_MODE)
        ),
        vec![TimelineIntent::ConvertSelectionToBezier],
        "bulk Custom must not broadcast one key's bezier to the rest"
    );
}

#[test]
fn a_pick_naming_a_key_the_document_no_longer_has_resolves_to_nothing() {
    let (st, target, _) = doc_unselected();
    assert_eq!(
        intents_for_pick(
            &st,
            pick(
                target,
                KeyId::new(999),
                c::CTX_MENU_TL_CUSTOM,
                TL_NO_EASE_MODE
            )
        ),
        vec![],
        "Custom reads the key; a deleted one must not fabricate an interp"
    );
    // The absolute presets do not need the key — `apply_intent` no-ops.
    assert!(
        !intents_for_pick(
            &st,
            pick(
                target,
                KeyId::new(999),
                c::CTX_MENU_TL_HOLD,
                TL_NO_EASE_MODE
            )
        )
        .is_empty()
    );
}

// ── a Summary column ─────────────────────────────────────────────────

#[test]
fn a_column_selects_every_key_at_its_time_and_then_retunes_them() {
    // The column names no key: it becomes the selection, and the ordinary bulk
    // edit follows. The ClearSelection MUST precede the AddToSelections, and
    // the edit MUST come last, or it retunes whatever was selected before.
    let st = doc_two_tracks();
    let tx = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .unwrap()
        .target;
    let op = st.doc.binding_for(1, PropKind::Opacity).unwrap().target;
    let k_tx = st.doc.active_clip().track(tx).unwrap().ids()[0];
    let k_op = st.doc.active_clip().track(op).unwrap().ids()[0];

    let got = intents_for_pick(&st, column_pick(0.0, c::CTX_MENU_TL_HOLD));
    assert_eq!(
        got,
        vec![
            TimelineIntent::ClearSelection,
            TimelineIntent::AddToSelection(SelectedKey {
                target: tx,
                key: k_tx
            }),
            TimelineIntent::AddToSelection(SelectedKey {
                target: op,
                key: k_op
            }),
            TimelineIntent::SetSelectedInterp {
                interp: Interp::Hold
            },
        ]
    );
}

#[test]
fn a_selected_column_retunes_the_whole_selection_not_just_itself() {
    // THE multi-column bug: select the columns at t = 0 AND t = 1 (e.g. via
    // Shift on the Summary), then right-click the one at t = 0. The old
    // unconditional ClearSelection collapsed the selection to that column,
    // so the easing never reached the keys at t = 1.
    let mut st = doc_two_tracks();
    let mut ph = Playhead::new(1.0 / 60.0);
    for t in [0.0, 1.0] {
        for k in keys_at(&st, t) {
            apply_intent(&mut st, &mut ph, TimelineIntent::AddToSelection(k));
        }
    }
    assert_eq!(st.selection.len(), 3, "both columns are selected");
    let got = intents_for_pick(&st, column_pick(0.0, c::CTX_MENU_TL_HOLD));
    assert_eq!(
        got,
        vec![TimelineIntent::SetSelectedInterp {
            interp: Interp::Hold
        }],
        "no ClearSelection: the whole selection is retuned"
    );
    // Control: a column OUTSIDE the selection still becomes the selection
    // (t = 1's key deselected → the t = 1 pick clears + reselects).
    let mut st = doc_two_tracks();
    for k in keys_at(&st, 0.0) {
        apply_intent(&mut st, &mut ph, TimelineIntent::AddToSelection(k));
    }
    let got = intents_for_pick(&st, column_pick(1.0, c::CTX_MENU_TL_HOLD));
    assert_eq!(got.first(), Some(&TimelineIntent::ClearSelection));
}

#[test]
fn a_column_of_one_key_still_only_touches_that_key() {
    let st = doc_two_tracks();
    let got = intents_for_pick(&st, column_pick(1.0, c::CTX_MENU_TL_LINEAR));
    assert_eq!(
        got.iter()
            .filter(|i| matches!(i, TimelineIntent::AddToSelection(_)))
            .count(),
        1,
        "t=1 has no Opacity key: {got:?}"
    );
}

#[test]
fn custom_on_a_column_converts_each_of_its_keys_from_its_own_curve() {
    let st = doc_two_tracks();
    let got = intents_for_pick(&st, column_pick(0.0, c::CTX_MENU_TL_CUSTOM));
    assert_eq!(got.last(), Some(&TimelineIntent::ConvertSelectionToBezier));
}

#[test]
fn a_column_at_a_time_with_no_keys_changes_nothing() {
    // The snapshot the user right-clicked can be one frame stale.
    let st = doc_two_tracks();
    assert_eq!(
        intents_for_pick(&st, column_pick(7.5, c::CTX_MENU_TL_HOLD)),
        vec![],
        "no keys there: not even a ClearSelection"
    );
}

#[test]
fn keys_at_groups_by_exact_time_across_every_bound_track() {
    let st = doc_two_tracks();
    assert_eq!(keys_at(&st, 0.0).len(), 2);
    assert_eq!(keys_at(&st, 1.0).len(), 1);
    assert!(keys_at(&st, 0.5).is_empty());
    // A frame-exact time and its `to_seconds()` round-trip land together.
    let t = ph2d_anim::RationalTime::from_frame(60, 60).to_seconds();
    assert_eq!(keys_at(&st, t).len(), 1, "t = 1.0 s reached by frame math");
}
