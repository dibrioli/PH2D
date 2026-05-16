//! Make Square icon — Lucide `square` glyph as a `BezPath`.
//!
//! Port of Lucide's `square.svg` (MIT-licensed, ISC-compatible). One
//! closed rounded-rectangle subpath, drawn in a 24×24 grid with Y-down
//! (SVG convention — matches Vello's coordinate system; see
//! `ph2d-render::projection` for the world↔screen flip).
//!
//! Lucide source (`square.svg`):
//! ```svg
//! <rect width="18" height="18" x="3" y="3" rx="2" />
//! ```
//!
//! The returned path is stroked, not filled — the Coordenador should
//! draw it with `Stroke::new(2.0)` and `Stroke::with_caps(Round, Round)`
//! to match Lucide's `stroke-linecap="round"` styling, then scale via
//! `Affine::scale(chip_px / 24.0)` to fit any chip size. Visually
//! consistent with the `crop` glyph used by `trim_transparency::icon`
//! immediately to its left on the Image Tools row.

use ph2d_vector::{BezPath, RoundedRect, Shape};

/// Build the square glyph as a `BezPath` in a 24×24 design space
/// (Y-down, origin top-left — matches SVG/Vello).
///
/// One closed subpath: rounded rect from `(3, 3)` to `(21, 21)` with
/// corner radius 2, generated via `kurbo::RoundedRect::to_path` so the
/// corner Bézier approximation tracks kurbo's canonical tolerance
/// model. Tolerance of `0.1` is sub-pixel at the 24-unit design scale.
pub fn square_bezpath() -> BezPath {
    RoundedRect::new(3.0, 3.0, 21.0, 21.0, 2.0).to_path(0.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector::Shape;

    #[test]
    fn path_is_non_empty() {
        let p = square_bezpath();
        assert!(!p.elements().is_empty());
    }

    #[test]
    fn path_bounding_box_matches_24_grid_rect() {
        // The rounded rect spans (3, 3) to (21, 21). The BezPath's
        // bounding box cannot exceed those edges because every Bézier
        // segment is contained inside the analytical rect.
        let bb = square_bezpath().bounding_box();
        let eps = 0.05;
        assert!((bb.x0 - 3.0).abs() < eps, "x0 = {}", bb.x0);
        assert!((bb.y0 - 3.0).abs() < eps, "y0 = {}", bb.y0);
        assert!((bb.x1 - 21.0).abs() < eps, "x1 = {}", bb.x1);
        assert!((bb.y1 - 21.0).abs() < eps, "y1 = {}", bb.y1);
    }

    #[test]
    fn path_serializes_to_a_single_subpath_with_one_moveto() {
        // RoundedRect::to_path emits one MoveTo + arcs/lines + close.
        // We only assert there is exactly one MoveTo so the shape
        // remains a single closed subpath; the internal segmentation
        // is a kurbo-version detail and not load-bearing here.
        let s = square_bezpath().to_svg();
        let m_count = s.matches('M').count();
        assert_eq!(m_count, 1, "expected exactly one MoveTo; got {s:?}");
    }
}
