//! Event-sourced [`EditLog`] of [`VectorOp`]s.
//!
//! Per [ADR-0056 §2.8](../../../../docs/architecture/decisions/0056-vector-network-data-model.md).
//! Cap: **VectorOp ≤ 16 variants** (current = 13 explicit + 3 reserved —
//! one slot earmarked for the future `BatchOp` reintroduction with a
//! depth-bounded custom `Deserialize` per W1.T1.6).
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

// NOTE — `BatchOp` variant removed in W1.T1.2 R1 audit remediation
// (CRIT-D1 + CRIT-D2). Original spec (ADR-0056 §2.8) defined
// `BatchOp { ops: SmallVec<[Box<VectorOp>; 8]> }` for atomic
// transactions, but Box recursion through `serde` is unbounded — a
// malicious .ph2d-vector with nested `BatchOp` overflows the stack
// during postcard `from_bytes` BEFORE `bounded_decode` runs. W1.T1.6
// (CRDT) reintroduces the variant with a custom `Deserialize` impl
// using a thread-local depth counter capped at ≤ 32 levels; until
// then `EditLog::ops` is flat — atomicity is the editor's
// responsibility (group N ops + apply together + undo as N steps).
// Slot in the 16-cap is reserved.

/// One mutation of a [`VectorNetwork`].
///
/// **Cap (ADR-0056 §2.3 + §2.8):** ≤ **16 variants** (current: 13
/// explicit + 3 reserved — one of the reserved slots holds the future
/// `BatchOp` reintroduction in W1.T1.6 with a depth-bounded custom
/// `Deserialize`; the others are open for amendment).
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

    /// CRDT integrity checkpoint — a blake3 hash of the network state
    /// at this point. Periodically inserted (every 30 s per
    /// ADR-0057 §2.3) so out-of-sync replicas detect divergence.
    Checkpoint {
        /// blake3-256 hash (32 bytes) of the network at this point.
        hash: [u8; 32],
    },
}

/// Failure modes for [`VectorOp::apply_to_network`] / [`EditLog::push_and_apply`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VectorOpApplyError {
    /// Referenced [`VertexId`] not present in the network.
    UnknownVertex(VertexId),
    /// Referenced [`SegmentId`] not present in the network.
    UnknownSegment(SegmentId),
    /// Referenced [`RegionId`] not present in the network.
    UnknownRegion(RegionId),
    /// Operation requires the full [`crate::Ph2dVectorAsset`] context
    /// (`ApplyBoolean` needs the boolean engine W3+; `SetStrokeStyle`
    /// needs the `StyleTable`; `CrdtMerge` updates `Ph2dVectorAsset.crdt_state`).
    /// Use the asset-level apply variant on a higher-level wrapper.
    NeedsAssetContext,
}

impl std::fmt::Display for VectorOpApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownVertex(id) => write!(f, "unknown vertex id {id}"),
            Self::UnknownSegment(id) => write!(f, "unknown segment id {id}"),
            Self::UnknownRegion(id) => write!(f, "unknown region id {id}"),
            Self::NeedsAssetContext => write!(
                f,
                "VectorOp variant requires Ph2dVectorAsset context (StyleTable / boolean engine / crdt_state)"
            ),
        }
    }
}

impl std::error::Error for VectorOpApplyError {}

impl VectorOp {
    /// Apply this op to a [`VectorNetwork`] (network-only mutation).
    ///
    /// **W1 ergonomic helper (R4 audit Lens-G HIGH-G3)**: tools that
    /// mutate a network during interactive authoring can call this
    /// instead of duplicating the match-on-variant arms. The op is
    /// applied **but not pushed to any edit log** — use
    /// [`EditLog::push_and_apply`] to do both atomically.
    ///
    /// Variants that need external context return
    /// [`VectorOpApplyError::NeedsAssetContext`] without mutating:
    /// - [`Self::ApplyBoolean`] — boolean engine ships W3.
    /// - [`Self::SetStrokeStyle`] — needs [`crate::StyleTable`].
    /// - [`Self::CrdtMerge`] — updates [`crate::Ph2dVectorAsset::crdt_state`].
    ///
    /// Other variants either mutate the network directly or are no-ops
    /// at the network level ([`Self::Checkpoint`] is a sync marker).
    pub fn apply_to_network(&self, net: &mut VectorNetwork) -> Result<(), VectorOpApplyError> {
        match self {
            VectorOp::AddVertex { id, pos, kind } => {
                net.vertices
                    .push(crate::cubic::Vertex::new(*id, *pos, *kind));
                Ok(())
            }
            VectorOp::MoveVertex { id, new_pos } => {
                let Some(v) = net.vertices.iter_mut().find(|v| v.id == *id) else {
                    return Err(VectorOpApplyError::UnknownVertex(*id));
                };
                v.pos = *new_pos;
                Ok(())
            }
            VectorOp::RemoveVertex { id } => {
                let before = net.vertices.len();
                net.vertices.retain(|v| v.id != *id);
                if net.vertices.len() == before {
                    return Err(VectorOpApplyError::UnknownVertex(*id));
                }
                Ok(())
            }
            VectorOp::AddSegment {
                id,
                start,
                end,
                tangents,
            } => {
                net.segments.push(crate::style::Segment {
                    id: *id,
                    start: *start,
                    end: *end,
                    out_at_start: tangents.out_at_start,
                    in_at_end: tangents.in_at_end,
                    style_ref: None,
                });
                Ok(())
            }
            VectorOp::MoveTangent {
                seg,
                which,
                new_pos,
            } => {
                let Some(s) = net.segments.iter_mut().find(|s| s.id == *seg) else {
                    return Err(VectorOpApplyError::UnknownSegment(*seg));
                };
                match which {
                    TangentSide::OutAtStart => s.out_at_start = *new_pos,
                    TangentSide::InAtEnd => s.in_at_end = *new_pos,
                }
                Ok(())
            }
            VectorOp::RemoveSegment { id } => {
                let before = net.segments.len();
                net.segments.retain(|s| s.id != *id);
                if net.segments.len() == before {
                    return Err(VectorOpApplyError::UnknownSegment(*id));
                }
                Ok(())
            }
            VectorOp::AddRegion {
                id,
                segments,
                winding,
            } => {
                let mut r = crate::region::Region::new(*id, *winding);
                r.segments = segments.clone();
                net.regions.push(r);
                Ok(())
            }
            VectorOp::SetRegionFill { id, fill } => {
                let Some(r) = net.regions.iter_mut().find(|r| r.id == *id) else {
                    return Err(VectorOpApplyError::UnknownRegion(*id));
                };
                r.fill = *fill;
                Ok(())
            }
            VectorOp::SetAuthoringHint { mode } => {
                net.authoring_hint = *mode;
                Ok(())
            }
            VectorOp::Checkpoint { .. } => Ok(()), // sync marker; no-op on the network
            VectorOp::ApplyBoolean { .. }
            | VectorOp::SetStrokeStyle { .. }
            | VectorOp::CrdtMerge { .. } => Err(VectorOpApplyError::NeedsAssetContext),
        }
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
/// [`NetworkSnapshot`]s accelerate seek to arbitrary positions
/// without replaying from genesis — but **insertion is the editor's
/// responsibility**, not the log's. [`EditLog::push`] only appends to
/// `ops`. Callers that want periodic snapshots construct a
/// [`NetworkSnapshot`] and push to `snapshots` directly (recommended
/// every 100 ops or on explicit "save" boundary).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditLog {
    /// Append-only operation history.
    pub ops: Vec<VectorOp>,

    /// Pinned snapshots (op-index → network state). Inserted **by the
    /// editor** at policy-defined points (typically every 100 ops);
    /// seeking to op `N` replays at most `100` ops from the nearest
    /// snapshot.
    pub snapshots: Vec<(usize, NetworkSnapshot)>,
}

impl EditLog {
    /// Construct an empty log.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an op. Snapshotting policy is the editor's responsibility
    /// (this log is a passive container; see struct-level docs).
    pub fn push(&mut self, op: VectorOp) {
        self.ops.push(op);
    }

    /// Remove + return the most-recently-pushed op.
    ///
    /// **W1 pre-CRDT undo primitive (R4 audit Lens-G MED-G4)**. The
    /// caller is responsible for *reverting* the op's effect on its
    /// [`VectorNetwork`] (this is just the log mutation half). T1.6
    /// CRDT lands a proper undo stack with op-level inverse semantics;
    /// until then, single-user tools may call `pop()` + recompute
    /// network from genesis (or from nearest snapshot).
    pub fn pop(&mut self) -> Option<VectorOp> {
        self.ops.pop()
    }

    /// Apply `op` to `net` and, on success, push it to the log
    /// (transactional: if `apply_to_network` errors, the log is **not**
    /// mutated — preventing the log/network drift documented in
    /// Lens-G HIGH-G3).
    pub fn push_and_apply(
        &mut self,
        op: VectorOp,
        net: &mut VectorNetwork,
    ) -> Result<(), VectorOpApplyError> {
        op.apply_to_network(net)?;
        self.push(op);
        Ok(())
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
