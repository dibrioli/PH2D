//! The clipboard COPY (`copy_selection`) — split from `intent_apply.rs` under the
//! file LOC cap, one subject: reading the selected keys' real values onto the
//! clipboard. A sibling module that reaches the parent's state.

use ph2d_anim::{AnimTarget, Key};

use crate::state::TimelineState;

/// Copy the selected keys' real `(track, key)` — retime included — onto the
/// clipboard, time-rebased to the earliest. A selection that resolves to no live
/// key leaves the clipboard untouched (never clobber a good clipboard).
pub(super) fn copy_selection(state: &mut TimelineState) {
    let picked: Vec<(AnimTarget, Key)> = {
        let clip = state.doc.active_clip();
        state
            .selection
            .keys()
            .iter()
            .filter_map(|sk| Some((sk.target, clip.track(sk.target)?.key(sk.key)?)))
            .collect()
    };
    if picked.is_empty() {
        return;
    }
    state.clipboard.set_from_absolute(&picked);
}
