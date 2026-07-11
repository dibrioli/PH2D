//! `CheckpointRing` — the backwards-scrub cache for the Motion pump (plan §1.4,
//! M2.N2). Holds one [`CookCheckpoint`] per forward tick over a bounded recent
//! window, so scrubbing the playhead backwards restores the exact past
//! simulation state instead of reading the marching future (the naive-scrub
//! bug, falsified in `ph2d-nodegraph`'s cook tests).
//!
//! ## Policy: dense window + a tick-0 seed anchor
//!
//! The state-of-the-art survey (GGPO/GGRS rollback, Houdini DOP cache, Blender
//! bake, binjgb rewind) says the ONLY open choice here is the cache policy, and
//! the deciding rule (GGRS's own) is: go dense unless the state-copy cost
//! dominates `K × re-cook`. Motion-graphics state is small and a cook is cheap,
//! so this ring is **dense (one checkpoint per tick)** — recent scrubs are an
//! `O(1)` restore with zero re-sim (GGPO "save every frame"). The window is
//! bounded ([`RECENT_CAPACITY`]); a target older than the window falls back to
//! the **tick-0 seed** ([`CookCheckpoint::default`] — empty `pre`, tick 0) and
//! re-sims forward, so *any* past tick is reachable, bit-exact by determinism.
//!
//! **No staleness:** for a fixed graph the sim is a pure function of the tick,
//! so `checkpoint[T]` is time-invariant — recorded once, valid until the graph
//! changes. A graph edit is the only invalidation trigger: the pump calls
//! [`CheckpointRing::clear`] on `mark_dirty`.
//!
//! **Follow-ups (named, measured):** an `Arc`/COW column would make the deep
//! clone cheap for large particle sets; a coarse stride (checkpoint every K)
//! plus base+delta (binjgb) would bound a far scrub's re-sim and cover a long
//! timeline in less memory — both drop in behind this same record/anchor API
//! (the GGRS sparse-saving shape), and neither is needed while the state is
//! small.

use ph2d_nodegraph::cook::CookCheckpoint;
use std::collections::VecDeque;

/// How many recent per-tick checkpoints to retain. ~5 s at 60 Hz — comfortably
/// covers the fine-tuning scrub range (the hot path) at a few MB even for a
/// heavy particle scene, while a farther scrub re-sims from the tick-0 seed.
/// This is the memory knob; a coarse stride is the drop-in when it bites.
pub const RECENT_CAPACITY: usize = 300;

/// A bounded, dense ring of `(tick, checkpoint)` for backwards scrub.
#[derive(Default)]
pub struct CheckpointRing {
    /// Recent checkpoints, strictly ascending by tick, newest at the back.
    recent: VecDeque<(u64, CookCheckpoint)>,
}

impl CheckpointRing {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record the checkpoint that reproduces frame `tick` (the state captured
    /// **before** that frame's cook). Appends only a strictly-forward tick — a
    /// re-recorded tick already in the window is identical by determinism, so it
    /// is skipped rather than duplicated. Evicts the oldest past
    /// [`RECENT_CAPACITY`].
    pub fn record(&mut self, tick: u64, cp: CookCheckpoint) {
        if self.recent.back().is_some_and(|(bt, _)| tick <= *bt) {
            return; // not newer than the window's front edge — already covered
        }
        self.recent.push_back((tick, cp));
        while self.recent.len() > RECENT_CAPACITY {
            self.recent.pop_front();
        }
    }

    /// The anchor to scrub from for `target`: the newest recorded checkpoint
    /// with `tick ≤ target`, or the tick-0 seed ([`CookCheckpoint::default`]) if
    /// the target predates the window. Returns `(anchor_tick, checkpoint)`; the
    /// caller restores it, then re-cooks forward `target − anchor_tick` ticks
    /// (zero when the target itself is in the dense window).
    #[must_use]
    pub fn anchor_at_or_before(&self, target: u64) -> (u64, CookCheckpoint) {
        self.recent
            .iter()
            .rev()
            .find(|(t, _)| *t <= target)
            .map(|(t, cp)| (*t, cp.clone()))
            .unwrap_or_default() // (0, empty) = the deterministic tick-0 seed
    }

    /// Whether [`Self::record`] would keep a checkpoint for `tick` (a strictly
    /// forward tick). Lets a re-sim skip the deep state clone for ticks the ring
    /// already covers.
    #[must_use]
    pub fn should_record(&self, tick: u64) -> bool {
        self.recent.back().is_none_or(|(bt, _)| tick > *bt)
    }

    /// Drop every checkpoint — the cached sim is invalid once the graph changes
    /// (a `mark_dirty` edit). A subsequent scrub re-sims from the tick-0 seed
    /// under the new graph (the Blender/Houdini "edit invalidates the cache"
    /// semantics).
    pub fn clear(&mut self) {
        self.recent.clear();
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.recent.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A checkpoint tagged by a value we can read back, to prove the ring hands
    // out the RIGHT one. `CookCheckpoint` is opaque here, so we key indirectly:
    // the ring's contract is purely about ticks, so we assert on the tick it
    // pairs. (Bit-exact state reproduction is proved in ph2d-nodegraph.)
    fn cp() -> CookCheckpoint {
        CookCheckpoint::default()
    }

    #[test]
    fn anchor_returns_the_newest_at_or_before_the_target() {
        let mut ring = CheckpointRing::new();
        for t in [10, 20, 30, 40] {
            ring.record(t, cp());
        }
        assert_eq!(ring.anchor_at_or_before(35).0, 30, "newest ≤ 35");
        assert_eq!(ring.anchor_at_or_before(40).0, 40, "exact hit");
        assert_eq!(ring.anchor_at_or_before(1000).0, 40, "clamps to newest");
    }

    #[test]
    fn a_target_before_the_window_falls_back_to_the_tick_zero_seed() {
        let mut ring = CheckpointRing::new();
        ring.record(50, cp());
        ring.record(60, cp());
        // Nothing ≤ 20 recorded → the seed (tick 0), which re-sims from the start.
        assert_eq!(ring.anchor_at_or_before(20).0, 0, "seed anchor");
    }

    #[test]
    fn recording_is_strictly_forward_and_bounded() {
        let mut ring = CheckpointRing::new();
        ring.record(5, cp());
        ring.record(5, cp()); // same tick — skipped, not duplicated
        ring.record(3, cp()); // older — skipped (already covered)
        assert_eq!(ring.len(), 1, "only strictly-forward ticks append");

        let mut ring = CheckpointRing::new();
        for t in 0..(RECENT_CAPACITY as u64 + 50) {
            ring.record(t, cp());
        }
        assert_eq!(
            ring.len(),
            RECENT_CAPACITY,
            "evicts the oldest past capacity"
        );
        // The oldest 50 were evicted → a scrub to an evicted tick hits the seed.
        assert_eq!(ring.anchor_at_or_before(10).0, 0, "evicted → seed");
        assert_eq!(
            ring.anchor_at_or_before(RECENT_CAPACITY as u64 + 40).0,
            RECENT_CAPACITY as u64 + 40,
            "recent still dense"
        );
    }

    #[test]
    fn clear_drops_everything() {
        let mut ring = CheckpointRing::new();
        ring.record(1, cp());
        ring.record(2, cp());
        ring.clear();
        assert_eq!(ring.len(), 0);
        assert_eq!(ring.anchor_at_or_before(2).0, 0, "empty → seed");
    }
}
