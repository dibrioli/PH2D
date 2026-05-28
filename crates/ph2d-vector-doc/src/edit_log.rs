//! Event-sourced [`EditLog`] of [`VectorOp`]s.
//!
//! Per [ADR-0056 §2.8](../../../../docs/architecture/decisions/0056-vector-network-data-model.md).
//! Cap: **VectorOp ≤ 16 variants** (current = 14 explicit + 2 reserved
//! for ADR-0056-amendment-N expansion).
//!
//! CRDT semantics (LWW + RGA + custom merge for tangents) detailed in
//! [ADR-0057](../../../../docs/architecture/decisions/0057-vector-edit-dispatch-crdt.md);
//! the merge / replay machinery lives in [`crate::crdt`] (W1.T1.6).

use glam::Vec2;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::cubic::{TangentSide, TangentsCubic, VertexId, VertexKind};
use crate::network::{RepresentationMode, VectorNetwork};
use crate::region::{RegionId, SegmentRef, WindingRule};
use crate::style::{FillRef, SegmentId, StrokeStyle};

/// One mutation of a [`VectorNetwork`].
///
/// **Cap (ADR-0056 §2.3 + §2.8):** ≤ **16 variants** (current: 14
/// explicit + 2 reserved). Adding the 15th / 16th variant ships as
/// `0056-amendment-N.md`; the 17th requires a new ADR entirely.
///
/// Marked `#[non_exhaustive]` so downstream pattern matches force a
/// fallback arm and survive future variant additions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VectorOp {
    /// Add a vertex at `pos` with the given tangent-continuity `kind`.
    AddVertex {
        /// Stable id assigned to the new vertex.
        id: VertexId,
        /// Position in network-local coordinates.
        pos: Vec2,
        /// Tangent-continuity intent.
        kind: VertexKind,
    },

    /// Move an existing vertex to `new_pos`. Tangent geometry on
    /// incident segments is preserved (relative offsets unchanged).
    MoveVertex {
        /// Vertex to move.
        id: VertexId,
        /// New position.
        new_pos: Vec2,
    },

    /// Remove a vertex (and orphans every incident segment).
    RemoveVertex {
        /// Vertex to remove.
        id: VertexId,
    },

    /// Add a segment connecting two existing vertices.
    AddSegment {
        /// Stable id assigned to the new segment.
        id: SegmentId,
        /// Start vertex id.
        start: VertexId,
        /// End vertex id.
        end: VertexId,
        /// Cubic tangent pair.
        tangents: TangentsCubic,
    },

    /// Move one tangent of an existing segment.
    MoveTangent {
        /// Segment id.
        seg: SegmentId,
        /// Which side of the segment the move targets.
        which: TangentSide,
        /// New tangent vector.
        new_pos: Vec2,
    },

    /// Remove a segment (and orphans every region that references it).
    RemoveSegment {
        /// Segment to remove.
        id: SegmentId,
    },

    /// Add a region (closed loop of segments) with a winding rule.
    AddRegion {
        /// Stable id assigned to the new region.
        id: RegionId,
        /// Ordered loop of segment refs (with traversal direction).
        segments: SmallVec<[SegmentRef; 16]>,
        /// Winding rule.
        winding: WindingRule,
    },

    /// Set or clear a region's fill.
    SetRegionFill {
        /// Region id.
        id: RegionId,
        /// New fill ref (`None` clears the fill).
        fill: Option<FillRef>,
    },

    /// Apply a boolean operation across regions, producing a new region
    /// with `result_id`. The pipeline is draft+reconcile per ADR-0059
    /// (SDF GPU draft → Linesweeper exact reconcile).
    ApplyBoolean {
        /// Operation.
        op: BooleanOp,
        /// Operand region ids (typically 2; up to 4 for chain ops).
        regions: SmallVec<[RegionId; 4]>,
        /// Result region id.
        result_id: RegionId,
    },

    /// CRDT merge anchor — records that this site has incorporated
    /// peer `peer_id` up through sequence `peer_seq`.
    CrdtMerge {
        /// Peer site id.
        peer_id: u64,
        /// Peer's sequence number at the merge point.
        peer_seq: u64,
    },

    /// Set a segment's stroke style.
    SetStrokeStyle {
        /// Segment id.
        seg: SegmentId,
        /// New stroke style.
        style: StrokeStyle,
    },

    /// Set the authoring representation hint for the whole network.
    SetAuthoringHint {
        /// New representation mode.
        mode: RepresentationMode,
    },

    /// Atomic batch of operations applied as one transaction (undo /
    /// redo treats the batch as a single step).
    ///
    /// SmallVec inline cap: 8 ops per batch (covers typical Pen-tool
    /// click-and-extrude that adds vertex + segment + region in one
    /// gesture).
    BatchOp {
        /// Ordered sub-operations.
        ops: SmallVec<[BatchEntry; 8]>,
    },

    /// CRDT integrity checkpoint — a blake3 hash of the network state
    /// at this point. Periodically inserted (every 30 s per
    /// ADR-0057 §2.3) so out-of-sync replicas detect divergence.
    Checkpoint {
        /// blake3-256 hash (32 bytes) of the network at this point.
        hash: [u8; 32],
    },
}

/// One sub-op inside a [`VectorOp::BatchOp`].
///
/// Boxed to break the otherwise-infinite recursive size of `VectorOp`
/// (which would contain `SmallVec<[VectorOp; 8]>` containing more
/// `VectorOp`s). The `Box` adds an indirection per sub-op — acceptable
/// since `BatchOp` is rare (typically 1-5 per second during pen drag).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchEntry(pub Box<VectorOp>);

impl BatchEntry {
    /// Wrap a [`VectorOp`] into a [`BatchEntry`].
    #[must_use]
    pub fn new(op: VectorOp) -> Self {
        Self(Box::new(op))
    }
}

/// Boolean operation variants.
///
/// **Cap (ADR-0058):** 9 variants — covers full SVG / Illustrator /
/// Affinity Pathfinder vocabulary. FROZEN until the ADR-0058 graph
/// node `vector-boolean` lands in W3 and exercises the surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BooleanOp {
    /// A ∪ B.
    Union,
    /// A \ B.
    Subtract,
    /// A ∩ B.
    Intersect,
    /// (A ∪ B) \ (A ∩ B) — symmetric difference.
    Exclude,
    /// Cut A by B into multiple regions.
    Divide,
    /// Keep parts of A outside B and parts of B outside A.
    Trim,
    /// Merge co-incident regions (no operand bias).
    Merge,
    /// Crop A by B (clip to bounding intersection).
    Crop,
    /// Outline-only result (region → stroke path).
    Outline,
}

/// Event-sourced log of [`VectorOp`]s applied to a [`VectorNetwork`].
///
/// Replay of `ops[..N]` reconstructs the network at step `N`.
/// Periodic [`NetworkSnapshot`]s (every 100 ops) accelerate seek to
/// arbitrary positions without replaying from genesis.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditLog {
    /// Append-only operation history.
    pub ops: Vec<VectorOp>,

    /// Pinned snapshots (op-index → network state). Inserted every 100
    /// ops so a 10 000-op session has 100 snapshots; seeking to op
    /// 5 000 replays at most 100 ops from the nearest snapshot.
    pub snapshots: Vec<(usize, NetworkSnapshot)>,
}

impl EditLog {
    /// Construct an empty log.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an op without snapshotting. Snapshotting policy is the
    /// editor's responsibility (the log is a passive container).
    pub fn push(&mut self, op: VectorOp) {
        self.ops.push(op);
    }
}

/// Pinned snapshot of a [`VectorNetwork`] at a specific point in the
/// edit log.
///
/// Stored as a full clone of the network (postcard-roundtrip-able).
/// Memory cost is amortized — at 1 snapshot per 100 ops + ~1 KB per
/// snapshot, a 10k-op session spends ~100 KB on snapshots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkSnapshot {
    /// Cloned network state at the snapshot point.
    pub network: VectorNetwork,
}

impl From<VectorNetwork> for NetworkSnapshot {
    fn from(network: VectorNetwork) -> Self {
        Self { network }
    }
}
