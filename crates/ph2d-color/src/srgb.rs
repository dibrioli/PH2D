//! `SrgbRgba` — 8-bit sRGB-encoded RGBA. The wire format every PH2D
//! tool reads from disk and writes to disk, and the format the panel
//! chrome paints with.
//!
//! Use [`SrgbRgba::to_linear`] when entering render math; use
//! [`LinearRgba::to_srgb`] to come back.

use crate::LinearRgba;

/// sRGB-encoded RGBA, 8-bit per channel.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct SrgbRgba(pub [u8; 4]);

impl SrgbRgba {
    /// Construct from 4 raw bytes.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([r, g, b, a])
    }

    /// Opaque from RGB (alpha = 255).
    pub const fn opaque(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b, 255])
    }

    pub const fn r(self) -> u8 {
        self.0[0]
    }
    pub const fn g(self) -> u8 {
        self.0[1]
    }
    pub const fn b(self) -> u8 {
        self.0[2]
    }
    pub const fn a(self) -> u8 {
        self.0[3]
    }

    /// Underlying byte slice.
    pub const fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }

    /// Explicit conversion to linear-light RGBA. The sRGB transfer
    /// function is applied per RGB channel; alpha is treated as linear
    /// (the IEC 61966 spec does not gamma-encode alpha).
    pub fn to_linear(self) -> LinearRgba {
        LinearRgba::new(
            srgb_to_linear_byte(self.r()),
            srgb_to_linear_byte(self.g()),
            srgb_to_linear_byte(self.b()),
            self.a() as f32 / 255.0,
        )
    }
}

/// IEC 61966 sRGB → linear transfer function on a single 8-bit byte.
/// Returns linear-light intensity in `[0.0, 1.0]`.
#[inline]
pub fn srgb_to_linear_byte(byte: u8) -> f32 {
    let v = byte as f32 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// IEC 61966 linear → sRGB transfer function on a linear intensity
/// in `[0.0, 1.0]`. Returns an 8-bit byte.
#[inline]
pub fn linear_to_srgb_byte(linear: f32) -> u8 {
    let v = linear.clamp(0.0, 1.0);
    let encoded = if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0 + 0.5) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_sets_alpha_255() {
        assert_eq!(SrgbRgba::opaque(10, 20, 30).0, [10, 20, 30, 255]);
    }

    #[test]
    fn black_and_white_round_trip() {
        let black = SrgbRgba::opaque(0, 0, 0);
        let white = SrgbRgba::opaque(255, 255, 255);
        let black_l = black.to_linear();
        let white_l = white.to_linear();
        assert!(black_l.r().abs() < 1e-5);
        assert!((white_l.r() - 1.0).abs() < 1e-5);
        // Round-trip preserves byte values exactly at extremes.
        assert_eq!(black_l.to_srgb(), black);
        assert_eq!(white_l.to_srgb(), white);
    }

    #[test]
    fn middle_gray_is_nonlinear() {
        // sRGB 128 → linear ~0.215 (not 0.5 — that's the whole point of gamma).
        let g = SrgbRgba::opaque(128, 128, 128).to_linear();
        assert!((g.r() - 0.215_86).abs() < 1e-3, "got {}", g.r());
    }

    #[test]
    fn linear_byte_helpers_inverse() {
        for byte in [0u8, 1, 64, 128, 192, 254, 255] {
            let lin = srgb_to_linear_byte(byte);
            let back = linear_to_srgb_byte(lin);
            assert!(
                back.abs_diff(byte) <= 1,
                "round-trip drift at {byte}: {lin} → {back}"
            );
        }
    }
}
