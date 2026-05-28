//! [`Vertex`] + cubic Bézier tangent types.
//!
//! Per [ADR-0056 §2.3](../../../../docs/architecture/decisions/0056-vector-network-data-model.md):
//! `Vertex` is capped at **5 fields**; `VertexKind` is **4 FROZEN variants**.
//!
//! Tangents are stored **per-edge** on [`crate::Segment`], not per-vertex —
//! this is the canonical vector-network invariant (Figma 2017): a vertex
//! may participate in N segments, each with its own tangent at that vertex.

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Stable identifier for a [`Vertex`] within a [`crate::VectorNetwork`].
///
/// `u32` chosen over `usize` so `.ph2d-vector` postcard files are
/// bit-identical across 32-bit and 64-bit hosts (HR-5 determinism).
pub type VertexId = u32;

/// A point in the vector network graph.
///
/// **Caps (ADR-0056 §2.3):** ≤ 5 public fields. Currently uses 3 with room
/// for 2 reserved additions via amendment. Marked `#[non_exhaustive]` so
/// downstream crates use the constructor / pattern updates and survive
/// future field additions without recompilation breakage.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Vertex {
    /// Stable identifier.
    pub id: VertexId,

    /// Position in network-local coordinates (pixels).
    pub pos: Vec2,

    /// Tangent-continuity intent declared by the author.
    pub kind: VertexKind,
}

impl Vertex {
    /// Construct a vertex with explicit `kind`.
    #[must_use]
    pub const fn new(id: VertexId, pos: Vec2, kind: VertexKind) -> Self {
        Self { id, pos, kind }
    }

    /// Construct a vertex with [`VertexKind::Auto`] — the renderer picks
    /// the continuity mode from incident tangent geometry.
    #[must_use]
    pub const fn auto(id: VertexId, pos: Vec2) -> Self {
        Self::new(id, pos, VertexKind::Auto)
    }
}

/// Tangent-continuity intent at a vertex.
///
/// **FROZEN at 4 variants** (ADR-0056 §2.3) — covers parity with
/// Illustrator / Affinity / Figma editing semantics. NOT marked
/// `#[non_exhaustive]`: FROZEN explicitly promises exhaustive-match
/// stability to downstream code (R1 audit Lens-B MED-1 + Lens-D MED-D9
/// consistency with [`crate::WindingRule`] and
/// [`crate::RepresentationMode`], both FROZEN and non-`non_exhaustive`).
/// Expansion requires a new `0056-amendment-N.md` ADR.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VertexKind {
    /// Tangents on either side of the vertex are mirrored (smooth, equal magnitude).
    Mirror,
    /// Tangents are colinear but may have different magnitudes (smooth, unequal).
    Aligned,
    /// Tangents are independent (corner / cusp).
    Free,
    /// Renderer chooses based on incident edges. Default for newly drawn
    /// vertices via `vector-pen` until the user manipulates a tangent.
    #[default]
    Auto,
}

/// Cubic Bézier tangent pair carried by a [`crate::Segment`].
///
/// `out_at_start` is the tangent vector exiting the segment's `start`
/// vertex; `in_at_end` is the tangent vector entering the `end` vertex.
/// Both are stored in the same coordinate system as the vertex positions
/// (network-local pixels).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TangentsCubic {
    /// Tangent at the start vertex, pointing along the segment.
    pub out_at_start: Vec2,
    /// Tangent at the end vertex, pointing back along the segment.
    pub in_at_end: Vec2,
}

impl TangentsCubic {
    /// Zero-length tangents — degenerates the segment to a straight line
    /// between its endpoints. Used as the default when a `Pen` click
    /// deposits a vertex without dragging.
    pub const ZERO: Self = Self {
        out_at_start: Vec2::ZERO,
        in_at_end: Vec2::ZERO,
    };
}

/// Which tangent side of a [`crate::Segment`] a [`crate::VectorOp::MoveTangent`]
/// targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TangentSide {
    /// `out_at_start` — the tangent leaving the segment's start vertex.
    OutAtStart,
    /// `in_at_end` — the tangent entering the segment's end vertex.
    InAtEnd,
}
