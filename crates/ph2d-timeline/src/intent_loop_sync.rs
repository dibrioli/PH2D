//! **Arming a playhead's loop from the document** — the loop-sync family, split out of
//! `intent_apply.rs` (sibling module, LOC cap). Three doors, one act: publish a view's loop
//! (clip/arrange), a container's own loop, or the shared kernel both use. The `Playhead` is a
//! live copy; the document is the truth, and these are where the copy is refreshed.

use crate::doc::TimelineDoc;
use ph2d_core::Playhead;

/// Publish the ACTIVE CLIP's loop for one VIEW onto its transport — the doc is the
/// truth, the `Playhead` is the copy. `keys` picks the pair: the Keys-view clip-clock
/// loop (`clip_playhead`) or the Arrange timeline loop (`playhead`).
///
/// Every intent that can change which clip is active (or its range) calls this. **So must
/// anything that swaps the document under the transport** — loading a project, above all:
/// the loop lives in the clip (`NamedClip.loop_range` / `keys_loop_range`, DOC v3/v5), so
/// without this a saved loop never comes back, and — worse — the *previous* project's loop
/// stays armed on the `Playhead` and quietly loops the new project over a range that belongs
/// to a file the artist already closed. The shell also calls it on a TAB switch (which is not
/// an intent) so the now-active clock adopts its own view's loop.
pub fn sync_transport_loop(doc: &TimelineDoc, playhead: &mut Playhead, keys: bool) {
    sync_loop(doc, playhead, keys);
}

/// Arm `playhead` with container `index`'s OWN loop — what entering a container
/// installs on the CONTAINER clock (`timeline_bridge::on_nav_change`), and what
/// [`TimelineIntent::SetContainerLoop`] re-arms on a toggle. The scene clock is
/// never in reach from here: the caller hands in the container playhead, and
/// that is the whole independence the three modes promise (Enio, 2026-07-22).
pub fn sync_container_loop(doc: &TimelineDoc, index: usize, playhead: &mut Playhead) {
    let (range, ping_pong) = doc.container_loop(index);
    match range {
        Some((a, b)) => playhead.set_loop(a, b),
        None => playhead.clear_loop(),
    }
    playhead.set_loop_mode(if ping_pong {
        ph2d_core::LoopMode::PingPong
    } else {
        ph2d_core::LoopMode::Wrap
    });
}

pub(crate) fn sync_loop(doc: &TimelineDoc, playhead: &mut Playhead, keys: bool) {
    match doc.active_loop_for(keys) {
        Some((a, b)) => playhead.set_loop(a, b),
        None => playhead.clear_loop(),
    }
    playhead.set_loop_mode(if doc.active_ping_pong_for(keys) {
        ph2d_core::LoopMode::PingPong
    } else {
        ph2d_core::LoopMode::Wrap
    });
}

/// Which duration [`apply_length`] authors.
#[derive(Clone, Copy, Debug)]
pub(crate) enum LengthTarget {
    /// The scene's (Arrange).
    Scene,
    /// The active clip's (Keys).
    ActiveClip,
    /// A container's, by index.
    Container(usize),
}

/// **Author one explicit duration** (Enio, 2026-07-23): one undo step, then the
/// loop invariant (`clamp_loops_to_lengths` — shrinking a duration pulls every
/// loop it governs back inside) and the LIVE playhead loop resync, so the
/// transport shows the clamped brace this same frame. One function for the three
/// intents, or the clamp would be three copies waiting to drift.
pub(crate) fn apply_length(
    state: &mut crate::TimelineState,
    playhead: &mut ph2d_core::Playhead,
    target: LengthTarget,
    len: Option<f64>,
) {
    super::edit(state, |doc, _| {
        match target {
            LengthTarget::Scene => doc.set_scene_length(len),
            LengthTarget::ActiveClip => {
                let ix = doc.active_index();
                doc.set_clip_length_override(ix, len);
            }
            LengthTarget::Container(c) => doc.set_container_length_override(c, len),
        }
        doc.clamp_loops_to_lengths();
    });
    match target {
        LengthTarget::Container(c) => sync_container_loop(&state.doc, c, playhead),
        _ => sync_loop(&state.doc, playhead, state.keys_mode),
    }
}
