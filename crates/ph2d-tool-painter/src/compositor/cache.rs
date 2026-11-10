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
    /// Cached straight encoded accumulator just below each adjustment layer,
    /// keyed by the adjustment's [`LayerId`]. Stable key + deterministic
    /// iteration (HR-5 — no `std::HashMap` in the compositor, ADR-0022).
    pub(crate) cuts: std::collections::BTreeMap<LayerId, Vec<[f32; 4]>>,
    /// Recompose only this sub-rect (the dab/transform bbox); `None` = full.
    pub(crate) dirty_rect: Option<Region>,
    /// B19.9: the accumulator and the adjusted copy of a root adjustment, kept between drag frames
    /// (a fresh canvas-sized `Vec` per frame is ~65 k page faults at 4K). Scratch: never cloned.
    pub(crate) acc: Vec<[f32; 4]>,
    pub(crate) ajustado: Vec<[f32; 4]>,
}

impl Clone for CompositorCache {
    fn clone(&self) -> Self {
        Self {
            cuts: self.cuts.clone(),
            dirty_rect: self.dirty_rect,
            acc: Vec::new(),
            ajustado: Vec::new(),
        }
    }
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

/// Composite the FULL canvas with the cut-point cache (ADR-0045 §2.7) — the
/// slider-drag FPS lever. On a param-only change of a root adjustment (after
/// `cache.invalidate_above(adj, stack)`), this restarts from that adjustment's
/// cached accumulator-below instead of recomposing the layers underneath; on a
/// cold cache (or structural change → `invalidate_from`) it composes the whole
/// stack and (re)fills the cuts. **Bit-identical to [`composite`]** — the layers
/// below the restart point are unchanged, and an adjustment resets `clip_base`, so
/// the restart state matches the full walk (gate `cache_matches_full_recompose`).
#[must_use]
pub fn composite_with_cache(
    stack: &LayerStack,
    src: &impl LayerPixelSource,
    width: u32,
    height: u32,
    cache: &mut CompositorCache,
) -> Vec<u8> {
    let root = stack.root();
    // Highest root adjustment (smallest index = MOST below-layers cached) whose
    // cut is still valid. Its cut is the composite of everything below it.
    let start = root.iter().enumerate().find(|&(_, &id)| {
        matches!(
            stack.get(id).map(|l| &l.kind),
            Some(LayerKind::Adjustment(_))
        ) && cache.cuts.contains_key(&id)
    });
    // B19.9: the accumulator lives in the cache between drag frames (no canvas-sized alloc per frame).
    let mut acc = if copias_antigas() {
        Vec::new()
    } else {
        std::mem::take(&mut cache.acc)
    };
    acc.clear();
    let ids: &[LayerId] = match start {
        // `composite_into` walks `ids` REVERSED (panel order is top-first;
        // index 0 = topmost). So the cut at panel index `i` = composite of the
        // layers BELOW it (`root[i+1..]`). Seed `acc` with that cut, then hand
        // `root[..=i]` (the adjustment + everything above): the reversed walk
        // processes the adjustment FIRST (on the correct below-acc), then the
        // layers above it. Restarting from the smallest valid index reuses the
        // largest cut (most below-layers cached) → fewest layers recomposed.
        Some((i, &id)) => {
            acc.extend_from_slice(&cache.cuts[&id]);
            &root[..=i]
        }
        None => {
            acc.resize((width as usize) * (height as usize), [0.0f32; 4]);
            root
        }
    };
    super::compose::composite_into(
        &mut acc,
        ids,
        stack,
        src,
        (width, height),
        0,
        0,
        width,
        height,
        0,
        Some(&mut *cache),
        None,
    );
    let out = super::compose::encode(&acc);
    cache.acc = acc;
    out
}

/// B19.9: the three canvas copies per drag frame, back on only for the probe's same-process A/B.
#[cfg(not(test))]
#[inline]
pub(super) fn copias_antigas() -> bool {
    false
}
#[cfg(test)]
pub(super) static COPIAS_ANTIGAS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
#[cfg(test)]
pub(super) fn copias_antigas() -> bool {
    COPIAS_ANTIGAS.load(std::sync::atomic::Ordering::Relaxed)
}

#[cfg(test)]
mod cache_tests;
