//! Painter primary-color surface — sRGB8 ↔ OKLCH bridge + hex round-trip.
//!
//! # Why this module exists (W2.T2.3)
//!
//! The visual color picker (editor-core `blender_color_picker`) and the
//! top-bar color thumb work in **sRGB / hex** — that is what humans type
//! and what panels paint. The `PainterTool`, however, stores its primary
//! color as [`crate::params::OklchColor`] (`params.active_color`), with
//! **hue in RADIANS** (the painter-local stub convention — see
//! `tool::oklch_to_oklab`, which asserts `|h| ≤ 4π`).
//!
//! This module is the single, tested bridge between those two worlds so
//! the picker can drive `active_color` and read it back without ever
//! touching the radians/degrees hue trap or re-deriving the OKLab matrix.
//!
//! ## Conversion path (no math lives here that `ph2d-color` already owns)
//!
//! Every conversion delegates the heavy lifting to `ph2d-color`
//! primitives (the Ottosson 2020 OKLab matrix lives there, in ONE place):
//!
//! - **sRGB8 → OKLCH(rad):** [`SrgbRgba::to_linear`] →
//!   [`OklabColor::from_linear`] → cartesian `(a,b)` to polar `(c,h)` via
//!   `hypot`/`atan2` (atan2 already returns radians — the painter's native
//!   hue unit, so NO degree conversion is introduced).
//! - **OKLCH(rad) → sRGB8:** polar `(c,h)` to cartesian `(a,b)` via
//!   `cos`/`sin` → [`OklabColor::to_linear`] → [`LinearRgba::to_srgb`]
//!   (which clamps out-of-gamut at the display boundary).
//!
//! ## Hex
//!
//! `ph2d-color` has NO hex parse/format (verified by grep, 2026-05-31), so
//! the hex round-trip lives here. It operates on `[u8; 4]` sRGB bytes —
//! the wire format the picker and thumb already speak — and composes with
//! the sRGB↔OKLCH bridge above for the OKLCH-native call sites.

use crate::params::OklchColor;
use ph2d_color::{LinearRgba, OklabColor, SrgbRgba};

/// Error returned by [`parse_hex`] for malformed hex color strings.
///
/// Each variant carries enough context for a UI toast / inline validation
/// message without the caller re-inspecting the input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HexParseError {
    /// String was empty or contained only the leading `#`.
    Empty,
    /// Digit count after the optional `#` was not 3, 6, or 8.
    BadLength(usize),
    /// A character was not a valid hex digit (`0-9`, `a-f`, `A-F`).
    BadDigit(char),
}

impl core::fmt::Display for HexParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => write!(f, "empty hex color"),
            Self::BadLength(n) => write!(
                f,
                "hex color must have 3, 6, or 8 digits (#RGB / #RRGGBB / #RRGGBBAA); got {n}"
            ),
            Self::BadDigit(c) => write!(f, "invalid hex digit '{c}'"),
        }
    }
}

impl std::error::Error for HexParseError {}

/// Parse a `#RGB` / `#RRGGBB` / `#RRGGBBAA` (leading `#` optional) hex
/// color into sRGB `[r, g, b, a]` bytes.
///
/// - `#RGB` short form expands each nibble (e.g. `#f08` → `[255, 0, 136, 255]`),
///   matching CSS semantics.
/// - `#RRGGBB` is opaque (alpha = 255).
/// - `#RRGGBBAA` carries explicit alpha.
///
/// Surrounding ASCII whitespace is trimmed. Returns a [`HexParseError`]
/// with a human-readable `Display` for invalid input (empty / wrong digit
/// count / non-hex char) so the UI can surface a precise validation message.
///
/// # Examples
/// ```
/// use ph2d_tool_painter::color::parse_hex;
/// assert_eq!(parse_hex("#ff8800").unwrap(), [255, 136, 0, 255]);
/// assert_eq!(parse_hex("f80").unwrap(), [255, 136, 0, 255]);
/// assert_eq!(parse_hex("#11223344").unwrap(), [0x11, 0x22, 0x33, 0x44]);
/// assert!(parse_hex("#12345").is_err());
/// ```
pub fn parse_hex(s: &str) -> Result<[u8; 4], HexParseError> {
    let body = s.trim().strip_prefix('#').unwrap_or_else(|| s.trim());
    if body.is_empty() {
        return Err(HexParseError::Empty);
    }
    // Validate every digit up front so the error points at the offending
    // char rather than failing mid-parse with a generic message.
    if let Some(bad) = body.chars().find(|c| !c.is_ascii_hexdigit()) {
        return Err(HexParseError::BadDigit(bad));
    }
    let nibble = |c: u8| (c as char).to_digit(16).expect("validated above") as u8;
    let bytes = body.as_bytes();
    match bytes.len() {
        3 => {
            // #RGB → each nibble doubled (0xF → 0xFF).
            let r = nibble(bytes[0]);
            let g = nibble(bytes[1]);
            let b = nibble(bytes[2]);
            Ok([r * 17, g * 17, b * 17, 255])
        }
        6 => {
            let r = nibble(bytes[0]) * 16 + nibble(bytes[1]);
            let g = nibble(bytes[2]) * 16 + nibble(bytes[3]);
            let b = nibble(bytes[4]) * 16 + nibble(bytes[5]);
            Ok([r, g, b, 255])
        }
        8 => {
            let r = nibble(bytes[0]) * 16 + nibble(bytes[1]);
            let g = nibble(bytes[2]) * 16 + nibble(bytes[3]);
            let b = nibble(bytes[4]) * 16 + nibble(bytes[5]);
            let a = nibble(bytes[6]) * 16 + nibble(bytes[7]);
            Ok([r, g, b, a])
        }
        n => Err(HexParseError::BadLength(n)),
    }
}

/// Format sRGB `[r, g, b, a]` bytes as an uppercase hex string.
///
/// Emits `#RRGGBB` when the color is opaque (`a == 255`) and
/// `#RRGGBBAA` otherwise — the shortest lossless representation, which is
/// what color-input fields conventionally display.
///
/// # Examples
/// ```
/// use ph2d_tool_painter::color::format_hex;
/// assert_eq!(format_hex([255, 136, 0, 255]), "#FF8800");
/// assert_eq!(format_hex([0x11, 0x22, 0x33, 0x44]), "#11223344");
/// ```
#[must_use]
pub fn format_hex(rgba: [u8; 4]) -> String {
    let [r, g, b, a] = rgba;
    if a == 255 {
        format!("#{r:02X}{g:02X}{b:02X}")
    } else {
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }
}

/// Convert sRGB8 `[r, g, b, a]` bytes → painter [`OklchColor`] (**hue in
/// radians**, the painter-stub convention).
///
/// Path: sRGB → linear → OKLab → polar OKLCH. `atan2` yields hue directly
/// in radians, so no degree conversion is introduced. The hue is
/// normalized to `[0, 2π)` so downstream `oklch_to_oklab` (which asserts
/// `|h| ≤ 4π`) is always satisfied.
#[must_use]
pub fn srgb8_to_painter_oklch(rgba: [u8; 4]) -> OklchColor {
    let lab = OklabColor::from_linear(SrgbRgba(rgba).to_linear());
    let c = lab.a.hypot(lab.b);
    // atan2 → (-π, π]; normalize to [0, 2π) for a canonical hue.
    let mut h = lab.b.atan2(lab.a);
    if h < 0.0 {
        h += core::f32::consts::TAU;
    }
    OklchColor {
        l: lab.l,
        c,
        h,
        a: lab.alpha,
    }
}

/// Convert painter [`OklchColor`] (**hue in radians**) → sRGB8
/// `[r, g, b, a]` bytes, clamping out-of-gamut chroma at the display
/// boundary (via [`LinearRgba::to_srgb`]).
///
/// Mirror of [`srgb8_to_painter_oklch`]; the two compose to an identity
/// within ±1 LSB per channel for in-gamut colors (the standard 8-bit
/// quantization tolerance — see the round-trip tests).
#[must_use]
pub fn painter_oklch_to_srgb8(c: OklchColor) -> [u8; 4] {
    // polar (c, h-radians) → cartesian (a, b). Matches tool::oklch_to_oklab.
    let a = c.c * c.h.cos();
    let b = c.c * c.h.sin();
    let lab = OklabColor::new(c.l, a, b, c.a);
    let srgb: SrgbRgba = LinearRgba::to_srgb(lab.to_linear());
    srgb.0
}

/// Parse a hex string straight into a painter [`OklchColor`] (hue in
/// radians). Convenience composition of [`parse_hex`] +
/// [`srgb8_to_painter_oklch`] for OKLCH-native call sites (e.g. the
/// `apply_ui_edit(SetColor)` path).
pub fn parse_hex_to_painter_oklch(s: &str) -> Result<OklchColor, HexParseError> {
    parse_hex(s).map(srgb8_to_painter_oklch)
}

/// Format a painter [`OklchColor`] (hue in radians) as a hex string.
/// Convenience composition of [`painter_oklch_to_srgb8`] + [`format_hex`].
#[must_use]
pub fn painter_oklch_to_hex(c: OklchColor) -> String {
    format_hex(painter_oklch_to_srgb8(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- hex parse: valid ----

    #[test]
    fn parse_six_digit_opaque() {
        assert_eq!(parse_hex("#ff8800").unwrap(), [255, 136, 0, 255]);
        assert_eq!(parse_hex("FF8800").unwrap(), [255, 136, 0, 255]); // no '#', uppercase
    }

    #[test]
    fn parse_eight_digit_with_alpha() {
        assert_eq!(parse_hex("#11223344").unwrap(), [0x11, 0x22, 0x33, 0x44]);
    }

    #[test]
    fn parse_three_digit_short_form_expands() {
        // #f80 → each nibble doubled (CSS semantics).
        assert_eq!(parse_hex("#f80").unwrap(), [255, 136, 0, 255]);
        assert_eq!(parse_hex("fff").unwrap(), [255, 255, 255, 255]);
        assert_eq!(parse_hex("000").unwrap(), [0, 0, 0, 255]);
    }

    #[test]
    fn parse_trims_whitespace() {
        assert_eq!(parse_hex("  #ff8800  ").unwrap(), [255, 136, 0, 255]);
    }

    // ---- hex parse: invalid ----

    #[test]
    fn parse_empty_is_error() {
        assert_eq!(parse_hex("").unwrap_err(), HexParseError::Empty);
        assert_eq!(parse_hex("#").unwrap_err(), HexParseError::Empty);
        assert_eq!(parse_hex("   ").unwrap_err(), HexParseError::Empty);
    }

    #[test]
    fn parse_bad_length_is_error() {
        assert_eq!(parse_hex("#12345").unwrap_err(), HexParseError::BadLength(5));
        assert_eq!(parse_hex("#1").unwrap_err(), HexParseError::BadLength(1));
        assert_eq!(
            parse_hex("#1234567").unwrap_err(),
            HexParseError::BadLength(7)
        );
    }

    #[test]
    fn parse_bad_digit_is_error() {
        // 'g' is not a hex digit; error names the offending char.
        assert_eq!(parse_hex("#gg8800").unwrap_err(), HexParseError::BadDigit('g'));
    }

    #[test]
    fn parse_error_display_is_human_readable() {
        assert!(format!("{}", HexParseError::Empty).contains("empty"));
        assert!(format!("{}", HexParseError::BadLength(5)).contains("3, 6, or 8"));
        assert!(format!("{}", HexParseError::BadDigit('z')).contains('z'));
    }

    // ---- hex format ----

    #[test]
    fn format_opaque_drops_alpha() {
        assert_eq!(format_hex([255, 136, 0, 255]), "#FF8800");
    }

    #[test]
    fn format_non_opaque_keeps_alpha() {
        assert_eq!(format_hex([0x11, 0x22, 0x33, 0x44]), "#11223344");
    }

    #[test]
    fn hex_string_round_trips() {
        for hex in ["#FF8800", "#000000", "#FFFFFF", "#11223344", "#01FE7F80"] {
            let bytes = parse_hex(hex).unwrap();
            assert_eq!(format_hex(bytes), hex, "round-trip drift for {hex}");
        }
    }

    // ---- sRGB ↔ OKLCH bridge ----

    #[test]
    fn srgb_black_and_white_round_trip_within_lsb() {
        for rgba in [[0, 0, 0, 255], [255, 255, 255, 255], [255, 136, 0, 255]] {
            let oklch = srgb8_to_painter_oklch(rgba);
            let back = painter_oklch_to_srgb8(oklch);
            for ch in 0..4 {
                assert!(
                    back[ch].abs_diff(rgba[ch]) <= 1,
                    "channel {ch} drift: in={rgba:?} out={back:?}"
                );
            }
        }
    }

    #[test]
    fn srgb_to_oklch_hue_is_radians_in_range() {
        // A saturated orange must yield hue in [0, 2π) (radians), NOT degrees.
        let oklch = srgb8_to_painter_oklch([255, 136, 0, 255]);
        assert!(
            (0.0..core::f32::consts::TAU).contains(&oklch.h),
            "hue must be radians in [0, 2π); got {}",
            oklch.h
        );
        // Sanity: |h| ≤ 4π so tool::oklch_to_oklab's production assert holds.
        assert!(oklch.h.abs() <= 4.0 * core::f32::consts::PI);
        // Orange has real chroma.
        assert!(oklch.c > 0.05, "orange should have visible chroma; got {}", oklch.c);
    }

    #[test]
    fn srgb_alpha_preserved_through_bridge() {
        let oklch = srgb8_to_painter_oklch([255, 136, 0, 128]);
        assert!((oklch.a - 128.0 / 255.0).abs() < 1e-3);
        let back = painter_oklch_to_srgb8(oklch);
        assert!(back[3].abs_diff(128) <= 1);
    }

    #[test]
    fn full_hex_to_oklch_to_hex_round_trip() {
        // The headline path: a hex the picker emits → OKLCH the tool stores
        // → hex the thumb displays. In-gamut colors survive within ±1 LSB,
        // so the displayed hex matches the typed hex for these samples.
        for hex in ["#FF8800", "#3366CC", "#808080"] {
            let oklch = parse_hex_to_painter_oklch(hex).unwrap();
            assert_eq!(painter_oklch_to_hex(oklch), hex, "drift for {hex}");
        }
    }

    #[test]
    fn painter_default_color_round_trips_to_hex_and_back() {
        // The painter default (L0.70/C0.18/H0.5rad orange) must survive the
        // bridge so the picker opens showing the current swatch faithfully.
        let default = crate::params::PainterParams::default().active_color;
        let hex = painter_oklch_to_hex(default);
        let parsed = parse_hex_to_painter_oklch(&hex).unwrap();
        let a = painter_oklch_to_srgb8(default);
        let b = painter_oklch_to_srgb8(parsed);
        for ch in 0..4 {
            assert!(a[ch].abs_diff(b[ch]) <= 1, "channel {ch}: {a:?} vs {b:?}");
        }
    }
}
