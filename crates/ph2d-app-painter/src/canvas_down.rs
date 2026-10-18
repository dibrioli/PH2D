//! **Onde um Down no canvas do Painter abre um traço** — a lei pura do gesto. Saiu de
//! `shells/desktop/src/input_dispatch/painter_canvas_input.rs` (P46, a caneta): a shell só compõe.

/// Whether a canvas Down at image fraction `t` (u or v) is inside the acceptance region on that axis:
/// the footprint `[0,1]`, extended to the 3×3 neighbour tiles when Repeat-Image tiling is on for the
/// axis, and by `margin` (uv units) on both sides while a Deform Transform gizmo is live — its corner
/// handles sit exactly ON the footprint edge, so half their clickable disc lies outside `[0,1]`.
#[must_use]
pub fn canvas_down_accepts(t: f32, repeat_tiled: bool, margin: f32) -> bool {
    if repeat_tiled {
        (-1.0..2.0).contains(&t)
    } else {
        (-margin..=1.0 + margin).contains(&t)
    }
}

#[cfg(test)]
mod tests {
    use super::canvas_down_accepts;

    #[test]
    fn transform_corner_margin_extends_the_down_gate() {
        // P2 (Enio 2026-07-04): a whole-image Transform puts the gizmo's corner squares exactly on the
        // footprint edge — a Down 4 image-px OUTSIDE (u slightly < 0 or > 1) must still reach the painter
        // (the deform gizmo) instead of falling through to the sprite gizmo on the same screen corner.
        let margin = 12.0 / 2048.0; // a 12 image-px grab tol on a 2048 canvas
        assert!(canvas_down_accepts(-4.0 / 2048.0, false, margin));
        assert!(canvas_down_accepts(1.0 + 4.0 / 2048.0, false, margin));
        // Beyond the margin still falls through (pan / sprite selection keep working).
        assert!(!canvas_down_accepts(-2.0 * margin, false, margin));
        assert!(!canvas_down_accepts(1.0 + 2.0 * margin, false, margin));
        // No live transform (margin 0) → the plain footprint gate, boundary inclusive.
        assert!(canvas_down_accepts(0.0, false, 0.0) && canvas_down_accepts(1.0, false, 0.0));
        assert!(!canvas_down_accepts(-0.001, false, 0.0));
        // Repeat-Image tiling keeps its 3×3 region regardless of the margin.
        assert!(canvas_down_accepts(-0.9, true, 0.0) && !canvas_down_accepts(-1.1, true, 0.0));
    }
}
