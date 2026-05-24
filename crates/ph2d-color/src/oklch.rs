//! `OklchColor` — perceptual color in the OKLCH color space.
//!
//! Used by `docs/design/tokens.json` (every theme palette is OKLCH) and
//! by the hue-stable interpolation in the design system. OKLCH is
//! superior to HSL/HSV for theme arithmetic because lightness is
//! perceptually uniform — interpolating two OKLCH colors never produces
//! the muddy mid-grays that HSL interpolation famously does.
//!
//! Conversion to / from sRGB is provided via OKLab as an intermediate
//! (the standard reference path).

use crate::{LinearRgba, SrgbRgba};

/// A color in OKLCH coordinates.
///
/// - `l`: lightness `[0.0, 1.0]` (0 = black, 1 = display white)
/// - `c`: chroma `[0.0, ~0.4]` (display gamut clamp depends on hue)
/// - `h`: hue in degrees `[0.0, 360.0)`
/// - `a`: alpha `[0.0, 1.0]`
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct OklchColor {
    pub l: f32,
    pub c: f32,
    pub h: f32,
    pub a: f32,
}

impl OklchColor {
    pub const fn new(l: f32, c: f32, h: f32, a: f32) -> Self {
        Self { l, c, h, a }
    }

    pub const fn opaque(l: f32, c: f32, h: f32) -> Self {
        Self { l, c, h, a: 1.0 }
    }

    /// Convert to linear-light sRGB via OKLab → linear sRGB matrix.
    /// Out-of-gamut values are NOT clamped here; chain `.to_srgb()` to
    /// land in display range.
    pub fn to_linear(self) -> LinearRgba {
        let (l_lab, a_lab, b_lab) = oklch_to_oklab(self.l, self.c, self.h);
        let (r, g, b) = oklab_to_linear_rgb(l_lab, a_lab, b_lab);
        LinearRgba::new(r, g, b, self.a)
    }

    /// Convert to sRGB byte form.
    pub fn to_srgb(self) -> SrgbRgba {
        self.to_linear().to_srgb()
    }
}

#[inline]
fn oklch_to_oklab(l: f32, c: f32, h_deg: f32) -> (f32, f32, f32) {
    let h_rad = h_deg.to_radians();
    (l, c * h_rad.cos(), c * h_rad.sin())
}

/// Björn Ottosson OKLab → linear sRGB (D65). Standard reference
/// implementation; see <https://bottosson.github.io/posts/oklab/>.
#[inline]
fn oklab_to_linear_rgb(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
    let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    (
        4.076_741_7 * l3 - 3.307_711_6 * m3 + 0.230_969_94 * s3,
        -1.268_438 * l3 + 2.609_757_4 * m3 - 0.341_319_38 * s3,
        -0.004_196_086_3 * l3 - 0.703_418_6 * m3 + 1.707_614_7 * s3,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_sets_alpha_1() {
        let c = OklchColor::opaque(0.5, 0.1, 30.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn white_oklch_round_trips_to_srgb_white() {
        // L=1.0, C=0.0 → display white per OKLab spec.
        let white = OklchColor::opaque(1.0, 0.0, 0.0);
        let srgb = white.to_srgb();
        // Allow ±1 of round-trip drift on each channel.
        for ch in [srgb.r(), srgb.g(), srgb.b()] {
            assert!(ch.abs_diff(255) <= 1, "got {ch}");
        }
    }

    #[test]
    fn black_oklch_round_trips_to_srgb_black() {
        let black = OklchColor::opaque(0.0, 0.0, 0.0);
        let srgb = black.to_srgb();
        for ch in [srgb.r(), srgb.g(), srgb.b()] {
            assert!(ch <= 1, "got {ch}");
        }
    }

    #[test]
    fn chromatic_color_produces_nonzero_chroma_channels() {
        // Pure red-ish: L=0.6, C=0.2, H=30 (orange/red region) → r should dominate
        let c = OklchColor::opaque(0.6, 0.2, 30.0);
        let srgb = c.to_srgb();
        assert!(
            srgb.r() > srgb.b(),
            "expected r > b, got r={}, b={}",
            srgb.r(),
            srgb.b()
        );
    }
}
