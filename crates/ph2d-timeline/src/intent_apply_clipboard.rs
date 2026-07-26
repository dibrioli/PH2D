//! The clipboard COPY (`copy_selection`) — split from `intent_apply.rs` under the
//! file LOC cap, one subject: reading the selected keys' real values onto the
//! clipboard. A sibling module that reaches the parent's state.

use ph2d_anim::{AnimTarget, AnimValue, Interp};

use crate::state::TimelineState;

/// Copy the selected keys' real `(track, time, value, interp)` onto the
/// clipboard, time-rebased to the earliest. A selection that resolves to no live
/// key leaves the clipboard untouched (never clobber a good clipboard).
pub(super) fn copy_selection(state: &mut TimelineState) {
    let picked: Vec<(AnimTarget, f64, AnimValue, Interp)> = {
        let clip = state.doc.active_clip();
        state
            .selection
            .keys()
            .iter()
            .filter_map(|sk| {
                let k = clip.track(sk.target)?.key(sk.key)?;
                Some((sk.target, k.t.to_seconds(), k.value, k.interp))
            })
            .collect()
    };
    if picked.is_empty() {
        return;
    }
    state.clipboard.set_from_absolute(&picked);
}
