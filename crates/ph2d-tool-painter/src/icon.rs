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
/// SVG paths (Lucide v0.453, absolute coords):
///
/// ```svg
/// <path d="M5 12V4a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v8"/>
/// <path d="M14 19.9V16h3a2 2 0 0 0 2-2v-2H5v2c0 1.1.9 2 2 2h3v3.9
///          a2 2 0 1 0 4 0Z"/>
/// <path d="M8 2v4"/>
/// <path d="M16 2v4"/>
/// ```
///
/// SVG `a rx ry x-axis-rotation large-arc-flag sweep-flag x y` segments
/// translate to cubic Bézier via the kappa approximation (control distance
/// `c = 0.5522847498` for a 90° arc; for r=2 → cd = 1.1046). Each `a 2 2 0 0 1`
/// quarter-arc with start (x0,y0) and end (x1,y1) becomes:
/// `curve_to((x0 + dx*c, y0 + dy*c), (x1 - ex*c, y1 - ey*c), (x1, y1))`
/// where (dx,dy) is the unit tangent at start and (ex,ey) the unit tangent
/// at end. For sweep-flag=1 (clockwise as drawn) on a top-left corner
/// (e.g., bristle pad TL), the path goes right-then-down.
pub fn painter_bezpath() -> BezPath {
    use ph2d_vector::Point;
    let mut p = BezPath::new();

    // Lucide arc kappa constant for r=2 quarter-arcs.
    // c = (4/3) * tan(π/8) ≈ 0.5522847498. For r=2, control offset = 1.1046.
    const C2: f64 = 1.104_569_499_661_667;

    // ── Path 1 (bristle pad top): "M5 12 V4 a2 2 0 0 1 2-2 H17
    //                              a2 2 0 0 1 2 2 V12"
    // Sequence: line up-left, arc TL corner, line right, arc TR corner, line down.
    p.move_to(Point::new(5.0, 12.0));
    p.line_to(Point::new(5.0, 4.0));
    // Arc TL: (5,4) → (7,2). Sweep-flag=1, r=2. Tangent at start = (0,-1);
    // tangent at end = (1,0). Cubic: control1=(5, 4-C2), control2=(7-C2, 2).
    p.curve_to(
        Point::new(5.0, 4.0 - C2),
        Point::new(7.0 - C2, 2.0),
        Point::new(7.0, 2.0),
    );
    p.line_to(Point::new(17.0, 2.0));
    // Arc TR: (17,2) → (19,4). Sweep-flag=1, r=2. Tangent start = (1,0);
    // tangent end = (0,1). Cubic: control1=(17+C2, 2), control2=(19, 4-C2).
    p.curve_to(
        Point::new(17.0 + C2, 2.0),
        Point::new(19.0, 4.0 - C2),
        Point::new(19.0, 4.0),
    );
    p.line_to(Point::new(19.0, 12.0));

    // ── Path 2 (handle body): "M14 19.9 V16 H17 a2 2 0 0 0 2-2 V12
    //                          H5 V14 c0 1.1 .9 2 2 2 H10 V19.9
    //                          a2 2 0 1 0 4 0 Z"
    p.move_to(Point::new(14.0, 19.9));
    p.line_to(Point::new(14.0, 16.0));
    p.line_to(Point::new(17.0, 16.0));
    // Arc CCW (sweep=0): (17,16) → (19,14). Tangent start = (1,0);
    // tangent end = (0,-1). CCW means going up-right from (17,16).
    p.curve_to(
        Point::new(17.0 + C2, 16.0),
        Point::new(19.0, 14.0 + C2),
        Point::new(19.0, 14.0),
    );
    p.line_to(Point::new(19.0, 12.0));
    p.line_to(Point::new(5.0, 12.0));
    p.line_to(Point::new(5.0, 14.0));
    // SVG: c0 1.1 .9 2 2 2 — relative cubic from (5,14) to (5+2, 14+2) = (7,16).
    // Controls relative: (5+0, 14+1.1) = (5, 15.1); (5+0.9, 14+2) = (5.9, 16).
    p.curve_to(
        Point::new(5.0, 15.1),
        Point::new(5.9, 16.0),
        Point::new(7.0, 16.0),
    );
    p.line_to(Point::new(10.0, 16.0));
    p.line_to(Point::new(10.0, 19.9));
    // Arc large-arc=1 sweep=0: (10,19.9) → (14,19.9). Path goes down then back
    // up, forming a "tab" (the user-grip rounded bottom). Approximate via 2
    // cubics joined at (12, 21.9). Tangents: start (0,1) down → meeting tangent
    // (1,0) right → end tangent (0,-1) up.
    p.curve_to(
        Point::new(10.0, 19.9 + C2),
        Point::new(12.0 - C2, 21.9),
        Point::new(12.0, 21.9),
    );
    p.curve_to(
        Point::new(12.0 + C2, 21.9),
        Point::new(14.0, 19.9 + C2),
        Point::new(14.0, 19.9),
    );
    p.close_path();

    // ── Paths 3+4 (ferrule rivets): two vertical strokes M8 2 V6, M16 2 V6.
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
        assert!(!p.elements().is_empty(), "icon path must have segments");
    }

    #[test]
    fn painter_bezpath_has_curves_not_just_lines() {
        // Audit 2026-05-26 (F9): regression guard. Comment "mirror of SVG"
        // exige que o BezPath ESPELHA o SVG Lucide; SVG tem arcs (TL,TR
        // do bristle pad + arc CCW do handle + arc bottom rounded + cubic
        // do c-relative). Se alguém regredir para linhas retas
        // (simplificação), este test pega: pelo menos 4 CurveTo elements.
        //
        // PathEl não é re-exportado por ph2d_vector; uso Debug match
        // textual (BezPath::elements() Debug renderiza CurveTo(..)).
        let p = painter_bezpath();
        let dbg = format!("{:?}", p.elements());
        let curve_count = dbg.matches("CurveTo(").count();
        assert!(
            curve_count >= 4,
            "painter_bezpath must mirror SVG Lucide arcs: expected ≥4 CurveTo, got {}",
            curve_count
        );
    }
}
