//! Legacy `paint_slider_row` (used by the non-store
//! `paint_blender_color_picker` path) + `rgba_to_hsv` /
//! `hsv_to_rgba8` color-space helpers. The interactive channel
//! row is now drawn by `crate::widget::paint_slider_with_chip`.

use crate::paint::{fill_rounded_rect, paint_text_centered, resolve};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

pub fn paint_slider_row(
    label: &str,
    value: f32,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let label_w = 70.0;
    let val_w = 60.0;
    let track_x = rect.x + label_w + Spacing::Sm.px();
    let track_w = rect.w - label_w - val_w - Spacing::Sm.px() * 2.0;
    let label_rect = Rect::new(rect.x, rect.y, label_w, rect.h);
    let track_rect = Rect::new(track_x, rect.y + 6.0, track_w, rect.h - 12.0);
    let val_rect = Rect::new(rect.x + rect.w - val_w, rect.y, val_w, rect.h);

    // Plain text label, no pill background — the previous
    // AccentPress fill made every channel label read as a "selected
    // button". Channel rows are not interactive at the label, so a
    // chrome-less label keeps the eye on the slider track.
    paint_text_centered(
        text_system,
        scene,
        label,
        label_rect,
        TypeToken::Xs.px() - 1.0,
        resolve(ColorToken::Text2, theme),
    );

    // Canonical slider track (Bg2 + Accent fill) — shared with every
    // other slider in the app. Was a one-off Border-filled track.
    crate::widget::paint_slider_track(
        track_rect,
        value,
        crate::widget::SliderOrientation::Horizontal,
        scene,
        theme,
    );

    fill_rounded_rect(
        scene,
        val_rect,
        Radius::Xs.px(),
        resolve(ColorToken::Bg3, theme),
    );
    let display = format!("{value:.3}");
    paint_text_centered(
        text_system,
        scene,
        &display,
        val_rect,
        TypeToken::Xs.px(),
        resolve(ColorToken::Text1, theme),
    );
}

// `paint_channel_chip` was the picker-local interactive chip; it
// has been folded into `crate::widget::paint_number_chip`
// (the canonical app-wide chip used by `paint_slider_with_chip`).
// Callers should use the new module instead.

pub fn rgba_to_hsv(rgba: [u8; 4]) -> (f32, f32, f32, f32) {
    let r = rgba[0] as f32 / 255.0;
    let g = rgba[1] as f32 / 255.0;
    let b = rgba[2] as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let v = max;
    let s = if max == 0.0 { 0.0 } else { (max - min) / max };
    let h = if (max - min).abs() < f32::EPSILON {
        0.0
    } else if (max - r).abs() < f32::EPSILON {
        ((g - b) / (max - min) % 6.0) / 6.0
    } else if (max - g).abs() < f32::EPSILON {
        ((b - r) / (max - min) + 2.0) / 6.0
    } else {
        ((r - g) / (max - min) + 4.0) / 6.0
    };
    let h = h.rem_euclid(1.0);
    (h, s, v, rgba[3] as f32 / 255.0)
}

/// Inverse of [`rgba_to_hsv`]: HSV+A in 0..=1 → 8-bit RGBA.
pub fn hsv_to_rgba8(h: f32, s: f32, v: f32, a: f32) -> [u8; 4] {
    let h = (h * 6.0).rem_euclid(6.0);
    let c = v * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h.floor() as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [
        (((r + m) * 255.0).round()).clamp(0.0, 255.0) as u8,
        (((g + m) * 255.0).round()).clamp(0.0, 255.0) as u8,
        (((b + m) * 255.0).round()).clamp(0.0, 255.0) as u8,
        ((a.clamp(0.0, 1.0) * 255.0).round()) as u8,
    ]
}
