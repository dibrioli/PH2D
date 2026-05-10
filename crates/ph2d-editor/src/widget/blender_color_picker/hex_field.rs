//! Hex `#RRGGBBAA` text field + eyedropper button painters + parser.

use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ColorValue, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Parse a hex color string. Accepts:
/// - `#RGB` / `RGB`         → expands to `RRGGBB`, alpha = 255.
/// - `#RGBA` / `RGBA`       → expands to `RRGGBBAA`.
/// - `#RRGGBB` / `RRGGBB`   → alpha = 255.
/// - `#RRGGBBAA` / `RRGGBBAA`.
///
/// Whitespace around the input is trimmed. Returns `None` for any
/// other length or non-hex characters. Case-insensitive.
pub fn parse_hex(input: &str) -> Option<ColorValue> {
    let trimmed = input.trim();
    let stripped = trimmed.strip_prefix('#').unwrap_or(trimmed);
    if !stripped.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let bytes: [u8; 4] = match stripped.len() {
        3 => {
            let r = expand_nibble(&stripped[0..1])?;
            let g = expand_nibble(&stripped[1..2])?;
            let b = expand_nibble(&stripped[2..3])?;
            [r, g, b, 255]
        }
        4 => {
            let r = expand_nibble(&stripped[0..1])?;
            let g = expand_nibble(&stripped[1..2])?;
            let b = expand_nibble(&stripped[2..3])?;
            let a = expand_nibble(&stripped[3..4])?;
            [r, g, b, a]
        }
        6 => [
            u8::from_str_radix(&stripped[0..2], 16).ok()?,
            u8::from_str_radix(&stripped[2..4], 16).ok()?,
            u8::from_str_radix(&stripped[4..6], 16).ok()?,
            255,
        ],
        8 => [
            u8::from_str_radix(&stripped[0..2], 16).ok()?,
            u8::from_str_radix(&stripped[2..4], 16).ok()?,
            u8::from_str_radix(&stripped[4..6], 16).ok()?,
            u8::from_str_radix(&stripped[6..8], 16).ok()?,
        ],
        _ => return None,
    };
    Some(ColorValue::from_rgba8(
        bytes[0], bytes[1], bytes[2], bytes[3],
    ))
}

fn expand_nibble(s: &str) -> Option<u8> {
    let n = u8::from_str_radix(s, 16).ok()?;
    Some(n * 0x11)
}

pub fn paint_hex_field(
    hex: &str,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_hex_field_with_state(
        hex,
        None,
        0,
        None,
        crate::widget::TextInputState::Normal,
        rect,
        scene,
        text_system,
        theme,
    )
}

/// Like [`paint_hex_field`] but reads `buffer`/`caret` from the
/// caller's [`crate::interaction::WidgetStore`] entry and draws a
/// caret + accent border when `state == Focused`. Used by
/// `paint_blender_color_picker_with_store` so typing into the hex
/// field is visible.
#[allow(clippy::too_many_arguments)]
pub fn paint_hex_field_with_state(
    fallback_hex: &str,
    buffer: Option<&str>,
    caret: usize,
    selection_anchor: Option<usize>,
    state: crate::widget::TextInputState,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let focused = state == crate::widget::TextInputState::Focused;
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg2, theme));
    let stroke_w = if focused { 2.0 } else { 1.0 };
    let border = if focused {
        ColorToken::Accent
    } else {
        ColorToken::Border
    };
    stroke_rounded_rect(scene, rect, radius, stroke_w, resolve(border, theme));
    let pad = Spacing::Md.px();
    let label_w = 36.0;
    let label_rect = Rect::new(rect.x + pad, rect.y, label_w, rect.h);
    paint_text(
        text_system,
        scene,
        "Hex",
        label_rect.x,
        label_rect.y + (label_rect.h - TypeToken::Xs.px()) * 0.5,
        TypeToken::Xs.px(),
        label_w,
        resolve(ColorToken::Text2, theme),
    );
    // While focused, the user's typed buffer is the source of truth
    // (it contains the in-progress edit). Once blurred, fall back to
    // the canonical `#RRGGBBAA` derived from the picker's current
    // value, so wheel/slider/swatch picks immediately update the
    // visible hex even if `TextInput.text` is stale.
    let display = match buffer {
        Some(b) if focused => b,
        _ => fallback_hex,
    };
    let text_x = rect.x + pad + label_w;
    let text_y = rect.y + (rect.h - TypeToken::Sm.px()) * 0.5;
    let text_w = rect.w - pad * 2.0 - label_w;
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
                .layout(&display[..sel_start], TypeToken::Sm.px(), f32::INFINITY)
                .width()
        };
        let mid_w = if sel_start == sel_end {
            0.0
        } else {
            text_system
                .layout(
                    &display[sel_start..sel_end],
                    TypeToken::Sm.px(),
                    f32::INFINITY,
                )
                .width()
        };
        let sel_x = (text_x + prefix_w).min(text_x + text_w);
        let sel_w = mid_w.min(text_x + text_w - sel_x);
        if sel_w > 0.0 {
            let sel_top = rect.y + 4.0;
            let sel_bot = rect.y + rect.h - 4.0;
            let sel_rect = Rect::new(sel_x, sel_top, sel_w, (sel_bot - sel_top).max(2.0));
            fill_rounded_rect(scene, sel_rect, 1.0, resolve(ColorToken::AccentSoft, theme));
        }
    }
    paint_text(
        text_system,
        scene,
        display,
        text_x,
        text_y,
        TypeToken::Sm.px(),
        text_w,
        resolve(ColorToken::Text1, theme),
    );
    if focused {
        // Measure the prefix through `caret` using the real font
        // layout — fixed-advance approximations sit visibly off the
        // glyph edges (and worse for proportional fonts).
        let caret_clamped = caret.min(display.len());
        let prefix = &display[..caret_clamped];
        let advance = if prefix.is_empty() {
            0.0
        } else {
            text_system
                .layout(prefix, TypeToken::Sm.px(), f32::INFINITY)
                .width()
        };
        let caret_x = (text_x + advance).min(text_x + text_w);
        let caret_top = rect.y + 4.0;
        let caret_bot = rect.y + rect.h - 4.0;
        let caret_rect = Rect::new(caret_x, caret_top, 1.5, (caret_bot - caret_top).max(2.0));
        fill_rounded_rect(scene, caret_rect, 0.75, resolve(ColorToken::Accent, theme));
    }
}

pub fn paint_eyedropper(rect: Rect, scene: &mut VectorScene, theme: Theme) {
    paint_eyedropper_with_state(rect, false, scene, theme);
}

/// Like [`paint_eyedropper`] but draws an "active" highlight when
/// the picker is in pixel-pick mode (the next click outside this
/// button will sample the rendered scene).
pub fn paint_eyedropper_with_state(
    rect: Rect,
    active: bool,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let radius = Radius::Sm.px();
    let (bg, border, fg) = if active {
        (ColorToken::Accent, ColorToken::Accent, ColorToken::AccentFg)
    } else {
        (ColorToken::Bg2, ColorToken::Border, ColorToken::Text2)
    };
    fill_rounded_rect(scene, rect, radius, resolve(bg, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(border, theme));
    paint_icon(scene, IconId::EyePencil, rect, resolve(fg, theme), 1.5);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_with_hash() {
        let cv = parse_hex("#FF0080").unwrap();
        assert_eq!(cv.rgba, [0xFF, 0x00, 0x80, 0xFF]);
    }

    #[test]
    fn parse_full_with_alpha() {
        let cv = parse_hex("#0080FF80").unwrap();
        assert_eq!(cv.rgba, [0x00, 0x80, 0xFF, 0x80]);
    }

    #[test]
    fn parse_short_3_expands() {
        let cv = parse_hex("#F0A").unwrap();
        assert_eq!(cv.rgba, [0xFF, 0x00, 0xAA, 0xFF]);
    }

    #[test]
    fn parse_short_4_expands_with_alpha() {
        let cv = parse_hex("#F0A8").unwrap();
        assert_eq!(cv.rgba, [0xFF, 0x00, 0xAA, 0x88]);
    }

    #[test]
    fn parse_without_hash() {
        let cv = parse_hex("E7E7E7").unwrap();
        assert_eq!(cv.rgba, [0xE7, 0xE7, 0xE7, 0xFF]);
    }

    #[test]
    fn parse_case_insensitive() {
        let cv = parse_hex("#abcdef").unwrap();
        assert_eq!(cv.rgba, [0xAB, 0xCD, 0xEF, 0xFF]);
    }

    #[test]
    fn parse_rejects_wrong_length() {
        assert!(parse_hex("#FF").is_none());
        assert!(parse_hex("#FFFFF").is_none());
        assert!(parse_hex("#FFFFFFF").is_none());
        assert!(parse_hex("#FFFFFFFFF").is_none());
    }

    #[test]
    fn parse_rejects_non_hex() {
        assert!(parse_hex("#GGGGGG").is_none());
        assert!(parse_hex("#FF00ZZ").is_none());
        assert!(parse_hex("hello!!").is_none());
    }
}
