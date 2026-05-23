//! Equalize Sizes icon — `BezPath` glyph for the Image Tools chrome pill.
//!
//! Lucide-derived "align horizontal" mark: a rounded outer canvas frame
//! `(3,3)-(21,21)` with two horizontal rails at `y = 10` and `y = 14`
//! (the lines the algorithm normalizes against). The source of truth for
//! the design pipeline is `docs/design/icons/equalize-sizes.svg`, kept
//! byte-for-byte in sync with the path commands below.

use ph2d_vector::{BezPath, Point};

/// Append an axis-aligned rounded rectangle (4 quarter-cubics, matching
/// the icon-system convention used by sibling glyphs that need a rounded
/// frame). The radius is fixed (Lucide default `rx = 2`).
fn push_rrect(p: &mut BezPath, x0: f64, y0: f64, x1: f64, y1: f64, r: f64) {
    let r = r.min((x1 - x0) * 0.5).min((y1 - y0) * 0.5).max(0.0);
    // Move to start of top edge (just right of the top-left corner).
    p.move_to(Point::new(x0 + r, y0));
    // Top edge → top-right corner.
    p.line_to(Point::new(x1 - r, y0));
    p.quad_to(Point::new(x1, y0), Point::new(x1, y0 + r));
    // Right edge → bottom-right corner.
    p.line_to(Point::new(x1, y1 - r));
    p.quad_to(Point::new(x1, y1), Point::new(x1 - r, y1));
    // Bottom edge → bottom-left corner.
    p.line_to(Point::new(x0 + r, y1));
    p.quad_to(Point::new(x0, y1), Point::new(x0, y1 - r));
    // Left edge → top-left corner.
    p.line_to(Point::new(x0, y0 + r));
    p.quad_to(Point::new(x0, y0), Point::new(x0 + r, y0));
    p.close_path();
}

/// Append a straight horizontal line as its own subpath.
fn push_hline(p: &mut BezPath, x0: f64, x1: f64, y: f64) {
    p.move_to(Point::new(x0, y));
    p.line_to(Point::new(x1, y));
}

/// Build the Equalize Sizes glyph as a `BezPath` in a 24×24 design space
/// (Y-down, origin top-left): a rounded outer frame `(3,3)-(21,21)` plus
/// two horizontal rails at `y = 10` and `y = 14`. Mirrors
/// `docs/design/icons/equalize-sizes.svg`.
pub fn equalize_sizes_bezpath() -> BezPath {
    let mut p = BezPath::new();
    push_rrect(&mut p, 3.0, 3.0, 21.0, 21.0, 2.0);
    push_hline(&mut p, 7.0, 17.0, 10.0);
    push_hline(&mut p, 7.0, 17.0, 14.0);
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector::Shape;

    #[test]
    fn path_is_non_empty() {
        assert!(!equalize_sizes_bezpath().elements().is_empty());
    }

    #[test]
    fn path_fits_inside_24_grid() {
        let bb = equalize_sizes_bezpath().bounding_box();
        assert!(bb.x0 >= 0.0 && bb.y0 >= 0.0);
        assert!(bb.x1 <= 24.0 && bb.y1 <= 24.0);
    }
}
