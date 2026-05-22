//! Tool icon for Background Removal — Lucide `eraser` glyph as a
//! `kurbo::BezPath` in a 24×24 viewBox (same space as the source
//! SVG), to be scaled by the caller via `Affine::scale(chip_size /
//! 24.0)` at paint time.
//!
//! Mirror of `ph2d_tool_trim_transparency`'s icon approach — a Lucide
//! glyph hand-ported to a `BezPath` in 24×24 space.

use ph2d_vector::BezPath;

/// Lucide `eraser` icon as a `BezPath` in a 24×24 coordinate space.
///
/// Source: <https://lucide.dev/icons/eraser> (ISC-0 license, embeddable).
/// Three sub-paths combined into a single `BezPath` via `move_to`
/// breaks — caller does not need to know it is composite.
///
/// Original SVG paths (Lucide v0.453):
/// ```svg
/// <path d="m7 21-4.3-4.3c-1-1-1-2.5 0-3.4l9.6-9.6c1-1 2.5-1 3.4 0
///          l5.6 5.6c1 1 1 2.5 0 3.4L13 21"/>
/// <path d="M22 21H7"/>
/// <path d="m5 11 9 9"/>
/// ```
///
/// Converted to absolute coordinates below. Cubic Béziers from the
/// SVG `c …` segments translate directly into `curve_to` calls
/// (control1, control2, end). Open polylines — no `close_path`.
pub fn eraser_bezpath() -> BezPath {
    use ph2d_vector::Point;
    let mut p = BezPath::new();

    // ── Path 1: the eraser body (open polyline, 4 segments + 3 curves)
    // m7 21 -4.3 -4.3                       → line to (2.7, 16.7)
    // c -1 -1 -1 -2.5 0 -3.4                → curve to (2.7, 13.3)
    // l 9.6 -9.6                            → line to (12.3, 3.7)
    // c 1 -1 2.5 -1 3.4 0                   → curve to (15.7, 3.7)
    // l 5.6 5.6                             → line to (21.3, 9.3)
    // c 1 1 1 2.5 0 3.4                     → curve to (21.3, 12.7)
    // L 13 21                               → line to (13, 21)  (absolute)
    p.move_to(Point::new(7.0, 21.0));
    p.line_to(Point::new(2.7, 16.7));
    p.curve_to(
        Point::new(1.7, 15.7), // 2.7 + (-1, -1)
        Point::new(1.7, 14.2), // 2.7 + (-1, -2.5)
        Point::new(2.7, 13.3), // 2.7 + (0, -3.4)
    );
    p.line_to(Point::new(12.3, 3.7));
    p.curve_to(
        Point::new(13.3, 2.7), // 12.3 + (1, -1)
        Point::new(14.8, 2.7), // 12.3 + (2.5, -1)
        Point::new(15.7, 3.7), // 12.3 + (3.4, 0)
    );
    p.line_to(Point::new(21.3, 9.3));
    p.curve_to(
        Point::new(22.3, 10.3), // 21.3 + (1, 1)
        Point::new(22.3, 11.8), // 21.3 + (1, 2.5)
        Point::new(21.3, 12.7), // 21.3 + (0, 3.4)
    );
    p.line_to(Point::new(13.0, 21.0));

    // ── Path 2: ground line under the eraser — M22 21 H7
    p.move_to(Point::new(22.0, 21.0));
    p.line_to(Point::new(7.0, 21.0));

    // ── Path 3: diagonal cut line through the eraser body — m5 11 9 9
    p.move_to(Point::new(5.0, 11.0));
    p.line_to(Point::new(14.0, 20.0));

    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eraser_bezpath_has_expected_element_count() {
        let bp = eraser_bezpath();
        // Path 1: MoveTo + LineTo + CurveTo + LineTo + CurveTo + LineTo +
        // CurveTo + LineTo = 8.
        // Path 2: MoveTo + LineTo = 2.
        // Path 3: MoveTo + LineTo = 2.
        // Total = 12 elements. No ClosePath — Lucide eraser is open
        // strokes, matching the source SVG.
        assert_eq!(bp.elements().len(), 12);
    }
}
