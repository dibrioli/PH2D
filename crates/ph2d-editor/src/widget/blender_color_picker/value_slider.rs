//! Vertical value (lightness) slider painter — gradient from the
//! current hue+sat at full lightness down to black, with a thumb
//! handle marking the current OKLCH `L`.

use super::state::BlenderColorPicker;
use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, ColorValue, Radius, Theme};
use ph2d_vector::{
    Affine, Brush, Color as VelloColor, Fill, Gradient, Point, Rect as KurboRect, VectorScene,
};

pub fn paint_value_slider(
    cp: &BlenderColorPicker,
    rect: Rect,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let radius = Radius::Sm.px();
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    // Bright endpoint: keep current hue + chroma but force L=1.0.
    let (_, c, h, _) = cp.value.oklch;
    let bright = ColorValue::from_oklch(1.0, c, h, 1.0);
    let dark = ColorValue::from_oklch(0.0, 0.0, 0.0, 1.0);
    let top_color = VelloColor::from_rgba8(bright.rgba[0], bright.rgba[1], bright.rgba[2], 255);
    let bot_color = VelloColor::from_rgba8(dark.rgba[0], dark.rgba[1], dark.rgba[2], 255);

    // Linear gradient top→bottom (high-L → low-L).
    let gradient = Gradient::new_linear(
        Point::new(rect.x as f64, rect.y as f64),
        Point::new(rect.x as f64, (rect.y + rect.h) as f64),
    )
    .with_stops([top_color, bot_color]);
    let body = KurboRect::new(
        rect.x as f64,
        rect.y as f64,
        (rect.x + rect.w) as f64,
        (rect.y + rect.h) as f64,
    );
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Gradient(gradient),
        None,
        &body,
    );

    // Thumb: thin horizontal pill marking current L (top = L=1, bot = L=0).
    let l = cp.value.oklch.0 as f32;
    let thumb_h = 4.0_f32;
    let thumb_y = rect.y + (1.0 - l).clamp(0.0, 1.0) * (rect.h - thumb_h);
    let thumb_rect = Rect::new(rect.x - 2.0, thumb_y, rect.w + 4.0, thumb_h);
    fill_rounded_rect(scene, thumb_rect, 2.0, resolve(ColorToken::Text1, theme));
    stroke_rounded_rect(scene, thumb_rect, 2.0, 1.0, resolve(ColorToken::Bg0, theme));
}

/// Map a click `py` inside the value slider's `rect` to an OKLCH
/// lightness in `[0..1]` (top = 1.0, bottom = 0.0).
pub fn value_pick(rect: Rect, py: f32) -> f32 {
    if rect.h <= 0.0 {
        return 0.0;
    }
    (1.0 - (py - rect.y) / rect.h).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_top_is_full_lightness() {
        let r = Rect::new(0.0, 0.0, 24.0, 200.0);
        assert!((value_pick(r, 0.0) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn pick_bottom_is_zero_lightness() {
        let r = Rect::new(0.0, 0.0, 24.0, 200.0);
        assert!(value_pick(r, 200.0).abs() < 1e-3);
    }

    #[test]
    fn pick_middle_is_half_lightness() {
        let r = Rect::new(0.0, 0.0, 24.0, 200.0);
        assert!((value_pick(r, 100.0) - 0.5).abs() < 1e-3);
    }
}
