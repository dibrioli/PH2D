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

/// A 1-D variable-width profile for a stroke (Vector Module plan §8 T5.1).
///
/// Width along the stroke = `StrokeStyle.width × scale(t)`, where `scale`
/// interpolates `start` → `end` over the stroke parameter `t ∈ [0,1]` with a
/// midpoint `bulge` (calligraphic swell). `None` on a [`StrokeStyle`] = constant
/// width (the common case). Per-sample, pressure-/jitter-driven width is a
/// *render-time* concern (`ph2d_vector::draw_variable_width_stroke`), not part of
/// this persisted parametric profile.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WidthProfile {
    /// Width scale at the stroke start (× the base [`StrokeStyle::width`]).
    pub start: f32,
    /// Width scale at the stroke end.
    pub end: f32,
    /// Midpoint swell: `> 0` bulges (brush/calligraphic), `< 0` pinches, `0` linear.
    pub bulge: f32,
}

impl Default for WidthProfile {
    fn default() -> Self {
        // Identity: constant width along the stroke.
        Self {
            start: 1.0,
            end: 1.0,
            bulge: 0.0,
        }
    }
}

impl WidthProfile {
    /// Width scale at parameter `t ∈ [0,1]` along the stroke (clamped). Linear
    /// `start`→`end` taper plus a parabolic `bulge` peaking at the midpoint.
    #[must_use]
    pub fn scale_at(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let linear = self.start + (self.end - self.start) * t;
        linear + self.bulge * 4.0 * t * (1.0 - t)
    }
}

/// Stroke style — minimal placeholder. Full stroke vocabulary
/// (dash pattern, brush ref) still arrives later per the Vector Module plan.
///
/// **Cap (ADR-0056 §2.3):** ≤ 6 fields. Currently 5 (W5 added `width_profile`);
/// further expansion via 0056-amendment-N.md when the Studio panel (W15) lands.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StrokeStyle {
    /// Base width in network-local pixels (scaled per-`t` by `width_profile`).
    pub width: f32,

    /// OKLCH color of the stroke (alpha included).
    pub color: ph2d_color::OklchColor,

    /// End-cap style.
    pub cap: StrokeCap,

    /// Corner-join style.
    pub join: StrokeJoin,

    /// Variable-width profile (plan §8 T5.1); `None` = constant `width`. Appended
    /// (`Option`, default `None`) → backward-compatible with v1 assets, no schema
    /// bump (mirrors the `dormant_fractures` precedent).
    #[serde(default)]
    pub width_profile: Option<WidthProfile>,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: ph2d_color::OklchColor::opaque(0.0, 0.0, 0.0),
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
            width_profile: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_profile_tapers_and_bulges() {
        let taper = WidthProfile {
            start: 1.0,
            end: 3.0,
            bulge: 0.0,
        };
        assert!((taper.scale_at(0.0) - 1.0).abs() < 1e-6);
        assert!((taper.scale_at(0.5) - 2.0).abs() < 1e-6);
        assert!((taper.scale_at(1.0) - 3.0).abs() < 1e-6);
        // Out-of-range clamps to the endpoints.
        assert!((taper.scale_at(-1.0) - 1.0).abs() < 1e-6);
        assert!((taper.scale_at(2.0) - 3.0).abs() < 1e-6);
        // A pure bulge swells at the midpoint (4·0.5·0.5 = 1 × bulge).
        let bulge = WidthProfile {
            start: 1.0,
            end: 1.0,
            bulge: 0.5,
        };
        assert!((bulge.scale_at(0.5) - 1.5).abs() < 1e-6);
        assert!((bulge.scale_at(0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn stroke_style_default_is_constant_width() {
        assert!(StrokeStyle::default().width_profile.is_none());
    }
}
