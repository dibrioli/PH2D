//! The sim-state **lifecycle** of [`GpuCook`] — seed, live-edit reseed and the
//! backwards scrub — split from `lib.rs` at the workspace LOC cap. Semantics
//! unchanged; `cook()` (the walk) stays in `lib.rs`, this file owns the
//! questions asked BETWEEN cooks.

use crate::GpuCook;

impl GpuCook {
    /// Forget the simulation state AND its scrub history — the next cook seeds
    /// from scratch, exactly as at tick 0 (when the `pre` edge reads Empty).
    ///
    /// Call this when the **graph changes**: a cached state is a function of the
    /// graph that produced it, so an edit invalidates every checkpoint (the
    /// Blender/Houdini semantics the CPU ring already follows — its `clear` has
    /// the same trigger). It also releases the ring's pinned VRAM.
    pub fn forget_state(&mut self) {
        self.prev.clear();
        self.last_tick = None;
        self.last_playhead = None;
        self.ring.clear();
        self.reseed = false;
    }

    /// Drop the sim state and **restart it AT the next tick cooked**, rather than
    /// re-deriving the history the old parameters produced.
    ///
    /// This is the **live-edit** invalidation (ADR-0130 D7), and it exists because
    /// [`Self::forget_state`] is the wrong one for an edit in flight.
    /// `forget_state` means *"this state is invalid, re-derive it"* — and
    /// [`Self::rewind_for`] honours that by anchoring an empty ring at tick 0, so
    /// the caller re-cooks `0..=target`. For a discrete edit that is one honest
    /// bake (Blender/Houdini re-bake a sim when you edit it). For a param a user
    /// is **holding and dragging**, it is `O(current tick)` re-simulated EVERY
    /// FRAME — which is not a bake, it is a freeze (the smoke: *"re-bake travado"*).
    ///
    /// An artist dragging an emitter's `rate` is asking *"what does it look like
    /// with THIS rate?"*, not *"replay the last forty seconds with it"*. So the
    /// honest answer is a fountain that RESTARTS: seed at the tick on screen and
    /// step forward from there, `O(1)` per edit. The scrub ring is dropped too —
    /// its checkpoints are the old params' sim, and a scrub through them would
    /// show a history this document never had.
    pub fn reseed_from_next_tick(&mut self) {
        self.prev.clear();
        self.ring.clear();
        self.last_tick = None;
        self.last_playhead = None;
        self.reseed = true;
    }

    /// Rewind (if needed) so that cooking `first..=target` in order stands the
    /// sim at `target`; returns that first tick. Call BEFORE the march.
    ///
    /// Forward — the overwhelming case, one tick per frame — this is
    /// `last_tick + 1` and touches nothing. Backwards (a scrub), it restores the
    /// newest checkpoint at or before the target and hands back its tick to
    /// re-sim from: **GGPO save/load/advance, without leaving the device**
    /// (ADR-0127 D5). A target the ring no longer covers anchors at the tick-0
    /// seed and re-sims forward, which is slow but always right — and is the CPU
    /// ring's own answer to the same question.
    ///
    /// Without this, a backwards scrub would cook `target` against the state of
    /// a LATER tick: `dt` would clamp to zero (the integrator's guard) so nothing
    /// would explode — it would just quietly show the future's pose and call it
    /// the past.
    pub fn rewind_for(&mut self, target: u64) -> u64 {
        // A LIVE EDIT invalidated the sim (D7): seed AT the target — do NOT
        // re-derive the history the old params produced. One cook, not `target`
        // of them; see [`Self::reseed_from_next_tick`] for why that distinction
        // is the difference between a bake and a freeze.
        if std::mem::take(&mut self.reseed) {
            self.prev.clear();
            self.last_tick = None;
            self.last_playhead = None;
            return target;
        }
        match self.last_tick {
            Some(t) if target > t => t + 1,
            _ => {
                // The anchor restores the CLOCK along with the state — the CPU
                // checkpoint restores `prev_playhead` for the same reason: a
                // birth law derives `dt` from it, and a scrub that seeded only
                // the streams would skip its first replayed tick's births
                // (ADR-0136).
                let (anchor, state, playhead) = self.ring.anchor_at_or_before(target);
                self.prev = state;
                self.last_tick = None;
                self.last_playhead = playhead;
                anchor
            }
        }
    }

    /// Retune the scrub ring's VRAM cap (default [`ring::RING_BYTES`]).
    pub fn set_ring_budget(&mut self, bytes: u64) {
        self.ring.set_budget(bytes);
    }

    /// Checkpoints the scrub ring holds, and the VRAM they pin (an upper bound).
    pub fn ring_stats(&self) -> (usize, u64) {
        (self.ring.len(), self.ring.bytes())
    }
}
