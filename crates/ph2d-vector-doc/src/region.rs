//! [`Region`] + [`WindingRule`] + [`SegmentRef`].
//!
//! Per [ADR-0056 §2.3](../../../../docs/architecture/decisions/0056-vector-network-data-model.md):
//! `Region` is capped at **5 fields**; `WindingRule` is **2 FROZEN variants**.
//!
//! A region is a closed loop of segments with a winding rule and an
//! optional fill — Figma's "vector network" data model (2017): per-region
//! fills, shared edges between regions, auto-resolved crossings.

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::style::{FillRef, SegmentId};

/// Stable identifier for a [`Region`] within a [`crate::VectorNetwork`].
pub type RegionId = u32;

/// One reference to a segment inside a region loop.
///
/// The `bool` tracks traversal direction: `true` = forward (start → end),
/// `false` = reversed. Required because a single segment can participate
/// in multiple regions with different orientations (e.g. a shared edge
/// between two faces of a divided shape).
pub type SegmentRef = (SegmentId, bool);

/// A closed loop of segments forming a fillable face.
///
/// **Caps (ADR-0056 §2.3):** ≤ 5 fields. Currently 5 (exact match).
/// SmallVec inline cap for `segments`: **16** (matches per-region segment
/// budget for typical logos / icons).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Region {
    /// Stable identifier.
    pub id: RegionId,

    /// Ordered loop of segment refs (with traversal direction).
    pub segments: SmallVec<[SegmentRef; 16]>,

    /// Winding rule for hit-test / fill determination.
    pub winding: WindingRule,

    /// Optional fill style reference; `None` = stroke-only region.
    pub fill: Option<FillRef>,

    /// Z-order for paint ordering inside the network. Higher = on top.
    pub z: i32,
}

impl Region {
    /// Construct an empty region with the given id and winding rule.
    /// Fill defaults to `None`; z-order to `0`. Append segments via the
    /// `segments` field directly (typical authoring path is via
    /// [`crate::VectorOp::AddRegion`]).
    #[must_use]
    pub fn new(id: RegionId, winding: WindingRule) -> Self {
        Self {
            id,
            segments: SmallVec::new(),
            winding,
            fill: None,
            z: 0,
        }
    }
}

/// Region winding rule.
///
/// **FROZEN at 2 variants** (ADR-0056 §2.3). SVG canonical default is
/// [`WindingRule::NonZero`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WindingRule {
    /// Even-odd rule: a point is inside if a ray from it crosses an odd
    /// number of segments.
    EvenOdd,

    /// Non-zero rule: a point is inside if the winding number (counting
    /// clockwise crossings as +1, counter-clockwise as −1) is non-zero.
    #[default]
    NonZero,
}
