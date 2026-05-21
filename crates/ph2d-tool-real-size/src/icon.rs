//! Real Size icon — `BezPath` glyph for the Image Tools chrome pill.
//!
//! SCAFFOLD PLACEHOLDER (Coordinator): a Lucide-style "maximize" frame
//! (four corner brackets) in a 24×24 Y-down design space — reads as
//! "fit to real size". The Implementer should confirm/finalise the glyph
//! (port the chosen Lucide source, e.g. `maximize.svg` or `scaling.svg`,
//! and drop the SVG at `docs/design/icons/real-size.svg` so the design
//! pipeline has a source), then keep the `path_is_non_empty` test green.
//!
//! Stroked, not filled — chrome draws it with `Stroke::new(2.0)` round
//! caps and scales via `Affine::scale(chip_px / 24.0)`, identical to the
//! sibling `make_square` / `trim_transparency` glyphs on the same row.

use ph2d_vector::{BezPath, Point};

/// Build the Real Size glyph as a `BezPath` in a 24×24 design space
/// (Y-down, origin top-left). Placeholder: four L-shaped corner brackets
/// (top-left, top-right, bottom-right, bottom-left) suggesting "expand to
/// real size".
pub fn real_size_bezpath() -> BezPath {
    let mut p = BezPath::new();
    // Top-left bracket.
    p.move_to(Point::new(4.0, 9.0));
    p.line_to(Point::new(4.0, 4.0));
    p.line_to(Point::new(9.0, 4.0));
    // Top-right bracket.
    p.move_to(Point::new(15.0, 4.0));
    p.line_to(Point::new(20.0, 4.0));
    p.line_to(Point::new(20.0, 9.0));
    // Bottom-right bracket.
    p.move_to(Point::new(20.0, 15.0));
    p.line_to(Point::new(20.0, 20.0));
    p.line_to(Point::new(15.0, 20.0));
    // Bottom-left bracket.
    p.move_to(Point::new(9.0, 20.0));
    p.line_to(Point::new(4.0, 20.0));
    p.line_to(Point::new(4.0, 15.0));
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector::Shape;

    #[test]
    fn path_is_non_empty() {
        assert!(!real_size_bezpath().elements().is_empty());
    }

    #[test]
    fn path_fits_inside_24_grid() {
        let bb = real_size_bezpath().bounding_box();
        assert!(bb.x0 >= 0.0 && bb.y0 >= 0.0);
        assert!(bb.x1 <= 24.0 && bb.y1 <= 24.0);
    }
}
