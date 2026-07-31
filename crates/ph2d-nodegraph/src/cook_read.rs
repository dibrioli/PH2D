//! `Cook`'s cached-output readers — split from `cook.rs` for the HR-18 LOC cap; declared there as
//! a `#[path]` sibling, so `super` is `cook`. A descendant module reaches `Cook`'s private cache
//! fields, so these stay `pub(super)` inherent methods rather than a public surface.

use super::*;

impl Cook {
    /// This tick's output for `(node, key)` at `port` — the cache read the forward-edge resolver
    /// uses. `Empty` for a port that never cooked (a disconnected or unknown output).
    pub(super) fn cur_output(&self, node: NodeId, key: ScopeKey, port: usize) -> CookValue {
        self.cache
            .get(&(node, key))
            .and_then(|c| c.outputs.get(port))
            .cloned()
            .unwrap_or_default()
    }

    /// The PREVIOUS tick's output for `node` at `port` — what a `pre` edge reads without recursing.
    /// `Empty` before the node has ever produced (its feedback loop starts from nothing).
    pub(super) fn prev_output(&self, node: NodeId, port: usize) -> CookValue {
        self.prev_outputs
            .get(&node)
            .and_then(|outs| outs.get(port))
            .cloned()
            .unwrap_or_default()
    }
}
