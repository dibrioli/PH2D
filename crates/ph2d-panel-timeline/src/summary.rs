//! The **Summary** channel — the one row above the tracks that aggregates every
//! key by time, and lets you grab a whole column of them at once (Blender's
//! Summary; a "master row" of frames).
//!
//! It owns no data. A column is just the keys of the published snapshot that
//! share a time, recomputed each frame, so it can never disagree with the rows
//! below it. That is also why dragging one is not a new operation: pressing a
//! column makes it the selection, and the selection is what
//! `MoveSelectedKeys` already moves. Right-clicking one likewise resolves, shell
//! side, into "select this column, then retune the selection".
//!
//! Two keys share a column iff they share a time **exactly**. The snapshot's
//! `t_seconds` comes from `RationalTime::to_seconds()`, which is deterministic,
//! so frame-snapped keys at the same frame always land in the same column and
//! two keys a microsecond apart are honestly two columns.

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture};
use ph2d_timeline::{SelectedKey, TimelineIntent, TimelineViewSnapshot};

use crate::key_drag::emit_move;
use crate::state::{self, KeyDrag, SummaryPress, TimelinePanelState};

/// One column of the Summary channel: a time, and every key sitting on it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Column {
    /// The shared time, in seconds.
    pub t_seconds: f64,
    /// Every key at that time, across every track, in track order.
    pub keys: Vec<SelectedKey>,
    /// All of them are selected — the column reads as "grabbed".
    pub all_selected: bool,
}

impl Column {
    /// The opaque handle dispatch carries for this column.
    pub(crate) fn t_bits(&self) -> u64 {
        self.t_seconds.to_bits()
    }
}

/// The Summary channel's columns, in time order. Empty when no track is bound —
/// the row itself is then not drawn.
pub(crate) fn columns(snap: &TimelineViewSnapshot) -> Vec<Column> {
    // Bucket by the time's exact bit pattern, then sort: `f64` is not `Ord`, but
    // the times here are finite and come straight from the document.
    let mut by_time: Vec<Column> = Vec::new();
    for track in &snap.tracks {
        for k in &track.keys {
            let sel = SelectedKey::new(track.target.get(), k.id.get());
            match by_time
                .iter_mut()
                .find(|c| c.t_seconds.to_bits() == k.t_seconds.to_bits())
            {
                Some(c) => {
                    c.keys.push(sel);
                    c.all_selected &= k.selected;
                }
                None => by_time.push(Column {
                    t_seconds: k.t_seconds,
                    keys: vec![sel],
                    all_selected: k.selected,
                }),
            }
        }
    }
    by_time.sort_by(|a, b| a.t_seconds.total_cmp(&b.t_seconds));
    by_time
}

/// The column at `t_bits`, if the snapshot still has one.
pub(crate) fn column_at(snap: &TimelineViewSnapshot, t_bits: u64) -> Option<Column> {
    columns(snap).into_iter().find(|c| c.t_bits() == t_bits)
}

/// The `t_bits` handle of the column a given track key sits on — the bridge from
/// a `Key` hit to a `SummaryKey` gesture when the column lock is on.
pub(crate) fn key_t_bits(snap: &TimelineViewSnapshot, target: u64, key: u64) -> Option<u64> {
    snap.tracks
        .iter()
        .find(|t| t.target.get() == target)?
        .keys
        .iter()
        .find(|k| k.id.get() == key)
        .map(|k| k.t_seconds.to_bits())
}

/// Interpret one gesture on a Summary diamond.
///
/// Press makes the column the selection (Shift adds it to whatever is already
/// selected), and from there the drag is the ordinary streamed
/// `MoveSelectedKeys` — every key in the column travels together, which is the
/// whole point of the row.
pub(crate) fn apply_gesture(
    state: &mut TimelinePanelState,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    t_bits: u64,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => begin(state, snap, t_bits, g),
        GesturePhase::Update => {
            if let Some(d) = state.key_drag.as_mut() {
                d.cur_x = g.x;
            }
            emit_move(state, px_per_s, snap);
        }
        GesturePhase::End => {
            if let Some(d) = state.key_drag.as_mut() {
                d.cur_x = g.x;
            }
            emit_move(state, px_per_s, snap);
            state.key_drag = None;
            state.summary_press = None;
            state::push_intent(TimelineIntent::EndEdit);
        }
        GesturePhase::Click | GesturePhase::DoubleClick => {
            // A plain click on a column that was ALREADY part of a wider
            // selection collapses down to it — the same courtesy a diamond does.
            if let Some(p) = state.summary_press.take()
                && p.was_selected
                && let Some(c) = column_at(snap, p.t_bits)
            {
                select_column(&c, false);
            }
            state.key_drag = None;
            state::push_intent(TimelineIntent::EndEdit);
        }
    }
}

fn begin(
    state: &mut TimelinePanelState,
    snap: &TimelineViewSnapshot,
    t_bits: u64,
    g: TimelineGesture,
) {
    state::push_intent(TimelineIntent::BeginEdit);
    let Some(c) = column_at(snap, t_bits) else {
        // The column vanished between paint and press: close the bracket we just
        // opened rather than leaving it to swallow the next atomic edit.
        state::push_intent(TimelineIntent::EndEdit);
        return;
    };
    // A press on a fully-selected column keeps the selection (so a drag moves
    // whatever else is selected with it) and only collapses on a plain click.
    if !c.all_selected || g.mods.shift {
        select_column(&c, g.mods.shift);
    }
    state.summary_press = Some(SummaryPress {
        t_bits,
        was_selected: c.all_selected,
    });
    state.key_drag = Some(KeyDrag {
        start_x: g.x,
        cur_x: g.x,
        collapse_to: None,
        applied_s: 0.0,
    });
}

/// Raise the selection intents for `c`. Without `additive` the previous
/// selection is replaced. The `ClearSelection` lands before the `AddToSelection`s
/// in the same drained batch, so the move that follows sees the column.
fn select_column(c: &Column, additive: bool) {
    if !additive {
        state::push_intent(TimelineIntent::ClearSelection);
    }
    for k in &c.keys {
        state::push_intent(TimelineIntent::AddToSelection(*k));
    }
}

#[cfg(test)]
#[path = "summary_tests.rs"]
mod tests;
