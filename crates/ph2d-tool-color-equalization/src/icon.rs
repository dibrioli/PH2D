//! Color Equalization icon — `BezPath` glyph for the Image Tools chrome pill.
//!
//! Lucide `sliders-horizontal` portado para `BezPath` em 24×24, mantido
//! byte-a-byte em sintonia com `docs/design/icons/color-equalization.svg`
//! (origem do design pipeline). Nove sub-paths abertos compõem três fileiras
//! horizontais com seus "knobs" verticais — leitura: três sliders empilhados,
//! que é exatamente este tool (faders de ajuste de cor).
//!
//! Original SVG paths (Coord, `ccf0cf0`):
//! ```svg
//! <path d="M10 5H3"/>
//! <path d="M12 19H3"/>
//! <path d="M14 3v4"/>
//! <path d="M16 17v4"/>
//! <path d="M21 12h-9"/>
//! <path d="M21 19h-5"/>
//! <path d="M21 5h-7"/>
//! <path d="M8 10v4"/>
//! <path d="M8 12H3"/>
//! ```

use ph2d_vector::{BezPath, Point};

/// Add a straight-line segment as its own open sub-path (`move_to` +
/// `line_to`), so the resulting `BezPath` is the same composite of
/// disjoint strokes that the source SVG describes.
fn push_line(p: &mut BezPath, x0: f64, y0: f64, x1: f64, y1: f64) {
    p.move_to(Point::new(x0, y0));
    p.line_to(Point::new(x1, y1));
}

/// Build the Color Equalization glyph as a `BezPath` in a 24×24 design
/// space (Y-down, origin top-left). Nine open line segments matching the
/// source SVG.
pub fn color_equalization_bezpath() -> BezPath {
    let mut p = BezPath::new();
    // Row 1 — top slider track (y=5).
    push_line(&mut p, 10.0, 5.0, 3.0, 5.0);
    // Row 2 — middle slider track (y=12, right portion).
    push_line(&mut p, 21.0, 12.0, 12.0, 12.0);
    // Row 3 — bottom slider track (y=19, right portion).
    push_line(&mut p, 12.0, 19.0, 3.0, 19.0);
    // Knob handles, top to bottom.
    push_line(&mut p, 14.0, 3.0, 14.0, 7.0); // knob over top row
    push_line(&mut p, 8.0, 10.0, 8.0, 14.0); // knob over middle row
    push_line(&mut p, 16.0, 17.0, 16.0, 21.0); // knob over bottom row
    // Far-right track extensions (each row continues past its knob).
    push_line(&mut p, 21.0, 5.0, 14.0, 5.0);
    push_line(&mut p, 8.0, 12.0, 3.0, 12.0);
    push_line(&mut p, 21.0, 19.0, 16.0, 19.0);
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector::Shape;

    #[test]
    fn path_is_non_empty() {
        assert!(!color_equalization_bezpath().elements().is_empty());
    }

    #[test]
    fn path_fits_inside_24_grid() {
        let bb = color_equalization_bezpath().bounding_box();
        assert!(bb.x0 >= 0.0 && bb.y0 >= 0.0);
        assert!(bb.x1 <= 24.0 && bb.y1 <= 24.0);
    }

    #[test]
    fn has_nine_open_subpaths() {
        // 9 `move_to` + 9 `line_to` = 18 elements; no `close_path`.
        let p = color_equalization_bezpath();
        assert_eq!(p.elements().len(), 18);
    }
}
