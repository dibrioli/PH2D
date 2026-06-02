//! Tool icon for the Vector Shape — a pentagon as a closed
//! `kurbo::BezPath` in a 24×24 viewBox (matches
//! `docs/design/icons/vector-shape.svg`). Caller scales by
//! `Affine::scale(chip_size / 24.0)` at paint time.
//!
//! **W2 placeholder.** A regular pentagon signals "primitive shape"
//! unambiguously and is visually distinct from the Pen / Pencil glyphs.
//! Tool Studio (W15) may swap for a richer multi-shape glyph.

use ph2d_vector::BezPath;

/// Vector Shape icon as a `BezPath` in a 24×24 coordinate space.
#[must_use]
pub fn vector_shape_bezpath() -> BezPath {
    use ph2d_vector::Point;
    let mut p = BezPath::new();
    // Regular pentagon centered at (12, 12), radius ~9, first tip up.
    p.move_to(Point::new(12.0, 3.0));
    p.line_to(Point::new(20.6, 9.2));
    p.line_to(Point::new(17.3, 19.3));
    p.line_to(Point::new(6.7, 19.3));
    p.line_to(Point::new(3.4, 9.2));
    p.close_path();
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_shape_bezpath_has_expected_element_count() {
        // MoveTo + LineTo × 4 + ClosePath = 6.
        assert_eq!(vector_shape_bezpath().elements().len(), 6);
    }

    #[test]
    fn vector_shape_bezpath_fits_in_24x24_viewbox() {
        use ph2d_vector::Shape;
        let bb = vector_shape_bezpath().bounding_box();
        assert!(
            bb.x0 >= 0.0 && bb.y0 >= 0.0 && bb.x1 <= 24.0 && bb.y1 <= 24.0,
            "icon bbox {bb:?} outside 24x24 viewBox"
        );
    }
}
