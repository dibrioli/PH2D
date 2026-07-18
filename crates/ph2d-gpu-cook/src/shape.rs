//! **What the host knows about the last cook** — the sequencer's introspection,
//! and the only thing the graph panel can still read off a GPU-resident frame.
//!
//! A GPU cook does not feed the CPU memo, so the panel's usual source for "how
//! much is moving through this wire?" is empty and every wire flattens to the
//! same thread exactly when the counts got interesting. But neither fact here is
//! a *result*: the element count is what the host SIZED the dispatch with, and
//! the column set is what the sequencer decided to emit. Publishing them is
//! bookkeeping, not a readback — and the readback is measured-**negative** (see
//! `readback_tap_cost_probe`: 268 ms to pull 4,19 M back, worse than the 227 ms
//! the CPU takes to compute the whole thing).
//!
//! **Names, never buffers.** Holding the `GpuStream`s would pin them by refcount
//! and defeat [`crate::BufferPool::reclaim`], so this keeps a count and a list of
//! column names per node and nothing else.

use crate::GpuStream;
use ph2d_nodegraph::graph::NodeId;
use std::collections::BTreeMap;

/// The shape of every staged node's output on the last cook.
#[derive(Default)]
pub struct CookShape {
    counts: BTreeMap<NodeId, u32>,
    cols: BTreeMap<NodeId, Vec<String>>,
}

impl CookShape {
    /// Replace the record with this cook's streams. Called once per cook, after
    /// the stage walk — so a cook that failed early leaves the PREVIOUS shape
    /// rather than a half-filled one.
    pub(crate) fn record(&mut self, streams: &BTreeMap<NodeId, GpuStream>) {
        self.counts = streams.iter().map(|(n, s)| (*n, s.count)).collect();
        self.cols = streams
            .iter()
            .map(|(n, s)| (*n, s.cols.keys().cloned().collect()))
            .collect();
    }

    /// How many elements `node` carried — `None` if this plan did not stage it
    /// (a CPU boundary, or a node no sink reaches).
    ///
    /// `None` and not `0`: a zero-width wire is a claim, and "I did not run this
    /// node" is not the same statement as "it carried nothing".
    pub fn count(&self, node: NodeId) -> Option<u32> {
        self.counts.get(&node).copied()
    }

    /// The column names `node`'s output carried, sorted (`BTreeMap` key order).
    pub fn columns(&self, node: NodeId) -> Option<&[String]> {
        self.cols.get(&node).map(Vec::as_slice)
    }
}
