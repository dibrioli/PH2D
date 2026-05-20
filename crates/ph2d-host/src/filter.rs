//! Global image-filter mode — the single source of truth for how the
//! app samples every sprite/texture and the Vello preview.
//!
//! Lives in `ph2d-host` because it is the lowest-level crate shared by
//! both `ph2d-render` (which maps it to `wgpu::FilterMode` for the
//! sprite samplers) and `ph2d-editor-core` (which maps it to
//! `peniko::ImageQuality` for the Background-Removal Vello preview).
//! Keeping the enum here — a zero-dependency, `#![forbid(unsafe_code)]`
//! crate — avoids a circular dep and keeps the type free of any GPU /
//! vector-renderer baggage. The `wgpu` ↔ `peniko` mapping helpers live
//! next to their consumers (see `ph2d_render::image_filter` and
//! `ph2d_editor_core`), so this crate stays dependency-light.
//!
//! ## Why a single mode
//!
//! Before this type the atlas sampler hardcoded `FilterMode::Linear`
//! (smooth), the individual-texture sampler hardcoded
//! `FilterMode::Nearest` (pixelated), and the Vello preview used the
//! peniko default. The same sprite looked smooth in the BG-removal
//! preview but pixelated after Apply (which bakes into an Individual
//! texture). One mode, chosen once in Settings, threaded everywhere,
//! removes the divergence.

/// The app-wide image-sampling mode. Chosen by the user in
/// Config → "Image filter" and applied to **every** texture sample
/// (atlas + individual sprite textures) and the Vello image preview.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum ImageFilterMode {
    /// Nearest-neighbor sampling — crisp, blocky pixels. The default:
    /// PH2D is a sprite/pixel-art editor, and integer-scaled pixel art
    /// is the expected look out of the box. Maps to
    /// `wgpu::FilterMode::Nearest` and `peniko::ImageQuality::Low`.
    #[default]
    PixelArt,
    /// Bilinear sampling — smooth, anti-aliased edges. Better for
    /// HD-2D / hand-drawn sprites resampled at non-integer scale. Maps
    /// to `wgpu::FilterMode::Linear` and `peniko::ImageQuality::High`.
    Smooth,
}

impl ImageFilterMode {
    /// Stable string label for menu rows / telemetry.
    pub fn label(self) -> &'static str {
        match self {
            Self::PixelArt => "Pixel Art",
            Self::Smooth => "Smooth",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_pixel_art() {
        // PH2D is a pixel-art-first editor; the out-of-the-box look
        // must be crisp Nearest sampling.
        assert_eq!(ImageFilterMode::default(), ImageFilterMode::PixelArt);
    }

    #[test]
    fn labels_are_distinct_and_stable() {
        assert_eq!(ImageFilterMode::PixelArt.label(), "Pixel Art");
        assert_eq!(ImageFilterMode::Smooth.label(), "Smooth");
        assert_ne!(
            ImageFilterMode::PixelArt.label(),
            ImageFilterMode::Smooth.label()
        );
    }
}
