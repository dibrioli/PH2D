//! Tool icon for Vector Select — an arrow cursor as a closed
//! `kurbo::BezPath` in a 24×24 viewBox (matches
//! `docs/design/icons/vector-select.svg`).
//!
//! **W2 placeholder.** The classic NW-pointing selection arrow — the
//! universal "select / pick" affordance, distinct from the create-tool
//! glyphs.

use ph2d_vector::BezPath;

/// Vector Select icon (arrow cursor) in a 24×24 coordinate space.
#[must_use]
pub fn vector_select_bezpath() -> BezPath {
    use ph2d_vector::Point;
    let mut p = BezPath::new();
    p.move_to(Point::new(5.0, 3.0));
    p.line_to(Point::new(5.0, 18.0));
    p.line_to(Point::new(9.0, 14.0));
    p.line_to(Point::new(11.5, 19.5));
    p.line_to(Point::new(13.5, 18.5));
    p.line_to(Point::new(11.0, 13.5));
    p.line_to(Point::new(16.0, 13.5));
    p.close_path();
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_select_bezpath_has_expected_element_count() {
        // MoveTo + LineTo × 6 + ClosePath = 8.
        assert_eq!(vector_select_bezpath().elements().len(), 8);
    }

    #[test]
    fn vector_select_bezpath_fits_in_24x24_viewbox() {
        use ph2d_vector::Shape;
        let bb = vector_select_bezpath().bounding_box();
        assert!(
            bb.x0 >= 0.0 && bb.y0 >= 0.0 && bb.x1 <= 24.0 && bb.y1 <= 24.0,
            "icon bbox {bb:?} outside 24x24 viewBox"
        );
    }
}
