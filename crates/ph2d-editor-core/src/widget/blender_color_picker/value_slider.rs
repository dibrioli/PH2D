//! Horizontal hue strip painter — rainbow gradient with a vertical
//! thumb marking the current H. Pairs with the SV rectangle painted
//! by [`super::wheel`]: the strip controls hue, the rectangle
//! controls saturation + value at that hue.

use super::state::BlenderColorPicker;
use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Theme};
use ph2d_vector::{
    Affine, Brush, Color as VelloColor, Fill, Gradient, Point, RoundedRect, VectorScene,
};

const HUE_STOPS: [VelloColor; 7] = [
    VelloColor::from_rgba8(255, 0, 0, 255),   // red
    VelloColor::from_rgba8(255, 255, 0, 255), // yellow
    VelloColor::from_rgba8(0, 255, 0, 255),   // green
    VelloColor::from_rgba8(0, 255, 255, 255), // cyan
    VelloColor::from_rgba8(0, 0, 255, 255),   // blue
    VelloColor::from_rgba8(255, 0, 255, 255), // magenta
    VelloColor::from_rgba8(255, 0, 0, 255),   // red (close)
];

pub fn paint_value_slider(
    cp: &BlenderColorPicker,
    rect: Rect,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let radius = Radius::Sm.px();
    // Filled with a rounded path (not a sharp `KurboRect`) so the
    // gradient terminates inside the rounded outline — earlier the
    // fill leaked past the stroke at the strip's corners as tiny
    // colored "ears".
    let body = RoundedRect::new(
        rect.x as f64,
        rect.y as f64,
        (rect.x + rect.w) as f64,
        (rect.y + rect.h) as f64,
        radius as f64,
    );
    let gradient = Gradient::new_linear(
        Point::new(rect.x as f64, rect.y as f64),
        Point::new((rect.x + rect.w) as f64, rect.y as f64),
    )
    .with_stops(HUE_STOPS);
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Gradient(gradient),
        None,
        &body,
    );
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    // Thumb: thin vertical pill at retained `hsv_h × w`. Reading
    // hue from the picker's anchor (rather than rgba_to_hsv) keeps
    // the thumb where the user dragged it even when the resulting
    // RGBA loses chromaticity (V→0, S→0) — and avoids the right-
    // edge "wrap to left" jump where 1.0 would round-trip to 0.0.
    //
    // Thumb stays inside the strip bounds: x clamped so no part
    // hangs off the left/right edges, y matches the strip height
    // (no overshoot top/bottom). Earlier versions extended ±2 px
    // beyond the strip — visible as tiny "ears" at the corners.
    let h = cp.hsv_h;
    let thumb_w = 4.0_f32;
    let thumb_x = (rect.x + h.clamp(0.0, 1.0) * rect.w - thumb_w * 0.5)
        .clamp(rect.x, rect.x + rect.w - thumb_w); // CLAMP-OK: rect-bound visual thumb clamp (bounds well-formed by construction)
    let thumb_rect = Rect::new(thumb_x, rect.y, thumb_w, rect.h);
    fill_rounded_rect(scene, thumb_rect, 2.0, resolve(ColorToken::Text1, theme));
    stroke_rounded_rect(scene, thumb_rect, 2.0, 1.0, resolve(ColorToken::Bg0, theme));
}

/// Map a click at `(px, _py)` inside the hue strip's `rect` to a
/// hue in `[0..1]` (left = 0 / red, right = 1 / red wrap-around).
pub fn value_pick(rect: Rect, px: f32) -> f32 {
    if rect.w <= 0.0 {
        return 0.0;
    }
    ((px - rect.x) / rect.w).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_left_edge_is_hue_zero() {
        let r = Rect::new(0.0, 0.0, 200.0, 16.0);
        assert!(value_pick(r, 0.0).abs() < 1e-3);
    }

    #[test]
    fn pick_right_edge_is_hue_one() {
        let r = Rect::new(0.0, 0.0, 200.0, 16.0);
        assert!((value_pick(r, 200.0) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn pick_middle_is_half_hue() {
        let r = Rect::new(0.0, 0.0, 200.0, 16.0);
        assert!((value_pick(r, 100.0) - 0.5).abs() < 1e-3);
    }
}
