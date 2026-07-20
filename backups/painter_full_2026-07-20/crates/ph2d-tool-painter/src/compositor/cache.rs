//! `CompositorCache` (ADR-0045 §2.7 cut-point cache), split out of the
//! former `compositor.rs` (pure mechanical move).

use super::*;

/// W4 compositor recomposition cache (ADR-0045 §2.7) — **skeleton**. Each
/// adjustment layer is a "cut point": the accumulator BELOW it is cached, so a
/// change at layer N only invalidates cut points ≥ N (above stay valid). This
/// is the perf lever for the soft `adjustment_layer_recomposition_perf_4k` gate
/// (≤1 ms slider-drag @ 4K). v1 composites inline (no cache wired into the hot
/// path yet); this type + the dirty-rect field land the contract (HR-5
/// `BTreeMap`, not `HashMap`) for the impl/T4.x to wire.
#[derive(Default)]
pub struct CompositorCache {
    /// Cached straight-linear accumulator just below each adjustment layer,
    /// keyed by the adjustment's [`LayerId`]. Stable key + deterministic
    /// iteration (HR-5 — no `std::HashMap` in the compositor, ADR-0022).
    pub(crate) cuts: std::collections::BTreeMap<LayerId, Vec<[f32; 4]>>,
    /// Recompose only this sub-rect (the dab/transform bbox); `None` = full.
    pub(crate) dirty_rect: Option<Region>,
}

impl CompositorCache {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// STRUCTURAL change (add / remove / reorder / visibility / opacity / pixels
    /// of a layer): the composite below some cut points changed. Conservative-
    /// correct: clear ALL cuts (cheap — they refill on the next full compose). A
    /// finer LayerId→depth mapping could keep cuts strictly below `changed`, but
    /// "clear all on structural" is the safe default (ADR-0045 §2.7 note).
    pub fn invalidate_from(&mut self, _changed: LayerId, _stack: &LayerStack) {
        self.cuts.clear();
    }

    /// PARAM-only change of root adjustment `adj` (the slider-drag hot path): the
    /// layers BELOW `adj` are unchanged, so `cuts[adj]` (the acc-below) stays
    /// valid; only the cuts ABOVE `adj` (which consumed `adj`'s output) are
    /// dropped. `composite_with_cache` then restarts from `adj`'s cut instead of
    /// recomposing the stack. `adj` not in the root ⇒ clear all (conservative).
    pub fn invalidate_above(&mut self, adj: LayerId, stack: &LayerStack) {
        let root = stack.root();
        let pos = |id: LayerId| root.iter().position(|&x| x == id);
        let Some(adj_pos) = pos(adj) else {
            self.cuts.clear();
            return;
        };
        // Panel order is top-first → "above" = smaller index. Keep cuts at the
        // adjustment and below (index ≥ adj_pos); drop those above it.
        self.cuts
            .retain(|&k, _| pos(k).is_some_and(|p| p >= adj_pos));
    }

    /// Mark the sub-rect to recompose next drain.
    pub fn mark_dirty(&mut self, region: Region) {
        self.dirty_rect = Some(region);
    }

    #[must_use]
    pub fn dirty_rect(&self) -> Option<Region> {
        self.dirty_rect
    }
}
