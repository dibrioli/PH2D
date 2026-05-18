//! Trim Transparency icon — Lucide `crop` glyph as a `BezPath`.
//!
//! Direct port of Lucide's `crop.svg` (MIT-licensed, ISC-compatible).
//! Two open subpaths, identical to the SVG, drawn in a 24×24 grid with
//! Y-down (SVG convention — matches Vello's coordinate system; see
//! `ph2d-render::projection` for the world↔screen flip). The two
//! mirrored L-brackets read as a classic photographer's crop mark.
//!
//! The returned path is stroked, not filled — the Integrator should
//! draw it with `Stroke::new(2.0)` and `Stroke::with_caps(Round, Round)`
//! to match Lucide's `stroke-linecap="round"` styling, then scale via
//! `Affine::scale(chip_px / 24.0)` to fit any chip size.

use ph2d_vector::BezPath;

/// Build the crop glyph as a `BezPath` in a 24×24 design space
/// (Y-down, origin top-left — matches SVG/Vello).
///
/// ## SVG source (Lucide `crop.svg`)
/// ```svg
/// <path d="M6 2v14a2 2 0 0 0 2 2h14" />
/// <path d="M18 22V8a2 2 0 0 0-2-2H2" />
/// ```
///
/// Each path is one open polyline with a single rounded inner corner
/// (90° arc, radius 2). We approximate each arc with a cubic Bézier
/// using the classic 4*(sqrt(2)-1)/3 ≈ 0.5523 control-point ratio
/// for a quarter circle.
pub fn crop_bezpath() -> BezPath {
    use ph2d_vector::Point;

    // Quarter-arc Bézier control offset for radius r:
    // (1 - 4/3 * (sqrt(2) - 1)) * r  on the perpendicular axis,
    // simplified — at r=2 the offset is ~1.1.
    const K: f64 = 0.5522847498307936; // 4*(sqrt(2)-1)/3
    const R: f64 = 2.0; // corner radius matches SVG "a2 2 0 0 0"

    let mut p = BezPath::new();

    // ---- Path 1: M6 2 v14 a2 2 0 0 0 2 2 h14 -------------------------
    // Start (6, 2); down to (6, 16); arc CCW r=2 from (6, 16) to (8, 18);
    // right to (22, 18).
    p.move_to(Point::new(6.0, 2.0));
    p.line_to(Point::new(6.0, 16.0));
    // Arc replaces "a2 2 0 0 0 2 2" (sweep=0 in SVG = CCW for our axes).
    // From (6, 16) to (8, 18) with center at (8, 16).
    p.curve_to(
        Point::new(6.0, 16.0 + R * K),
        Point::new(8.0 - R * K, 18.0),
        Point::new(8.0, 18.0),
    );
    p.line_to(Point::new(22.0, 18.0));

    // ---- Path 2: M18 22 V8 a2 2 0 0 0 -2 -2 H2 -----------------------
    // Start (18, 22); up to (18, 8); arc r=2 from (18, 8) to (16, 6);
    // left to (2, 6).
    p.move_to(Point::new(18.0, 22.0));
    p.line_to(Point::new(18.0, 8.0));
    // From (18, 8) to (16, 6) with center at (16, 8).
    p.curve_to(
        Point::new(18.0, 8.0 - R * K),
        Point::new(16.0 + R * K, 6.0),
        Point::new(16.0, 6.0),
    );
    p.line_to(Point::new(2.0, 6.0));

    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_has_two_subpaths_of_expected_length() {
        // Each subpath is MoveTo + LineTo + CurveTo + LineTo = 4 elements;
        // two subpaths = 8 total. No `ClosePath` — both L-brackets are
        // open polylines exactly like the source SVG.
        let path = crop_bezpath();
        assert_eq!(
            path.elements().len(),
            8,
            "expected two open subpaths of 4 elements each",
        );
    }

    #[test]
    fn path_serializes_to_two_moveto_subpaths_in_svg_form() {
        // `BezPath::to_svg()` writes standard SVG path syntax.
        // Two subpaths == two `M` commands appear in the output.
        let s = crop_bezpath().to_svg();
        let m_count = s.matches('M').count();
        assert_eq!(
            m_count, 2,
            "expected exactly two MoveTo commands; got {s:?}"
        );
        // Spot-check the starting points the SVG source defines.
        // kurbo serializes coordinate pairs with a comma separator
        // (`M6,2`); fall back to space form (`M 6 2`) if a future
        // version changes the formatting.
        assert!(
            s.contains("M6,2") || s.contains("M 6 2") || s.contains("M6 2"),
            "path 1 starts at (6, 2): {s:?}"
        );
        assert!(
            s.contains("M18,22") || s.contains("M 18 22") || s.contains("M18 22"),
            "path 2 starts at (18, 22): {s:?}"
        );
    }
}
