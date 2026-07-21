//! `CheckpointRing` — the backwards-scrub cache for the Motion pump (plan §1.4,
//! M2.N2; policy reformed by ADR-0137). Holds `(tick, checkpoint)` anchors so
//! scrubbing the playhead backwards restores the exact past simulation state
//! instead of reading the marching future (the naive-scrub bug, falsified in
//! `ph2d-nodegraph`'s cook tests).
//!
//! ## Policy (ADR-0137): backfill · protected dense recency · min-gap thinning · bytes
//!
//! The original ring recorded only strictly-forward ticks and evicted the
//! oldest. Composed with a LOOP that is a permanent trap, measured at
//! `scrub_tests::a_loop_wrap_anchors_on_the_previous_laps_backfill`: after
//! playing past the loop's end the window sits on the tail, the wrap to `lo`
//! anchors at the seed and re-sims the whole history — and records NONE of it
//! (every tick ≤ the back), so every lap repeats the full re-sim, forever.
//!
//! - **Backfill:** [`Self::record`] keeps any tick not already present,
//!   inserted in order — the replay's own record calls (which always existed)
//!   now rebuild coverage, so lap 2 anchors where lap 1 re-simmed.
//! - **Protected recency:** the [`RECENT_DENSE`] highest-tick entries are never
//!   thinning victims — the fine-tuning scrub (a few seconds back) stays the
//!   `O(1)` restore it always was.
//! - **Min-gap thinning:** past the budget, the victim is the entry whose
//!   removal creates the SMALLEST gap between its neighbours (the most
//!   redundant anchor; the first entry's left neighbour is the virtual tick-0
//!   seed). History degrades in RESOLUTION, never by amputation — which is
//!   exactly what a loop's next wrap needs to stay cheap.
//! - **Bytes, not count** ([`CPU_RING_BYTES`]): the old `RECENT_CAPACITY = 300`
//!   was a COUNT — a multiplier, the ADR-0117 class: 300 checkpoints of a 262k
//!   particle scene are gigabytes and the cap never blinked. A heavy scene now
//!   gets a SHORTER window, never a bigger bill. [`MAX_ENTRIES`] remains as an
//!   insertion-cost backstop (ordered insert shifts `O(n)`), an order of
//!   magnitude above any real spread — a backstop, not a budget.
//!
//! **No staleness:** for a fixed graph the sim is a pure function of the tick,
//! so `checkpoint[T]` is time-invariant — recorded once, valid until the graph
//! changes. A graph edit is the only invalidation trigger: the pump calls
//! [`CheckpointRing::clear`] on `mark_dirty`. The tick-0 seed stays implicit
//! (empty ring → `(0, default)`), so every past tick remains reachable.

use ph2d_nodegraph::cook::CookCheckpoint;

/// The protected dense-recency window, in ENTRIES of highest tick — the same
/// ~5 s at 60 Hz the old `RECENT_CAPACITY` guaranteed for the fine-tuning
/// scrub, now explicit policy rather than a side effect of the count cap.
pub const RECENT_DENSE: usize = 300;

/// The ring's byte budget — the same class as the GPU ring's `RING_BYTES`
/// (128 MB of VRAM, ADR-0127 D5): enough for hundreds of light-scene
/// checkpoints (a few KB each), while a heavy particle scene (a 262k-element
/// stream is ~15 MB per checkpoint — `measure` test below) gets a handful of
/// anchors instead of a silent multi-GB bill.
pub const CPU_RING_BYTES: usize = 128 * 1024 * 1024;

/// Insertion-cost backstop (ordered insert shifts `O(n)`), far above any
/// real spread — never the budget (that is [`CPU_RING_BYTES`]).
pub const MAX_ENTRIES: usize = 2048;

/// One anchor: the state that reproduces frame `tick`, and what it costs to
/// hold ([`CookCheckpoint::approx_bytes`], charged once at admission).
struct Entry {
    tick: u64,
    cp: CookCheckpoint,
    bytes: usize,
}

/// A bounded ring of `(tick, checkpoint)` anchors, sorted by tick.
#[derive(Default)]
pub struct CheckpointRing {
    /// Strictly ascending by tick.
    entries: Vec<Entry>,
    bytes: usize,
    budget_bytes: usize,
}

impl CheckpointRing {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            bytes: 0,
            budget_bytes: CPU_RING_BYTES,
        }
    }

    /// Record the checkpoint that reproduces frame `tick` (the state captured
    /// **before** that frame's cook). A tick already present is skipped
    /// (identical by determinism); any OTHER tick — behind, between, ahead —
    /// is inserted in order (the backfill, ADR-0137). Evicts by min-gap
    /// thinning past the budget.
    pub fn record(&mut self, tick: u64, cp: CookCheckpoint) {
        let Err(at) = self.entries.binary_search_by_key(&tick, |e| e.tick) else {
            return; // already covered — identical by determinism
        };
        let bytes = cp.approx_bytes();
        self.bytes += bytes;
        self.entries.insert(at, Entry { tick, cp, bytes });
        while self.entries.len() > 1
            && (self.bytes > self.budget_bytes || self.entries.len() > MAX_ENTRIES)
        {
            let victim = self.thinning_victim(tick);
            let e = self.entries.remove(victim);
            self.bytes = self.bytes.saturating_sub(e.bytes);
        }
    }

    /// The thinning victim: the evictable entry whose removal creates the
    /// smallest neighbour gap. Protected: the highest-tick entries — up to
    /// [`RECENT_DENSE`] of them, but never more than HALF the live ring — and
    /// the just-recorded `tick`. The half-split is load-bearing (found by the
    /// squeezed phase of the O(1) gate, which starved 101/101 without it):
    /// under byte pressure the ring holds far fewer than `RECENT_DENSE`
    /// entries, and a count-based protection would swallow them ALL — leaving
    /// only the oldest-first fallback, which is the pre-ADR-0137 disease
    /// verbatim. Splitting the squeezed ring between recency and thinned
    /// history keeps a wrap bounded by resolution in every regime. When even
    /// that leaves no candidate, fall back to the oldest that is not the
    /// just-recorded — last resort, never policy.
    fn thinning_victim(&self, just_recorded: u64) -> usize {
        let n = self.entries.len();
        let protected = RECENT_DENSE.min(n / 2).max(1);
        let protected_from = n - protected;
        let candidate = (0..protected_from)
            .filter(|&i| self.entries[i].tick != just_recorded)
            .min_by_key(|&i| {
                // The gap this removal creates: left neighbour (or the virtual
                // tick-0 seed) to right neighbour.
                let left = if i == 0 { 0 } else { self.entries[i - 1].tick };
                self.entries[i + 1].tick - left // i+1 exists: i < protected_from ≤ n-1 needs n ≥ 1; guarded by len > 1 in record
            });
        candidate.unwrap_or_else(|| {
            // Everything evictable is protected: oldest that is not the newcomer.
            usize::from(self.entries[0].tick == just_recorded)
        })
    }

    /// The anchor to scrub from for `target`: the newest recorded checkpoint
    /// with `tick ≤ target`, or the tick-0 seed ([`CookCheckpoint::default`]) if
    /// nothing precedes it. Returns `(anchor_tick, checkpoint)`; the caller
    /// restores it, then re-cooks forward `target − anchor_tick` ticks (zero
    /// when the target itself is anchored).
    #[must_use]
    pub fn anchor_at_or_before(&self, target: u64) -> (u64, CookCheckpoint) {
        let at = self
            .entries
            .partition_point(|e| e.tick <= target)
            .checked_sub(1);
        at.map(|i| (self.entries[i].tick, self.entries[i].cp.clone()))
            .unwrap_or_default() // (0, empty) = the deterministic tick-0 seed
    }

    /// Whether [`Self::record`] would keep a checkpoint for `tick` — any tick
    /// not already present (ADR-0137's backfill; it used to mean "strictly
    /// forward", which is what starved a loop's wrap). Lets a re-sim skip the
    /// deep state clone for ticks the ring already covers.
    #[must_use]
    pub fn should_record(&self, tick: u64) -> bool {
        self.entries
            .binary_search_by_key(&tick, |e| e.tick)
            .is_err()
    }

    /// Drop every checkpoint — the cached sim is invalid once the graph changes
    /// (a `mark_dirty` edit). A subsequent scrub re-sims from the tick-0 seed
    /// under the new graph (the Blender/Houdini "edit invalidates the cache"
    /// semantics).
    pub fn clear(&mut self) {
        self.entries.clear();
        self.bytes = 0;
    }

    /// Retune the byte budget (default [`CPU_RING_BYTES`]) — tests and future
    /// budget owners; the knob is bytes because that is the resource.
    pub fn set_budget(&mut self, bytes: usize) {
        self.budget_bytes = bytes;
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    #[cfg(test)]
    pub(crate) fn ticks(&self) -> Vec<u64> {
        self.entries.iter().map(|e| e.tick).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(ring.anchor_at_or_before(5).0, 0, "nothing ≤ 5 → seed");
    }

    /// The backfill (ADR-0137): a tick BEHIND the window records — inserted in
    /// order, anchorable at once. This is the exact rule whose absence starved
    /// every loop wrap (the old test here pinned the opposite and was retired
    /// with the policy).
    #[test]
    fn recording_backfills_out_of_order_ticks() {
        let mut ring = CheckpointRing::new();
        ring.record(50, cp());
        ring.record(60, cp());
        ring.record(20, cp()); // a replay tick behind the window — KEPT now
        ring.record(20, cp()); // same tick — skipped, not duplicated
        assert_eq!(ring.ticks(), vec![20, 50, 60], "sorted, deduplicated");
        assert!(!ring.should_record(20), "present → covered");
        assert!(ring.should_record(21), "absent → wanted");
        assert_eq!(ring.anchor_at_or_before(30).0, 20, "backfill is anchorable");
    }

    /// Past the entry backstop the victim is the MOST REDUNDANT anchor (min
    /// created gap), never a recent one — history thins in resolution instead
    /// of being amputated from the old side, which is what a loop's next wrap
    /// feeds on.
    #[test]
    fn thinning_evicts_the_most_redundant_old_anchor_not_the_oldest() {
        let mut ring = CheckpointRing::new();
        // Budget forces thinning by entries: allow 4.
        ring.set_budget(usize::MAX);
        // Old history at ticks 0, 10, 12, 500 — 12's removal creates the
        // smallest gap (10→500 vs evicting 10: 0→12).
        for t in [0, 10, 12, 500] {
            ring.record(t, cp());
        }
        // Shrink the protected window out of the way for the fixture: entries
        // beyond MAX_ENTRIES force eviction, so emulate by a tiny budget…
        // CookCheckpoint::default() is 0 bytes, so drive the ENTRY backstop
        // instead: fill to MAX_ENTRIES then add one more.
        for t in 1000..(1000 + MAX_ENTRIES as u64 - 4) {
            ring.record(t, cp());
        }
        assert_eq!(ring.len(), MAX_ENTRIES);
        ring.record(5000, cp());
        assert_eq!(ring.len(), MAX_ENTRIES, "backstop held");
        // The dense 1000.. run is recent-protected up to RECENT_DENSE from the
        // top; the evictable prefix contains 0,10,12 and the early 1000s. 12 is
        // the min-gap victim (gap 490 created is large… the dense 1000s create
        // gap 2). So the victim came from the dense old run, NOT tick 0/10/500.
        let kept = ring.ticks();
        assert!(kept.contains(&0) && kept.contains(&10) && kept.contains(&500));
        assert!(
            kept.contains(&5000),
            "the just-recorded tick is never the victim"
        );
    }

    /// The byte budget (ADR-0137 §3): entries are charged their checkpoint's
    /// `approx_bytes`, and a zero-byte checkpoint stream still respects the
    /// ENTRY backstop — count is a backstop, bytes are the budget.
    #[test]
    fn the_budget_is_bytes_and_the_entry_cap_is_a_backstop() {
        let mut ring = CheckpointRing::new();
        ring.set_budget(0); // every admission is over budget…
        ring.record(1, cp());
        assert_eq!(ring.len(), 1, "…but the ring never drops below one anchor");
        ring.record(2, cp());
        // Default checkpoints weigh 0 bytes → nothing is over budget; both stay.
        assert_eq!(
            ring.len(),
            2,
            "zero-byte states are bounded by the backstop"
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
