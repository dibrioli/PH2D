//! Padding icon — `BezPath` glyph for the Image Tools chrome pill.
//!
//! SCAFFOLD PLACEHOLDER (Coordinator): two concentric squares — an outer
//! canvas frame and an inner content frame — reading as "padding around
//! content". The Implementer should confirm/finalise the glyph (a Lucide
//! source such as `expand` / `move` / `scaling`, or this concentric
//! frame), drop the SVG at `docs/design/icons/padding.svg` for the design
//! pipeline, and keep `path_is_non_empty` + `path_fits_inside_24_grid`
//! green. Stroked, not filled — drawn like the sibling row glyphs.

use ph2d_vector::{BezPath, Point};

/// Append an axis-aligned rectangle as its own closed subpath.
fn push_rect(p: &mut BezPath, x0: f64, y0: f64, x1: f64, y1: f64) {
    p.move_to(Point::new(x0, y0));
    p.line_to(Point::new(x1, y0));
    p.line_to(Point::new(x1, y1));
    p.line_to(Point::new(x0, y1));
    p.close_path();
}

/// Build the Padding glyph as a `BezPath` in a 24×24 design space
/// (Y-down, origin top-left): an outer canvas frame `(2,2)-(22,22)` and
/// an inner content frame `(8,8)-(16,16)`.
pub fn padding_bezpath() -> BezPath {
    let mut p = BezPath::new();
    push_rect(&mut p, 2.0, 2.0, 22.0, 22.0);
    push_rect(&mut p, 8.0, 8.0, 16.0, 16.0);
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector::Shape;

    #[test]
    fn path_is_non_empty() {
        assert!(!padding_bezpath().elements().is_empty());
    }

    #[test]
    fn path_fits_inside_24_grid() {
        let bb = padding_bezpath().bounding_box();
        assert!(bb.x0 >= 0.0 && bb.y0 >= 0.0);
        assert!(bb.x1 <= 24.0 && bb.y1 <= 24.0);
    }
}
