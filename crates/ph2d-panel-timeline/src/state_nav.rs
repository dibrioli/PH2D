//! **Where the animator IS, and which of it the shell is told** — the panel's navigation half
//! (ADR-0133 §5, amended 2026-07-21).
//!
//! Split out of `state.rs` when it crossed the 600-LOC panel cap (HR-18), and a unit in its
//! own right: everything here answers one question — *which stack is on screen?* — and the
//! three parties that ask it (the tab strip, the breadcrumb, the shell) have to get the SAME
//! answer or an edit lands somewhere the artist is not looking.
//!
//! The state itself stays in the parent's `thread_local!` block: these are its accessors, not
//! a second copy of it.

use super::{KEYS_MODE, OPEN_CONTAINER, SCENE_ROOT, TimelinePanelState, drop_row_gestures};
use std::cell::Cell;

/// Publish whether the panel is on the Keys tab (called by `paint` each frame).
pub(crate) fn publish_keys_mode(on: bool) {
    KEYS_MODE.with(|c| c.set(on));
}

/// Publish whether the panel is showing the SCENE's stack (called by `paint` each frame,
/// beside [`publish_keys_mode`]). See [`SCENE_ROOT`].
pub(crate) fn publish_scene_root(on: bool) {
    SCENE_ROOT.with(|c| c.set(on));
}

/// **Switch tabs** — the ONE door, because switching is three things at once.
///
/// It moves the panel's view state, publishes which STACK is now on screen (so
/// [`edit_path`] answers correctly *before* the next paint — the shell reads the host at the
/// top of a frame, and a tab that only took effect at paint time would route one frame of
/// edits into the stack the animator just left), and drops any gesture whose rows went away.
/// Assigning `state.tab` directly does the first and forgets the other two.
pub(crate) fn set_tab(state: &mut TimelinePanelState, want: crate::tab::Tab) {
    publish_scene_root(want.scene_root());
    if state.tab == want {
        return;
    }
    state.tab = want;
    drop_row_gestures(state);
}

/// **Is the timeline panel on the Keys tab?** Read by the shell to drive the CLIP
/// playhead and solo the active clip (the AE precomp model). Reflects the panel's
/// last paint, so it lags a tab switch by at most one frame — imperceptible, and
/// the same latency every panel→shell mirror here already has.
#[must_use]
pub fn keys_mode() -> bool {
    KEYS_MODE.with(Cell::get)
}

/// **Enter a container** (or leave, with `None`) — the breadcrumb's whole job.
///
/// Panel-local, and taking effect on the next frame's publish: the shell reads
/// [`edit_host`] before draining intents, so the edit that follows a click lands in the
/// stack the animator is looking at.
pub(crate) fn enter_container(step: ph2d_timeline::EnterStep) {
    OPEN_CONTAINER.with(|p| p.borrow_mut().push(step));
}

/// Pop the trail back to `depth` levels (0 = the scene root). Clicking where you already are
/// truncates to the length it already has, which is why the trailing segment is safe to paint
/// as part of the trail rather than special-cased.
pub(crate) fn pop_to_depth(depth: usize) {
    OPEN_CONTAINER.with(|p| p.borrow_mut().truncate(depth));
}

/// **How deep the animator has walked**, outermost first — read by the shell each frame and
/// stamped onto `TimelineState::edit_path` before intents drain, the same channel
/// [`keys_mode`] rides.
/// **The trail as the panel holds it**, whatever tab is on screen — what the breadcrumb pops
/// and what a tab switch reads. [`edit_path`] is what gets PUBLISHED, which is not the same
/// thing on the Arrange tab.
pub(crate) fn trail_len() -> usize {
    OPEN_CONTAINER.with(|p| p.borrow().len())
}

#[must_use]
pub fn edit_path() -> Vec<ph2d_timeline::EnterStep> {
    if SCENE_ROOT.with(Cell::get) {
        // Arrange IS the scene's stack. The trail is not cleared — leaving the tab and
        // coming back has to land where you were — it simply does not apply here.
        return Vec::new();
    }
    OPEN_CONTAINER.with(|p| p.borrow().clone())
}

/// **Which stack an edit lands in** — the innermost of [`edit_path`]. Derived, never stored
/// beside it: two answers to "where is the animator" would drift the first time one of them
/// was updated alone.
#[must_use]
pub fn edit_host() -> ph2d_timeline::StackHost {
    // Off [`edit_path`], never off `OPEN_CONTAINER`: the published path already answers
    // "which stack is on screen" (Arrange drops the trail), and a host derived from the raw
    // trail would route an Arrange edit into a container the tab is not showing.
    edit_path()
        .last()
        .map_or(ph2d_timeline::StackHost::Document, |step| {
            ph2d_timeline::StackHost::Container(step.container)
        })
}

/// **Which container is open**, if any — derived from the published trail, so what the lanes
/// show and what the breadcrumb says can never name different containers.
///
/// `None` at the scene root, which on the Containers tab is the LIST.
#[must_use]
pub fn open_container() -> Option<usize> {
    edit_path().last().map(|s| s.container)
}

/// **Show `container` in the Containers tab** — the one door for "the tab is showing THIS",
/// walked by a double-click on that container's bar in the list.
///
/// It REPLACES the trail rather than pushing onto it: the list is the tab's ROOT, so a bar
/// you double-click there is one level deep by definition. Pushing would read as *"inside the
/// one I was already in"*, which is a different place with a different clock.
pub(crate) fn open_container_root(state: &mut TimelinePanelState, container: usize) {
    pop_to_depth(0);
    enter_container(ph2d_timeline::EnterStep {
        container,
        lane: 0,
        strip: None,
    });
    set_tab(state, crate::tab::Tab::Containers);
}
