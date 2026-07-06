//! Tool icon for the Vector tool — a pen nib as a closed `kurbo::BezPath` in a
//! 24×24 viewBox (matches `docs/design/icons/vector.svg`). Caller scales by
//! `Affine::scale(chip_size / 24.0)` at paint time.
//!
//! A slanted fountain-pen nib reads as "vector pen / draw" and is visually
//! distinct from the other topbar glyphs. Placeholder — Tool Studio may swap it
//! for a richer glyph later.

use ph2d_vector::BezPath;

/// Vector tool icon as a `BezPath` in a 24×24 coordinate space.
#[must_use]
pub fn vector_bezpath() -> BezPath {
    use ph2d_vector::Point;
    let mut p = BezPath::new();
    // Slanted nib: tip at lower-left, body up to the upper-right.
    p.move_to(Point::new(4.0, 20.0));
    p.line_to(Point::new(6.5, 11.5));
    p.line_to(Point::new(11.5, 6.5));
    p.line_to(Point::new(20.0, 4.0));
    p.line_to(Point::new(17.5, 12.5));
    p.line_to(Point::new(12.5, 17.5));
    p.close_path();
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_bezpath_has_expected_element_count() {
        // MoveTo + LineTo × 5 + ClosePath = 7.
        assert_eq!(vector_bezpath().elements().len(), 7);
    }

    #[test]
    fn vector_bezpath_fits_in_24x24_viewbox() {
        use ph2d_vector::Shape;
        let bb = vector_bezpath().bounding_box();
        assert!(
            bb.x0 >= 0.0 && bb.y0 >= 0.0 && bb.x1 <= 24.0 && bb.y1 <= 24.0,
            "icon bbox {bb:?} outside 24x24 viewBox"
        );
    }
}
