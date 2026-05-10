//! Saturation × Value rectangle picker (web-standard layout).
//!
//! The rectangle is filled in three layers:
//!   1. Solid hue color at full S+V (the "purest" version of the
//!      currently selected hue).
//!   2. Horizontal gradient: white at the left (S=0) → transparent
//!      at the right (S=1). Multiplies saturation onto the hue.
//!   3. Vertical gradient: transparent at the top (V=1) → black at
//!      the bottom (V=0). Multiplies brightness.
//!
//! Result is a continuous SV picker for a fixed hue. Hue itself is
//! controlled by the horizontal hue strip rendered separately by
//! [`super::value_slider`].
//!
//! Cursor: small ring stroked black-on-white at the current
//! `(S * w, (1 − V) * h)` so the user can locate it on any
//! background color.

use super::channels::{hsv_to_rgba8, rgba_to_hsv};
use super::state::BlenderColorPicker;
use crate::zones::Rect;
use ph2d_vector::{
    Affine, Brush, Circle, Color as VelloColor, Fill, Gradient, Point, Rect as KurboRect, Stroke,
    VectorScene,
};

pub fn paint_color_wheel(cp: &BlenderColorPicker, rect: Rect, scene: &mut VectorScene) {
    // Hue + S come from the retained anchor on the picker so the
    // SV-rect tint and cursor stay put when V→0 collapses the RGBA
    // to (0,0,0) (which has no recoverable hue). V is recoverable
    // from RGBA so we read it directly.
    let (_, _, v, _) = rgba_to_hsv(cp.value.rgba);
    let h = cp.hsv_h;
    let s = cp.hsv_s;
    let body = KurboRect::new(
        rect.x as f64,
        rect.y as f64,
        (rect.x + rect.w) as f64,
        (rect.y + rect.h) as f64,
    );

    // Layer 1: solid pure hue (S=1, V=1) at the current H.
    let hue_rgba = hsv_to_rgba8(h, 1.0, 1.0, 1.0);
    let hue_color = VelloColor::from_rgba8(hue_rgba[0], hue_rgba[1], hue_rgba[2], 255);
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(hue_color),
        None,
        &body,
    );

    // Layer 2: white (left) → transparent (right). Saturation overlay.
    let saturation = Gradient::new_linear(
        Point::new(rect.x as f64, rect.y as f64),
        Point::new((rect.x + rect.w) as f64, rect.y as f64),
    )
    .with_stops([
        VelloColor::from_rgba8(255, 255, 255, 255),
        VelloColor::from_rgba8(255, 255, 255, 0),
    ]);
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Gradient(saturation),
        None,
        &body,
    );

    // Layer 3: transparent (top) → black (bottom). Value overlay.
    let value = Gradient::new_linear(
        Point::new(rect.x as f64, rect.y as f64),
        Point::new(rect.x as f64, (rect.y + rect.h) as f64),
    )
    .with_stops([
        VelloColor::from_rgba8(0, 0, 0, 0),
        VelloColor::from_rgba8(0, 0, 0, 255),
    ]);
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Gradient(value),
        None,
        &body,
    );

    // Cursor at (S × w, (1 − V) × h). Clamped inwards by the cursor
    // radius (6) + outer-stroke half-width (1.5) so the ring never
    // pokes past the SV rect's edge — visible as a tiny "ear" at
    // the corners when S/V land at 0 or 1.
    const CURSOR_R: f32 = 6.0;
    const CURSOR_OUTER_HALF: f32 = 1.5;
    let inset = CURSOR_R + CURSOR_OUTER_HALF;
    let cur_x =
        (rect.x + s.clamp(0.0, 1.0) * rect.w).clamp(rect.x + inset, rect.x + rect.w - inset);
    let cur_y = (rect.y + (1.0 - v.clamp(0.0, 1.0)) * rect.h)
        .clamp(rect.y + inset, rect.y + rect.h - inset);
    let cursor = Circle::new(Point::new(cur_x as f64, cur_y as f64), CURSOR_R as f64);
    let outer = Stroke::new(3.0);
    let inner = Stroke::new(1.5);
    scene.inner_mut().stroke(
        &outer,
        Affine::IDENTITY,
        &Brush::Solid(VelloColor::from_rgba8(0, 0, 0, 220)),
        None,
        &cursor,
    );
    scene.inner_mut().stroke(
        &inner,
        Affine::IDENTITY,
        &Brush::Solid(VelloColor::WHITE),
        None,
        &cursor,
    );
}

/// Map a click at `(px, py)` inside `rect` to the (S, V) the SV
/// rectangle's cursor lives in. Both are normalised to `[0..1]`;
/// out-of-bounds clicks clamp to the nearest edge so the user can
/// drag past the rectangle and still get a valid pick.
pub fn wheel_pick(rect: Rect, px: f32, py: f32) -> Option<(f32, f32)> {
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return None;
    }
    let s = ((px - rect.x) / rect.w).clamp(0.0, 1.0);
    let v = 1.0 - ((py - rect.y) / rect.h).clamp(0.0, 1.0);
    Some((s, v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_top_left_is_zero_sat_full_value() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        let (s, v) = wheel_pick(r, 0.0, 0.0).unwrap();
        assert!(s.abs() < 1e-3);
        assert!((v - 1.0).abs() < 1e-3);
    }

    #[test]
    fn pick_bottom_right_is_full_sat_zero_value() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        let (s, v) = wheel_pick(r, 100.0, 100.0).unwrap();
        assert!((s - 1.0).abs() < 1e-3);
        assert!(v.abs() < 1e-3);
    }

    #[test]
    fn pick_clamps_outside_rect_instead_of_returning_none() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        let (s, v) = wheel_pick(r, 999.0, -50.0).unwrap();
        assert!((s - 1.0).abs() < 1e-3);
        assert!((v - 1.0).abs() < 1e-3);
    }
}
