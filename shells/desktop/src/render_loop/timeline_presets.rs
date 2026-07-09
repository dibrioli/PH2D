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

/// Turn a picked preset into the intent that applies it.
///
/// **Scope follows the selection**, exactly as a drag does: right-clicking a key
/// that is part of the current selection retunes the WHOLE selection — any number
/// of tracks, any number of times, one undo step. Right-clicking a key outside it
/// retunes only that key, and leaves the selection alone. Same disambiguation the
/// dope-sheet diamond uses for press-and-drag, so the menu never surprises.
///
/// `None` when the row is unknown or the key has since been deleted.
pub(crate) fn interp_for_pick(
    state: &TimelineState,
    pick: ph2d_editor::interaction::TimelineInterpPick,
) -> Option<TimelineIntent> {
    use ph2d_anim::{AnimTarget, Interp, KeyId};
    use ph2d_timeline::SelectedKey;
    let target = AnimTarget::new(pick.target);
    let key = KeyId::new(pick.key);
    let preset = preset_for(pick.item, pick.mode)?;
    let bulk = state.selection.contains(SelectedKey { target, key });

    // `Custom` has no single `Interp` to broadcast: every key freezes ITS own
    // handles, so it crosses as its own intent rather than a shared value.
    if matches!(preset, Preset::Custom) {
        return Some(if bulk {
            TimelineIntent::ConvertSelectionToBezier
        } else {
            TimelineIntent::SetInterp {
                target,
                key,
                interp: state
                    .doc
                    .active_clip()
                    .track(target)?
                    .key(key)?
                    .interp
                    .to_bezier(),
            }
        });
    }
    let interp = match preset {
        Preset::Hold => Interp::Hold,
        Preset::Linear => Interp::Linear,
        Preset::Eased(e) => Interp::Eased(e),
        Preset::Custom => unreachable!("handled above"),
    };
    Some(if bulk {
        TimelineIntent::SetSelectedInterp { interp }
    } else {
        TimelineIntent::SetInterp {
            target,
            key,
            interp,
        }
    })
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
    use ph2d_editor::interaction::{TL_NO_EASE_MODE, TimelineInterpPick};
    use ph2d_timeline::{PropKind, SelectedKey, apply_intent};

    /// One track (entity 1, TranslationX) with a single Cubic-InOut key. `AddKey`
    /// leaves it selected, so callers that want the single-key scope must
    /// `ClearSelection` first — which is the point of this whole module.
    fn doc_with_one_key() -> (TimelineState, AnimTarget, KeyId) {
        let mut st = TimelineState::new();
        let mut ph = Playhead::new(1.0 / 60.0);
        apply_intent(
            &mut st,
            &mut ph,
            TimelineIntent::AddKey {
                entity: 1,
                prop: PropKind::TranslationX,
                t: ph2d_anim::RationalTime::from_seconds(0.0),
                value: AnimValue::Float(0.0),
                interp: default_interp(),
            },
        );
        let target = st
            .doc
            .binding_for(1, PropKind::TranslationX)
            .unwrap()
            .target;
        let key = st.doc.active_clip().track(target).unwrap().ids()[0];
        (st, target, key)
    }

    /// The same doc with nothing selected — the "right-clicked a key outside the
    /// selection" case.
    fn doc_unselected() -> (TimelineState, AnimTarget, KeyId) {
        let (mut st, target, key) = doc_with_one_key();
        st.selection.clear();
        (st, target, key)
    }

    fn pick(
        target: AnimTarget,
        key: KeyId,
        item: ph2d_editor::NodeId,
        mode: u8,
    ) -> TimelineInterpPick {
        TimelineInterpPick {
            target: target.get(),
            key: key.get(),
            item,
            mode,
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
                interp_for_pick(&st, pick(target, key, item, TL_NO_EASE_MODE)),
                Some(TimelineIntent::SetInterp {
                    target,
                    key,
                    interp: want
                }),
                "an unselected key must not drag the selection along"
            );
        }
    }

    #[test]
    fn right_clicking_a_selected_key_retunes_every_selected_key() {
        // The whole point: pick once, and every key selected across any number of
        // tracks and times gets the same curve, in one undo step.
        let (st, target, key) = doc_with_one_key();
        assert!(st.selection.contains(SelectedKey { target, key }));
        assert_eq!(
            interp_for_pick(
                &st,
                pick(target, key, c::CTX_MENU_TL_FAM_BACK, c::TL_EASE_MODE_OUT)
            ),
            Some(TimelineIntent::SetSelectedInterp {
                interp: Interp::Eased(Easing::new(EasingFamily::Back, EasingMode::Out))
            })
        );
    }

    #[test]
    fn a_family_row_becomes_that_family_in_the_mode_its_cascade_carried() {
        let (st, target, key) = doc_unselected();
        assert_eq!(
            interp_for_pick(
                &st,
                pick(target, key, c::CTX_MENU_TL_FAM_BACK, c::TL_EASE_MODE_OUT)
            ),
            Some(TimelineIntent::SetInterp {
                target,
                key,
                interp: Interp::Eased(Easing::new(EasingFamily::Back, EasingMode::Out))
            })
        );
    }

    // ── Custom: no single Interp to broadcast ────────────────────────────

    #[test]
    fn custom_freezes_the_handles_the_graph_is_already_drawing() {
        // Not a fresh bezier: the one whose control points are the tangent
        // handles the editor paints for the key's CURRENT easing.
        let (st, target, key) = doc_unselected();
        let current = default_interp();
        let got = interp_for_pick(
            &st,
            pick(target, key, c::CTX_MENU_TL_CUSTOM, TL_NO_EASE_MODE),
        );
        assert_eq!(
            got,
            Some(TimelineIntent::SetInterp {
                target,
                key,
                interp: current.to_bezier()
            })
        );
        let Some(TimelineIntent::SetInterp { interp, .. }) = got else {
            panic!()
        };
        assert_eq!(interp.handles(), Some(current.tangent_handles()));
    }

    #[test]
    fn custom_over_a_selection_converts_each_key_from_its_own_curve() {
        // A shared `Interp` would flatten a mixed selection onto one shape, so
        // Custom crosses as its own intent and each key reads its own handles.
        let (st, target, key) = doc_with_one_key();
        assert_eq!(
            interp_for_pick(
                &st,
                pick(target, key, c::CTX_MENU_TL_CUSTOM, TL_NO_EASE_MODE)
            ),
            Some(TimelineIntent::ConvertSelectionToBezier),
            "bulk Custom must not broadcast one key's bezier to the rest"
        );
    }

    #[test]
    fn a_pick_naming_a_key_the_document_no_longer_has_resolves_to_nothing() {
        // Custom reads the key; a delete between the right-click and the pick
        // must not panic or fabricate an interpolation.
        let (st, target, _) = doc_unselected();
        assert_eq!(
            interp_for_pick(
                &st,
                pick(
                    target,
                    KeyId::new(999),
                    c::CTX_MENU_TL_CUSTOM,
                    TL_NO_EASE_MODE
                )
            ),
            None
        );
        // The absolute presets do not need the key, but an unknown TRACK still
        // yields an intent — `apply_intent` no-ops on it. Only Custom must look.
        assert!(
            interp_for_pick(
                &st,
                pick(
                    target,
                    KeyId::new(999),
                    c::CTX_MENU_TL_HOLD,
                    TL_NO_EASE_MODE
                )
            )
            .is_some()
        );
    }
}
