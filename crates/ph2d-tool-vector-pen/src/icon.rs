//! Tool icon for the Vector Pen — stylized pen body as a closed
//! `kurbo::BezPath` in a 24×24 viewBox (matches the SVG source in
//! `docs/design/icons/vector-pen.svg`). Caller scales by
//! `Affine::scale(chip_size / 24.0)` at paint time.
//!
//! **W1 placeholder.** Hand-traced quadrilateral pen body — simpler
//! than Lucide's curved `pen` glyph but unambiguous as a pen at
//! 24×24. W2 / Tool Studio may swap for the canonical Lucide port
//! (requires kurbo conversion of SVG arc commands `a` → cubic
//! Béziers, deferred until the icon system grows a generic
//! `arc_to_cubic` helper).

use ph2d_vector::BezPath;

/// Vector Pen icon as a `BezPath` in a 24×24 coordinate space.
///
/// Polyline-only (no curves) — kurbo handles straight cubic / line
/// segments uniformly. The closed shape is one continuous
/// `move_to → line_to* → close_path` sequence so callers using
/// `Scene::stroke` get a single brush draw.
#[must_use]
pub fn vector_pen_bezpath() -> BezPath {
    use ph2d_vector::Point;
    let mut p = BezPath::new();

    // Pen body — closed quadrilateral tracing the pen outline.
    // Coordinates roughly mirror Lucide's `pen` glyph bbox but use
    // straight segments for W1 simplicity.
    //
    //  (18,4)─(20,6)
    //         │
    //         │ (pen shaft running diagonally)
    //         │
    //  (8,18)─(6,12)
    //  │
    //  (4,20) ← writing tip
    p.move_to(Point::new(18.0, 4.0));
    p.line_to(Point::new(20.0, 6.0));
    p.line_to(Point::new(8.0, 18.0));
    p.line_to(Point::new(4.0, 20.0));
    p.line_to(Point::new(6.0, 12.0));
    p.close_path();

    // Cap detail — a short line across the upper end of the pen
    // hinting at the cap / hold edge.
    p.move_to(Point::new(14.0, 8.0));
    p.line_to(Point::new(16.0, 10.0));

    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_pen_bezpath_has_expected_element_count() {
        // Path 1: MoveTo + LineTo × 4 + ClosePath = 6
        // Path 2: MoveTo + LineTo = 2
        // Total: 8 elements.
        let bp = vector_pen_bezpath();
        assert_eq!(bp.elements().len(), 8);
    }

    #[test]
    fn vector_pen_bezpath_fits_in_24x24_viewbox() {
        // Tight-bounding-box check — Shape::bounding_box returns the
        // smallest axis-aligned rect that contains the path. Lighter
        // than iterating raw PathEls (kurbo's `PathEl` is not in
        // `ph2d_vector`'s re-export surface).
        use ph2d_vector::Shape;
        let bb = vector_pen_bezpath().bounding_box();
        assert!(
            bb.x0 >= 0.0 && bb.y0 >= 0.0 && bb.x1 <= 24.0 && bb.y1 <= 24.0,
            "icon bbox {:?} outside 24x24 viewBox",
            bb
        );
    }
}
