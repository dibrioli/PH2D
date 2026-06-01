//! Tool icon for the Vector Pencil — a sharpened pencil as a closed
//! `kurbo::BezPath` in a 24×24 viewBox (matches the SVG source in
//! `docs/design/icons/vector-pencil.svg`). Caller scales by
//! `Affine::scale(chip_size / 24.0)` at paint time.
//!
//! **W2 placeholder.** Hand-traced pencil body + sharpened tip using
//! straight segments — unambiguous as a pencil at 24×24 and visually
//! distinct from the Pen glyph. Tool Studio (W15) may swap for the
//! canonical Lucide `pencil` port once the icon system grows a generic
//! `arc_to_cubic` helper.

use ph2d_vector::BezPath;

/// Vector Pencil icon as a `BezPath` in a 24×24 coordinate space.
///
/// Polyline-only (kurbo handles line segments uniformly). The body +
/// sharpened tip is one closed `move_to → line_to* → close_path`; a short
/// second sub-path marks the ferrule band near the cap.
#[must_use]
pub fn vector_pencil_bezpath() -> BezPath {
    use ph2d_vector::Point;
    let mut p = BezPath::new();

    // Pencil body — a slanted parallelogram running top-right → bottom-left,
    // sharpened to a writing point.
    //
    //  (18,3)─(21,6)
    //          │
    //          │  (shaft)
    //          │
    //  (6,15)  (9,18)
    //     \    /
    //     (4,21)  ← writing tip
    p.move_to(Point::new(18.0, 3.0)); // cap corner (top)
    p.line_to(Point::new(21.0, 6.0)); // cap corner (right)
    p.line_to(Point::new(9.0, 18.0)); // right edge down to the point
    p.line_to(Point::new(4.0, 21.0)); // sharpened writing tip
    p.line_to(Point::new(6.0, 15.0)); // left edge back up
    p.close_path();

    // Ferrule band — a short line across the shaft just below the cap.
    p.move_to(Point::new(15.0, 6.0));
    p.line_to(Point::new(18.0, 9.0));

    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_pencil_bezpath_has_expected_element_count() {
        // Path 1: MoveTo + LineTo × 4 + ClosePath = 6
        // Path 2: MoveTo + LineTo = 2
        // Total: 8 elements.
        let bp = vector_pencil_bezpath();
        assert_eq!(bp.elements().len(), 8);
    }

    #[test]
    fn vector_pencil_bezpath_fits_in_24x24_viewbox() {
        use ph2d_vector::Shape;
        let bb = vector_pencil_bezpath().bounding_box();
        assert!(
            bb.x0 >= 0.0 && bb.y0 >= 0.0 && bb.x1 <= 24.0 && bb.y1 <= 24.0,
            "icon bbox {bb:?} outside 24x24 viewBox"
        );
    }
}
