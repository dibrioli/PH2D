//! Real Size icon — `BezPath` glyph for the Image Tools chrome pill.
//!
//! Port of Lucide [`maximize`](https://lucide.dev/icons/maximize) (24×24,
//! Y-down, origin top-left): four L-shaped corner brackets with rounded
//! `r = 2` corners, reading as "expand to real size". The canonical SVG
//! source lives at `docs/design/icons/real-size.svg` for the design
//! pipeline; this `BezPath` mirrors it 1:1.
//!
//! Each rounded corner approximates Lucide's quarter-circle arc with a
//! single cubic Bézier (κ·r where κ = 0.552_284_75, so the control offset
//! is ≈ 1.104_57). Stroked, not filled — chrome draws it with
//! `Stroke::new(2.0)` round caps and scales via
//! `Affine::scale(chip_px / 24.0)`, identical to the sibling
//! `make_square` / `trim_transparency` glyphs on the same row.

use ph2d_vector::{BezPath, Point};

/// Cubic control offset for a quarter-circle of radius 2 (κ · r).
const KR: f64 = 1.104_569_5;

/// Build the Real Size glyph as a `BezPath` in a 24×24 design space
/// (Y-down, origin top-left): four rounded corner brackets — the Lucide
/// `maximize` frame.
pub fn real_size_bezpath() -> BezPath {
    let mut p = BezPath::new();

    // Top-left bracket: down-left from (8,3), round the (3,3) corner, to (3,8).
    p.move_to(Point::new(8.0, 3.0));
    p.line_to(Point::new(5.0, 3.0));
    p.curve_to(
        Point::new(5.0 - KR, 3.0),
        Point::new(3.0, 3.0 + KR),
        Point::new(3.0, 5.0),
    );
    p.line_to(Point::new(3.0, 8.0));

    // Top-right bracket: up from (21,8), round the (21,3) corner, to (16,3).
    p.move_to(Point::new(21.0, 8.0));
    p.line_to(Point::new(21.0, 5.0));
    p.curve_to(
        Point::new(21.0, 5.0 - KR),
        Point::new(19.0 + KR, 3.0),
        Point::new(19.0, 3.0),
    );
    p.line_to(Point::new(16.0, 3.0));

    // Bottom-left bracket: down from (3,16), round the (3,21) corner, to (8,21).
    p.move_to(Point::new(3.0, 16.0));
    p.line_to(Point::new(3.0, 19.0));
    p.curve_to(
        Point::new(3.0, 19.0 + KR),
        Point::new(5.0 - KR, 21.0),
        Point::new(5.0, 21.0),
    );
    p.line_to(Point::new(8.0, 21.0));

    // Bottom-right bracket: right from (16,21), round the (21,21) corner, to (21,16).
    p.move_to(Point::new(16.0, 21.0));
    p.line_to(Point::new(19.0, 21.0));
    p.curve_to(
        Point::new(19.0 + KR, 21.0),
        Point::new(21.0, 19.0 + KR),
        Point::new(21.0, 19.0),
    );
    p.line_to(Point::new(21.0, 16.0));

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
