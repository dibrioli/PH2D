//! W3.E4 — turning a right-click on a timeline key into a `SetInterp`.
//!
//! editor-core paints the segment preset menu and parks the clicked row as an
//! opaque `(item, mode)` [`TimelineInterpPick`] — it cannot depend on `ph2d-anim`
//! and so never names an easing. This module is where that pair becomes anim
//! vocabulary, because the shell is where the document lives.
//!
//! Split from `timeline_bridge` (the per-frame apply) under the HR-18 shell cap.

use ph2d_timeline::{TimelineIntent, TimelineState};

/// What a row of the timeline segment menu (W3.E4) means, once the shell has
/// paired the clicked id with the mode its cascade carried.
///
/// editor-core parks an opaque `(item, mode)`; this is where it becomes anim
/// vocabulary. [`Preset::Custom`] needs the key's current interpolation (it
/// freezes the handles the graph already draws), so it stays a variant instead
/// of an `Interp` — see [`interp_for_pick`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Preset {
    /// Hold the value, then jump.
    Hold,
    /// Straight line.
    Linear,
    /// Freeze the drawn handles into an editable bézier.
    Custom,
    /// One of the 30 easing family × mode combinations.
    Eased(ph2d_anim::Easing),
}

/// Resolve one menu row. `mode` is `TL_NO_EASE_MODE` for the rows that carry
/// none. `None` for an id that is not a leaf of either table (the three cascade
/// rows never reach here — editor-core swallows them).
///
/// A gate below walks BOTH published tables through this function: a row painted
/// into the menu but forgotten here would be an item that silently does nothing,
/// which is the classic way a context menu ships dead.
pub(crate) fn preset_for(item: ph2d_editor::NodeId, mode: u8) -> Option<Preset> {
    use ph2d_anim::{Easing, EasingFamily as F, EasingMode as M};
    use ph2d_editor::ids as c;
    if item == c::CTX_MENU_TL_HOLD {
        return Some(Preset::Hold);
    }
    if item == c::CTX_MENU_TL_LINEAR {
        return Some(Preset::Linear);
    }
    if item == c::CTX_MENU_TL_CUSTOM {
        return Some(Preset::Custom);
    }
    let mode = match mode {
        c::TL_EASE_MODE_IN => M::In,
        c::TL_EASE_MODE_OUT => M::Out,
        c::TL_EASE_MODE_INOUT => M::InOut,
        _ => return None,
    };
    let family = match item {
        _ if item == c::CTX_MENU_TL_FAM_SINE => F::Sine,
        _ if item == c::CTX_MENU_TL_FAM_QUAD => F::Quad,
        _ if item == c::CTX_MENU_TL_FAM_CUBIC => F::Cubic,
        _ if item == c::CTX_MENU_TL_FAM_QUART => F::Quart,
        _ if item == c::CTX_MENU_TL_FAM_QUINT => F::Quint,
        _ if item == c::CTX_MENU_TL_FAM_EXPO => F::Expo,
        _ if item == c::CTX_MENU_TL_FAM_CIRC => F::Circ,
        _ if item == c::CTX_MENU_TL_FAM_BACK => F::Back,
        _ if item == c::CTX_MENU_TL_FAM_ELASTIC => F::Elastic,
        _ if item == c::CTX_MENU_TL_FAM_BOUNCE => F::Bounce,
        _ => return None,
    };
    Some(Preset::Eased(Easing::new(family, mode)))
}

/// Turn a picked preset into the intents that apply it. Returns them in the
/// order the bridge must drain them (a selection change, then the edit that
/// reads it); empty when the row is unknown or its keys are gone.
///
/// **Scope follows the selection**, exactly as a drag does: right-clicking a key
/// that is part of the current selection retunes the WHOLE selection — any number
/// of tracks, any number of times, one undo step. Right-clicking a key outside it
/// retunes only that key, and leaves the selection alone. Same disambiguation the
/// dope-sheet diamond uses for press-and-drag, so the menu never surprises.
///
/// A **Summary column** names no key at all: it stands for every key at one time.
/// It resolves by *making* that column the selection and then running the very
/// same bulk edit — which is why the column also stays visibly selected
/// afterwards, and why no third code path was needed.
pub(crate) fn intents_for_pick(
    state: &TimelineState,
    pick: ph2d_editor::interaction::TimelineInterpPick,
) -> Vec<TimelineIntent> {
    use ph2d_editor::interaction::TimelineInterpScope as S;
    let Some(preset) = preset_for(pick.item, pick.mode) else {
        return Vec::new();
    };
    match pick.scope {
        S::Key { target, key } => single_key(state, preset, target, key),
        S::Column { t_bits } => column(state, preset, f64::from_bits(t_bits)),
    }
}

/// A key: bulk when it is selected, otherwise just itself.
fn single_key(state: &TimelineState, preset: Preset, target: u64, key: u64) -> Vec<TimelineIntent> {
    use ph2d_anim::{AnimTarget, KeyId};
    use ph2d_timeline::SelectedKey;
    let target = AnimTarget::new(target);
    let key = KeyId::new(key);
    let bulk = state.selection.contains(SelectedKey { target, key });
    // `Custom` has no single `Interp` to broadcast: every key freezes ITS own
    // handles, so it crosses as its own intent rather than a shared value.
    if matches!(preset, Preset::Custom) {
        if bulk {
            return vec![TimelineIntent::ConvertSelectionToBezier];
        }
        let Some(k) = state
            .doc
            .active_clip()
            .track(target)
            .and_then(|t| t.key(key))
        else {
            return Vec::new();
        };
        return vec![TimelineIntent::SetInterp {
            target,
            key,
            interp: k.interp.to_bezier(),
        }];
    }
    let interp = absolute(preset);
    vec![if bulk {
        TimelineIntent::SetSelectedInterp { interp }
    } else {
        TimelineIntent::SetInterp {
            target,
            key,
            interp,
        }
    }]
}

/// A Summary column: select every key at `t`, then bulk-edit the selection.
fn column(state: &TimelineState, preset: Preset, t: f64) -> Vec<TimelineIntent> {
    let keys = keys_at(state, t);
    if keys.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(keys.len() + 2);
    out.push(TimelineIntent::ClearSelection);
    out.extend(keys.into_iter().map(TimelineIntent::AddToSelection));
    out.push(if matches!(preset, Preset::Custom) {
        TimelineIntent::ConvertSelectionToBezier
    } else {
        TimelineIntent::SetSelectedInterp {
            interp: absolute(preset),
        }
    });
    out
}

/// Every key sitting at exactly `t`, across every bound track of the active clip.
///
/// Exact `f64` equality is right here, not sloppy: the time came from the panel,
/// which read it out of `RationalTime::to_seconds()` on these very keys, and that
/// map is deterministic. Two keys share a Summary column iff they share a time.
pub(crate) fn keys_at(state: &TimelineState, t: f64) -> Vec<ph2d_timeline::SelectedKey> {
    use ph2d_timeline::SelectedKey;
    let clip = state.doc.active_clip();
    state
        .doc
        .bindings()
        .iter()
        .filter_map(|b| Some((b.target, clip.track(b.target)?)))
        .flat_map(|(target, track)| {
            track
                .keys()
                .iter()
                .zip(track.ids())
                .filter(move |(k, _)| k.t.to_seconds() == t)
                .map(move |(_, &key)| SelectedKey { target, key })
        })
        .collect()
}

/// The three presets that name one interpolation outright.
fn absolute(preset: Preset) -> ph2d_anim::Interp {
    use ph2d_anim::Interp;
    match preset {
        Preset::Hold => Interp::Hold,
        Preset::Linear => Interp::Linear,
        Preset::Eased(e) => Interp::Eased(e),
        Preset::Custom => unreachable!("Custom is relative to each key's own curve"),
    }
}

#[cfg(test)]
mod preset_tests {
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
}

#[cfg(test)]
mod pick_tests {
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

    fn pick(
        target: AnimTarget,
        key: KeyId,
        item: ph2d_editor::NodeId,
        mode: u8,
    ) -> TimelineInterpPick {
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
}
