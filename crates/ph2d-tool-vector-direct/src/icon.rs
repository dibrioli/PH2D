//! Tool icon for Vector Direct Select — an arrow cursor with a grabbed
//! vertex node, in a 24×24 viewBox (matches
//! `docs/design/icons/vector-direct.svg`).
//!
//! **W2 placeholder.** Arrow + a small square node distinguishes
//! "direct / vertex select" from the plain Select arrow (Illustrator's
//! white-arrow analogue).

use ph2d_vector::BezPath;

/// Vector Direct Select icon in a 24×24 coordinate space.
#[must_use]
pub fn vector_direct_bezpath() -> BezPath {
    use ph2d_vector::Point;
    let mut p = BezPath::new();
    // Arrow cursor (lower-right).
    p.move_to(Point::new(7.0, 5.0));
    p.line_to(Point::new(7.0, 17.0));
    p.line_to(Point::new(10.5, 13.8));
    p.line_to(Point::new(12.7, 18.5));
    p.line_to(Point::new(14.4, 17.7));
    p.line_to(Point::new(12.2, 13.2));
    p.line_to(Point::new(16.5, 13.2));
    p.close_path();
    // Grabbed vertex node (upper-left square).
    p.move_to(Point::new(3.0, 3.0));
    p.line_to(Point::new(7.0, 3.0));
    p.line_to(Point::new(7.0, 7.0));
    p.line_to(Point::new(3.0, 7.0));
    p.close_path();
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_direct_bezpath_has_expected_element_count() {
        // Arrow: MoveTo + LineTo×6 + ClosePath = 8.
        // Node:  MoveTo + LineTo×3 + ClosePath = 5.
        // Total: 13.
        assert_eq!(vector_direct_bezpath().elements().len(), 13);
    }

    #[test]
    fn vector_direct_bezpath_fits_in_24x24_viewbox() {
        use ph2d_vector::Shape;
        let bb = vector_direct_bezpath().bounding_box();
        assert!(
            bb.x0 >= 0.0 && bb.y0 >= 0.0 && bb.x1 <= 24.0 && bb.y1 <= 24.0,
            "icon bbox {bb:?} outside 24x24 viewBox"
        );
    }
}
