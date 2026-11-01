//! O par sRGB 8-bit + OKLCH em sincronia — irmão do [`crate::color`] pelo tecto de 700 LOC (o
//! endereço `ph2d_tokens::color::ColorValue` não muda: aquele ficheiro re-exporta-o).

use crate::color_space::{oklch_to_srgb, srgb_to_oklch};

/// Color value carrying both sRGB 8-bit and OKLCH in sync. Editor
/// code (paint helpers, ColorSwatch) consumes `rgba`; design tokens
/// and ColorPicker math operate on `oklch`. Construct via
/// [`ColorValue::from_rgba8`] or [`ColorValue::from_oklch`] — both
/// constructors compute the missing representation so the pair stays
/// consistent.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ColorValue {
    pub rgba: [u8; 4],
    /// `(L, C, H_deg, alpha)` — alpha mirrors `rgba[3] / 255.0`.
    pub oklch: (f64, f64, f64, f64),
}

impl ColorValue {
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        let (l, c, h) = srgb_to_oklch(r, g, b);
        Self {
            rgba: [r, g, b, a],
            oklch: (l, c, h, a as f64 / 255.0),
        }
    }

    pub fn from_oklch(l: f64, c: f64, h_deg: f64, alpha: f64) -> Self {
        let [r, g, b] = oklch_to_srgb(l, c, h_deg);
        let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
        Self {
            rgba: [r, g, b, a],
            oklch: (l, c, h_deg, alpha.clamp(0.0, 1.0)),
        }
    }

    pub const TRANSPARENT: Self = Self {
        rgba: [0, 0, 0, 0],
        oklch: (0.0, 0.0, 0.0, 0.0),
    };

    pub const BLACK: Self = Self {
        rgba: [0, 0, 0, 255],
        oklch: (0.0, 0.0, 0.0, 1.0),
    };

    pub const WHITE: Self = Self {
        rgba: [255, 255, 255, 255],
        oklch: (1.0, 0.0, 0.0, 1.0),
    };
}
