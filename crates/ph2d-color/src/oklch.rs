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

use crate::{LinearRgba, OklabColor, SrgbRgba};

/// A color in OKLCH coordinates.
///
/// - `l`: lightness `[0.0, 1.0]` (0 = black, 1 = display white)
/// - `c`: chroma `[0.0, ~0.4]` (display gamut clamp depends on hue)
/// - `h`: hue in degrees `[0.0, 360.0)`
/// - `a`: alpha `[0.0, 1.0]`
#[derive(Copy, Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
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

    /// Convert to linear-light sRGB via the OKLab → linear sRGB matrix.
    /// Out-of-gamut values are NOT clamped here; chain `.to_srgb()` to
    /// land in display range.
    ///
    /// Implementation: convert polar OKLCH → cartesian OKLab → linear,
    /// delegating the LMS-cubed matrix to [`OklabColor::to_linear`] so
    /// the 9 matrix coefficients live in a single place (no drift
    /// risk between the two color types).
    #[inline]
    #[must_use]
    pub fn to_linear(self) -> LinearRgba {
        let h_rad = self.h.to_radians();
        OklabColor::new(self.l, self.c * h_rad.cos(), self.c * h_rad.sin(), self.a).to_linear()
    }

    /// Convert to sRGB byte form.
    #[inline]
    #[must_use]
    pub fn to_srgb(self) -> SrgbRgba {
        self.to_linear().to_srgb()
    }
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
