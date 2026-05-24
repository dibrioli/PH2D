//! `LinearRgba` — linear-light RGBA in `[0.0, 1.0]`. The canonical
//! representation for render math: alpha blending, bilinear filtering,
//! convolution kernels and luminance integrals are only physically
//! correct on linear samples.

use crate::SrgbRgba;
use crate::srgb::linear_to_srgb_byte;

/// Linear-light RGBA with normalized `f32` channels.
///
/// Range is `[0.0, 1.0]` for in-gamut values; some pipelines tolerate
/// HDR (`> 1.0`) — we don't clamp here, callers decide.
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct LinearRgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl LinearRgba {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn opaque(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn black() -> Self {
        Self::opaque(0.0, 0.0, 0.0)
    }

    pub const fn white() -> Self {
        Self::opaque(1.0, 1.0, 1.0)
    }

    /// Fully transparent (RGB irrelevant — kept at 0).
    pub const fn transparent() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub const fn r(self) -> f32 {
        self.r
    }
    pub const fn g(self) -> f32 {
        self.g
    }
    pub const fn b(self) -> f32 {
        self.b
    }
    pub const fn a(self) -> f32 {
        self.a
    }

    pub const fn as_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Explicit conversion back to sRGB byte form via the IEC 61966
    /// transfer function. Out-of-range values are clamped to `[0, 1]`
    /// before quantization.
    pub fn to_srgb(self) -> SrgbRgba {
        SrgbRgba::new(
            linear_to_srgb_byte(self.r),
            linear_to_srgb_byte(self.g),
            linear_to_srgb_byte(self.b),
            (self.a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        )
    }

    /// Per-channel linear interpolation (`t` clamped to `[0.0, 1.0]`).
    /// Interpolation is correct in linear space; doing the same with
    /// sRGB bytes produces visibly darker midpoints.
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self::new(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
            self.a + (other.a - self.a) * t,
        )
    }

    /// Compute the BT.709 luminance (Y) of this color, treating it as
    /// opaque (alpha ignored). Useful for monochrome previews and
    /// histogram brightness rows.
    pub fn luminance(self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_set_alpha() {
        assert_eq!(LinearRgba::opaque(0.1, 0.2, 0.3).a(), 1.0);
        assert_eq!(LinearRgba::transparent().a(), 0.0);
    }

    #[test]
    fn lerp_midpoint() {
        let a = LinearRgba::black();
        let b = LinearRgba::white();
        let mid = a.lerp(b, 0.5);
        assert!((mid.r() - 0.5).abs() < 1e-6);
        assert!((mid.a() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn lerp_clamps_t() {
        let a = LinearRgba::black();
        let b = LinearRgba::white();
        assert_eq!(a.lerp(b, -0.5), a);
        assert_eq!(a.lerp(b, 1.5), b);
    }

    #[test]
    fn luminance_white_is_one() {
        let y = LinearRgba::white().luminance();
        assert!((y - 1.0).abs() < 1e-6);
        let y = LinearRgba::black().luminance();
        assert!(y.abs() < 1e-6);
    }
}
