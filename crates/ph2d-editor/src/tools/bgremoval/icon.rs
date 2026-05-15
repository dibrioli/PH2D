//! Tool icon for Background Removal — Lucide `eraser` glyph as
//! `kurbo::BezPath` in a 24×24 viewBox (same space as the source
//! SVG), to be scaled by the caller via `Affine::scale(chip_size /
//! 24.0)` at paint time.
//!
//! Mirror of [`crate::tools::trim_transparency::icon`] which provides
//! `crop_bezpath()` the same way.

use ph2d_vector::BezPath;

/// Lucide `eraser` icon as a `BezPath` in a 24×24 coordinate space.
///
/// Source: <https://lucide.dev/icons/eraser> (ISC-0 license, embeddable).
/// Two paths combined into a single `BezPath` by `move_to`-segments —
/// caller does not need to know it is composite.
///
/// TODO(M1-final): swap stub for the actual ported path. The stub
/// returns a placeholder square so the integration smoke-test can
/// still paint *something*.
pub fn eraser_bezpath() -> BezPath {
    use ph2d_vector::Point;
    let mut p = BezPath::new();
    // Placeholder: 16×16 centred square inside the 24×24 viewBox.
    // Real eraser glyph lands when M1 + audit close.
    p.move_to(Point::new(4.0, 4.0));
    p.line_to(Point::new(20.0, 4.0));
    p.line_to(Point::new(20.0, 20.0));
    p.line_to(Point::new(4.0, 20.0));
    p.close_path();
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eraser_bezpath_is_nonempty() {
        let bp = eraser_bezpath();
        assert!(bp.elements().len() >= 4);
    }
}
