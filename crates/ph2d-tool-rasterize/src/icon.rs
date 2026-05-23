//! Rasterize icon — `BezPath` glyph for the Image Tools chrome pill.
//!
//! Port of the canonical SVG at `docs/design/icons/rasterize.svg`
//! (Lucide-style, 24×24, Y-down origin top-left). The SVG source is:
//!
//! ```svg
//! <path d="M12 3v18"/>
//! <path d="M3 12h18"/>
//! <rect x="3" y="3" width="18" height="18" rx="2"/>
//! ```
//!
//! Visual semantics: a rounded square subdivided by a centered cross —
//! reads as "rasterize / commit to the pixel grid". Three subpaths
//! co-existing inside a single `BezPath` (the chrome stroke pass walks
//! every subpath uniformly). Stroked, not filled — chrome draws it with
//! `Stroke::new(2.0)` round caps and scales via
//! `Affine::scale(chip_px / 24.0)`, identical to the sibling Image
//! Tools glyphs.

use ph2d_vector::{BezPath, Point, RoundedRect, Shape};

/// Build the Rasterize glyph as a `BezPath` in a 24×24 design space
/// (Y-down, origin top-left).
///
/// Three subpaths in one `BezPath`:
///   1. The outer rounded rect from `(3, 3)` to `(21, 21)` with corner
///      radius 2, via `kurbo::RoundedRect::to_path` (tolerance `0.1`
///      = sub-pixel at the 24-unit scale).
///   2. Vertical bar: `move_to(12, 3) → line_to(12, 21)`.
///   3. Horizontal bar: `move_to(3, 12) → line_to(21, 12)`.
pub fn rasterize_bezpath() -> BezPath {
    let mut p = RoundedRect::new(3.0, 3.0, 21.0, 21.0, 2.0).to_path(0.1);

    // Vertical bar through the centre.
    p.move_to(Point::new(12.0, 3.0));
    p.line_to(Point::new(12.0, 21.0));

    // Horizontal bar through the centre.
    p.move_to(Point::new(3.0, 12.0));
    p.line_to(Point::new(21.0, 12.0));

    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector::Shape;

    #[test]
    fn path_is_non_empty() {
        assert!(!rasterize_bezpath().elements().is_empty());
    }

    #[test]
    fn path_fits_inside_24_grid() {
        // Every segment lives within (3, 3) → (21, 21); the bounding
        // box must therefore stay inside the 24×24 design space.
        let bb = rasterize_bezpath().bounding_box();
        let eps = 0.05;
        assert!(bb.x0 >= 3.0 - eps && bb.y0 >= 3.0 - eps);
        assert!(bb.x1 <= 21.0 + eps && bb.y1 <= 21.0 + eps);
    }

    #[test]
    fn path_has_three_subpaths() {
        // The chrome stroke pass treats every `M` (MoveTo) as a fresh
        // subpath; the glyph must serialize with exactly three —
        // outer rect (one `M` from RoundedRect::to_path) + vertical bar
        // + horizontal bar.
        let s = rasterize_bezpath().to_svg();
        assert_eq!(
            s.matches('M').count(),
            3,
            "expected exactly three MoveTo subpaths; got {s:?}",
        );
    }
}
