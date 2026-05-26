//! `DecodedImage` — the canonical result of [`crate::ImageImporter::import`].
//!
//! **Cap FROZEN at 5 variants** by ADR-0054 §2.1 arch-gate
//! `decoded_image_variant_count_is_capped`. The 5 variants cover the
//! product of {LDR, HDR, layered, animated, vector} the three waves will
//! populate. `Vector` is modelled from W0 with a stub body so the cap
//! locks at 5 before W3 needs to widen it.
//!
//! ## What is NOT arch-gated (audit clarification 2026-05-26)
//!
//! Per ADR-0054 §2.1 explicit non-capped list, the following **data
//! model** types may grow without amendment:
//!
//! - [`BlendMode`] — PSD/ORA carry 27+ blend modes; the enum can grow
//!   freely + `Custom(u16)` preserves PSD opcode for byte-exact
//!   round-trip on unknown modes.
//! - [`Layer`] — fields, layer kinds, effects.
//! - [`LayerStack`] — fields.
//! - [`AnimFrame`] — fields (offset_xy, dispose, blend_op, …).
//! - [`VectorDoc`] — fields populated by W3 SVG.
//!
//! These are *implementation details every format encoder/decoder reads*,
//! NOT *trait surfaces every implementer must satisfy*. The latter are
//! capped; the former grow with the formats they describe.

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
    /// Frame sequence — GIF/APNG/WebP-anim/AVIF-anim. SVG SMIL animation
    /// rasterizes here at the importer's default DPI (W3 ADR-0054
    /// §2.1 explicit decision — vector-preserved animation would need a
    /// 6th variant; current cap keeps it raster).
    Animated(Vec<AnimFrame>),
    /// Vector document with `BezPath` paint stack — SVG (static; SMIL
    /// animation goes to `Animated`)/PDF. Wave 3. Stub in W0 to lock
    /// the cap; W3 populates [`VectorDoc`] fields without bumping the
    /// `DecodedImage` cap.
    Vector(VectorDoc),
}

/// Multi-layer document. `canvas_*` dimensions are the document bounds;
/// individual layer pixels may be smaller (offset by position metadata).
#[derive(Debug, Clone)]
pub struct LayerStack {
    /// HR-14 save-format versioning. Bumped when [`Layer`] / [`LayerStack`]
    /// schema breaks. W1 = `1`. Migration via
    /// `fn migrate_v{N}_to_v{N+1}(old) -> Result<New>` lives in the
    /// `.ph2d-native` crate (W1.T5).
    pub version: u32,
    /// Document width in pixels.
    pub canvas_width: u32,
    /// Document height in pixels.
    pub canvas_height: u32,
    /// Layers in **paint order** (bottom-up; `layers[0]` paints first).
    pub layers: Vec<Layer>,
    /// Document-wide color profile. Per-layer override available via
    /// [`Layer::color_profile`] (audit A-M2 follow-up — PSD smart
    /// objects and TIFF multi-page carry per-layer profiles).
    pub color_profile: ColorProfile,
}

impl Default for LayerStack {
    fn default() -> Self {
        Self {
            version: 1,
            canvas_width: 0,
            canvas_height: 0,
            layers: Vec::new(),
            color_profile: ColorProfile::Srgb,
        }
    }
}

/// One layer in a [`LayerStack`]. Audit A-C2 (2026-05-26): grew to model
/// PSD/ORA layer richness — groups (folders), adjustment layers (no
/// pixels), text layers (vector-content), smart objects (embedded),
/// per-layer color profiles, layer effects (drop shadow / glow / stroke).
#[derive(Debug, Clone)]
pub struct Layer {
    /// HR-14 save-format versioning.
    pub version: u32,
    /// User-visible name (UTF-8). PSD/ORA preserve original casing.
    pub name: String,
    /// What this layer represents. Drives whether `pixels` is `Some`.
    pub kind: LayerKind,
    /// Rendered pixels. `None` for non-pixel layers (groups,
    /// adjustment layers, unrasterized text/smart objects).
    pub pixels: Option<ImageBuffer<SrgbRgba>>,
    /// Layer opacity ∈ [0.0, 1.0].
    pub opacity: f32,
    /// Blend mode against the composite below. Aligns with PSD blend
    /// opcodes; the importer maps PSD enum → [`BlendMode`].
    pub blend_mode: BlendMode,
    /// `false` = hidden (skipped in composite, but preserved on save).
    pub visible: bool,
    /// 8-bit alpha mask. `None` = no mask attached.
    pub mask: Option<ImageBuffer<u8>>,
    /// Layer effects (drop shadow, glow, stroke, …). PSD/ORA preserve
    /// these; flat formats ignore. Audit A-C2.
    pub effects: Vec<LayerEffect>,
    /// Per-layer color profile override. `None` = inherit from
    /// [`LayerStack::color_profile`]. Audit A-M2.
    pub color_profile: Option<ColorProfile>,
}

/// What a [`Layer`] represents. Audit A-C2 (2026-05-26): PSD/ORA
/// distinguish pixel layers from groups (folders), adjustment layers
/// (HSL/Levels/etc — no pixels, only operation parameters), text
/// layers (vector content + font metadata), and smart objects
/// (embedded raster/vector with non-destructive transforms).
///
/// Importer behaviour for non-`Pixel` kinds is W2-specific:
/// - PSD: preserve metadata; runtime composite either flattens or
///   honours kind based on UI policy.
/// - ORA: only `Pixel` + `Group` (ORA spec); other kinds are
///   importer-error.
/// - `.ph2d-native`: preserves all kinds round-trip.
#[derive(Debug, Clone, Default)]
pub enum LayerKind {
    /// Standard raster pixel layer — `Layer::pixels` is `Some`.
    #[default]
    Pixel,
    /// Group / folder. Children render as a sub-composite then blend
    /// into the parent.
    Group {
        /// Child layers, paint order bottom-up (same convention as
        /// [`LayerStack::layers`]).
        children: Vec<Layer>,
    },
    /// Non-destructive adjustment (HSL / Curves / Levels / etc.).
    /// `Layer::pixels` is `None`; the adjustment's parameters are
    /// `Layer::effects` or a future dedicated field. W2.0.2 expand
    /// when ICC pipeline lands.
    Adjustment,
    /// Text layer — vector content with font + character/paragraph
    /// metadata. `Layer::pixels` is `Some` only if the importer
    /// rasterized; PSD typically keeps unrasterized.
    Text,
    /// Smart object — embedded raster/vector with non-destructive
    /// transform. Preserve metadata; rasterize at runtime as needed.
    Smart,
}

/// Blend modes recognized by [`Layer::blend_mode`]. Audit A-C1
/// (2026-05-26): grew from 6 to ~24 mainstream modes (PSD canonical
/// set) plus `Custom(u16)` to preserve unknown PSD blend opcodes
/// byte-exact for round-trip. **Not arch-gated** per ADR-0054 §2.1
/// non-capped clarification — data model, not trait surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    /// Standard `over` Porter-Duff composite. Default.
    #[default]
    Normal,
    /// `1 - (1-src) × (1-dst)`.
    Multiply,
    /// `1 - (1-src) × (1-dst)`.
    Screen,
    /// Combines Multiply (dark) + Screen (light) by dst luminance.
    Overlay,
    /// `min(src, dst)`.
    Darken,
    /// `max(src, dst)`.
    Lighten,
    /// `dst / (1 - src)` clamped.
    ColorDodge,
    /// `1 - (1-dst) / src` clamped.
    ColorBurn,
    /// Photoshop "Hard Light".
    HardLight,
    /// Photoshop "Soft Light" (W3C variant).
    SoftLight,
    /// `|src - dst|`.
    Difference,
    /// `src + dst - 2·src·dst`.
    Exclusion,
    /// HSL: take hue from src, sat+lum from dst.
    Hue,
    /// HSL: take saturation from src.
    Saturation,
    /// HSL: take hue+saturation from src.
    Color,
    /// HSL: take luminosity from src.
    Luminosity,
    /// `dst + src - 1` (linear burn).
    LinearBurn,
    /// `dst + src` (linear dodge / "add").
    LinearDodge,
    /// Combines LinearBurn + LinearDodge by src luminance.
    LinearLight,
    /// Combines ColorBurn + ColorDodge by src luminance.
    VividLight,
    /// Combines Darken + Lighten by src luminance.
    PinLight,
    /// Combines VividLight then threshold to 0/1 per channel.
    HardMix,
    /// `dst / src` clamped.
    Divide,
    /// `dst - src`.
    Subtract,
    /// Unknown PSD blend opcode preserved byte-exact for round-trip.
    /// `u16` = PSD blend mode signature (4-CC packed) for fidelity.
    Custom(u16),
}

/// Layer effect (drop shadow, glow, stroke, …). PSD/ORA layer effects
/// stored opaquely until W2.T4 + W2.T1 wire format-specific encoding.
/// Audit A-C2.
#[derive(Debug, Clone)]
pub struct LayerEffect {
    /// What kind of effect.
    pub kind: LayerEffectKind,
}

/// Categories of layer effect. Mirrors PSD `lfx2`/`lmfx` opcodes at
/// the granularity W2 needs.
#[derive(Debug, Clone)]
pub enum LayerEffectKind {
    /// Drop shadow (color + offset + spread + size + opacity + blend).
    DropShadow,
    /// Inner / outer glow.
    Glow,
    /// Stroke (color + width + position).
    Stroke,
    /// Bevel & emboss.
    Bevel,
    /// Color overlay.
    ColorOverlay,
    /// Gradient overlay.
    GradientOverlay,
    /// Pattern overlay.
    PatternOverlay,
    /// Unknown / future PSD effect preserved opaquely for round-trip.
    Custom(String),
}

/// One frame in a [`DecodedImage::Animated`] sequence. Audit A-C3
/// (2026-05-26): grew from `{ image, delay_ms }` to model GIF/APNG/
/// WebP-anim disposal + blend correctly. Round-trip without these
/// would silently re-emit broken animations.
#[derive(Debug, Clone)]
pub struct AnimFrame {
    /// Frame pixels (top-left = `offset_xy`).
    pub image: ImageBuffer<SrgbRgba>,
    /// Display duration in milliseconds before advancing to the next
    /// frame. GIF/APNG/WebP-anim all carry this.
    pub delay_ms: u32,
    /// Sub-rectangle offset (APNG/WebP-anim support partial frames;
    /// GIF: always `(0, 0)`).
    pub offset_xy: (i32, i32),
    /// What to do with the canvas region this frame occupies *after*
    /// it has been displayed (before drawing the next frame). GIF and
    /// APNG `dispose_op`.
    pub dispose_op: DisposeOp,
    /// How to combine this frame's pixels with the canvas region
    /// (APNG only — GIF is always source-over with palette
    /// transparency).
    pub blend_op: AnimBlendOp,
    /// Paletted transparency: index into the source palette that is
    /// rendered transparent. GIF-specific; `None` for full-alpha
    /// formats.
    pub transparent_index: Option<u32>,
}

/// Animation frame disposal. Audit A-C3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisposeOp {
    /// Leave the canvas as-is (next frame draws on top).
    #[default]
    None,
    /// Clear the frame region to the background colour.
    Background,
    /// Restore the canvas to the previous frame.
    Previous,
}

/// Animation frame blend operation (APNG `blend_op`). Audit A-C3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimBlendOp {
    /// Replace the canvas region with the frame's pixels
    /// (`BLEND_OP_SOURCE` in APNG spec).
    #[default]
    Source,
    /// Composite the frame over the canvas region using its alpha
    /// (`BLEND_OP_OVER`).
    Over,
}

/// Vector document preserved from SVG/PDF — `kurbo::BezPath` paint
/// stack plus transforms and clip paths.
///
/// **Wave 3 stub.** Modelled in W0 so the `DecodedImage` cap locks at
/// 5 before W3 populates the body. Until then, the variant exists but
/// no importer constructs it.
///
/// **SMIL decision (audit A-H3)**: SVG SMIL animation rasterizes to
/// `DecodedImage::Animated` at the importer's default DPI; preserving
/// vector-animation would need a 6th `DecodedImage` variant which is
/// out of cap scope. Documented in `DecodedImage::Animated` doc.
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
        let _ = DecodedImage::Vector(VectorDoc::default());
    }

    #[test]
    fn blend_mode_default_is_normal() {
        assert_eq!(BlendMode::default(), BlendMode::Normal);
    }

    #[test]
    fn blend_mode_custom_preserves_opcode_for_roundtrip() {
        // Audit A-C1: PSD unknown blend modes preserve their raw
        // opcode so re-export is byte-exact.
        let custom = BlendMode::Custom(0x1234);
        let BlendMode::Custom(opcode) = custom else {
            panic!("expected Custom");
        };
        assert_eq!(opcode, 0x1234);
    }

    #[test]
    fn layer_kind_default_is_pixel() {
        assert!(matches!(LayerKind::default(), LayerKind::Pixel));
    }

    #[test]
    fn layer_kind_group_holds_children() {
        let g = LayerKind::Group {
            children: vec![Layer {
                version: 1,
                name: "nested".into(),
                kind: LayerKind::Pixel,
                pixels: None,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                visible: true,
                mask: None,
                effects: Vec::new(),
                color_profile: None,
            }],
        };
        if let LayerKind::Group { children } = g {
            assert_eq!(children.len(), 1);
            assert_eq!(children[0].name, "nested");
        } else {
            panic!("expected Group");
        }
    }

    #[test]
    fn anim_frame_disposes_correctly() {
        // Audit A-C3: GIF dispose_op fidelity for round-trip.
        let frame = AnimFrame {
            image: ImageBuffer {
                width: 1,
                height: 1,
                pixels: vec![SrgbRgba([0, 0, 0, 255])],
                color_profile: ColorProfile::Srgb,
            },
            delay_ms: 100,
            offset_xy: (0, 0),
            dispose_op: DisposeOp::Background,
            blend_op: AnimBlendOp::Source,
            transparent_index: Some(0),
        };
        assert_eq!(frame.dispose_op, DisposeOp::Background);
        assert_eq!(frame.blend_op, AnimBlendOp::Source);
        assert_eq!(frame.transparent_index, Some(0));
    }

    #[test]
    fn layer_stack_default_has_version_1() {
        // HR-14: every save-format struct begins with version: u32.
        let stack = LayerStack::default();
        assert_eq!(stack.version, 1);
    }
}
