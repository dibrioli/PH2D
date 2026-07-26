//! **Buffer Curves** intent bodies (§5) — Store the graph editor's current curve
//! and Swap it with the buffered one. Split from `intent_apply.rs` under the LOC
//! cap; a CHILD module reaching the parent's `edit` for the swap's undo bracket.

use super::edit;
use crate::TimelineState;
use ph2d_anim::AnimTarget;

/// Body of [`TimelineIntent::StoreTrackBuffer`](crate::TimelineIntent::StoreTrackBuffer):
/// capture `target`'s exact curve into the buffer. Not a document edit — reads
/// the track and stashes a snapshot in session state, so it raises no undo step.
/// A missing track leaves the buffer as it was (nothing to capture).
pub(crate) fn store(state: &mut TimelineState, target: AnimTarget) {
    if let Some(track) = state.doc.active_clip().track(target) {
        state.curve_buffer = Some((target, track.snapshot_curve()));
    }
}

/// Body of [`TimelineIntent::SwapTrackBuffer`](crate::TimelineIntent::SwapTrackBuffer):
/// replace `target`'s curve with the buffered one and keep the just-replaced
/// curve as the new buffer — the A/B toggle. A no-op unless the buffer belongs to
/// THIS target (swapping one track's curve onto another would be a silent
/// corruption). The restore is one undo step; the buffer swap is transient.
pub(crate) fn swap(state: &mut TimelineState, target: AnimTarget) {
    // Take the buffer only if it is for this target; otherwise put it back and stop.
    let buffered = match state.curve_buffer.take() {
        Some((t, snap)) if t == target => snap,
        other => {
            state.curve_buffer = other;
            return;
        }
    };
    // Restore the buffered curve, capturing what was there to become the new buffer.
    let mut replaced = None;
    edit(state, |doc, _sel| {
        if let Some(track) = doc.active_clip_mut().track_mut(target) {
            replaced = Some(track.snapshot_curve());
            track.restore_curve(&buffered);
        }
    });
    // The displaced curve becomes the buffer (so a second swap returns to it). If
    // the track vanished mid-swap, keep the curve we took so it is not lost.
    state.curve_buffer = Some((target, replaced.unwrap_or(buffered)));
}
