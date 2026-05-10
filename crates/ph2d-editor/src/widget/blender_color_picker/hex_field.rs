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
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg2, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
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
    paint_text(
        text_system,
        scene,
        hex,
        rect.x + pad + label_w,
        rect.y + (rect.h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        rect.w - pad * 2.0 - label_w,
        resolve(ColorToken::Text1, theme),
    );
}

pub fn paint_eyedropper(rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg2, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    paint_icon(
        scene,
        IconId::EyePencil,
        rect,
        resolve(ColorToken::Text2, theme),
        1.5,
    );
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
