//! The segment-menu resolution tests of `timeline_presets` (`preset_tests`),
//! extracted to a sibling file under the HR-18 shell LOC cap (same pattern as
//! `timeline_presets_tests.rs`).

use super::*;
use ph2d_editor::ids as c;

/// Every row the overlay paints must resolve — the anti-dead-menu gate.
#[test]
fn every_published_menu_row_resolves_to_a_preset() {
    // Leaves of the top-level menu (the three cascade rows are editor-core's).
    for (id, label, _) in c::TIMELINE_SEGMENT_MENU {
        let is_cascade = id == c::CTX_MENU_TL_EASE_IN
            || id == c::CTX_MENU_TL_EASE_OUT
            || id == c::CTX_MENU_TL_EASE_INOUT;
        if is_cascade {
            continue;
        }
        assert!(
            preset_for(id, ph2d_editor::interaction::TL_NO_EASE_MODE).is_some(),
            "top-level row {label:?} paints but resolves to nothing"
        );
    }
    // Every family, under every mode the three cascades can open.
    for mode in [
        c::TL_EASE_MODE_IN,
        c::TL_EASE_MODE_OUT,
        c::TL_EASE_MODE_INOUT,
    ] {
        for (id, label, _) in c::TIMELINE_EASE_MENU {
            assert!(
                matches!(preset_for(id, mode), Some(Preset::Eased(_))),
                "family {label:?} paints but resolves to nothing under mode {mode}"
            );
        }
    }
}

#[test]
fn a_family_row_without_a_mode_resolves_to_nothing() {
    // Only the cascade grants a mode; a family id arriving without one must
    // not silently become "In".
    assert_eq!(
        preset_for(
            c::CTX_MENU_TL_FAM_BOUNCE,
            ph2d_editor::interaction::TL_NO_EASE_MODE
        ),
        None
    );
}

#[test]
fn the_three_cascade_rows_are_not_leaves() {
    for id in [
        c::CTX_MENU_TL_EASE_IN,
        c::CTX_MENU_TL_EASE_OUT,
        c::CTX_MENU_TL_EASE_INOUT,
    ] {
        assert_eq!(
            preset_for(id, ph2d_editor::interaction::TL_NO_EASE_MODE),
            None,
            "a cascade row must open a submenu, never set an interp"
        );
    }
}

#[test]
fn each_mode_and_family_reaches_its_own_easing() {
    use ph2d_anim::{Easing, EasingFamily as F, EasingMode as M};
    assert_eq!(
        preset_for(c::CTX_MENU_TL_FAM_ELASTIC, c::TL_EASE_MODE_OUT),
        Some(Preset::Eased(Easing::new(F::Elastic, M::Out)))
    );
    assert_eq!(
        preset_for(c::CTX_MENU_TL_FAM_SINE, c::TL_EASE_MODE_INOUT),
        Some(Preset::Eased(Easing::new(F::Sine, M::InOut)))
    );
}

/// The 30 combinations are distinct — a copy-paste slip in the family match
/// would map two rows to the same easing and half the menu would be a lie.
#[test]
fn the_thirty_easing_rows_are_all_different() {
    let mut seen = std::collections::BTreeSet::new();
    for mode in [
        c::TL_EASE_MODE_IN,
        c::TL_EASE_MODE_OUT,
        c::TL_EASE_MODE_INOUT,
    ] {
        for (id, _, _) in c::TIMELINE_EASE_MENU {
            let Some(Preset::Eased(e)) = preset_for(id, mode) else {
                panic!("not an easing")
            };
            assert!(seen.insert(format!("{e:?}")), "duplicate easing: {e:?}");
        }
    }
    assert_eq!(seen.len(), 30);
}

// ─── Rove Across Time (W5) ───────────────────────────────────────────────────

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_timeline::{PropKind, SelectedKey, TimelineIntent, TimelineState, apply_intent};

/// Three TX keys (0 s → 0, 3 s → 10, 4 s → 30); returns `(state, target raw,
/// middle key raw)` with an empty selection.
fn rove_rig() -> (TimelineState, u64, u64) {
    let mut st = TimelineState::new();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    for (t, v) in [(0.0, 0.0f32), (3.0, 10.0), (4.0, 30.0)] {
        apply_intent(
            &mut st,
            &mut ph,
            TimelineIntent::AddKey {
                entity: 1,
                prop: PropKind::TranslationX,
                t: RationalTime::from_seconds(t),
                value: AnimValue::Float(v),
                interp: Interp::Linear,
            },
        );
    }
    apply_intent(&mut st, &mut ph, TimelineIntent::ClearSelection);
    let target = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .unwrap()
        .target;
    let mid = st.doc.active_clip().track(target).unwrap().ids()[1];
    (st, target.get(), mid.get())
}

#[test]
fn the_rove_row_resolves_and_toggles_by_current_state() {
    // The row resolves (the anti-dead-menu gate above also walks it).
    assert_eq!(
        preset_for(
            c::CTX_MENU_TL_ROVE,
            ph2d_editor::interaction::TL_NO_EASE_MODE
        ),
        Some(Preset::Rove)
    );
    let (mut st, target, key) = rove_rig();
    // Unselected key → per-key intent, first press turns roving ON.
    let picks = single_key(&st, Preset::Rove, target, key);
    assert_eq!(
        picks,
        vec![TimelineIntent::SetRove {
            target: ph2d_anim::AnimTarget::new(target),
            key: ph2d_anim::KeyId::new(key),
            on: true,
        }]
    );
    // Apply it; the same row now toggles OFF.
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    for p in picks {
        apply_intent(&mut st, &mut ph, p);
    }
    let picks = single_key(&st, Preset::Rove, target, key);
    assert_eq!(
        picks,
        vec![TimelineIntent::SetRove {
            target: ph2d_anim::AnimTarget::new(target),
            key: ph2d_anim::KeyId::new(key),
            on: false,
        }]
    );
}

#[test]
fn a_selected_key_roves_the_whole_selection_mixed_converges_on() {
    let (mut st, target, key) = rove_rig();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    // Select the middle AND last keys, mark only the middle as roving.
    let t = ph2d_anim::AnimTarget::new(target);
    let last = st.doc.active_clip().track(t).unwrap().ids()[2];
    for k in [ph2d_anim::KeyId::new(key), last] {
        apply_intent(
            &mut st,
            &mut ph,
            TimelineIntent::AddToSelection(SelectedKey { target: t, key: k }),
        );
    }
    apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::SetRove {
            target: t,
            key: ph2d_anim::KeyId::new(key),
            on: true,
        },
    );
    // Mixed selection (one roving, one not) → the bulk toggle turns ON.
    let picks = single_key(&st, Preset::Rove, target, key);
    assert_eq!(picks, vec![TimelineIntent::SetSelectedRove { on: true }]);
    apply_intent(&mut st, &mut ph, picks[0].clone());
    // Now ALL selected rove → the toggle flips OFF.
    let picks = single_key(&st, Preset::Rove, target, key);
    assert_eq!(picks, vec![TimelineIntent::SetSelectedRove { on: false }]);
}

#[test]
fn a_column_rove_selects_the_column_and_toggles_it() {
    let (st, target, _) = rove_rig();
    // No key of the 3 s column is selected → the pick selects it, then roves.
    let picks = column(&st, Preset::Rove, 3.0);
    assert_eq!(
        picks.last(),
        Some(&TimelineIntent::SetSelectedRove { on: true }),
        "column rove ends in the bulk toggle"
    );
    assert!(
        matches!(picks.first(), Some(TimelineIntent::ClearSelection)),
        "unselected column becomes the selection first"
    );
    let _ = target;
}
