//! Tool icon for Painter — Lucide `paintbrush-2` glyph as a `BezPath` in
//! a 24×24 viewBox (mirror of `docs/design/icons/painter.svg`).
//!
//! Caller scales via `Affine::scale(chip_size / 24.0)` at paint time.

use ph2d_vector::BezPath;

/// Lucide `paintbrush-2` icon as a `BezPath` in a 24×24 coordinate space.
///
/// Source: <https://lucide.dev/icons/paintbrush-2> (ISC license, embeddable).
/// Mirrors `docs/design/icons/painter.svg`. Four sub-paths combined into
/// a single `BezPath` via `move_to` breaks — caller doesn't need to know
/// it is composite.
///
/// Path semantics:
/// - **Bristle pad top:** `M5 12 V4 a2 2 0 0 1 2-2 h10 a2 2 0 0 1 2 2 V12`
/// - **Handle body:** `M14 19.9 V16 H17 a2 2 0 0 0 2-2 V12 H5 V14 c0 1.1 .9 2 2 2 H10 V19.9 a2 2 0 1 0 4 0 Z`
/// - **Ferrule rivets:** `M8 2 V6` and `M16 2 V6` (vertical strokes)
///
/// Para T1.1 stub: implementação placeholder simples (retângulo do bristle pad
/// + linha do handle). W1 T-icon-polish fine-tunes os arcs/curves.
pub fn painter_bezpath() -> BezPath {
    use ph2d_vector::Point;
    let mut p = BezPath::new();

    // ── Bristle pad: rectangle (5,2)..(19,12) (sem arcs por enquanto; T-polish)
    p.move_to(Point::new(5.0, 12.0));
    p.line_to(Point::new(5.0, 4.0));
    p.line_to(Point::new(7.0, 2.0)); // approx top-left round corner
    p.line_to(Point::new(17.0, 2.0));
    p.line_to(Point::new(19.0, 4.0)); // approx top-right round corner
    p.line_to(Point::new(19.0, 12.0));

    // ── Handle body: vertical strip from bristle bottom down to (14,21)
    p.move_to(Point::new(14.0, 19.9));
    p.line_to(Point::new(14.0, 16.0));
    p.line_to(Point::new(17.0, 16.0));
    p.line_to(Point::new(19.0, 14.0)); // approx
    p.line_to(Point::new(19.0, 12.0));
    p.line_to(Point::new(5.0, 12.0));
    p.line_to(Point::new(5.0, 14.0));
    p.line_to(Point::new(7.0, 16.0)); // approx
    p.line_to(Point::new(10.0, 16.0));
    p.line_to(Point::new(10.0, 19.9));
    p.close_path();

    // ── Ferrule rivets (vertical strokes M8 2 V6, M16 2 V6)
    p.move_to(Point::new(8.0, 2.0));
    p.line_to(Point::new(8.0, 6.0));
    p.move_to(Point::new(16.0, 2.0));
    p.line_to(Point::new(16.0, 6.0));

    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn painter_bezpath_is_non_empty() {
        let p = painter_bezpath();
        // BezPath::elements() exposes the underlying Vec<PathEl>.
        assert!(!p.elements().is_empty(), "icon path must have segments");
    }
}
