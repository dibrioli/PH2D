//! **The runtime scratch plumbing** — how this frame's live strips, clocks, and composed
//! `LinkFrame` are borrowed out for the apply and parked back on the document.
//!
//! Split out of `doc.rs` under the 700-LOC workspace cap, and a unit in its own right: the
//! scratch is a CACHE keyed on the playhead, and every reader (key authoring, the autokey
//! diff) must be answered "what is the stack doing NOW" — which only holds because the apply
//! primed it at this same instant. A CHILD module (not a sibling) so it reads the document's
//! private `scratch` field, the idiom `extent`/`loops` already use.

use super::TimelineDoc;
use crate::stack_frames::StackScratch;

impl TimelineDoc {
    /// Move this frame's scratch out for the apply to refill (it needs the
    /// document immutably while writing to the index). The buffers' capacity
    /// rides along, so [`Self::put_scratch`] returns them warm.
    pub(crate) fn take_scratch(&mut self) -> StackScratch {
        core::mem::take(&mut self.scratch)
    }

    /// Park the scratch back on the document, capacity and all.
    pub(crate) fn put_scratch(&mut self, scratch: StackScratch) {
        self.scratch = scratch;
    }

    /// **Stash the frame's composed `LinkFrame` onto the scratch** (ADR-0146 W6, C2), for a
    /// view that keeps no local scratch ([`crate::apply_active_clip`]) — the autokey diff then
    /// READS it (`shown_value`/`curve_value`) instead of re-deriving. `put_scratch` would
    /// overwrite it, so the views that DO take a scratch set the field on theirs directly.
    pub(crate) fn stash_composed_links(&mut self, links: crate::frame_solve::LinkFrame) {
        self.scratch.composed_links = links;
    }

    /// This frame's resolved strips + clocks, as the apply left them. Key
    /// authoring runs AFTER the apply, on the same playhead, so what it reads
    /// here is exactly what the scene is showing.
    pub(crate) fn scratch(&self) -> &StackScratch {
        &self.scratch
    }

    /// **Make the stack's scratch describe `t`.** Idempotent, and free when it
    /// already does (the apply built it at this same `t` a moment ago).
    ///
    /// Every reader of the scratch — where a key lands, whether a pose is even
    /// reachable — is asking "what is the stack doing *now*", and was being
    /// answered "what was it doing when the apply last ran". In production those
    /// are the same instant, which is exactly what makes the coupling invisible
    /// and exactly the shape of bug that has broken this module three times over
    /// ([[feedback_derived_coordinate_seed_must_match_sample]]). A caller that
    /// authors keys calls this first and is simply right, whether or not an apply
    /// ran before it.
    pub fn prime_stack(&mut self, t: f64) {
        // Unconditional. A "has the time changed" guard would be blind to the half
        // of the dependency that matters more — the DOCUMENT changing at the same
        // instant (a strip moved, a clip deleted) — and a cache that is fresh for
        // one of its two inputs is a cache that lies. The rebuild is a scan of the
        // live strips; the apply already pays it once a frame.
        let mut scratch = self.take_scratch();
        scratch.rebuild(self, t);
        self.put_scratch(scratch);
    }
}
