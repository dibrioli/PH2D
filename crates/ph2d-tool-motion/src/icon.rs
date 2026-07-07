//! Tool icon for the Motion Nodes tool — a three-node graph as a stroked
//! `kurbo::BezPath` in a 24×24 viewBox (matches `docs/design/icons/motion-nodes.svg`).
//! Caller scales by `Affine::scale(chip_size / 24.0)` at paint time; the pill
//! painter strokes it (`paint_icon_path`), so this is an outline, not a fill.
//!
//! Two source nodes (left) feed one sink (right) — the canonical "node graph"
//! silhouette, distinct from the other topbar glyphs.

use ph2d_vector::{BezPath, Point};

/// Cubic-bezier circle approximation constant (4-arc unit circle). Baked so the
/// icon is transcendental-free (HR-5): no `sin`/`cos` at build or paint.
const KAPPA: f64 = 0.552_284_75;

/// Append a circle (centre `c`, radius `r`) to `p` as four cubic beziers.
fn push_circle(p: &mut BezPath, cx: f64, cy: f64, r: f64) {
    let k = KAPPA * r;
    p.move_to(Point::new(cx + r, cy));
    p.curve_to(
        Point::new(cx + r, cy + k),
        Point::new(cx + k, cy + r),
        Point::new(cx, cy + r),
    );
    p.curve_to(
        Point::new(cx - k, cy + r),
        Point::new(cx - r, cy + k),
        Point::new(cx - r, cy),
    );
    p.curve_to(
        Point::new(cx - r, cy - k),
        Point::new(cx - k, cy - r),
        Point::new(cx, cy - r),
    );
    p.curve_to(
        Point::new(cx + k, cy - r),
        Point::new(cx + r, cy - k),
        Point::new(cx + r, cy),
    );
    p.close_path();
}

/// Motion Nodes tool icon as a `BezPath` in a 24×24 coordinate space.
#[must_use]
pub fn motion_bezpath() -> BezPath {
    let mut p = BezPath::new();
    // Three nodes: two sources on the left, one sink on the right.
    push_circle(&mut p, 6.0, 7.0, 2.5);
    push_circle(&mut p, 6.0, 17.0, 2.5);
    push_circle(&mut p, 18.0, 12.0, 2.5);
    // Two connecting edges (source → sink).
    p.move_to(Point::new(8.5, 8.0));
    p.line_to(Point::new(15.5, 11.0));
    p.move_to(Point::new(8.5, 16.0));
    p.line_to(Point::new(15.5, 13.0));
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector::Shape;

    #[test]
    fn motion_bezpath_fits_in_24x24_viewbox() {
        let bb = motion_bezpath().bounding_box();
        assert!(
            bb.x0 >= 0.0 && bb.y0 >= 0.0 && bb.x1 <= 24.0 && bb.y1 <= 24.0,
            "icon bbox {bb:?} outside 24x24 viewBox"
        );
    }

    #[test]
    fn motion_bezpath_has_three_nodes_and_two_edges() {
        // 3 circles (MoveTo + 4 CurveTo + ClosePath = 6 els each = 18) + 2 edges
        // (MoveTo + LineTo = 2 els each = 4) = 22 elements.
        assert_eq!(motion_bezpath().elements().len(), 22);
    }
}
