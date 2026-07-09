//! The graph-editor anchor gesture (W3.E5) — drag a key in the `(time, value)`
//! plane.
//!
//! Two axes, two scopes, and the difference is not an accident:
//!
//! - **Sideways** the anchor rides the *shared* time axis — the same ruler every
//!   row is drawn against — so it moves the whole selection, exactly as its
//!   diamond does in the strip above (one streamed `MoveSelectedKeys` per frame).
//! - **Up and down** it rides a *band-local* value axis: each row auto-fits its
//!   own range, so the same pixel offset means +5 metres in a translation band
//!   and +5 (out of 1) in an opacity band. A value delta therefore only travels
//!   as far as the band it was made in — the selected keys of THIS track.
//!
//! Value edits go out as absolute `SetKeyValue`s rebuilt from the values captured
//! at Begin, not as increments: an `f32` accumulated once per frame drifts away
//! from the cursor over a slow drag.
//!
//! The whole press-to-release sits inside one `BeginEdit`/`EndEdit` bracket, so
//! streaming per frame still undoes in a single Ctrl+Z. Like the handle drag, the
//! value half is geometry-bound: the band's value↔pixel mapping only exists
//! during `paint`, so the gesture merely *records* the pointer and
//! [`resolve_drag`] turns it into intents there.

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture};
use ph2d_timeline::{
    AnimValue, KeyView, SelectedKey, TimelineIntent, TimelineViewSnapshot, TrackView,
};

use crate::graph::Band;
use crate::key_drag::drag_delta_seconds;
use crate::state::{self, AnchorDrag, TimelinePanelState};

/// Interpret one gesture on a key's anchor dot. `target`/`key` name the pressed
/// key; `snap` supplies the selection and the base values captured at Begin.
pub(crate) fn apply_gesture(
    state: &mut TimelinePanelState,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    target: u64,
    key: u64,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => begin(state, snap, target, key, g),
        GesturePhase::Update => {
            if let Some(d) = state.anchor_drag.as_mut() {
                d.cur = (g.x, g.y);
            }
            emit_move(state, px_per_s, snap);
        }
        GesturePhase::End => {
            // `paint` still owes one resolve for the value half (it alone knows
            // the band), and that pass closes the bracket.
            if let Some(d) = state.anchor_drag.as_mut() {
                (d.cur, d.ending) = ((g.x, g.y), true);
            }
            emit_move(state, px_per_s, snap);
        }
        GesturePhase::Click | GesturePhase::DoubleClick => {
            // Nothing moved: collapse a preserved group to the pressed key and
            // close the bracket here — no resolve is owed.
            if let Some(d) = state.anchor_drag.take()
                && let Some(one) = d.collapse_to
            {
                state::push_intent(TimelineIntent::SelectSingle(one));
            }
            state::push_intent(TimelineIntent::EndEdit);
        }
    }
}

/// Open the undo bracket, settle the selection, and capture the base values of
/// every key this drag will retune.
fn begin(
    state: &mut TimelinePanelState,
    snap: &TimelineViewSnapshot,
    target: u64,
    key: u64,
    g: TimelineGesture,
) {
    state::push_intent(TimelineIntent::BeginEdit);
    let sel = SelectedKey::new(target, key);
    let was_selected = is_selected(snap, target, key);
    // Same disambiguation as the dope-sheet diamond: Shift toggles, a press on a
    // selected key preserves the group (a drag moves it, a click collapses it),
    // and a press on an unselected key selects it alone.
    let collapse_to = if g.mods.shift {
        state::push_intent(TimelineIntent::ToggleSelect(sel));
        None
    } else if was_selected {
        Some(sel)
    } else {
        state::push_intent(TimelineIntent::SelectSingle(sel));
        None
    };
    state.anchor_drag = Some(AnchorDrag {
        target,
        start: (g.x, g.y),
        cur: (g.x, g.y),
        base: drag_set(snap, target, key, g.mods.shift, was_selected),
        applied_s: 0.0,
        applied_v: None,
        collapse_to,
        range: None,
        ending: false,
    });
}

/// The keys this drag retunes, with the value each holds right now.
///
/// Selection intents raised at Begin only land next frame, so this predicts what
/// the press *will* have selected on this track rather than reading the stale
/// snapshot:
///
/// - Shift on an unselected key ADDS it — the group grows and drags together.
/// - Shift on a selected key REMOVES it — nothing on this track is being grabbed,
///   so the drag retunes nothing (the pointer is dragging a deselected anchor).
/// - No shift on a selected key preserves the group.
/// - No shift on an unselected key collapses the selection to it.
fn drag_set(
    snap: &TimelineViewSnapshot,
    target: u64,
    key: u64,
    shift: bool,
    was_selected: bool,
) -> Vec<(u64, f32)> {
    let Some(track) = snap.tracks.iter().find(|t| t.target.get() == target) else {
        return Vec::new();
    };
    let take_group = shift || was_selected;
    track
        .keys
        .iter()
        .filter(|k| {
            let id = k.id.get();
            if shift && was_selected {
                // The press deselects it; the rest of the group stays.
                return k.selected && id != key;
            }
            if take_group {
                return k.selected || id == key;
            }
            id == key
        })
        .map(|k| (k.id.get(), k.value))
        .collect()
}

/// Emit whatever frame-snapped time delta accrued since the last emit. The
/// anchor rides the shared ruler, so this moves the whole selection — the same
/// intent its diamond raises one strip above.
fn emit_move(state: &mut TimelinePanelState, px_per_s: f64, snap: &TimelineViewSnapshot) {
    let Some(d) = state.anchor_drag.as_ref() else {
        return;
    };
    let want = drag_delta_seconds(d.start.0, d.cur.0, px_per_s, snap);
    let delta = want - d.applied_s;
    if delta == 0.0 {
        return;
    }
    if let Some(d) = state.anchor_drag.as_mut() {
        d.applied_s = want;
    }
    state::push_intent(TimelineIntent::MoveSelectedKeys {
        delta_seconds: delta,
    });
    // Deliberately no `pending_move_dx`: unlike a diamond drag, everything this
    // gesture touches (anchors, curve, handles) is painted from the snapshot, so
    // it must all lag the same single frame. Sliding the anchor ahead of its own
    // curve would look worse than the frame it saves.
}

/// Turn this frame of an in-flight anchor drag into one `SetKeyValue` per key it
/// grabbed, now that the band's value↔pixel mapping is known. Closes the undo
/// bracket on the last one. Called from `graph_paint::paint_track`.
pub(crate) fn resolve_drag(state: &mut TimelinePanelState, band: &Band, track: &TrackView) {
    let Some(d) = state.anchor_drag.as_ref() else {
        return;
    };
    if d.target != track.target.get() {
        return;
    }
    // The band maps pixels to values affinely, so the offset is the difference of
    // the two endpoints' values — no need to know where zero is.
    let delta = band.value(d.cur.1) - band.value(d.start.1);
    let ending = d.ending;
    if d.applied_v != Some(delta) {
        for &(key, base) in &d.base {
            state::push_intent(TimelineIntent::SetKeyValue {
                target: track.target,
                key: ph2d_timeline::KeyId::new(key),
                value: AnimValue::Float((f64::from(base) + delta) as f32),
            });
        }
        if let Some(d) = state.anchor_drag.as_mut() {
            d.applied_v = Some(delta);
        }
    }
    if ending {
        state::push_intent(TimelineIntent::EndEdit);
        state.anchor_drag = None;
    }
}

/// The band range an in-flight anchor drag on `target` froze, if any.
pub(crate) fn frozen_range(state: &TimelinePanelState, target: u64) -> Option<(f64, f64)> {
    let d = state.anchor_drag.as_ref()?;
    (d.target == target).then_some(d.range).flatten()
}

/// Freeze the band range on the drag's first paint (see [`AnchorDrag::range`]).
pub(crate) fn freeze_range(state: &mut TimelinePanelState, target: u64, range: (f64, f64)) {
    if let Some(d) = state.anchor_drag.as_mut()
        && d.target == target
        && d.range.is_none()
    {
        d.range = Some(range);
    }
}

/// Whether the anchor of `key` on `target` is drawn as being dragged.
pub(crate) fn is_dragging(state: &TimelinePanelState, target: u64, key: &KeyView) -> bool {
    state
        .anchor_drag
        .as_ref()
        .is_some_and(|d| d.target == target && d.base.iter().any(|&(id, _)| id == key.id.get()))
}

/// Whether `(target, key)` is selected in the published snapshot.
fn is_selected(snap: &TimelineViewSnapshot, target: u64, key: u64) -> bool {
    snap.tracks
        .iter()
        .filter(|t| t.target.get() == target)
        .flat_map(|t| &t.keys)
        .any(|k| k.id.get() == key && k.selected)
}

#[cfg(test)]
#[path = "anchor_drag_tests.rs"]
mod tests;
