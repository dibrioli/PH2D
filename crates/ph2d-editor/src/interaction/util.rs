//! Standalone helpers used by the interaction module.
//!
//! Extracted from [`super::state`] (Track D2). Pure functions — no
//! `WidgetStore` access, no shared state — so they're trivial to
//! reason about in isolation and to test.

use ph2d_tokens::ColorValue;

/// Convert HSV (all in [0..1]) + alpha to [`ColorValue`].
/// Inverse of [`crate::widget::blender_color_picker::channels::rgba_to_hsv`].
pub fn hsv_to_color_value(h: f32, s: f32, v: f32, a: f32) -> ColorValue {
    let h6 = h * 6.0;
    let i = h6.floor() as u32 % 6;
    let f = h6 - h6.floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    ColorValue::from_rgba8(
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
        (a * 255.0).round() as u8,
    )
}

/// Pretty-print a `f64` for NumberInput buffer initialisation:
/// integers without trailing `.0`, fractions with up to 3 decimals.
/// Mirrors `widget::number_input::format_number` to keep both reps
/// in sync without crossing the module boundary.
pub fn format_number(v: f64) -> String {
    if (v - v.round()).abs() < 1e-6 {
        format!("{}", v as i64)
    } else {
        format!("{v:.3}")
    }
}
