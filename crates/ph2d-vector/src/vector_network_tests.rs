//! Testes de `vector_network.rs` — irmão por tecto de LOC (`workspace_src_files_under_loc_cap`).
//!
//! Corte mecânico: o `mod tests` saiu inteiro, verbatim, do ficheiro que o continha.

use super::*;
use glam::Vec2;
use ph2d_vector_doc::{
    FillSolid, ProceduralFill, Region, Segment, StrokeStyle, StyleTable, Vertex, WindingRule,
};
use vello::Scene;
use vello::kurbo::Shape;

#[test]
fn variable_width_band_expands_to_a_tapered_quad() {
    // Straight segment, half-widths 1 → 2: the band spans x∈[0,10], y∈[-2,2].
    let line = [Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)];
    let band = variable_width_band(&line, &[2.0, 4.0]).expect("two points expand");
    let bb = band.bounding_box();
    assert!((bb.x0 - 0.0).abs() < 1e-6 && (bb.x1 - 10.0).abs() < 1e-6);
    assert!((bb.y0 + 2.0).abs() < 1e-6 && (bb.y1 - 2.0).abs() < 1e-6);
}

#[test]
fn variable_width_band_rejects_degenerate() {
    assert!(
        variable_width_band(&[Vec2::ZERO], &[1.0]).is_none(),
        "needs ≥2 points"
    );
    assert!(
        variable_width_band(&[Vec2::ZERO, Vec2::X], &[1.0]).is_none(),
        "len mismatch"
    );
}

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

#[test]
fn procedural_fill_falls_back_to_solid_without_an_image() {
    // A region whose fill resolves to a procedural fill (ADR-0056-amendment-3)
    // but no image is supplied → fallback solid is drawn (graceful degrade).
    let net = make_triangle_network(); // region.fill = Some(0)
    let mut styles = StyleTable::default();
    let pref = styles.insert_procedural(ProceduralFill::diffusion(
        7,
        OklchColor::opaque(0.6, 0.1, 200.0),
    ));
    assert_eq!(
        pref, 0,
        "shares the FillRef namespace; region.fill = Some(0)"
    );
    let mut scene = Scene::new();
    let drawn = draw_vector_network(&mut scene, &net, &styles, Affine::IDENTITY);
    assert_eq!(drawn, 1);
}

#[test]
fn procedural_fill_renders_image_when_resolver_supplies_one() {
    let net = make_triangle_network();
    let mut styles = StyleTable::default();
    styles.insert_procedural(ProceduralFill::shader_graph(
        3,
        OklchColor::opaque(0.5, 0.0, 0.0),
    ));
    let rgba = std::sync::Arc::new(vec![255u8; 2 * 2 * 4]);
    let mut scene = Scene::new();
    let drawn = draw_vector_network_with_fills(&mut scene, &net, &styles, Affine::IDENTITY, |fr| {
        (fr == 0).then(|| ProceduralFillImage {
            rgba: rgba.clone(),
            width: 2,
            height: 2,
        })
    });
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
