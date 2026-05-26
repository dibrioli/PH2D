//! `DecodedImage` — the canonical result of [`crate::ImageImporter::import`].
//!
//! **Cap FROZEN at 5 variants** by ADR-0054 §2.1 arch-gate
//! `decoded_image_variant_count_is_capped`. The 5 variants cover the
//! product of {LDR, HDR, layered, animated, vector} the three waves will
//! populate. `Vector` is modelled from W0 with a stub body so the cap
//! locks at 5 before W3 needs to widen it.

use crate::{ColorProfile, ImageBuffer};
use ph2d_color::{LinearRgba, SrgbRgba};

/// Result of decoding an image file. Importers return exactly one
/// variant; the caller (Painter shell, asset cooker, smoke test)
/// matches on it to decide rendering / commit semantics.
#[derive(Debug, Clone)]
pub enum DecodedImage {
    /// 8-bit sRGB-encoded RGBA — PNG/JPEG/WebP/GIF (1st frame)/BMP/TGA.
    /// Wave 1.
    Flat(ImageBuffer<SrgbRgba>),
    /// HDR scene-linear float RGBA — EXR/HDR-Radiance/AVIF-10bit/JXL-HDR.
    /// Wave 3.
    FlatHdr(ImageBuffer<LinearRgba>),
    /// Multi-layer document with masks/blend modes — PSD/ORA/KRA/.ph2d.
    /// Waves 1 (.ph2d) + 2 (PSD/ORA).
    Layered(LayerStack),
    /// Frame sequence — GIF/APNG/WebP-anim/AVIF-anim.
    /// Waves 1 (GIF) + 2 (APNG) + 3 (AVIF-anim).
    Animated(Vec<AnimFrame>),
    /// Vector document with `BezPath` paint stack — SVG/PDF.
    /// Wave 3. Stub in W0 to lock the cap.
    Vector(VectorDoc),
}

/// Multi-layer document. `canvas_*` dimensions are the document bounds;
/// individual layer pixels may be smaller (offset by position metadata).
#[derive(Debug, Clone, Default)]
pub struct LayerStack {
    /// Document width in pixels.
    pub canvas_width: u32,
    /// Document height in pixels.
    pub canvas_height: u32,
    /// Layers in **paint order** (bottom-up; `layers[0]` paints first).
    pub layers: Vec<Layer>,
    /// Document-wide color profile. Per-layer override unsupported in
    /// W1; PSD/ORA spec allows it in theory but no real authoring tool
    /// uses it.
    pub color_profile: ColorProfile,
}

/// One layer in a [`LayerStack`].
#[derive(Debug, Clone)]
pub struct Layer {
    /// User-visible name (UTF-8). PSD/ORA preserve original casing.
    pub name: String,
    /// 8-bit sRGB-encoded RGBA pixels (`width × height` in `pixels`).
    pub pixels: ImageBuffer<SrgbRgba>,
    /// Layer opacity ∈ [0.0, 1.0].
    pub opacity: f32,
    /// Blend mode against the composite below. Aligns with
    /// `RenderingMode = 6 FROZEN` of ADR-0044 Painter brush engine.
    pub blend_mode: BlendMode,
    /// `false` = hidden (skipped in composite, but preserved on save).
    pub visible: bool,
    /// 8-bit alpha mask. `None` = no mask attached.
    pub mask: Option<ImageBuffer<u8>>,
}

/// Blend modes recognized by [`Layer::blend_mode`]. Subset mirroring
/// ADR-0044 §RenderingMode; extended cautiously (each new variant is
/// extra work for every exporter to translate to its format-specific
/// blend code).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    /// Standard `over` Porter-Duff composite. Default.
    #[default]
    Normal,
    /// Multiply src × dst.
    Multiply,
    /// `1 - (1-src) × (1-dst)`.
    Screen,
    /// Combines Multiply (dark) + Screen (light) by dst luminance.
    Overlay,
    /// Photoshop "Soft Light" formulation.
    SoftLight,
    /// Photoshop "Hard Light" formulation.
    HardLight,
}

/// One frame in a [`DecodedImage::Animated`] sequence.
#[derive(Debug, Clone)]
pub struct AnimFrame {
    /// Frame pixels.
    pub image: ImageBuffer<SrgbRgba>,
    /// Display duration in milliseconds before advancing to the next
    /// frame. GIF/APNG/WebP-anim all carry this.
    pub delay_ms: u32,
}

/// Vector document preserved from SVG/PDF — `kurbo::BezPath` paint stack
/// plus transforms and clip paths.
///
/// **Wave 3 stub.** Modelled in W0 so the `DecodedImage` cap locks at 5
/// before W3 populates the body. Until then, the variant exists but no
/// importer constructs it; constructing one means hand-rolling a vector
/// fixture for arch-gate tests.
#[derive(Debug, Clone, Default)]
pub struct VectorDoc {
    /// W3 populates with `kurbo::BezPath` paint stack, transforms,
    /// viewBox. Empty in W0/W1/W2; presence of the field is what locks
    /// the `DecodedImage` cap at 5 without an amendment in W3.
    _reserved_for_w3: (),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_doc_default_is_constructible() {
        // Pin the W3 stub: keep the variant present, body empty, so a
        // future amendment in W3 can fill `_reserved_for_w3` without
        // changing the variant set.
        let _ = DecodedImage::Vector(VectorDoc::default());
    }

    #[test]
    fn blend_mode_default_is_normal() {
        assert_eq!(BlendMode::default(), BlendMode::Normal);
    }
}
