//! [`absorb_where_released`] — what a streamed key drag absorbs, where it was released. Split from
//! `intent_apply.rs` under the LOC cap.

use super::{Selection, TimelineState, for_selected_tracks, settle};

/// The keys a streamed drag moved absorb the stationary keys under them where it was released.
/// Also on a `BeginEdit` that finds a bracket never closed (pointer capture lost).
pub(super) fn absorb_where_released(state: &mut TimelineState) {
    let moved = state.history.take_moved();
    if moved.is_empty() {
        return;
    }
    let mut sel = Selection::default();
    for k in moved {
        sel.add(k);
    }
    for_selected_tracks(&mut state.doc, &sel, |track, ids| {
        track.absorb_under_moved(ids);
    });
    settle(&mut state.doc);
}
