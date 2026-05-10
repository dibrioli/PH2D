//! HSV color wheel disc + cursor crosshair painter.
//!
//! v1 paints a neutral disc plus 12 hue dots at the rim (cheap
//! approximation while Vello sweep gradients are wired in Phase 3 of
//! [`docs/plans/2026-05-color-picker-fix.md`]). The cursor crosshair
//! still tracks the picker's current color via the OKLCH chroma+hue.

use super::state::BlenderColorPicker;
use crate::zones::Rect;
use ph2d_tokens::ColorValue;
use ph2d_vector::{Affine, Brush, Circle, Color as VelloColor, Fill, Point, Stroke, VectorScene};

pub fn paint_color_wheel(cp: &BlenderColorPicker, rect: Rect, scene: &mut VectorScene) {
    let cx = rect.x + rect.w * 0.5;
    let cy = rect.y + rect.h * 0.5;
    let radius = (rect.w.min(rect.h)) * 0.5 - 2.0;
    let disc = Circle::new(Point::new(cx as f64, cy as f64), radius as f64);
    let disc_brush = Brush::Solid(VelloColor::from_rgba8(120, 120, 120, 255));
    scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, &disc_brush, None, &disc);

    let inner_r = radius * 0.78;
    for step in 0..12 {
        let h = step as f32 * 30.0;
        let theta = h.to_radians();
        let mid_r = (radius + inner_r) * 0.5;
        let px = cx + theta.cos() * mid_r;
        let py = cy + theta.sin() * mid_r;
        let dot = Circle::new(
            Point::new(px as f64, py as f64),
            ((radius - inner_r) * 0.5) as f64,
        );
        let cv = ColorValue::from_oklch(0.7, 0.18, h as f64, 1.0);
        let brush = Brush::Solid(VelloColor::from_rgba8(
            cv.rgba[0], cv.rgba[1], cv.rgba[2], 255,
        ));
        scene
            .inner_mut()
            .fill(Fill::NonZero, Affine::IDENTITY, &brush, None, &dot);
    }

    let (_, c, h, _) = cp.value.oklch;
    let normalized_c = (c / 0.4).clamp(0.0, 1.0) as f32;
    let theta = (h as f32).to_radians();
    let cur_x = cx + theta.cos() * radius * normalized_c;
    let cur_y = cy + theta.sin() * radius * normalized_c;
    let cursor_r: f64 = 6.0;
    let cursor = Circle::new(Point::new(cur_x as f64, cur_y as f64), cursor_r);
    let stroke = Stroke::new(2.0);
    scene.inner_mut().stroke(
        &stroke,
        Affine::IDENTITY,
        &Brush::Solid(VelloColor::WHITE),
        None,
        &cursor,
    );
}
