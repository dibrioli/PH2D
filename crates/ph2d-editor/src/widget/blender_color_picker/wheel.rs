//! HSV color wheel disc + cursor crosshair painter.
//!
//! Two-pass paint:
//!   1. Sweep gradient (conic): pure-saturation hues around the rim
//!      — red 0° → yellow 60° → green 120° → cyan 180° → blue 240°
//!      → magenta 300° → red 360°.
//!   2. Radial gradient overlay: white at the center (sat=0) fading
//!      to transparent at the edge (sat=1). Composited additively
//!      gives the canonical HSV disc you see in Blender / Photoshop.
//!
//! Cursor: 6px circle stroked white over black for visibility on any
//! hue, positioned at `(sat·r·cos(hue), sat·r·sin(hue))` in disc-
//! local coords. `hue` and `sat` are derived from the picker's
//! current `value.oklch` chroma + hue, normalised to [0..1].

use super::state::BlenderColorPicker;
use crate::zones::Rect;
use ph2d_vector::{
    Affine, Brush, Circle, Color as VelloColor, Fill, Gradient, Point, Stroke, VectorScene,
};

/// Hue stops walked CCW around the wheel. Repeats red at the end so
/// the sweep gradient closes seamlessly.
const HUE_STOPS: [VelloColor; 7] = [
    VelloColor::from_rgba8(255, 0, 0, 255),   // red
    VelloColor::from_rgba8(255, 255, 0, 255), // yellow
    VelloColor::from_rgba8(0, 255, 0, 255),   // green
    VelloColor::from_rgba8(0, 255, 255, 255), // cyan
    VelloColor::from_rgba8(0, 0, 255, 255),   // blue
    VelloColor::from_rgba8(255, 0, 255, 255), // magenta
    VelloColor::from_rgba8(255, 0, 0, 255),   // red (close)
];

pub fn paint_color_wheel(cp: &BlenderColorPicker, rect: Rect, scene: &mut VectorScene) {
    let cx = rect.x + rect.w * 0.5;
    let cy = rect.y + rect.h * 0.5;
    let radius = (rect.w.min(rect.h)) * 0.5 - 2.0;
    let center = Point::new(cx as f64, cy as f64);
    let disc = Circle::new(center, radius as f64);

    // Pass 1: sweep gradient (conic) for the hue ring.
    let sweep = Gradient::new_sweep(center, 0.0, std::f32::consts::TAU).with_stops(HUE_STOPS);
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Gradient(sweep),
        None,
        &disc,
    );

    // Pass 2: radial overlay — white center fading to transparent
    // at the rim. Multiplies the saturation onto the hue beneath.
    let radial = Gradient::new_radial(center, radius).with_stops([
        VelloColor::from_rgba8(255, 255, 255, 255),
        VelloColor::from_rgba8(255, 255, 255, 0),
    ]);
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Gradient(radial),
        None,
        &disc,
    );

    // Cursor crosshair at (sat, hue) — chroma is normalised by 0.4
    // (the OKLCH max chroma we expose in this widget).
    let (_, c, h, _) = cp.value.oklch;
    let normalized_c = (c / 0.4).clamp(0.0, 1.0) as f32;
    let theta = (h as f32).to_radians();
    let cur_x = cx + theta.cos() * radius * normalized_c;
    let cur_y = cy + theta.sin() * radius * normalized_c;
    let cursor_r: f64 = 6.0;
    let cursor = Circle::new(Point::new(cur_x as f64, cur_y as f64), cursor_r);
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

/// Map a click at `(px, py)` inside `rect` to the (hue°, sat[0..1])
/// coordinates the wheel cursor lives in. Returns `None` if the
/// point is outside the disc (so dispatch can ignore it).
pub fn wheel_pick(rect: Rect, px: f32, py: f32) -> Option<(f32, f32)> {
    let cx = rect.x + rect.w * 0.5;
    let cy = rect.y + rect.h * 0.5;
    let radius = (rect.w.min(rect.h)) * 0.5 - 2.0;
    let dx = px - cx;
    let dy = py - cy;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist > radius {
        return None;
    }
    let sat = (dist / radius).clamp(0.0, 1.0);
    let hue_rad = dy.atan2(dx);
    let hue_deg = hue_rad.to_degrees().rem_euclid(360.0);
    Some((hue_deg, sat))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_center_is_zero_sat() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        let (_, sat) = wheel_pick(r, 50.0, 50.0).unwrap();
        assert!(sat.abs() < 1e-3);
    }

    #[test]
    fn pick_right_edge_is_full_sat_zero_hue() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        // Right edge is hue 0°, full saturation.
        let (hue, sat) = wheel_pick(r, 98.0, 50.0).unwrap();
        assert!(hue.abs() < 1.0 || (hue - 360.0).abs() < 1.0);
        assert!(sat > 0.95);
    }

    #[test]
    fn pick_outside_disc_returns_none() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(wheel_pick(r, 0.0, 0.0).is_none());
    }
}
