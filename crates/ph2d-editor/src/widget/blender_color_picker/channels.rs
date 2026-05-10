//! 4-row channel sliders (R/G/B/A or H/S/V/A) + chip painter +
//! `rgba_to_hsv` helper.

use crate::paint::{fill_rounded_rect, paint_text_centered, resolve, stroke_rounded_rect};
use crate::widget::TextInputState;
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

/// Paint an interactive channel value chip (replaces the static
/// chip drawn by [`paint_slider_row`] when the orchestrator wants
/// edit/focus/caret behavior). `value` is the channel value in
/// 0..=1; the chip displays it as a 3-decimal float. When `state`
/// is `Focused` and `buffer` is `Some`, the buffer is shown instead
/// (so live typing is visible) and a caret is drawn at the byte
/// offset given by `caret`.
#[allow(clippy::too_many_arguments)]
pub fn paint_channel_chip(
    rect: Rect,
    state: TextInputState,
    value: f64,
    buffer: Option<&str>,
    caret: usize,
    selection_anchor: Option<usize>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let focused = state == TextInputState::Focused;
    let radius = Radius::Xs.px();
    let bg = if focused {
        ColorToken::Bg2
    } else {
        ColorToken::Bg3
    };
    fill_rounded_rect(scene, rect, radius, resolve(bg, theme));
    if focused {
        stroke_rounded_rect(scene, rect, radius, 2.0, resolve(ColorToken::Accent, theme));
    }
    let display_owned;
    let display = match buffer {
        Some(b) if focused => b,
        _ => {
            display_owned = format!("{value:.3}");
            display_owned.as_str()
        }
    };
    let font_size = TypeToken::Xs.px();
    let total_w = if display.is_empty() {
        0.0
    } else {
        text_system
            .layout(display, font_size, f32::INFINITY)
            .width()
    };
    let text_start = rect.x + (rect.w - total_w) * 0.5;
    if focused
        && let Some(anchor) = selection_anchor
        && anchor != caret
    {
        let (sel_start, sel_end) = if anchor < caret {
            (anchor, caret)
        } else {
            (caret, anchor)
        };
        let sel_start = sel_start.min(display.len());
        let sel_end = sel_end.min(display.len());
        let prefix_w = if sel_start == 0 {
            0.0
        } else {
            text_system
                .layout(&display[..sel_start], font_size, f32::INFINITY)
                .width()
        };
        let mid_w = if sel_start == sel_end {
            0.0
        } else {
            text_system
                .layout(&display[sel_start..sel_end], font_size, f32::INFINITY)
                .width()
        };
        let sel_top = rect.y + 4.0;
        let sel_bot = rect.y + rect.h - 4.0;
        let sel_x = (text_start + prefix_w).clamp(rect.x + 2.0, rect.x + rect.w - 2.0);
        let sel_w = mid_w.min(rect.x + rect.w - 2.0 - sel_x);
        if sel_w > 0.0 {
            let sel_rect = Rect::new(sel_x, sel_top, sel_w, (sel_bot - sel_top).max(2.0));
            fill_rounded_rect(scene, sel_rect, 1.0, resolve(ColorToken::AccentSoft, theme));
        }
    }
    paint_text_centered(
        text_system,
        scene,
        display,
        rect,
        font_size,
        resolve(ColorToken::Text1, theme),
    );
    if focused {
        let caret_clamped = caret.min(display.len());
        let prefix = &display[..caret_clamped];
        let prefix_w = if prefix.is_empty() {
            0.0
        } else {
            text_system.layout(prefix, font_size, f32::INFINITY).width()
        };
        let caret_x = (text_start + prefix_w).clamp(rect.x + 2.0, rect.x + rect.w - 2.0);
        let caret_top = rect.y + 4.0;
        let caret_bot = rect.y + rect.h - 4.0;
        let caret_rect = Rect::new(caret_x, caret_top, 1.5, (caret_bot - caret_top).max(2.0));
        fill_rounded_rect(scene, caret_rect, 0.75, resolve(ColorToken::Accent, theme));
    }
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
