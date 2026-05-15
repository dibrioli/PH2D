// Derived from OpenCV modules/imgproc/src/grabcut.cpp and
// modules/imgproc/include/opencv2/imgproc/detail/gcgraph.hpp
// (Apache-2.0). © OpenCV contributors. Modifications: ported to
// Rust, struct-of-arrays Node/Edge layout, f32 capacities,
// BgRemovalScratch-backed working memory. See
// THIRD_PARTY_LICENSES.md at the repo root for full text.

//! Boykov–Kolmogorov augmenting-paths max-flow / min-cut.
//!
//! Algorithm: maintains two trees (S rooted at the source, T rooted
//! at the sink) and runs three phases per iteration:
//!
//! - **Growth**: each active tree expands toward the other along
//!   edges with positive residual capacity. Phase stops when a
//!   tree reaches a node from the opposite tree → an augmenting
//!   path is found.
//! - **Augmentation**: saturate the path; orphaned nodes (the ones
//!   whose parent edge becomes saturated) go on the orphan list.
//! - **Adoption**: each orphan attempts to find a new valid parent
//!   in its own tree. Failures detach the orphan and add its
//!   downstream children to the orphan list.
//!
//! The implementation is single-thread (BK is inherently sequential
//! on grid graphs).
//!
//! This file is a STUB — the algorithm body lands in the M2 BK
//! port pass. Signatures are fixed so the surrounding orchestrator
//! and tests can build against them.

/// Stub maxflow solver. Holds the (n-link, t-link) graph and the
/// per-node BK working state.
#[derive(Clone, Debug, Default)]
pub struct BkGraph {
    pub width: u32,
    pub height: u32,
}

impl BkGraph {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Configure the per-pixel source/sink capacities and the
    /// 4-direction n-link weights produced by `super::graph`.
    /// `source_caps[i]` and `sink_caps[i]` give the t-link weight
    /// from pixel `i` to source / sink respectively;
    /// `n_link_edges` packs `[right, down_right, down, down_left]`
    /// per pixel.
    pub fn load(
        &mut self,
        _source_caps: &[f32],
        _sink_caps: &[f32],
        _n_link_edges: &[f32],
    ) {
        // STUB.
    }

    /// Run max-flow to saturation. Returns the maximum flow value.
    pub fn run_max_flow(&mut self) -> f32 {
        // STUB.
        0.0
    }

    /// After [`Self::run_max_flow`], return `true` iff pixel `i`
    /// is in the source-side cut (foreground).
    pub fn is_source_side(&self, _i: usize) -> bool {
        // STUB returns "all foreground" so the orchestrator's
        // surrounding plumbing can run unit tests.
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_graph_records_dimensions() {
        let g = BkGraph::new(64, 32);
        assert_eq!(g.width, 64);
        assert_eq!(g.height, 32);
    }

    #[test]
    fn stub_max_flow_returns_zero() {
        let mut g = BkGraph::new(4, 4);
        let zeros = vec![0.0f32; 16];
        let n_links = vec![0.0f32; 16 * 4];
        g.load(&zeros, &zeros, &n_links);
        assert_eq!(g.run_max_flow(), 0.0);
    }

    #[test]
    fn stub_is_source_side_returns_true() {
        let g = BkGraph::new(4, 4);
        assert!(g.is_source_side(0));
    }
}
