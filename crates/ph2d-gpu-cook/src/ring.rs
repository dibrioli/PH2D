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
//! The CPU ring is dense and bounded by a COUNT (`RECENT_CAPACITY = 300`) —
//! sound when the state is small, and exactly the shape ADR-0117 named as the
//! bug it was: a count is a **multiplier**, not a ceiling. 300 checkpoints of a
//! 2M-element sim is ~24 GB. So this ring bounds the thing that actually runs
//! out ([`RING_BYTES`]), and a scene big enough to fill it with one checkpoint
//! simply keeps one.
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

/// A bounded ring of `(tick, state-before-that-tick)`, newest at the back.
pub struct GpuCheckpointRing {
    entries: Vec<(u64, State)>,
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
    /// [`RING_STRIDE`] grid, and only a strictly forward one: a re-simmed tick
    /// already in the window is identical by determinism (same device, same
    /// graph), so it is skipped rather than duplicated — the CPU ring's rule.
    pub fn should_record(&self, tick: u64) -> bool {
        tick.is_multiple_of(RING_STRIDE) && self.entries.last().is_none_or(|(t, _)| tick > *t)
    }

    /// Record the state to cook `tick` from. Evicts the oldest while over
    /// budget — never the one just recorded, so a scene whose single checkpoint
    /// exceeds the cap still scrubs to the last anchor rather than keeping
    /// nothing.
    pub fn record(&mut self, tick: u64, state: &State) {
        if !self.should_record(tick) {
            return;
        }
        self.bytes += state_bytes(state);
        self.entries.push((tick, state.clone()));
        while self.entries.len() > 1 && self.bytes > self.budget {
            let (_, old) = self.entries.remove(0);
            self.bytes = self.bytes.saturating_sub(state_bytes(&old));
        }
    }

    /// The anchor to scrub from for `target`: the newest checkpoint at or before
    /// it, or `(0, empty)` — the tick-0 seed — when the target predates the
    /// window. The caller restores the state and re-cooks `anchor..=target`.
    pub fn anchor_at_or_before(&self, target: u64) -> (u64, State) {
        self.entries
            .iter()
            .rev()
            .find(|(t, _)| *t <= target)
            .map(|(t, s)| (*t, s.clone()))
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
    fn only_forward_ticks_on_the_stride_grid_are_recorded() {
        let mut r = GpuCheckpointRing::default();
        assert!(r.should_record(0));
        assert!(!r.should_record(1), "off the stride grid");
        r.record(0, &state());
        r.record(1, &state());
        assert_eq!(r.len(), 1, "the off-grid tick pins nothing");
        r.record(RING_STRIDE, &state());
        assert!(
            !r.should_record(RING_STRIDE),
            "already covered — determinism"
        );
        assert!(!r.should_record(0));
        r.record(RING_STRIDE, &state());
        assert_eq!(r.len(), 2, "a re-simmed tick must not duplicate");
    }

    #[test]
    fn the_anchor_is_the_newest_checkpoint_at_or_before_the_target() {
        let mut r = GpuCheckpointRing::default();
        for k in 0..4u64 {
            r.record(k * RING_STRIDE, &state());
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
            r.record(t, &state());
        }
        let (tick, s) = r.anchor_at_or_before(4);
        assert_eq!(tick, 0, "fall back to the seed and re-sim forward");
        assert!(s.is_empty(), "no state = what `pre` reads at tick 0");
    }

    #[test]
    fn clearing_releases_everything() {
        let mut r = GpuCheckpointRing::default();
        r.record(0, &state());
        r.clear();
        assert!(r.is_empty() && r.bytes() == 0);
        assert!(r.should_record(0), "a cleared ring re-records from scratch");
    }
}
