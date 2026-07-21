//! The GPU sim's backwards-scrub ring (GPU/M5 Fase 3, ADR-0127 **D5**) — the
//! device-side mirror of `ph2d-eval-motion`'s `CheckpointRing`.
//!
//! ## A checkpoint is a refcount, not a copy
//!
//! The CPU ring's own docs name the follow-up it wanted: *"an `Arc`/COW column
//! would make the deep clone cheap for large particle sets"*. On the GPU that is
//! already the case — a `GpuStream`'s columns are `Arc<wgpu::Buffer>` and a
//! written column is **immutable** ([`crate::stream`]), so last tick's state is
//! last tick's buffers and checkpointing it is cloning a map of refcounts. Not
//! even the `copy_buffer_to_buffer` the ADR budgeted for: nothing can overwrite
//! a checkpointed column, because nothing overwrites any column.
//!
//! What it costs is **VRAM residency**: a held buffer is one the pool will not
//! recycle, so the sim allocates a fresh one next tick. That is the whole price
//! of the scrub, and it is why the cap is in **bytes**.
//!
//! ## Bytes, not ticks
//!
//! The CPU ring used to be bounded by a COUNT (`RECENT_CAPACITY = 300`) —
//! sound when the state is small, and exactly the shape ADR-0117 named as the
//! bug it was: a count is a **multiplier**, not a ceiling. 300 checkpoints of a
//! 2M-element sim is ~24 GB. This ring always bounded the thing that actually
//! runs out ([`RING_BYTES`]) — and ADR-0137 brought the CPU ring to the same
//! rule (`CPU_RING_BYTES`). A scene big enough to fill the budget with one
//! checkpoint simply keeps one.
//!
//! ## Backfill + min-gap thinning (ADR-0137)
//!
//! Recording used to be strictly forward with oldest-first eviction — which a
//! LOOP turns into a permanent trap (the audit's §A2: after playing past the
//! loop's end, every wrap re-simmed the whole history and recorded none of it).
//! Now any on-stride tick not already present records (inserted in order — the
//! cook's replay ticks rebuild coverage), and the eviction victim is the entry
//! whose removal creates the SMALLEST neighbour gap, never the newest and never
//! the just-recorded: history thins in resolution instead of being amputated
//! from the side the next wrap needs.
//!
//! ## Missing the window is not an error
//!
//! A target older than the ring anchors at **tick 0 with no state**, which is
//! the deterministic seed, and re-sims forward — so *any* past tick is
//! reachable. This is the CPU ring's own policy, and the reason it is right here
//! too: the alternative D5 sketched (fall back to the CPU) cannot work
//! mid-session, because the CPU pump has not been marching and its clock is
//! stale — it would answer with a *different* simulation, not a rewind.

use crate::stream::GpuStream;
use ph2d_nodegraph::graph::NodeId;
use std::collections::BTreeMap;

/// The sim state at the top of one tick: what `pre` edges read to cook it.
type State = BTreeMap<NodeId, GpuStream>;

/// VRAM the ring may pin, in bytes. The knob, measured in the resource that
/// actually runs out; [`GpuCheckpointRing::set_budget`] retunes it.
pub const RING_BYTES: u64 = 128 * 1024 * 1024;

/// Checkpoint one tick in every `RING_STRIDE`; a scrub re-sims the ≤ 7 ticks
/// between anchors (a few ms — the cook is what this whole engine made cheap).
///
/// **Sparse, where the CPU ring is dense** — and the divergence is the point.
/// That ring reasoned "go dense unless the state-copy cost dominates `K ×
/// re-cook`", and copying a small CPU state is what it was weighing. Here the
/// copy is FREE (a refcount) and the cost moved somewhere else entirely:
/// **residency**. A checkpointed buffer is one the pool cannot recycle, so a
/// dense ring makes the sim allocate its whole state EVERY tick — it would spend
/// the ping-pong's entire win to buy a window a stride gets for an eighth of the
/// VRAM. Same reasoning, different dominant cost, opposite answer (ADR-0127 D5,
/// which asked for sparse).
pub const RING_STRIDE: u64 = 8;

/// A bounded ring of `(tick, state-before-that-tick, the previous cooked
/// playhead)`, newest at the back. The playhead rides along because a birth law
/// derives `dt` from it (ADR-0136): the CPU checkpoint restores `prev_playhead`
/// for exactly the same reason, and a scrub that seeded the state but not the
/// clock would cook its first replayed tick with `dt = 0` and silently skip
/// that tick's births.
pub struct GpuCheckpointRing {
    entries: Vec<(u64, State, Option<f64>)>,
    bytes: u64,
    budget: u64,
}

impl Default for GpuCheckpointRing {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            bytes: 0,
            budget: RING_BYTES,
        }
    }
}

/// The VRAM a state pins. Columns are shared by `Arc` between checkpoints (an
/// untouched column rides through every tick), so summing per checkpoint is an
/// **upper bound** on what is actually resident — the direction a cap must err.
fn state_bytes(state: &State) -> u64 {
    state
        .values()
        .flat_map(|s| s.cols.values())
        .map(|c| c.buffer.size())
        .sum()
}

impl GpuCheckpointRing {
    /// Retune the VRAM cap (default [`RING_BYTES`]).
    pub fn set_budget(&mut self, bytes: u64) {
        self.budget = bytes;
    }

    /// Would [`Self::record`] keep a checkpoint for `tick`? Only one on the
    /// [`RING_STRIDE`] grid, and only a tick not already present — a re-simmed
    /// tick in the window is identical by determinism (same device, same graph)
    /// and is skipped rather than duplicated; any ABSENT on-grid tick records,
    /// in either direction (the backfill, ADR-0137 — "strictly forward" was
    /// what starved every loop wrap).
    pub fn should_record(&self, tick: u64) -> bool {
        tick.is_multiple_of(RING_STRIDE)
            && self
                .entries
                .binary_search_by_key(&tick, |(t, _, _)| *t)
                .is_err()
    }

    /// Record the state to cook `tick` from, inserted in tick order (the
    /// backfill — a replayed tick behind the window is coverage the next wrap
    /// anchors on). Past the budget, evicts by min-gap thinning
    /// ([`Self::thinning_victim`]) — never the one just recorded, so a scene
    /// whose single checkpoint exceeds the cap still scrubs to the last anchor
    /// rather than keeping nothing.
    pub fn record(&mut self, tick: u64, state: &State, last_playhead: Option<f64>) {
        if !self.should_record(tick) {
            return;
        }
        let at = self.entries.partition_point(|(t, _, _)| *t < tick);
        self.bytes += state_bytes(state);
        self.entries
            .insert(at, (tick, state.clone(), last_playhead));
        while self.entries.len() > 1 && self.bytes > self.budget {
            let victim = self.thinning_victim(tick);
            let (_, old, _) = self.entries.remove(victim);
            self.bytes = self.bytes.saturating_sub(state_bytes(&old));
        }
    }

    /// The thinning victim (ADR-0137): the evictable entry whose removal
    /// creates the smallest neighbour gap — the most redundant anchor. The
    /// NEWEST entry and the just-recorded tick are protected (the recent
    /// scrub's anchor, and the admission rule above); when only those remain,
    /// fall back to the oldest that is not the newcomer.
    fn thinning_victim(&self, just_recorded: u64) -> usize {
        let n = self.entries.len();
        let candidate = (0..n - 1) // n-1: the newest entry is protected
            .filter(|&i| self.entries[i].0 != just_recorded)
            .min_by_key(|&i| {
                let left = if i == 0 { 0 } else { self.entries[i - 1].0 };
                self.entries[i + 1].0 - left
            });
        candidate.unwrap_or_else(|| usize::from(self.entries[0].0 == just_recorded))
    }

    /// The anchor to scrub from for `target`: the newest checkpoint at or before
    /// it, or `(0, empty)` — the tick-0 seed — when the target predates the
    /// window. The caller restores the state and re-cooks `anchor..=target`.
    pub fn anchor_at_or_before(&self, target: u64) -> (u64, State, Option<f64>) {
        self.entries
            .iter()
            .rev()
            .find(|(t, _, _)| *t <= target)
            .map(|(t, s, p)| (*t, s.clone(), *p))
            .unwrap_or_default()
    }

    /// Drop every checkpoint — the cached sim is invalid once the graph changes
    /// (the CPU ring's `clear`, same trigger). Releases the pinned VRAM.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.bytes = 0;
    }

    /// Checkpoints currently held.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// VRAM the ring pins (upper bound — see [`state_bytes`]).
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> State {
        State::new()
    }

    #[test]
    fn on_grid_absent_ticks_record_and_present_ones_do_not() {
        let mut r = GpuCheckpointRing::default();
        assert!(r.should_record(0));
        assert!(!r.should_record(1), "off the stride grid");
        r.record(0, &state(), None);
        r.record(1, &state(), None);
        assert_eq!(r.len(), 1, "the off-grid tick pins nothing");
        r.record(RING_STRIDE, &state(), None);
        assert!(
            !r.should_record(RING_STRIDE),
            "already covered — determinism"
        );
        assert!(!r.should_record(0));
        r.record(RING_STRIDE, &state(), None);
        assert_eq!(r.len(), 2, "a re-simmed tick must not duplicate");
    }

    /// The backfill (ADR-0137): an on-grid tick BEHIND the window records —
    /// sorted in, anchorable at once. The strictly-forward rule this replaces
    /// is what made every loop wrap re-sim the whole history, forever.
    #[test]
    fn a_replayed_tick_behind_the_window_backfills_and_anchors() {
        let mut r = GpuCheckpointRing::default();
        r.record(10 * RING_STRIDE, &state(), None);
        r.record(11 * RING_STRIDE, &state(), None);
        assert!(
            r.should_record(2 * RING_STRIDE),
            "behind the window and absent — wanted"
        );
        r.record(2 * RING_STRIDE, &state(), Some(0.25));
        assert_eq!(
            r.anchor_at_or_before(3 * RING_STRIDE).0,
            2 * RING_STRIDE,
            "the backfilled anchor serves the next wrap"
        );
        assert_eq!(
            r.anchor_at_or_before(3 * RING_STRIDE).2,
            Some(0.25),
            "…with its clock"
        );
    }

    /// Min-gap thinning (ADR-0137): past the budget the victim is the most
    /// REDUNDANT anchor, never the newest and never the just-recorded — the
    /// oldest-first rule this replaces amputated exactly the anchors a loop's
    /// next wrap needs.
    #[test]
    fn thinning_evicts_the_most_redundant_anchor_not_the_oldest() {
        let mut r = GpuCheckpointRing::default();
        r.set_budget(0);
        // With a zero budget every insert triggers eviction down to one entry —
        // record widely-spaced history plus a redundant middle anchor and
        // verify WHO survives each squeeze.
        // Zero-byte states never exceed a 0 budget (0 > 0 is false), so nothing
        // evicts by pressure here: exercise the VICTIM CHOICE directly.
        // Entries [5S, 6S, 8S]: removing 5S creates gap 6S−0 (the virtual seed
        // is the left neighbour); removing 6S creates gap 8S−5S = 3S — the
        // middle is the most redundant.
        r.record(5 * RING_STRIDE, &state(), None);
        r.record(6 * RING_STRIDE, &state(), None);
        r.record(8 * RING_STRIDE, &state(), None);
        assert_eq!(r.len(), 3);
        let victim = r.thinning_victim(8 * RING_STRIDE);
        assert_eq!(
            r.entries[victim].0,
            6 * RING_STRIDE,
            "the middle anchor is the most redundant (smallest created gap)"
        );
        let protected = r.thinning_victim(5 * RING_STRIDE);
        assert_ne!(
            r.entries[protected].0,
            8 * RING_STRIDE,
            "the newest entry is never the victim"
        );
        // …and an anchor AT tick 0 is always redundant against the virtual
        // seed: [0, 5S, 8S] evicts 0 (created gap 5S beats 8S−0).
        let mut r2 = GpuCheckpointRing::default();
        r2.record(0, &state(), None);
        r2.record(5 * RING_STRIDE, &state(), None);
        r2.record(8 * RING_STRIDE, &state(), None);
        assert_eq!(
            r2.entries[r2.thinning_victim(8 * RING_STRIDE)].0,
            0,
            "tick 0 duplicates the implicit seed — the cheapest anchor to shed"
        );
    }

    #[test]
    fn the_anchor_is_the_newest_checkpoint_at_or_before_the_target() {
        let mut r = GpuCheckpointRing::default();
        for k in 0..4u64 {
            r.record(k * RING_STRIDE, &state(), None);
        }
        let s = RING_STRIDE;
        assert_eq!(
            r.anchor_at_or_before(2 * s).0,
            2 * s,
            "exact hit: no re-sim"
        );
        assert_eq!(
            r.anchor_at_or_before(2 * s + 3).0,
            2 * s,
            "newest ≤ target — the caller re-sims the 3 ticks since"
        );
        assert_eq!(r.anchor_at_or_before(99 * s).0, 3 * s);
    }

    #[test]
    fn a_target_older_than_the_window_anchors_at_the_tick_zero_seed() {
        let mut r = GpuCheckpointRing::default();
        // A window that starts well past tick 4.
        for t in [10 * RING_STRIDE, 11 * RING_STRIDE] {
            r.record(t, &state(), None);
        }
        let (tick, s, _) = r.anchor_at_or_before(4);
        assert_eq!(tick, 0, "fall back to the seed and re-sim forward");
        assert!(s.is_empty(), "no state = what `pre` reads at tick 0");
    }

    /// The anchor restores the CLOCK with the state (ADR-0136): a birth law
    /// derives `dt` from the previous playhead, and a scrub that seeded only
    /// the streams would skip its first replayed tick's births — the CPU
    /// checkpoint restores `prev_playhead` for exactly this.
    #[test]
    fn the_anchor_carries_the_previous_playhead() {
        let mut r = GpuCheckpointRing::default();
        r.record(0, &state(), None);
        r.record(RING_STRIDE, &state(), Some(0.1166));
        assert_eq!(r.anchor_at_or_before(RING_STRIDE + 3).2, Some(0.1166));
        assert_eq!(r.anchor_at_or_before(0).2, None, "the seed has no history");
    }

    #[test]
    fn clearing_releases_everything() {
        let mut r = GpuCheckpointRing::default();
        r.record(0, &state(), None);
        r.clear();
        assert!(r.is_empty() && r.bytes() == 0);
        assert!(r.should_record(0), "a cleared ring re-records from scratch");
    }
}
