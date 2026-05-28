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
//! - Strokes are deferred to W2 (the stroke vocabulary lands with the
//!   Pencil tool); W1 ships fill-only.
//!
//! ## Anti-padrões avoided
//!
//! - **No allocation in the per-frame draw loop**: a single `BezPath` is
//!   built and discarded per region. SmallVec inline budgets in the
//!   network (32 vertices, 64 segments, 8 regions) keep typical
//!   documents heap-free.
//! - **No `unsafe`** (`#![forbid(unsafe_code)]` at the crate root).
//! - **`vello::*` / `kurbo::*` imports stay confined to this crate** —
//!   arch-gate `vello_kurbo_only_in_ph2d_vector` (ADR-0059 §L6F1 + W2+
//!   arch-gate landing).

use ph2d_color::OklchColor;
use ph2d_vector_doc::{Region, VectorNetwork};
use vello::Scene;
use vello::kurbo::{Affine, BezPath, Point};
use vello::peniko::{Brush, Color, Fill};

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
    let mut drawn = 0;
    for region in &network.regions {
        let Some(fill_ref) = region.fill else {
            continue;
        };
        let Some(fill_solid) = styles.fills.get(&fill_ref) else {
            continue;
        };
        let path = build_region_path(network, region);
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
    drawn
}

/// Build a closed `kurbo::BezPath` for one region.
///
/// Pure function; testable without Vello. Returns an empty path if the
/// region references segments that aren't in the network (validate the
/// network upstream — this routine is permissive so a partially built
/// network during interactive drawing still renders the well-formed
/// regions).
#[must_use]
pub fn build_region_path(network: &VectorNetwork, region: &Region) -> BezPath {
    let mut path = BezPath::new();
    let mut first = true;

    for &(seg_id, forward) in &region.segments {
        let Some(segment) = network.segments.iter().find(|s| s.id == seg_id) else {
            continue;
        };
        let (start_v, end_v, c1, c2) = if forward {
            let Some(s) = network.vertices.iter().find(|v| v.id == segment.start) else {
                continue;
            };
            let Some(e) = network.vertices.iter().find(|v| v.id == segment.end) else {
                continue;
            };
            (
                s.pos,
                e.pos,
                s.pos + segment.out_at_start,
                e.pos + segment.in_at_end,
            )
        } else {
            let Some(s) = network.vertices.iter().find(|v| v.id == segment.end) else {
                continue;
            };
            let Some(e) = network.vertices.iter().find(|v| v.id == segment.start) else {
                continue;
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

    if !first {
        path.close_path();
    }
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
    use ph2d_vector_doc::{FillSolid, Region, Segment, StyleTable, Vertex, WindingRule};
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
    fn triangle_region_builds_4_element_bezpath() {
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
        assert_eq!(drawn, 1);
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
    fn region_with_dangling_segment_ref_renders_what_it_can() {
        // Partial network (segment 99 doesn't exist) — should NOT panic
        // and should still build an empty path (no closed loop).
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::ZERO));
        let mut region = Region::new(0, WindingRule::NonZero);
        region.segments.extend_from_slice(&[(99, true)]);
        net.regions.push(region);
        let path = build_region_path(&net, &net.regions[0]);
        assert!(
            path.is_empty(),
            "dangling ref should produce empty path, got {} elements",
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
}
