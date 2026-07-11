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

/// A Summary column: bulk when it is already part of the selection, otherwise
/// select every key at `t` and then bulk-edit the new selection.
fn column(state: &TimelineState, preset: Preset, t: f64) -> Vec<TimelineIntent> {
    let keys = keys_at(state, t);
    if keys.is_empty() {
        return Vec::new();
    }
    let bulk_edit = if matches!(preset, Preset::Custom) {
        TimelineIntent::ConvertSelectionToBezier
    } else {
        TimelineIntent::SetSelectedInterp {
            interp: absolute(preset),
        }
    };
    // The same disambiguation the Key scope uses: right-clicking a column whose
    // keys are ALL already selected retunes the WHOLE selection — several
    // selected columns must not collapse to the clicked one first (the original
    // bug: the unconditional ClearSelection below threw the rest away).
    if keys.iter().all(|k| state.selection.contains(*k)) {
        return vec![bulk_edit];
    }
    let mut out = Vec::with_capacity(keys.len() + 2);
    out.push(TimelineIntent::ClearSelection);
    out.extend(keys.into_iter().map(TimelineIntent::AddToSelection));
    out.push(bulk_edit);
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
#[path = "timeline_presets_tests.rs"]
mod pick_tests;
