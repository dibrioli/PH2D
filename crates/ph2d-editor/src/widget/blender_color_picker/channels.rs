//! 4-row channel sliders (R/G/B/A or H/S/V/A) + chip painter +
//! `rgba_to_hsv` helper.

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

    fill_rounded_rect(
        scene,
        label_rect,
        Radius::Xs.px(),
        resolve(ColorToken::AccentPress, theme),
    );
    paint_text_centered(
        text_system,
        scene,
        label,
        label_rect,
        TypeToken::Xs.px() - 1.0,
        resolve(ColorToken::AccentFg, theme),
    );

    fill_rounded_rect(
        scene,
        track_rect,
        Radius::Xs.px(),
        resolve(ColorToken::Bg2, theme),
    );
    let fill_w = track_rect.w * value.clamp(0.0, 1.0);
    if fill_w > 0.0 {
        let filled = Rect::new(track_rect.x, track_rect.y, fill_w, track_rect.h);
        fill_rounded_rect(
            scene,
            filled,
            Radius::Xs.px(),
            resolve(ColorToken::Border, theme),
        );
    }

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
