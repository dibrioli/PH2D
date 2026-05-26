//! `OklabColor` — perceptual color in the OKLab linear-light space.
//!
//! Companion to [`crate::OklchColor`] (polar form). The cartesian OKLab
//! is what sits behind the `Stamp.color_oklab: [f32; 4]` ABI field in
//! the brush engine compute shader (ADR-0044 §2.3) — the GPU shader
//! reads the 4 floats as OKLab `(L, a, b, alpha)` and converts to
//! linear sRGB on the GPU side using the Björn Ottosson 2020 matrices
//! (mirrored here on the CPU side for parity).
//!
//! OKLab is also the canonical working space for the design system's
//! hue-stable interpolation (4 themes × N tokens), and the entry point
//! for Mixbox pigment mixing (ADR-0044 §2.5 + [`crate::MixboxLinearSrgb`]):
//! Mixbox itself runs in linear sRGB, but the brush input arrives in
//! OKLab via `Stamp`, so the boundary conversion lives here.
//!
//! ## Reference
//!
//! Björn Ottosson (2020). *A perceptual color space for image
//! processing.* <https://bottosson.github.io/posts/oklab/>
//!
//! ## ABI / wire compatibility
//!
//! `#[repr(C)]` + `bytemuck::Pod + Zeroable` so a `&[OklabColor]` slice
//! is a zero-copy view of any `[f32; 4]` block — including the
//! `Stamp.color_oklab` slot in the painter brush engine GPU buffer.
//! Field order is `(l, a, b, alpha)` and **frozen by ADR-0044 §2.3**
//! (Stamp ABI offset 32..48); reordering breaks the shader.

use bytemuck::{Pod, Zeroable};

use crate::LinearRgba;

/// A color in OKLab coordinates.
///
/// - `l`: lightness `[0.0, 1.0]` — 0 = black, 1 = display white.
/// - `a`: green ↔ red axis (typically `[-0.4, 0.4]` in-gamut).
/// - `b`: blue ↔ yellow axis (typically `[-0.4, 0.4]` in-gamut).
/// - `alpha`: straight alpha `[0.0, 1.0]` (NOT premultiplied — the
///   brush engine premultiplies downstream when emitting stamps).
///
/// Out-of-gamut values (e.g. high chroma colors that fall outside
/// sRGB) are tolerated here; conversion clamps at the sRGB boundary.
///
/// **`Default::default()` is fully-transparent black** `(L=0, a=0, b=0,
/// alpha=0)` — matches [`LinearRgba`]'s convention. Use [`Self::black`]
/// for opaque black or [`Self::white`] for opaque D65 white.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Default, Pod, Zeroable)]
pub struct OklabColor {
    pub l: f32,
    pub a: f32,
    pub b: f32,
    pub alpha: f32,
}

impl OklabColor {
    #[must_use]
    pub const fn new(l: f32, a: f32, b: f32, alpha: f32) -> Self {
        Self { l, a, b, alpha }
    }

    #[must_use]
    pub const fn opaque(l: f32, a: f32, b: f32) -> Self {
        Self {
            l,
            a,
            b,
            alpha: 1.0,
        }
    }

    /// Achromatic black: `L = 0, a = 0, b = 0`.
    #[must_use]
    pub const fn black() -> Self {
        Self::opaque(0.0, 0.0, 0.0)
    }

    /// Achromatic white: `L = 1, a = 0, b = 0` — exactly D65 white per
    /// the OKLab spec.
    #[must_use]
    pub const fn white() -> Self {
        Self::opaque(1.0, 0.0, 0.0)
    }

    #[must_use]
    pub const fn transparent() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Zero-copy view as a `[f32; 4]` array in `(l, a, b, alpha)` order.
    /// Matches the wire format used by `Stamp.color_oklab` (ADR-0044 §2.3).
    #[must_use]
    pub const fn as_array(self) -> [f32; 4] {
        [self.l, self.a, self.b, self.alpha]
    }

    #[must_use]
    pub const fn from_array(rgba: [f32; 4]) -> Self {
        Self::new(rgba[0], rgba[1], rgba[2], rgba[3])
    }

    /// Convert from linear sRGB (D65) to OKLab.
    ///
    /// Reference: Björn Ottosson 2020. Three-matrix path
    /// (RGB → LMS → cbrt → OKLab). Preserves `alpha` straight (no
    /// premultiplication is applied or removed).
    #[inline]
    #[must_use]
    pub fn from_linear(linear: LinearRgba) -> Self {
        let r = linear.r();
        let g = linear.g();
        let b = linear.b();

        let l = 0.412_221_47 * r + 0.536_332_5 * g + 0.051_445_993 * b;
        let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
        let s = 0.088_302_46 * r + 0.281_718_84 * g + 0.629_978_7 * b;

        let l_ = cbrt(l);
        let m_ = cbrt(m);
        let s_ = cbrt(s);

        Self {
            l: 0.210_454_26 * l_ + 0.793_617_8 * m_ - 0.004_072_047 * s_,
            a: 1.977_998_5 * l_ - 2.428_592_2 * m_ + 0.450_593_7 * s_,
            b: 0.025_904_037 * l_ + 0.782_771_77 * m_ - 0.808_675_77 * s_,
            alpha: linear.a(),
        }
    }

    /// Convert from OKLab to linear sRGB (D65). Mirror of
    /// [`Self::from_linear`].
    ///
    /// **Out-of-gamut values are NOT clamped** — the inverse matrix can
    /// emit negative RGB for high-chroma OKLab. Chain `.to_srgb()` on
    /// the result to clamp+quantize at the display boundary.
    ///
    /// The brush engine WGSL shader (`stamp.wgsl::oklab_to_linear_srgb`)
    /// is bit-identical to this function (same Ottosson 2020 matrix; all
    /// 15 coefficients round to the same `f32` bit pattern).
    #[inline]
    #[must_use]
    pub fn to_linear(self) -> LinearRgba {
        let l_ = self.l + 0.396_337_78 * self.a + 0.215_803_76 * self.b;
        let m_ = self.l - 0.105_561_346 * self.a - 0.063_854_17 * self.b;
        let s_ = self.l - 0.089_484_18 * self.a - 1.291_485_5 * self.b;

        let l3 = l_ * l_ * l_;
        let m3 = m_ * m_ * m_;
        let s3 = s_ * s_ * s_;

        LinearRgba::new(
            4.076_741_7 * l3 - 3.307_711_6 * m3 + 0.230_969_94 * s3,
            -1.268_438 * l3 + 2.609_757_4 * m3 - 0.341_319_38 * s3,
            -0.004_196_086_3 * l3 - 0.703_418_6 * m3 + 1.707_614_7 * s3,
            self.alpha,
        )
    }

    /// Per-channel linear interpolation (`t` clamped to `[0.0, 1.0]`).
    ///
    /// Perceptually-uniform: lightness (`L`) interpolates linearly in
    /// perceived brightness, and the `(a, b)` opponent axes lerp without
    /// HSL's muddy mid-greys. This is **NOT hue-preserving** — chroma
    /// direction shifts along the Cartesian chord when both endpoints
    /// have non-zero, non-parallel chroma. For hue-preserving gradients
    /// use [`crate::OklchColor`] (polar form). For mixing pigments use
    /// the Mixbox path in `ph2d-painter-brush::mixbox` (ADR-0044 §2.5).
    #[inline]
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self::new(
            self.l + (other.l - self.l) * t,
            self.a + (other.a - self.a) * t,
            self.b + (other.b - self.b) * t,
            self.alpha + (other.alpha - self.alpha) * t,
        )
    }
}

/// Pure-Rust cube root that handles negative inputs (the OKLab inverse
/// path can feed negative LMS values for out-of-gamut OKLab). `f32::cbrt`
/// already does this on stable Rust 1.92+, but going via a local fn
/// keeps the implementation explicit (and ready for `--features
/// det-painter` Q-fixed-point swap in a later wave, per ADR-0044 §2.5.1).
#[inline]
fn cbrt(x: f32) -> f32 {
    x.cbrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    fn close(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn constructors_set_alpha() {
        assert_eq!(OklabColor::opaque(0.5, 0.0, 0.0).alpha, 1.0);
        assert_eq!(OklabColor::transparent().alpha, 0.0);
        assert_eq!(OklabColor::white().l, 1.0);
        assert_eq!(OklabColor::black().l, 0.0);
    }

    #[test]
    fn as_array_round_trip() {
        let c = OklabColor::new(0.5, 0.1, -0.2, 0.7);
        let arr = c.as_array();
        assert_eq!(arr, [0.5, 0.1, -0.2, 0.7]);
        let c2 = OklabColor::from_array(arr);
        assert_eq!(c2, c);
    }

    #[test]
    fn white_oklab_round_trips_to_linear_white() {
        let white = OklabColor::white();
        let linear = white.to_linear();
        assert!(close(linear.r(), 1.0, 1e-3), "r got {}", linear.r());
        assert!(close(linear.g(), 1.0, 1e-3), "g got {}", linear.g());
        assert!(close(linear.b(), 1.0, 1e-3), "b got {}", linear.b());
        assert!(close(linear.a(), 1.0, EPS));
    }

    #[test]
    fn black_oklab_round_trips_to_linear_black() {
        let black = OklabColor::black();
        let linear = black.to_linear();
        assert!(close(linear.r(), 0.0, EPS));
        assert!(close(linear.g(), 0.0, EPS));
        assert!(close(linear.b(), 0.0, EPS));
        assert!(close(linear.a(), 1.0, EPS));
    }

    #[test]
    fn round_trip_linear_to_oklab_to_linear_preserves_values() {
        // Hand-picked in-gamut linear sRGB samples.
        let samples = [
            LinearRgba::new(0.5, 0.5, 0.5, 1.0),
            LinearRgba::new(1.0, 0.0, 0.0, 1.0),
            LinearRgba::new(0.0, 1.0, 0.0, 1.0),
            LinearRgba::new(0.0, 0.0, 1.0, 1.0),
            LinearRgba::new(0.3, 0.6, 0.9, 0.5),
        ];
        for lin in samples {
            let lab = OklabColor::from_linear(lin);
            let back = lab.to_linear();
            assert!(
                close(back.r(), lin.r(), 1e-3)
                    && close(back.g(), lin.g(), 1e-3)
                    && close(back.b(), lin.b(), 1e-3)
                    && close(back.a(), lin.a(), EPS),
                "round-trip drift: in={:?} out=({},{},{},{})",
                lin,
                back.r(),
                back.g(),
                back.b(),
                back.a(),
            );
        }
    }

    #[test]
    fn reference_red_oklab_matches_ottosson_paper() {
        // From Björn Ottosson 2020: linear-sRGB (1, 0, 0) → OKLab roughly
        // (L=0.6279, a=0.2249, b=0.1258). Allow ±0.002 for f32 drift.
        let lab = OklabColor::from_linear(LinearRgba::new(1.0, 0.0, 0.0, 1.0));
        assert!(close(lab.l, 0.627_955, 2e-3), "L got {}", lab.l);
        assert!(close(lab.a, 0.224_863, 2e-3), "a got {}", lab.a);
        assert!(close(lab.b, 0.125_846, 2e-3), "b got {}", lab.b);
    }

    #[test]
    fn lerp_midpoint() {
        let a = OklabColor::black();
        let b = OklabColor::white();
        let mid = a.lerp(b, 0.5);
        assert!(close(mid.l, 0.5, EPS));
        assert!(close(mid.a, 0.0, EPS));
        assert!(close(mid.b, 0.0, EPS));
        assert!(close(mid.alpha, 1.0, EPS));
    }

    #[test]
    fn lerp_clamps_t() {
        let a = OklabColor::black();
        let b = OklabColor::white();
        assert_eq!(a.lerp(b, -0.5), a);
        assert_eq!(a.lerp(b, 1.5), b);
    }

    #[test]
    fn pod_zero_copy_round_trip_through_bytes() {
        // ABI: a &[OklabColor] is a zero-copy view of any [f32; 4] block.
        // This is the contract that Stamp.color_oklab relies on (ADR-0044 §2.3
        // offset 32..48). If layout drifts, bytemuck::cast_slice would either
        // panic at runtime or — worse — feed garbage into the shader.
        let c = OklabColor::new(0.5, -0.1, 0.2, 0.75);
        let bytes = bytemuck::bytes_of(&c);
        assert_eq!(bytes.len(), 16);
        let c2: OklabColor = bytemuck::pod_read_unaligned(bytes);
        assert_eq!(c2, c);

        // Round-trip via &[f32; 4] cast — what the GPU buffer write path uses.
        let arr: &[f32; 4] = bytemuck::cast_ref(&c);
        assert_eq!(arr, &[0.5, -0.1, 0.2, 0.75]);
    }

    #[test]
    fn struct_size_is_16_bytes_aligned_4() {
        // OklabColor must serialize as 4×f32 = 16 bytes — matches
        // Stamp.color_oklab slot offset 32..48 (ADR-0044 §2.3).
        assert_eq!(core::mem::size_of::<OklabColor>(), 16);
        assert_eq!(core::mem::align_of::<OklabColor>(), 4);
    }
}
