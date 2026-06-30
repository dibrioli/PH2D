//! [`BrushTextureImage`] — an imported brush-texture image (owned grayscale luminance + dims). Split out
//! of `brush_settings` for the workspace LOC cap; re-exported there so the `super::brush_settings::…`
//! import paths stay stable. Held in [`super::PaintState`] (the heavy pixels can't live in the `Copy`
//! `BrushSpec`); the engine borrows it as an [`ImageMask`].

use ph2d_painter_brush::ImageMask;

/// An imported brush-texture image: owned grayscale luminance + dims, held in [`super::PaintState`]
/// (heavy pixels can't live in the `Copy` `BrushSpec`); the engine borrows it as an [`ImageMask`].
pub(super) struct BrushTextureImage {
    lum: Vec<u8>,
    width: u32,
    height: u32,
}

impl BrushTextureImage {
    /// Construct from owned luminance + dims (fields are private; `shape_settings` builds via this).
    pub(super) fn new(lum: Vec<u8>, width: u32, height: u32) -> Self {
        Self { lum, width, height }
    }
    /// Borrow as `(luminance, w, h)` for the panel previews.
    pub(super) fn parts(&self) -> (&[u8], u32, u32) {
        (self.lum.as_slice(), self.width, self.height)
    }
    /// Borrow as the engine's [`ImageMask`].
    pub(super) fn as_mask(&self) -> ImageMask<'_> {
        ImageMask {
            lum: &self.lum,
            width: self.width,
            height: self.height,
        }
    }
}
