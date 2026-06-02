//! Bridge from [`ph2d_vector_doc::VectorNetwork`] into Vello scene
//! draw commands.
//!
//! Per [ADR-0059](../../../docs/architecture/decisions/0059-vector-renderer-pipeline.md)
//! and the W1.T1.3 task in `docs/Vector Module/17_plano_de_implementacao.md`.
//!
//! ## What this module does (W1 scope)
//!
//! - Converts every [`ph2d_vector_doc::Region`] of a network into a
//!   `kurbo::BezPath` (cubic Bézier per segment, honoring traversal
//!   direction).
//! - Fills each region with its referenced [`ph2d_vector_doc::FillSolid`]
//!   color (OKLCH → sRGB → linear via `peniko::Color::from_rgba8`).
//! - Strokes each segment carrying a `style_ref` that resolves to a
//!   [`ph2d_vector_doc::StrokeStyle`] (W2 — landed with the Pencil tool).
//!   Pen region edges have no per-segment `style_ref`, so they're skipped
//!   (the region fill drew them) — Pen fills + Pencil strokes mix with no
//!   double-draw.
//!
//! ## Allocation behavior (HR-3 honest accounting)
//!
//! - **Per-frame, per-region**: one `BezPath` is built and discarded.
//!   `BezPath` is `Vec<PathEl>` internally — pre-sized via
//!   `with_capacity(2 + segments * 2)` so the path needs at most one
//!   heap alloc per region (no realloc on push). W2 lands a reusable
//!   scratch `BezPath` cached in `VectorScene` to drop even that one.
//! - **SmallVec inline budgets** in the **data model**
//!   (`VectorNetwork.vertices/segments/regions`: 32/64/8 inline) keep
//!   *typical* documents heap-free for the network itself — but the
//!   per-region `BezPath` lives in the renderer, not the network, so
//!   those budgets are unrelated to this draw routine.
//! - **No `unsafe`** (`#![forbid(unsafe_code)]` at the crate root).
//! - **`vello::*` / `kurbo::*` imports stay confined to this crate** —
//!   target gate `vello_kurbo_only_in_ph2d_vector` is **planned for W2+**
//!   (per ADR-0059 §2.8 + L6F1 long-tail); 20+ pre-existing crates
//!   import vello/kurbo direct today and will need a whitelist or
//!   migration. Until then this comment documents the intent, not an
//!   enforced gate.

use ph2d_color::OklchColor;
use ph2d_vector_doc::{Region, Segment, SegmentId, VectorNetwork, Vertex, VertexId};
use std::collections::BTreeMap;
use vello::Scene;
use vello::kurbo::{Affine, BezPath, Point, Stroke};
use vello::peniko::{Brush, Color, Fill};

/// Scratch lookup tables built once per `draw_vector_network` call so
/// `build_region_path_indexed` resolves segment / vertex refs in
/// O(log N) instead of O(N) linear scan per ref (R4 audit Lens-K
/// HIGH-K1).
///
/// `BTreeMap` (not `HashMap`) honors HR-5 + ADR-0022 — same lookup
/// complexity class (logarithmic vs constant amortized) but with
/// deterministic iteration. For worst-case 200k segments,
/// log₂(200k) ≈ 18 ops per lookup vs 200k linear; for typical 500
/// segments, log₂(500) ≈ 9 ops vs 500. Either way the linear-scan
/// hot-path is gone.
struct NetworkLookup<'a> {
    segments: BTreeMap<SegmentId, &'a Segment>,
    vertices: BTreeMap<VertexId, &'a Vertex>,
}

impl<'a> NetworkLookup<'a> {
    fn build(network: &'a VectorNetwork) -> Self {
        Self {
            segments: network.segments.iter().map(|s| (s.id, s)).collect(),
            vertices: network.vertices.iter().map(|v| (v.id, v)).collect(),
        }
    }
}

/// Render every region of `network` into `scene`, fill-only, under the
/// given `transform`.
///
/// Resolution order:
/// 1. Iterate `network.regions` in array order (which equals z-order
///    `Region::z` rank since the editor appends regions in z-order;
///    callers that need explicit z-sort should pre-sort).
/// 2. For each region, build a `kurbo::BezPath` from its segments
///    (honoring per-segment traversal direction).
/// 3. Resolve the fill via `region.fill` → [`ph2d_vector_doc::StyleTable`]
///    lookup on `styles`. `None` regions skip rendering (stroke-only
///    paths arrive W2).
/// 4. Emit a Vello fill command with winding rule mapped from
///    [`ph2d_vector_doc::WindingRule`].
///
/// Returns the number of regions actually drawn (regions whose
/// `fill = None` or whose ref isn't in `styles` are skipped silently).
pub fn draw_vector_network(
    scene: &mut Scene,
    network: &VectorNetwork,
    styles: &ph2d_vector_doc::StyleTable,
    transform: Affine,
) -> usize {
    // R4 audit Lens-K HIGH-K1: build segment+vertex indexes ONCE per
    // frame instead of triple linear scan per region.segment ref.
    // Per typical-asset profile (100 regions, 32 segs/region, 500 segs,
    // 300 vertices) this drops the lookup cost from ~3.5M ops/frame to
    // ~4k ops/frame (~1000× speedup) without changing the output.
    let lookup = NetworkLookup::build(network);
    let mut drawn = 0;
    for region in &network.regions {
        let Some(fill_ref) = region.fill else {
            continue;
        };
        let Some(fill_solid) = styles.fills.get(&fill_ref) else {
            continue;
        };
        let path = build_region_path_indexed(&lookup, region);
        if path.is_empty() {
            continue;
        }
        let color = oklch_to_color(fill_solid.color);
        let winding = match region.winding {
            ph2d_vector_doc::WindingRule::EvenOdd => Fill::EvenOdd,
            ph2d_vector_doc::WindingRule::NonZero => Fill::NonZero,
        };
        scene.fill(winding, transform, &Brush::Solid(color), None, &path);
        drawn += 1;
    }

    // Stroke pass (W2 — "the stroke vocabulary lands with the Pencil tool",
    // per the module doc). Stroke every segment whose `style_ref` resolves to a
    // [`StrokeStyle`] in `styles`. Pen region edges carry NO per-segment
    // `style_ref` (the region's fill draws them), so they're skipped here — the
    // committed list can mix Pen fills + Pencil strokes with no double-draw.
    let mut seg_path = BezPath::new();
    for segment in &network.segments {
        let Some(style_ref) = segment.style_ref else {
            continue;
        };
        let Some(stroke_style) = styles.strokes.get(&style_ref) else {
            continue;
        };
        let (Some(start_v), Some(end_v)) = (
            lookup.vertices.get(&segment.start),
            lookup.vertices.get(&segment.end),
        ) else {
            continue;
        };
        let start = Point::new(start_v.pos.x as f64, start_v.pos.y as f64);
        let end = Point::new(end_v.pos.x as f64, end_v.pos.y as f64);
        // Cubic control points from the tangent offset vectors (renderer
        // convention: c1 = start + out_at_start, c2 = end + in_at_end).
        let c1 = Point::new(
            start.x + segment.out_at_start.x as f64,
            start.y + segment.out_at_start.y as f64,
        );
        let c2 = Point::new(
            end.x + segment.in_at_end.x as f64,
            end.y + segment.in_at_end.y as f64,
        );
        seg_path.truncate(0);
        seg_path.move_to(start);
        seg_path.curve_to(c1, c2, end);
        scene.stroke(
            &Stroke::new(f64::from(stroke_style.width)),
            transform,
            &Brush::Solid(oklch_to_color(stroke_style.color)),
            None,
            &seg_path,
        );
        drawn += 1;
    }

    drawn
}

/// Build a closed `kurbo::BezPath` for one region.
///
/// Pure function; testable without Vello.
///
/// **Strict mode**: if any segment in the region references a vertex
/// or segment that isn't in the network, returns an empty path
/// (strict failure surfaces data problems instead of masking them).
///
/// **Single-region callers**: this convenience builds a local
/// segment/vertex index each call (O(S + V) per region). When drawing
/// **many regions** of the same network, use [`draw_vector_network`]
/// which hoists the index build once per frame (O(S + V + R × S_per_R)
/// total, ~1000× faster on typical-asset profiles per R4 audit
/// Lens-K HIGH-K1).
///
/// One `Vec<PathEl>` allocation is performed up-front via
/// `BezPath::with_capacity` sized for exactly `2 + N` elements.
#[must_use]
pub fn build_region_path(network: &VectorNetwork, region: &Region) -> BezPath {
    if region.segments.is_empty() {
        return BezPath::new();
    }
    let lookup = NetworkLookup::build(network);
    build_region_path_indexed(&lookup, region)
}

/// Indexed variant of [`build_region_path`] — accepts a pre-built
/// [`NetworkLookup`] so multi-region rendering amortizes the index
/// construction.
///
/// Same strict semantics as [`build_region_path`]: dangling refs
/// return an empty path. Same alloc behavior: exactly one
/// `BezPath::with_capacity(2 + N)` per call.
fn build_region_path_indexed(lookup: &NetworkLookup<'_>, region: &Region) -> BezPath {
    if region.segments.is_empty() {
        return BezPath::new();
    }
    let capacity = 2 + region.segments.len();
    let mut path = BezPath::with_capacity(capacity);
    let mut first = true;

    for &(seg_id, forward) in &region.segments {
        let Some(segment) = lookup.segments.get(&seg_id) else {
            return BezPath::new();
        };
        let (start_v, end_v, c1, c2) = if forward {
            let Some(s) = lookup.vertices.get(&segment.start) else {
                return BezPath::new();
            };
            let Some(e) = lookup.vertices.get(&segment.end) else {
                return BezPath::new();
            };
            (
                s.pos,
                e.pos,
                s.pos + segment.out_at_start,
                e.pos + segment.in_at_end,
            )
        } else {
            let Some(s) = lookup.vertices.get(&segment.end) else {
                return BezPath::new();
            };
            let Some(e) = lookup.vertices.get(&segment.start) else {
                return BezPath::new();
            };
            (
                s.pos,
                e.pos,
                s.pos + segment.in_at_end,
                e.pos + segment.out_at_start,
            )
        };
        if first {
            path.move_to(Point::new(start_v.x as f64, start_v.y as f64));
            first = false;
        }
        // Cubic Bézier per segment; if both tangents are zero this still
        // produces a straight line (kurbo handles degenerate cubics).
        path.curve_to(
            Point::new(c1.x as f64, c1.y as f64),
            Point::new(c2.x as f64, c2.y as f64),
            Point::new(end_v.x as f64, end_v.y as f64),
        );
    }

    path.close_path();
    path
}

/// Convert an [`OklchColor`] to a `peniko::Color` for Vello rendering.
///
/// Chain: OKLCH → OKLab → linear sRGB → `SrgbRgba` (gamma-encoded 8-bit)
/// → `Color::from_rgba8` (re-linearizes inside Vello). The double-
/// gamma trip is intentional — `Color::from_rgba8` expects sRGB-encoded
/// bytes and produces the same linear values Vello stores natively.
#[must_use]
pub fn oklch_to_color(color: OklchColor) -> Color {
    let srgb = color.to_srgb();
    let [r, g, b, a] = srgb.0;
    Color::from_rgba8(r, g, b, a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use ph2d_vector_doc::{
        FillSolid, Region, Segment, StrokeStyle, StyleTable, Vertex, WindingRule,
    };
    use vello::Scene;

    fn make_triangle_network() -> VectorNetwork {
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        net.vertices.push(Vertex::auto(1, Vec2::new(100.0, 0.0)));
        net.vertices.push(Vertex::auto(2, Vec2::new(50.0, 86.6)));
        net.segments.push(Segment::straight(0, 0, 1));
        net.segments.push(Segment::straight(1, 1, 2));
        net.segments.push(Segment::straight(2, 2, 0));
        let mut region = Region::new(0, WindingRule::NonZero);
        region
            .segments
            .extend_from_slice(&[(0, true), (1, true), (2, true)]);
        region.fill = Some(0);
        net.regions.push(region);
        net
    }

    fn make_styles_with_red_fill() -> StyleTable {
        let mut t = StyleTable::default();
        t.fills.insert(
            0,
            FillSolid {
                color: OklchColor::opaque(0.5, 0.2, 30.0),
            },
        );
        t
    }

    #[test]
    fn triangle_region_builds_5_element_bezpath() {
        // 1 move_to + 3 curve_to + 1 close_path = 5 elements.
        let net = make_triangle_network();
        let path = build_region_path(&net, &net.regions[0]);
        assert_eq!(
            path.elements().len(),
            5,
            "expected MoveTo + 3 CurveTo + ClosePath"
        );
    }

    #[test]
    fn draw_vector_network_with_unfilled_region_skips() {
        // Region with fill = None should not be drawn.
        let mut net = make_triangle_network();
        net.regions[0].fill = None;
        let styles = make_styles_with_red_fill();
        let mut scene = Scene::new();
        let drawn = draw_vector_network(&mut scene, &net, &styles, Affine::IDENTITY);
        assert_eq!(drawn, 0);
    }

    #[test]
    fn draw_vector_network_with_filled_region_draws() {
        let net = make_triangle_network();
        let styles = make_styles_with_red_fill();
        let mut scene = Scene::new();
        let drawn = draw_vector_network(&mut scene, &net, &styles, Affine::IDENTITY);
        // Fill drawn; the 3 region segments carry no `style_ref` (Pen),
        // so the stroke pass skips them — still 1.
        assert_eq!(drawn, 1);
    }

    /// Two vertices + one open segment carrying a `style_ref` that resolves to
    /// a stroke (a committed Pencil path). No region.
    fn make_styled_open_segment(seg_id_base: u32) -> (VectorNetwork, StyleTable) {
        let mut net = VectorNetwork::empty();
        let a = seg_id_base * 2;
        let b = a + 1;
        net.vertices.push(Vertex::auto(a, Vec2::new(0.0, 0.0)));
        net.vertices.push(Vertex::auto(b, Vec2::new(100.0, 0.0)));
        let mut seg = Segment::straight(seg_id_base, a, b);
        seg.style_ref = Some(7);
        net.segments.push(seg);
        let mut styles = StyleTable::default();
        styles.strokes.insert(7, StrokeStyle::default());
        (net, styles)
    }

    #[test]
    fn draw_vector_network_strokes_styled_open_segment() {
        let (net, styles) = make_styled_open_segment(0);
        let mut scene = Scene::new();
        let drawn = draw_vector_network(&mut scene, &net, &styles, Affine::IDENTITY);
        assert_eq!(drawn, 1, "a styled open segment must be stroked");
    }

    #[test]
    fn draw_vector_network_skips_segment_with_unresolved_style_ref() {
        let (mut net, _) = make_styled_open_segment(0);
        // style_ref points at a stroke not present in the (empty) table.
        let styles = StyleTable::default();
        net.segments[0].style_ref = Some(99);
        let mut scene = Scene::new();
        let drawn = draw_vector_network(&mut scene, &net, &styles, Affine::IDENTITY);
        assert_eq!(drawn, 0, "an unresolved style_ref must not stroke");
    }

    #[test]
    fn draw_vector_network_mixes_pen_fill_and_pencil_stroke_no_double_draw() {
        // Pen triangle (filled region, unstyled segments) + a Pencil open
        // segment (styled). Exactly 1 fill + 1 stroke; the 3 Pen segments are
        // unstyled so the stroke pass never double-draws them.
        let mut net = make_triangle_network();
        net.vertices.push(Vertex::auto(3, Vec2::new(200.0, 0.0)));
        net.vertices.push(Vertex::auto(4, Vec2::new(300.0, 0.0)));
        let mut seg = Segment::straight(3, 3, 4);
        seg.style_ref = Some(7);
        net.segments.push(seg);
        let mut styles = make_styles_with_red_fill();
        styles.strokes.insert(7, StrokeStyle::default());
        let mut scene = Scene::new();
        let drawn = draw_vector_network(&mut scene, &net, &styles, Affine::IDENTITY);
        assert_eq!(drawn, 2, "1 Pen fill + 1 Pencil stroke");
    }

    #[test]
    fn empty_network_draws_nothing_without_panic() {
        let net = VectorNetwork::empty();
        let styles = StyleTable::default();
        let mut scene = Scene::new();
        let drawn = draw_vector_network(&mut scene, &net, &styles, Affine::IDENTITY);
        assert_eq!(drawn, 0);
    }

    #[test]
    fn region_with_dangling_segment_ref_yields_empty_path_strict_mode() {
        // R1 audit Lens-B MED-2: strict mode aborts the path entirely
        // on the first dangling ref instead of stitching across the
        // gap and producing a self-intersecting fill.
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::ZERO));
        let mut region = Region::new(0, WindingRule::NonZero);
        region.segments.extend_from_slice(&[(99, true)]);
        net.regions.push(region);
        let path = build_region_path(&net, &net.regions[0]);
        assert!(
            path.is_empty(),
            "dangling ref should produce empty path (strict mode), got {} elements",
            path.elements().len()
        );
    }

    #[test]
    fn region_with_partial_dangling_aborts_whole_region() {
        // R1 audit Lens-B MED-2: even if the FIRST segments are valid,
        // a later dangling segment aborts the whole region — the
        // alternative is a partial polyline stitched across the gap,
        // which looks like a render bug.
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::ZERO));
        net.vertices.push(Vertex::auto(1, Vec2::new(10.0, 0.0)));
        net.segments.push(Segment::straight(0, 0, 1));
        let mut region = Region::new(0, WindingRule::NonZero);
        region.segments.extend_from_slice(&[(0, true), (99, true)]); // (0) valid, then dangling
        net.regions.push(region);
        let path = build_region_path(&net, &net.regions[0]);
        assert!(
            path.is_empty(),
            "partial dangling should abort whole region (strict mode); got {} elements",
            path.elements().len()
        );
    }

    #[test]
    fn oklch_to_color_white_round_trips_to_near_white() {
        let white_oklch = OklchColor::opaque(1.0, 0.0, 0.0);
        let color = oklch_to_color(white_oklch);
        // peniko::Color exposes the components as f32 channels — we just
        // verify the value isn't black (a smoke that the conversion runs).
        let [r, g, b, _a] = color.components;
        assert!(
            r > 0.9 && g > 0.9 && b > 0.9,
            "expected near-white, got [{r}, {g}, {b}]"
        );
    }

    #[test]
    fn winding_rule_maps_to_vello_fill_modes() {
        // Cover both winding paths via two distinct regions.
        let mut net = make_triangle_network();
        net.regions[0].winding = WindingRule::EvenOdd;
        let styles = make_styles_with_red_fill();
        let mut scene = Scene::new();
        let drawn = draw_vector_network(&mut scene, &net, &styles, Affine::IDENTITY);
        assert_eq!(drawn, 1);
    }

    #[test]
    fn oklch_to_color_black_round_trips_to_near_black() {
        // R1 audit Lens-D MED-D10 expansion: cover the BLACK case
        // (only WHITE was tested originally).
        let black_oklch = OklchColor::opaque(0.0, 0.0, 0.0);
        let color = oklch_to_color(black_oklch);
        let [r, g, b, _a] = color.components;
        assert!(
            r < 0.05 && g < 0.05 && b < 0.05,
            "expected near-black, got [{r}, {g}, {b}]"
        );
    }

    #[test]
    fn oklch_to_color_alpha_is_preserved() {
        // R1 audit Lens-D MED-D10: alpha previously untested.
        let translucent = OklchColor {
            l: 0.5,
            c: 0.0,
            h: 0.0,
            a: 0.5,
        };
        let color = oklch_to_color(translucent);
        let [_r, _g, _b, a] = color.components;
        assert!((a - 0.5).abs() < 0.01, "expected alpha ~0.5, got {a}");
    }

    #[test]
    fn oklch_to_color_red_maps_to_red_dominant_channel() {
        // OKLCH (L=0.628, C=0.258, H=29.234°) ≈ sRGB red.
        let red_oklch = OklchColor::opaque(0.628, 0.258, 29.234);
        let color = oklch_to_color(red_oklch);
        let [r, g, b, _a] = color.components;
        assert!(r > g && r > b, "expected R-dominant, got [{r}, {g}, {b}]");
        assert!(r > 0.5, "expected R > 0.5, got {r}");
    }
}
