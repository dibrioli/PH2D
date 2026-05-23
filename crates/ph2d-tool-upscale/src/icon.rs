//! Tool icon for Upscale — a small framed image with an "out" arrow
//! pointing up-right, suggesting "scale up". Ported from
//! `docs/design/icons/upscale.svg` (Lucide `image-up`-style glyph) into
//! a `kurbo::BezPath` in a 24×24 viewBox (same space as the source
//! SVG), to be scaled by the caller via `Affine::scale(chip_size /
//! 24.0)` at paint time.
//!
//! Strokes only (no fills) like the rest of the Image Tools row. Arc
//! corners (`a 2 2 0 0 0 2 -2` in the SVG) become a single cubic Bézier
//! using the canonical quarter-circle κ ≈ 0.5523 control distance.

use ph2d_vector::{BezPath, Point};

/// Canonical kappa for approximating a quarter-circle arc with one
/// cubic Bézier (4·(√2 − 1) / 3 ≈ 0.5523). Off-curve control points sit
/// at `κ·radius` along the tangent from each arc endpoint.
const KAPPA: f64 = 0.5522847498307933;

/// Append an axis-aligned rectangle as its own closed subpath.
fn push_rect(p: &mut BezPath, x0: f64, y0: f64, x1: f64, y1: f64) {
    p.move_to(Point::new(x0, y0));
    p.line_to(Point::new(x1, y0));
    p.line_to(Point::new(x1, y1));
    p.line_to(Point::new(x0, y1));
    p.close_path();
}

/// Quarter-circle arc from the current pen position `(x0,y0)` to
/// `(x1,y1)` with radius `r`. `tx_start/ty_start` is the unit tangent
/// at the start (toward control1); `tx_end/ty_end` is the unit tangent
/// at the end (away from control2). Control distances are `κ · r`.
///
/// Caller must already have placed the pen at `(x0,y0)` via
/// `move_to` or `line_to` — this just appends the cubic.
///
/// 10 args is intentional: this is the canonical signature for an SVG
/// arc segment (two endpoints + radius + two tangent unit vectors),
/// and bundling them into a struct just to satisfy a lint would
/// make the four call-sites less readable.
#[allow(clippy::too_many_arguments)]
fn arc_quarter(
    p: &mut BezPath,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    r: f64,
    tx_start: f64,
    ty_start: f64,
    tx_end: f64,
    ty_end: f64,
) {
    let k = KAPPA * r;
    let c1 = Point::new(x0 + tx_start * k, y0 + ty_start * k);
    let c2 = Point::new(x1 - tx_end * k, y1 - ty_end * k);
    p.curve_to(c1, c2, Point::new(x1, y1));
}

/// Upscale glyph as a `BezPath` in a 24×24 design space.
///
/// Source SVG paths (Lucide-ported, `docs/design/icons/upscale.svg`):
/// ```svg
/// <path d="M16 3h5v5"/>
/// <path d="M17 21h2a2 2 0 0 0 2-2"/>
/// <path d="M21 12v3"/>
/// <path d="m21 3-5 5"/>
/// <path d="M3 7V5a2 2 0 0 1 2-2"/>
/// <path d="m5 21 4.144-4.144a1.21 1.21 0 0 1 1.712 0L13 19"/>
/// <path d="M9 3h3"/>
/// <rect x="3" y="11" width="10" height="10" rx="1"/>
/// ```
pub fn upscale_bezpath() -> BezPath {
    let mut p = BezPath::new();

    // ── Path 1: top-right arrow corner — M16 3 h5 v5
    p.move_to(Point::new(16.0, 3.0));
    p.line_to(Point::new(21.0, 3.0));
    p.line_to(Point::new(21.0, 8.0));

    // ── Path 2: bottom-right outer frame — M17 21 h2 a2 2 0 0 0 2 -2
    p.move_to(Point::new(17.0, 21.0));
    p.line_to(Point::new(19.0, 21.0));
    // Arc from (19,21) to (21,19), radius 2, sweep clockwise (corner
    // bending right-then-up). Start tangent +X, end tangent −Y.
    arc_quarter(&mut p, 19.0, 21.0, 21.0, 19.0, 2.0, 1.0, 0.0, 0.0, -1.0);

    // ── Path 3: right vertical — M21 12 v3
    p.move_to(Point::new(21.0, 12.0));
    p.line_to(Point::new(21.0, 15.0));

    // ── Path 4: diagonal arrow shaft — m21 3 -5 5
    p.move_to(Point::new(21.0, 3.0));
    p.line_to(Point::new(16.0, 8.0));

    // ── Path 5: top-left outer frame — M3 7 V5 a2 2 0 0 1 2 -2
    p.move_to(Point::new(3.0, 7.0));
    p.line_to(Point::new(3.0, 5.0));
    // Arc from (3,5) to (5,3), radius 2, sweep counter-clockwise.
    // Start tangent −Y, end tangent +X.
    arc_quarter(&mut p, 3.0, 5.0, 5.0, 3.0, 2.0, 0.0, -1.0, 1.0, 0.0);

    // ── Path 6: mountain inside the framed image —
    //         m5 21 4.144 -4.144 a1.21 1.21 0 0 1 1.712 0 L13 19
    p.move_to(Point::new(5.0, 21.0));
    p.line_to(Point::new(9.144, 16.856));
    // Tiny arc joining the two mountain peaks: from (9.144, 16.856) to
    // (9.144 + 1.712, 16.856) = (10.856, 16.856), radius 1.21,
    // counter-clockwise (the peak crests outward / upward).
    arc_quarter(
        &mut p, 9.144, 16.856, 10.856, 16.856, 1.21, 1.0, -1.0, 1.0, 1.0,
    );
    p.line_to(Point::new(13.0, 19.0));

    // ── Path 7: top horizontal stub — M9 3 h3
    p.move_to(Point::new(9.0, 3.0));
    p.line_to(Point::new(12.0, 3.0));

    // ── Path 8: bottom-left framed image rect — x=3 y=11 w=10 h=10
    // (`rx=1` is rendered as sharp here; the icon is small enough that
    // the radius would be invisible at 24px and the bezpath layer
    // doesn't expose rounded-rect natively).
    push_rect(&mut p, 3.0, 11.0, 13.0, 21.0);

    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector::Shape;

    #[test]
    fn path_is_non_empty() {
        assert!(!upscale_bezpath().elements().is_empty());
    }

    #[test]
    fn path_fits_inside_24_grid() {
        let bb = upscale_bezpath().bounding_box();
        // The kappa-approximated arcs can swing outside the exact SVG
        // endpoints by a fraction of a pixel; allow a small tolerance.
        const TOL: f64 = 0.1;
        assert!(bb.x0 >= -TOL && bb.y0 >= -TOL, "bbox top-left {bb:?}");
        assert!(
            bb.x1 <= 24.0 + TOL && bb.y1 <= 24.0 + TOL,
            "bbox bottom-right {bb:?}"
        );
    }
}
