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

    /// Solid fill styles indexed by [`FillRef`].
    pub fills: BTreeMap<FillRef, FillSolid>,

    /// Procedural fills (W6 shader graph / W7 diffusion-curve mesh gradient),
    /// keyed in the **same [`FillRef`] namespace** as [`Self::fills`]: a region's
    /// `fill` ref resolves here FIRST (procedural), else falls back to a solid
    /// (see [`Self::resolve_fill`]). The doc holds only an opaque registry id +
    /// kind + a solid fallback — the actual graph/curve-set lives render-side
    /// (`ph2d-vector-fill`), so this crate stays decoupled from it.
    ///
    /// Appended `#[serde(default)]` → additive over the v1 wire (empty map for
    /// assets without procedural fills), mirroring the [`StrokeStyle::width_profile`]
    /// / `dormant_fractures` precedent. ADR-0056-amendment-3. Bounded on decode by
    /// `AssetBounds::max_style_procedural`.
    #[serde(default)]
    pub procedural: BTreeMap<FillRef, ProceduralFill>,
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

    /// Insert a solid `fill` and return the auto-allocated [`FillRef`]. The id is
    /// the max over BOTH fill maps + 1, so solid and procedural fills share one
    /// [`FillRef`] namespace and never collide.
    pub fn insert_fill(&mut self, fill: FillSolid) -> FillRef {
        let id = self.next_fill_id();
        self.fills.insert(id, fill);
        id
    }

    /// Insert a [`ProceduralFill`] and return the auto-allocated [`FillRef`]
    /// (shared namespace with [`Self::insert_fill`]). ADR-0056-amendment-3.
    pub fn insert_procedural(&mut self, fill: ProceduralFill) -> FillRef {
        let id = self.next_fill_id();
        self.procedural.insert(id, fill);
        id
    }

    /// The next free [`FillRef`] across BOTH the solid and procedural maps
    /// (max existing key + 1, or 0 when both are empty). O(log N).
    fn next_fill_id(&self) -> FillRef {
        let solid = self.fills.keys().next_back().copied();
        let proc = self.procedural.keys().next_back().copied();
        match (solid, proc) {
            (Some(a), Some(b)) => a.max(b) + 1,
            (Some(a), None) | (None, Some(a)) => a + 1,
            (None, None) => 0,
        }
    }

    /// Resolve a [`FillRef`] to its fill. **Procedural takes precedence** over a
    /// solid in the shared namespace (a ref present in both — which `insert_*`
    /// never produces — prefers the procedural). `None` = a dangling ref (neither
    /// map has it; the region paints stroke-only).
    #[must_use]
    pub fn resolve_fill(&self, fill: FillRef) -> Option<ResolvedFill<'_>> {
        if let Some(p) = self.procedural.get(&fill) {
            return Some(ResolvedFill::Procedural(p));
        }
        self.fills.get(&fill).map(ResolvedFill::Solid)
    }
}

/// What a [`FillRef`] resolves to in a [`StyleTable`] ([`StyleTable::resolve_fill`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResolvedFill<'a> {
    /// A procedural fill (W6 shader graph / W7 diffusion) — checked first.
    Procedural(&'a ProceduralFill),
    /// A solid color fill.
    Solid(&'a FillSolid),
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

/// Which render-side procedural pipeline a [`ProceduralFill`]'s `id` indexes.
///
/// **Cap (ADR-0056-amendment-3):** ≤ 4 variants (gate
/// `architecture_vector_contract_surface`) — room for the resource-bound fills
/// (pattern/image) to grow without unbounded surface. Currently 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProceduralFillKind {
    /// W6 procedural shader graph (`ph2d_vector_fill::FillGraph`) — the `id`
    /// indexes the render-side fill-graph registry.
    ShaderGraph,
    /// W7 diffusion-curve mesh gradient (`DiffusionCurveSet`) — the `id` indexes
    /// the render-side diffusion registry.
    Diffusion,
}

/// A procedural fill a region's [`FillRef`] can resolve to (alongside the solid
/// [`FillSolid`]). The data model holds ONLY a `kind` + an opaque registry `id` +
/// a solid `fallback`, keeping `ph2d-vector-doc` decoupled from the render-side
/// graph/curve-set crate (`ph2d-vector-fill`) — the renderer (or a tier without
/// the procedural pipeline) resolves `id` against its registry, or paints
/// `fallback` when it cannot. W6/W7, ADR-0056-amendment-3.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProceduralFill {
    /// Which render-side pipeline resolves `id`.
    pub kind: ProceduralFillKind,

    /// Opaque id into the render-side registry for `kind`. Stable within a
    /// document; the registry (and the kind→registry mapping) lives render-side.
    pub id: u32,

    /// Solid OKLCH fallback — painted for preview, on a tier without the
    /// procedural pipeline, or when `id` is unresolved (graceful degradation per
    /// the tier policy, ADR-0053/0068).
    pub fallback: ph2d_color::OklchColor,
}

impl ProceduralFill {
    /// A shader-graph (W6) procedural fill with the given registry `id` and
    /// solid `fallback`.
    #[must_use]
    pub fn shader_graph(id: u32, fallback: ph2d_color::OklchColor) -> Self {
        Self {
            kind: ProceduralFillKind::ShaderGraph,
            id,
            fallback,
        }
    }

    /// A diffusion-curve mesh-gradient (W7) procedural fill with the given
    /// registry `id` and solid `fallback`.
    #[must_use]
    pub fn diffusion(id: u32, fallback: ph2d_color::OklchColor) -> Self {
        Self {
            kind: ProceduralFillKind::Diffusion,
            id,
            fallback,
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

    #[test]
    fn fill_refs_share_one_namespace_across_solid_and_procedural() {
        let mut t = StyleTable::default();
        let a = t.insert_fill(FillSolid::default());
        let b = t.insert_procedural(ProceduralFill::shader_graph(7, FillSolid::default().color));
        let c = t.insert_fill(FillSolid::default());
        // Ids never collide across the two maps (max-of-both + 1).
        assert_eq!((a, b, c), (0, 1, 2));
        assert!(
            t.fills.contains_key(&a) && t.procedural.contains_key(&b) && t.fills.contains_key(&c)
        );
    }

    #[test]
    fn resolve_fill_prefers_procedural_then_solid_then_none() {
        let mut t = StyleTable::default();
        let solid = t.insert_fill(FillSolid::default());
        let proc = t.insert_procedural(ProceduralFill::diffusion(3, FillSolid::default().color));
        assert!(matches!(
            t.resolve_fill(solid),
            Some(ResolvedFill::Solid(_))
        ));
        assert!(matches!(
            t.resolve_fill(proc),
            Some(ResolvedFill::Procedural(p)) if p.kind == ProceduralFillKind::Diffusion && p.id == 3
        ));
        assert!(t.resolve_fill(999).is_none());
    }

    #[test]
    fn style_table_with_procedural_round_trips_postcard() {
        let mut t = StyleTable::default();
        t.insert_fill(FillSolid::default());
        t.insert_procedural(ProceduralFill::shader_graph(
            42,
            ph2d_color::OklchColor::opaque(0.6, 0.1, 200.0),
        ));
        let bytes = postcard::to_allocvec(&t).expect("serialize");
        let back: StyleTable = postcard::from_bytes(&bytes).expect("deserialize");
        assert_eq!(t, back);
    }
}
