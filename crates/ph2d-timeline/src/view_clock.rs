//! **Which `Playhead` the view on screen drives** — one door for the question every reader of
//! the transport asks (ADR-0115 R8 emenda 3).
//!
//! The shell holds three clocks: the EDITOR's (the scene — the one the Motion, the physics, the
//! Flip and the sprites read), the CLIP's (the precomp of R8 emenda 2) and the open CONTAINER's.
//! **Without a stack there is ONE clock**: the Keys tab rules the editor's, so a scrub or a play
//! there moves the simulation (Enio, 2026-10-13: *«na aba Keys arrastar a régua e dar play tem
//! de mexer a simulação»*). The clip clock exists only under a stack — and there, when the clip
//! plays exactly once, [`follow_clip_clock`] carries the editor's clock to the same instant.
//!
//! ⚠️ `keys_mode` stays the Keys TAB (the SCOPE: solo the clip, its Dur, its loop — Enio
//! 2026-07-27); this module is the CLOCK, a separate fact the two used to share by accident.

use ph2d_core::Playhead;

use crate::{TimelineDoc, TimelineState};

/// The clock a view rules.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewClock {
    /// The editor's (scene) clock.
    #[default]
    Editor,
    /// The active clip's own clock — the Keys tab under a stack.
    Clip,
    /// The open container's own clock.
    Container,
}

impl ViewClock {
    /// The law: the Keys tab rules the clip clock **only under a stack**; a container view its
    /// own; everything else — the Keys tab without a stack included — the editor's.
    #[must_use]
    pub fn of(keys_mode: bool, container: Option<usize>, stacked: bool) -> Self {
        match (keys_mode, container) {
            (true, _) if stacked => Self::Clip,
            (true, _) | (false, None) => Self::Editor,
            (false, Some(_)) => Self::Container,
        }
    }

    /// The one of the three this view rules.
    pub fn pick<T>(self, editor: T, clip: T, container: T) -> T {
        match self {
            Self::Editor => editor,
            Self::Clip => clip,
            Self::Container => container,
        }
    }
}

impl TimelineState {
    /// The clock of the view on screen — the editor's while the scene RUNS, whatever the tab.
    #[must_use]
    pub fn view_clock(&self) -> ViewClock {
        if self.runtime {
            return ViewClock::Editor;
        }
        ViewClock::of(
            self.keys_mode,
            self.container_open,
            !self.doc.stack().is_empty(),
        )
    }
}

/// **Arm the clock of the view with that view's loop**, on the edge where the view changes
/// (the tab, or a stack appearing/vanishing under the Keys tab). The loop is per view
/// (`TimelineDoc::active_loop_for`) and a `Playhead` is its live copy: without a stack the
/// editor's clock carries the Keys loop on the Keys tab and the Arrange loop on Arrange.
/// Entering the clip clock from the one clock seeds it there, so the Keys playhead does not jump.
pub fn arm_view_loop(state: &mut TimelineState, editor: &mut Playhead, clip: &mut Playhead) {
    let keys = state.keys_mode;
    let on_clip = keys && !state.doc.stack().is_empty();
    let was = std::mem::replace(&mut state.armed_view, (keys, on_clip));
    if was == (keys, on_clip) {
        return;
    }
    if on_clip {
        if was == (true, false) {
            clip.seek(editor.time());
        }
        crate::sync_transport_loop(&state.doc, clip, true);
        crate::sync_transport_loop(&state.doc, editor, false);
    } else {
        crate::sync_transport_loop(&state.doc, editor, keys);
    }
}

/// **Under a stack, the editor's clock follows the Keys tab** — to the scene second where the
/// active clip reads the clip clock's second, when the clip plays exactly once
/// ([`crate::clip_scene_map`]). Only when the clip clock MOVED (`scrubbed`, or playing): a tab
/// switch alone never relocates the scene. Where the clip plays zero or two times the editor
/// stays, and the panel says why (the snapshot's `keys_scene`).
pub fn follow_clip_clock(
    doc: &TimelineDoc,
    clip: &Playhead,
    editor: &mut Playhead,
    scrubbed: bool,
) {
    if !(scrubbed || clip.is_playing()) {
        return;
    }
    if let Ok(map) = crate::clip_scene_map(doc, doc.active_index()) {
        editor.seek(map.host_time(clip.time()));
    }
}

#[cfg(test)]
#[path = "view_clock_tests.rs"]
mod tests;
