//! **The per-clip, per-view loop pair** — where each clip parks its Arrange loop
//! and its Keys loop, and the accessors the transport syncs them through.
//!
//! Split from `doc.rs` under the 700-LOC workspace cap; a unit in its own right:
//! "which loop brackets this view" is one family of questions with one owner
//! (the loop is PER CLIP, Enio 2026-07-12, and PER VIEW, 2026-07-16 — a
//! container's own loop lives with the container, in `nest.rs`).

use super::TimelineDoc;

impl TimelineDoc {
    /// The active clip's loop range for one VIEW, if it has one. `keys` picks the
    /// Keys-view loop (the clip's own clock); `false` picks the Arrange/timeline
    /// loop. The two are stored and returned independently — the loop area is
    /// per clip AND per view (Enio, 2026-07-16).
    #[must_use]
    pub fn active_loop_for(&self, keys: bool) -> Option<(f64, f64)> {
        let c = &self.clips[self.active_clip];
        if keys {
            c.keys_loop_range
        } else {
            c.loop_range
        }
    }

    /// Park `range` on the ACTIVE clip's `keys`/Arrange loop. The caller mirrors it
    /// into the matching `Playhead` (clip clock in Keys, timeline in Arrange), which
    /// owns the live loop — this is the copy that survives a clip switch.
    pub fn set_active_loop_for(&mut self, keys: bool, range: Option<(f64, f64)>) {
        let c = &mut self.clips[self.active_clip];
        if keys {
            c.keys_loop_range = range;
        } else {
            c.loop_range = range;
        }
    }

    /// Whether the active clip's `keys`/Arrange loop ping-pongs.
    #[must_use]
    pub fn active_ping_pong_for(&self, keys: bool) -> bool {
        let c = &self.clips[self.active_clip];
        if keys {
            c.keys_loop_ping_pong
        } else {
            c.loop_ping_pong
        }
    }

    /// Set whether the active clip's `keys`/Arrange loop ping-pongs.
    pub fn set_active_ping_pong_for(&mut self, keys: bool, on: bool) {
        let c = &mut self.clips[self.active_clip];
        if keys {
            c.keys_loop_ping_pong = on;
        } else {
            c.loop_ping_pong = on;
        }
    }

    /// The active clip's Arrange (timeline) loop range — [`Self::active_loop_for`]`(false)`.
    /// The project-load path and older callers read the timeline loop through this.
    #[must_use]
    pub fn active_loop(&self) -> Option<(f64, f64)> {
        self.active_loop_for(false)
    }

    /// Park `range` on the active clip's Arrange loop.
    pub fn set_active_loop(&mut self, range: Option<(f64, f64)>) {
        self.set_active_loop_for(false, range);
    }

    /// Whether the active clip's Arrange loop ping-pongs.
    #[must_use]
    pub fn active_ping_pong(&self) -> bool {
        self.active_ping_pong_for(false)
    }

    /// Set whether the active clip's Arrange loop ping-pongs.
    pub fn set_active_ping_pong(&mut self, on: bool) {
        self.set_active_ping_pong_for(false, on);
    }

    /// **No loop reaches past the authored duration that governs it** (Enio,
    /// 2026-07-23: *"faça a área do loop/pingpong não ultrapassar a área definida
    /// pela duração"*). One invariant, enforced whole: every clip's Arrange loop
    /// against the SCENE's authored length, every clip's Keys loop against that
    /// clip's own, every container's loop against its own. A loop lying wholly
    /// beyond the end brackets nothing and is CLEARED. Called after every edit
    /// that can break it — arming or dragging a brace, and shrinking a duration —
    /// so the five arms share one law instead of five copies of it.
    ///
    /// Only an AUTHORED duration constrains (a derived end never did), and a loop
    /// already inside is byte-untouched.
    pub(crate) fn clamp_loops_to_lengths(&mut self) {
        let clamp = |range: &mut Option<(f64, f64)>, end: Option<f64>| {
            let Some(end) = end else { return };
            if let Some((a, b)) = *range {
                *range = (a < end).then_some((a, b.min(end)));
            }
        };
        let scene = self.scene_length;
        for i in 0..self.clips().len() {
            let own = self.clip_length_override(i);
            let c = &mut self.clips[i];
            clamp(&mut c.loop_range, scene);
            clamp(&mut c.keys_loop_range, own);
        }
        for i in 0..self.containers().len() {
            let own = self.container_length_override(i);
            if let Some(c) = self.containers_mut().get_mut(i) {
                clamp(&mut c.loop_range, own);
            }
        }
    }
}
