//! Pixel buffer carried inside `DecodedImage::Flat` / `FlatHdr` / `Layered`.
//!
//! Generic over the pixel type `P` — the canonical instantiations are
//! `ImageBuffer<SrgbRgba>` (LDR 8-bit wire format) and
//! `ImageBuffer<LinearRgba>` (HDR scene-linear). Per ADR-0054 §2.3 the
//! internal Painter buffer is converted to OKLCH/linear-f32 at the
//! editor boundary; this struct is purely the I/O wire shape.

use crate::ColorProfile;

/// Row-major RGBA pixel array tagged with its source color profile and
/// dimensions. `pixels.len()` must equal `width as usize * height as
/// usize` — importer impls are responsible for upholding this invariant.
#[derive(Debug, Clone)]
pub struct ImageBuffer<P> {
    /// Pixel count along the X axis.
    pub width: u32,
    /// Pixel count along the Y axis.
    pub height: u32,
    /// Row-major pixel data, top-left origin (sRGB convention; the
    /// engine flips for Vello/PostScript when needed — see
    /// `ph2d-render::projection`).
    pub pixels: Vec<P>,
    /// Color space of the encoded values. Importers populate from
    /// embedded ICC (W2+) or from the format's default (W1: `Srgb`).
    pub color_profile: ColorProfile,
}

impl<P> ImageBuffer<P> {
    /// Total pixel count (`width × height`).
    #[must_use]
    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    /// `true` if `pixels.len()` matches `width × height`. Importer impls
    /// should assert this after decode.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.pixels.len() == self.pixel_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_color::SrgbRgba;

    #[test]
    fn well_formed_buffer_passes_invariant() {
        let buf = ImageBuffer::<SrgbRgba> {
            width: 2,
            height: 3,
            pixels: vec![SrgbRgba([0, 0, 0, 255]); 6],
            color_profile: ColorProfile::Srgb,
        };
        assert!(buf.is_well_formed());
        assert_eq!(buf.pixel_count(), 6);
    }

    #[test]
    fn mismatched_buffer_fails_invariant() {
        let buf = ImageBuffer::<SrgbRgba> {
            width: 2,
            height: 3,
            pixels: vec![SrgbRgba([0, 0, 0, 255]); 5], // one short
            color_profile: ColorProfile::Srgb,
        };
        assert!(!buf.is_well_formed());
    }
}
