//! Style references + [`Segment`] (which carries one style ref) + minimal
//! [`StyleTable`] / [`StrokeStyle`] stubs.
//!
//! Per [ADR-0056 §2.3](../../../../docs/architecture/decisions/0056-vector-network-data-model.md):
//! `Segment` is capped at **6 fields**. Stroke / fill styles themselves
//! live in a per-document table and are referenced by `u32` index so the
//! per-segment / per-region payload stays compact.

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::BTreeMap;

use crate::cubic::VertexId;

/// Stable identifier for a [`Segment`] within a [`crate::VectorNetwork`].
pub type SegmentId = u32;

/// Index into [`StyleTable::strokes`].
pub type StyleRef = u32;

/// Index into [`StyleTable::fills`].
pub type FillRef = u32;

/// One edge in the vector network graph.
///
/// **Caps (ADR-0056 §2.3):** ≤ 6 fields (exact match below). Marked
/// `#[non_exhaustive]` to lock external exhaustive construction.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Segment {
    /// Stable identifier.
    pub id: SegmentId,

    /// Vertex this segment leaves.
    pub start: VertexId,

    /// Vertex this segment arrives at.
    pub end: VertexId,

    /// Cubic tangent at the start vertex (vector pointing along the segment).
    pub out_at_start: glam::Vec2,

    /// Cubic tangent at the end vertex (vector pointing back along the segment).
    pub in_at_end: glam::Vec2,

    /// Optional stroke style; `None` = inherit from parent / region.
    pub style_ref: Option<StyleRef>,
}

impl Segment {
    /// Construct a straight-line segment (zero tangents) between two vertices.
    #[must_use]
    pub const fn straight(id: SegmentId, start: VertexId, end: VertexId) -> Self {
        Self {
            id,
            start,
            end,
            out_at_start: glam::Vec2::ZERO,
            in_at_end: glam::Vec2::ZERO,
            style_ref: None,
        }
    }
}

/// Stroke style — minimal placeholder. Full stroke vocabulary
/// (variable width, dash pattern, caps, joins, brush ref) arrives in
/// W2-W5 per the Vector Module plan §§5-8.
///
/// **Cap (ADR-0056 §2.3):** ≤ 6 fields. Currently 4; expansion via
/// 0056-amendment-N.md when the Studio panel (W15) lands.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StrokeStyle {
    /// Constant width in network-local pixels (variable-width via per-segment
    /// width-profile arrives W5).
    pub width: f32,

    /// OKLCH color of the stroke (alpha included).
    pub color: ph2d_color::OklchColor,

    /// End-cap style.
    pub cap: StrokeCap,

    /// Corner-join style.
    pub join: StrokeJoin,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: ph2d_color::OklchColor::opaque(0.0, 0.0, 0.0),
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
        }
    }
}

/// SVG-compatible stroke end-cap modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrokeCap {
    /// Cut flat at the endpoint.
    Butt,
    /// Round semicircle past the endpoint.
    Round,
    /// Square extension past the endpoint by half the stroke width.
    Square,
}

/// SVG-compatible stroke corner-join modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrokeJoin {
    /// Sharp miter (clipped beyond miter-limit).
    Miter,
    /// Round arc filling the corner.
    Round,
    /// Bevel — flat chamfer.
    Bevel,
}

/// Document-level style storage. Strokes / fills referenced by `u32`
/// index from segments / regions — keeps per-element payloads small
/// and de-duplicates style data shared across many segments.
///
/// HR-5: `BTreeMap` (not `HashMap`) for deterministic iteration.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StyleTable {
    /// Stroke styles indexed by [`StyleRef`].
    pub strokes: BTreeMap<StyleRef, StrokeStyle>,

    /// Fill styles indexed by [`FillRef`]. W1 stores only solid colors;
    /// gradient / pattern / procedural fills arrive W2-W6.
    pub fills: BTreeMap<FillRef, FillSolid>,
}

impl StyleTable {
    /// Insert `stroke` and return the auto-allocated [`StyleRef`].
    ///
    /// **W1 ergonomic helper** (R4 audit Lens-G HIGH-G2) — assigns the
    /// next free id (existing max + 1, or 0 if empty). O(log N) via
    /// `BTreeMap::keys().next_back()`.
    pub fn insert_stroke(&mut self, stroke: StrokeStyle) -> StyleRef {
        let id = self.strokes.keys().next_back().map_or(0, |m| m + 1);
        self.strokes.insert(id, stroke);
        id
    }

    /// Insert `fill` and return the auto-allocated [`FillRef`].
    ///
    /// **W1 ergonomic helper** (R4 audit Lens-G HIGH-G2) — same
    /// allocation semantics as [`Self::insert_stroke`].
    pub fn insert_fill(&mut self, fill: FillSolid) -> FillRef {
        let id = self.fills.keys().next_back().map_or(0, |m| m + 1);
        self.fills.insert(id, fill);
        id
    }
}

/// Solid color fill — W1 minimal. Gradient / pattern / procedural arrive
/// later (W2+ / W6+).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FillSolid {
    /// OKLCH color of the fill (alpha included).
    pub color: ph2d_color::OklchColor,
}

impl Default for FillSolid {
    fn default() -> Self {
        Self {
            color: ph2d_color::OklchColor::opaque(0.5, 0.0, 0.0),
        }
    }
}

/// Per-network style-ref index. SmallVec inline 8 covers small documents
/// (most logos / icons need ≤ 8 distinct styles).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StyleRefMap {
    /// Direct style references the network uses.
    pub refs: SmallVec<[StyleRef; 8]>,
}
