//! `Premultiplied<T>` — marker wrapper indicating an alpha-premultiplied
//! color value.
//!
//! Premultiplied alpha is the canonical blending format for `wgpu`'s
//! over-blending modes and for stb_image / PNG decoded with the
//! "premultiplied" loader option. Mixing premultiplied with
//! straight-alpha samples in the same pass is the textbook recipe for
//! the "dark halo on edges" artifact (UI_Bugs §4).
//!
//! The wrapper carries NO runtime cost (zero-sized newtype around `T`)
//! — it exists purely for type-level documentation. Methods that
//! materialize the wrapped value are explicit (`premultiply`,
//! `unmultiply`, `into_inner`).

use crate::{LinearRgba, SrgbRgba};

/// Alpha-premultiplied wrapper around a color type.
///
/// Convention: every RGB channel of the inner value has already been
/// multiplied by the alpha channel. To recover straight-alpha values
/// call [`Premultiplied::unmultiply`].
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Premultiplied<T>(T);

impl<T> Premultiplied<T> {
    /// Unsafe-but-not-unsafe constructor: marks the inner value as
    /// already premultiplied. Use ONLY when the inner value provably
    /// satisfies the invariant (e.g. came from a `*.png` decoded with
    /// premultiplied alpha). Otherwise use `premultiply()` on a typed
    /// straight-alpha value.
    pub const fn from_already_premultiplied(inner: T) -> Self {
        Self(inner)
    }

    /// Borrow the wrapped value. Caller asserts they will only use it
    /// in a context that expects premultiplied data (or convert back
    /// explicitly).
    pub const fn inner(&self) -> &T {
        &self.0
    }

    /// Move the wrapped value out, dropping the marker.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl Premultiplied<LinearRgba> {
    /// Multiply each RGB channel by alpha.
    pub fn premultiply(c: LinearRgba) -> Self {
        let a = c.a();
        Self(LinearRgba::new(c.r() * a, c.g() * a, c.b() * a, a))
    }

    /// Recover straight-alpha. Divides RGB by alpha; transparent
    /// pixels collapse to `(0, 0, 0, 0)` rather than NaN.
    pub fn unmultiply(self) -> LinearRgba {
        let p = self.0;
        let a = p.a();
        if a <= f32::EPSILON {
            LinearRgba::new(0.0, 0.0, 0.0, 0.0)
        } else {
            LinearRgba::new(p.r() / a, p.g() / a, p.b() / a, a)
        }
    }
}

impl Premultiplied<SrgbRgba> {
    /// Premultiplied sRGB is unusual but legal — used for PNG IO where
    /// the file was authored with `straight-then-premultiplied-on-load`.
    /// Multiplication is in 8-bit integer space (`(r * a + 127) / 255`).
    pub fn premultiply(c: SrgbRgba) -> Self {
        let a = c.a() as u16;
        let mul = |x: u8| (((x as u16) * a + 127) / 255) as u8;
        Self(SrgbRgba::new(mul(c.r()), mul(c.g()), mul(c.b()), c.a()))
    }

    /// Recover straight-alpha sRGB from premultiplied bytes. Division
    /// is in 8-bit integer space (`(r * 255 + a/2) / a`); fully
    /// transparent inputs collapse to `(0, 0, 0, 0)` rather than
    /// dividing by zero.
    ///
    /// Note: sRGB premultiplied math is not strictly correct under the
    /// gamma transfer — for filter-grade fidelity, prefer
    /// `unmultiply()` on a `Premultiplied<LinearRgba>` instead. This
    /// method exists for the IO boundary where the on-disk wire format
    /// is sRGB+premultiplied (e.g. some PNG loaders).
    pub fn unmultiply(self) -> SrgbRgba {
        let p = self.0;
        let a = p.a();
        if a == 0 {
            return SrgbRgba::new(0, 0, 0, 0);
        }
        let a16 = a as u16;
        let unmul = |x: u8| {
            let scaled = (x as u16) * 255 + a16 / 2;
            (scaled / a16).min(255) as u8
        };
        SrgbRgba::new(unmul(p.r()), unmul(p.g()), unmul(p.b()), a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_premultiply_round_trip() {
        let c = LinearRgba::new(0.4, 0.6, 0.8, 0.5);
        let p = <Premultiplied<LinearRgba>>::premultiply(c);
        assert!((p.inner().r() - 0.2).abs() < 1e-6);
        assert!((p.inner().a() - 0.5).abs() < 1e-6);
        let back = p.unmultiply();
        assert!((back.r() - 0.4).abs() < 1e-6);
        assert!((back.a() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn transparent_unmultiply_does_not_nan() {
        let p = <Premultiplied<LinearRgba>>::premultiply(LinearRgba::transparent());
        let back = p.unmultiply();
        assert_eq!(back.r(), 0.0);
        assert_eq!(back.a(), 0.0);
        // Crucially: no NaN.
        assert!(!back.r().is_nan() && !back.a().is_nan());
    }

    #[test]
    fn srgb_premultiply_byte_math() {
        // (200 * 128 + 127) / 255 = 25728 / 255 = 100
        let c = SrgbRgba::new(200, 200, 200, 128);
        let p = <Premultiplied<SrgbRgba>>::premultiply(c);
        assert_eq!(p.inner().r(), 100);
        assert_eq!(p.inner().a(), 128);
    }

    #[test]
    fn srgb_round_trip_within_quantization_error() {
        // 8-bit premultiplied storage has intrinsic round-trip error
        // bounded by `ceil(255 / a)` — the forward multiply quantizes
        // the input to multiples of `255/a`. At a=64 the quantum is
        // ~4; at a=255 it's 1. The reverse divide has another ±1.
        //
        // For each (r, a) we accept drift ≤ ceil(256 / a) + 1, which
        // bounds the worst-case round-trip. Render pipelines that need
        // sub-byte fidelity at low alpha must use
        // `Premultiplied<LinearRgba>`.
        for r in [0u8, 50, 128, 200, 255] {
            for a in [64u8, 128, 200, 255] {
                let c = SrgbRgba::new(r, r, r, a);
                let p = <Premultiplied<SrgbRgba>>::premultiply(c);
                let back = p.unmultiply();
                let drift = back.r().abs_diff(r) as u32;
                let bound = 256_u32.div_ceil(a as u32) + 1;
                assert!(
                    drift <= bound,
                    "round-trip drift at r={r}, a={a}: {} → {} (drift {drift}, bound {bound})",
                    p.inner().r(),
                    back.r()
                );
                assert_eq!(back.a(), a);
            }
        }
    }

    #[test]
    fn srgb_transparent_unmultiply_is_zero() {
        let p = Premultiplied::<SrgbRgba>::from_already_premultiplied(SrgbRgba::new(0, 0, 0, 0));
        let back = p.unmultiply();
        assert_eq!(back.0, [0, 0, 0, 0]);
    }
}
