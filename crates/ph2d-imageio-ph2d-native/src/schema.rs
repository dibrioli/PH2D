//! Wire schema for the `.ph2d` save format. Mirror of
//! [`ph2d_imageio::DecodedImage`] with `#[derive(Serialize, Deserialize)]`
//! so postcard can encode it; conversions to/from `DecodedImage` live
//! at the file boundary so the foundation crate (`ph2d-imageio`) stays
//! free of a serde dep.
//!
//! ### HR-14 versioning
//!
//! Every wire struct begins with `version: u32` (top-level
//! [`Ph2dNativeV1`] + nested [`LayerStackV1`] + [`LayerV1`]). The
//! current schema is v1. Future schema diffs add `Ph2dNativeV2` +
//! `migrate_v1_to_v2(v1) -> v2` and the importer dispatches on the
//! header's `version` field.

use ph2d_color::{LinearRgba, SrgbRgba};
use ph2d_imageio::{
    AnimBlendOp, AnimFrame, BlendMode, ColorProfile, DecodedImage, DisposeOp, Error, ImageBuffer,
    Layer, LayerEffect, LayerEffectKind, LayerKind, LayerStack, MAX_ICC_PROFILE_LEN,
    MAX_RASTER_DIMENSION, VectorDoc,
};
use serde::{Deserialize, Serialize};

/// Top-level wire envelope. Always starts with `version: u32` per HR-14.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ph2dNativeV1 {
    /// Schema version. Always `1` in this struct.
    pub version: u32,
    /// What's stored — one of the 5 [`DecodedImage`] variants.
    pub content: Ph2dContentV1,
}

/// Mirror of [`DecodedImage`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Ph2dContentV1 {
    Flat(ImageBufferSrgbV1),
    FlatHdr(ImageBufferHdrV1),
    Layered(LayerStackV1),
    Animated(Vec<AnimFrameV1>),
    Vector(VectorDocV1),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBufferSrgbV1 {
    pub width: u32,
    pub height: u32,
    /// Pixels as packed [r, g, b, a] sequences; postcard encodes
    /// `Vec<[u8; 4]>` byte-for-byte.
    pub pixels: Vec<[u8; 4]>,
    pub color_profile: ColorProfileV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBufferHdrV1 {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[f32; 4]>,
    pub color_profile: ColorProfileV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBufferMaskV1 {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub color_profile: ColorProfileV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorProfileV1 {
    Srgb,
    DisplayP3,
    AdobeRgb,
    ProPhoto,
    LinearRec709,
    LinearRec2020,
    Custom(Vec<u8>),
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStackV1 {
    pub version: u32,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub layers: Vec<LayerV1>,
    pub color_profile: ColorProfileV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerV1 {
    pub version: u32,
    pub name: String,
    pub kind: LayerKindV1,
    pub pixels: Option<ImageBufferSrgbV1>,
    pub opacity: f32,
    pub blend_mode: BlendModeV1,
    pub visible: bool,
    pub mask: Option<ImageBufferMaskV1>,
    pub effects: Vec<LayerEffectV1>,
    pub color_profile: Option<ColorProfileV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerKindV1 {
    Pixel,
    Group { children: Vec<LayerV1> },
    Adjustment,
    Text,
    Smart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerEffectV1 {
    pub kind: LayerEffectKindV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerEffectKindV1 {
    DropShadow,
    Glow,
    Stroke,
    Bevel,
    ColorOverlay,
    GradientOverlay,
    PatternOverlay,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlendModeV1 {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    LinearBurn,
    LinearDodge,
    LinearLight,
    VividLight,
    PinLight,
    HardMix,
    Divide,
    Subtract,
    Custom(u16),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimFrameV1 {
    pub image: ImageBufferSrgbV1,
    pub delay_ms: u32,
    pub offset_xy: (i32, i32),
    pub dispose_op: DisposeOpV1,
    pub blend_op: AnimBlendOpV1,
    pub transparent_index: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisposeOpV1 {
    None,
    Background,
    Previous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimBlendOpV1 {
    Source,
    Over,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorDocV1 {
    /// W3 expand. Empty in W1.
    _reserved: (),
}

// ----- conversions -----

// ----- inner version + cap validation -----

/// Walk a freshly-decoded [`Ph2dContentV1`] confirming every nested
/// `version: u32` matches the schema this build knows about (= 1),
/// and that no `ColorProfile::Custom` ICC blob exceeds
/// [`MAX_ICC_PROFILE_LEN`].
///
/// Audit E-1 CRITICAL (2026-05-26): without this walk, a Painter-v2
/// `.ph2d` written by a build that ships `LayerStack { version: 2, … }`
/// or `Layer { version: 2, … }` would deserialize fine through serde
/// and postcard (those are dumb integers in v1's shape) and lose every
/// v2-only field silently as `From<LayerV1>` copies the bare integer
/// without validation. This is exactly the data-loss class HR-14 was
/// written to prevent.
pub fn validate_v1_inner_versions(content: &Ph2dContentV1) -> Result<(), Error> {
    match content {
        Ph2dContentV1::Flat(b) => {
            validate_dimensions_v1(b.width, b.height)?;
            validate_color_profile_v1(&b.color_profile)
        }
        Ph2dContentV1::FlatHdr(b) => {
            validate_dimensions_v1(b.width, b.height)?;
            validate_color_profile_v1(&b.color_profile)
        }
        Ph2dContentV1::Layered(stack) => validate_layer_stack_v1(stack),
        Ph2dContentV1::Animated(frames) => {
            for f in frames {
                validate_dimensions_v1(f.image.width, f.image.height)?;
                validate_color_profile_v1(&f.image.color_profile)?;
            }
            Ok(())
        }
        Ph2dContentV1::Vector(_) => Ok(()),
    }
}

fn validate_layer_stack_v1(s: &LayerStackV1) -> Result<(), Error> {
    if s.version != 1 {
        return Err(Error::Unsupported(format!(
            "LayerStack schema version {} not supported by this build (max: 1)",
            s.version
        )));
    }
    validate_dimensions_v1(s.canvas_width, s.canvas_height)?;
    validate_color_profile_v1(&s.color_profile)?;
    for l in &s.layers {
        validate_layer_v1(l)?;
    }
    Ok(())
}

fn validate_layer_v1(l: &LayerV1) -> Result<(), Error> {
    if l.version != 1 {
        return Err(Error::Unsupported(format!(
            "Layer schema version {} not supported by this build (max: 1)",
            l.version
        )));
    }
    if let Some(cp) = &l.color_profile {
        validate_color_profile_v1(cp)?;
    }
    if let Some(b) = &l.pixels {
        validate_dimensions_v1(b.width, b.height)?;
        validate_color_profile_v1(&b.color_profile)?;
    }
    if let Some(m) = &l.mask {
        validate_dimensions_v1(m.width, m.height)?;
    }
    // Recurse into nested Group children.
    if let LayerKindV1::Group { children } = &l.kind {
        for child in children {
            validate_layer_v1(child)?;
        }
    }
    Ok(())
}

/// Audit-7 Lens J CRITICAL J-3 (2026-05-26): post-deserialize walker
/// that enforces [`MAX_RASTER_DIMENSION`] on every nested
/// `ImageBuffer*` in the tree. The pre-existing `MAX_PH2D_PAYLOAD_LEN`
/// cap (4 GiB) bounds the raw bytes, but a postcard payload can
/// legitimately fit under 4 GiB while declaring multiple nested
/// `width: 65535, height: 65535` raster fields that, when reified
/// into `Vec<SrgbRgba>`, blow past the per-buffer budget. This walker
/// catches that class before the `From<ImageBufferSrgbV1>` reify
/// allocates anything.
fn validate_dimensions_v1(width: u32, height: u32) -> Result<(), Error> {
    if width as u64 > MAX_RASTER_DIMENSION as u64 || height as u64 > MAX_RASTER_DIMENSION as u64 {
        return Err(Error::DimensionExceedsLimit);
    }
    Ok(())
}

fn validate_color_profile_v1(cp: &ColorProfileV1) -> Result<(), Error> {
    if let ColorProfileV1::Custom(bytes) = cp
        && bytes.len() > MAX_ICC_PROFILE_LEN
    {
        return Err(Error::DimensionExceedsLimit);
    }
    Ok(())
}

// ----- conversions -----

impl From<&ColorProfile> for ColorProfileV1 {
    fn from(p: &ColorProfile) -> Self {
        match p {
            ColorProfile::Srgb => Self::Srgb,
            ColorProfile::DisplayP3 => Self::DisplayP3,
            ColorProfile::AdobeRgb => Self::AdobeRgb,
            ColorProfile::ProPhoto => Self::ProPhoto,
            ColorProfile::LinearRec709 => Self::LinearRec709,
            ColorProfile::LinearRec2020 => Self::LinearRec2020,
            ColorProfile::Custom(b) => Self::Custom(b.clone()),
            ColorProfile::Unknown => Self::Unknown,
        }
    }
}

impl From<ColorProfileV1> for ColorProfile {
    fn from(p: ColorProfileV1) -> Self {
        match p {
            ColorProfileV1::Srgb => Self::Srgb,
            ColorProfileV1::DisplayP3 => Self::DisplayP3,
            ColorProfileV1::AdobeRgb => Self::AdobeRgb,
            ColorProfileV1::ProPhoto => Self::ProPhoto,
            ColorProfileV1::LinearRec709 => Self::LinearRec709,
            ColorProfileV1::LinearRec2020 => Self::LinearRec2020,
            ColorProfileV1::Custom(b) => Self::Custom(b),
            ColorProfileV1::Unknown => Self::Unknown,
        }
    }
}

impl From<&BlendMode> for BlendModeV1 {
    fn from(b: &BlendMode) -> Self {
        match b {
            BlendMode::Normal => Self::Normal,
            BlendMode::Multiply => Self::Multiply,
            BlendMode::Screen => Self::Screen,
            BlendMode::Overlay => Self::Overlay,
            BlendMode::Darken => Self::Darken,
            BlendMode::Lighten => Self::Lighten,
            BlendMode::ColorDodge => Self::ColorDodge,
            BlendMode::ColorBurn => Self::ColorBurn,
            BlendMode::HardLight => Self::HardLight,
            BlendMode::SoftLight => Self::SoftLight,
            BlendMode::Difference => Self::Difference,
            BlendMode::Exclusion => Self::Exclusion,
            BlendMode::Hue => Self::Hue,
            BlendMode::Saturation => Self::Saturation,
            BlendMode::Color => Self::Color,
            BlendMode::Luminosity => Self::Luminosity,
            BlendMode::LinearBurn => Self::LinearBurn,
            BlendMode::LinearDodge => Self::LinearDodge,
            BlendMode::LinearLight => Self::LinearLight,
            BlendMode::VividLight => Self::VividLight,
            BlendMode::PinLight => Self::PinLight,
            BlendMode::HardMix => Self::HardMix,
            BlendMode::Divide => Self::Divide,
            BlendMode::Subtract => Self::Subtract,
            BlendMode::Custom(op) => Self::Custom(*op),
        }
    }
}

impl From<BlendModeV1> for BlendMode {
    fn from(b: BlendModeV1) -> Self {
        match b {
            BlendModeV1::Normal => Self::Normal,
            BlendModeV1::Multiply => Self::Multiply,
            BlendModeV1::Screen => Self::Screen,
            BlendModeV1::Overlay => Self::Overlay,
            BlendModeV1::Darken => Self::Darken,
            BlendModeV1::Lighten => Self::Lighten,
            BlendModeV1::ColorDodge => Self::ColorDodge,
            BlendModeV1::ColorBurn => Self::ColorBurn,
            BlendModeV1::HardLight => Self::HardLight,
            BlendModeV1::SoftLight => Self::SoftLight,
            BlendModeV1::Difference => Self::Difference,
            BlendModeV1::Exclusion => Self::Exclusion,
            BlendModeV1::Hue => Self::Hue,
            BlendModeV1::Saturation => Self::Saturation,
            BlendModeV1::Color => Self::Color,
            BlendModeV1::Luminosity => Self::Luminosity,
            BlendModeV1::LinearBurn => Self::LinearBurn,
            BlendModeV1::LinearDodge => Self::LinearDodge,
            BlendModeV1::LinearLight => Self::LinearLight,
            BlendModeV1::VividLight => Self::VividLight,
            BlendModeV1::PinLight => Self::PinLight,
            BlendModeV1::HardMix => Self::HardMix,
            BlendModeV1::Divide => Self::Divide,
            BlendModeV1::Subtract => Self::Subtract,
            BlendModeV1::Custom(op) => Self::Custom(op),
        }
    }
}

impl From<&LayerEffectKind> for LayerEffectKindV1 {
    fn from(k: &LayerEffectKind) -> Self {
        match k {
            LayerEffectKind::DropShadow => Self::DropShadow,
            LayerEffectKind::Glow => Self::Glow,
            LayerEffectKind::Stroke => Self::Stroke,
            LayerEffectKind::Bevel => Self::Bevel,
            LayerEffectKind::ColorOverlay => Self::ColorOverlay,
            LayerEffectKind::GradientOverlay => Self::GradientOverlay,
            LayerEffectKind::PatternOverlay => Self::PatternOverlay,
            LayerEffectKind::Custom(s) => Self::Custom(s.clone()),
        }
    }
}

impl From<LayerEffectKindV1> for LayerEffectKind {
    fn from(k: LayerEffectKindV1) -> Self {
        match k {
            LayerEffectKindV1::DropShadow => Self::DropShadow,
            LayerEffectKindV1::Glow => Self::Glow,
            LayerEffectKindV1::Stroke => Self::Stroke,
            LayerEffectKindV1::Bevel => Self::Bevel,
            LayerEffectKindV1::ColorOverlay => Self::ColorOverlay,
            LayerEffectKindV1::GradientOverlay => Self::GradientOverlay,
            LayerEffectKindV1::PatternOverlay => Self::PatternOverlay,
            LayerEffectKindV1::Custom(s) => Self::Custom(s),
        }
    }
}

impl From<&DisposeOp> for DisposeOpV1 {
    fn from(d: &DisposeOp) -> Self {
        match d {
            DisposeOp::None => Self::None,
            DisposeOp::Background => Self::Background,
            DisposeOp::Previous => Self::Previous,
        }
    }
}

impl From<DisposeOpV1> for DisposeOp {
    fn from(d: DisposeOpV1) -> Self {
        match d {
            DisposeOpV1::None => Self::None,
            DisposeOpV1::Background => Self::Background,
            DisposeOpV1::Previous => Self::Previous,
        }
    }
}

impl From<&AnimBlendOp> for AnimBlendOpV1 {
    fn from(b: &AnimBlendOp) -> Self {
        match b {
            AnimBlendOp::Source => Self::Source,
            AnimBlendOp::Over => Self::Over,
        }
    }
}

impl From<AnimBlendOpV1> for AnimBlendOp {
    fn from(b: AnimBlendOpV1) -> Self {
        match b {
            AnimBlendOpV1::Source => Self::Source,
            AnimBlendOpV1::Over => Self::Over,
        }
    }
}

impl From<&ImageBuffer<SrgbRgba>> for ImageBufferSrgbV1 {
    fn from(b: &ImageBuffer<SrgbRgba>) -> Self {
        Self {
            width: b.width,
            height: b.height,
            pixels: b.pixels.iter().map(|p| p.0).collect(),
            color_profile: (&b.color_profile).into(),
        }
    }
}

impl From<ImageBufferSrgbV1> for ImageBuffer<SrgbRgba> {
    fn from(b: ImageBufferSrgbV1) -> Self {
        Self {
            width: b.width,
            height: b.height,
            pixels: b.pixels.into_iter().map(SrgbRgba).collect(),
            color_profile: b.color_profile.into(),
        }
    }
}

impl From<&ImageBuffer<LinearRgba>> for ImageBufferHdrV1 {
    fn from(b: &ImageBuffer<LinearRgba>) -> Self {
        Self {
            width: b.width,
            height: b.height,
            pixels: b.pixels.iter().map(|p| p.as_array()).collect(),
            color_profile: (&b.color_profile).into(),
        }
    }
}

impl From<ImageBufferHdrV1> for ImageBuffer<LinearRgba> {
    fn from(b: ImageBufferHdrV1) -> Self {
        Self {
            width: b.width,
            height: b.height,
            pixels: b
                .pixels
                .into_iter()
                .map(|a| LinearRgba::new(a[0], a[1], a[2], a[3]))
                .collect(),
            color_profile: b.color_profile.into(),
        }
    }
}

impl From<&ImageBuffer<u8>> for ImageBufferMaskV1 {
    fn from(b: &ImageBuffer<u8>) -> Self {
        Self {
            width: b.width,
            height: b.height,
            pixels: b.pixels.clone(),
            color_profile: (&b.color_profile).into(),
        }
    }
}

impl From<ImageBufferMaskV1> for ImageBuffer<u8> {
    fn from(b: ImageBufferMaskV1) -> Self {
        Self {
            width: b.width,
            height: b.height,
            pixels: b.pixels,
            color_profile: b.color_profile.into(),
        }
    }
}

impl From<&LayerEffect> for LayerEffectV1 {
    fn from(e: &LayerEffect) -> Self {
        Self {
            kind: (&e.kind).into(),
        }
    }
}

impl From<LayerEffectV1> for LayerEffect {
    fn from(e: LayerEffectV1) -> Self {
        Self {
            kind: e.kind.into(),
        }
    }
}

impl From<&LayerKind> for LayerKindV1 {
    fn from(k: &LayerKind) -> Self {
        match k {
            LayerKind::Pixel => Self::Pixel,
            LayerKind::Group { children } => Self::Group {
                children: children.iter().map(LayerV1::from).collect(),
            },
            LayerKind::Adjustment => Self::Adjustment,
            LayerKind::Text => Self::Text,
            LayerKind::Smart => Self::Smart,
        }
    }
}

impl From<LayerKindV1> for LayerKind {
    fn from(k: LayerKindV1) -> Self {
        match k {
            LayerKindV1::Pixel => Self::Pixel,
            LayerKindV1::Group { children } => Self::Group {
                children: children.into_iter().map(Layer::from).collect(),
            },
            LayerKindV1::Adjustment => Self::Adjustment,
            LayerKindV1::Text => Self::Text,
            LayerKindV1::Smart => Self::Smart,
        }
    }
}

impl From<&Layer> for LayerV1 {
    fn from(l: &Layer) -> Self {
        Self {
            version: l.version,
            name: l.name.clone(),
            kind: (&l.kind).into(),
            pixels: l.pixels.as_ref().map(ImageBufferSrgbV1::from),
            opacity: l.opacity,
            blend_mode: (&l.blend_mode).into(),
            visible: l.visible,
            mask: l.mask.as_ref().map(ImageBufferMaskV1::from),
            effects: l.effects.iter().map(LayerEffectV1::from).collect(),
            color_profile: l.color_profile.as_ref().map(ColorProfileV1::from),
        }
    }
}

impl From<LayerV1> for Layer {
    fn from(l: LayerV1) -> Self {
        Self {
            version: l.version,
            name: l.name,
            kind: l.kind.into(),
            pixels: l.pixels.map(ImageBuffer::<SrgbRgba>::from),
            opacity: l.opacity,
            blend_mode: l.blend_mode.into(),
            visible: l.visible,
            mask: l.mask.map(ImageBuffer::<u8>::from),
            effects: l.effects.into_iter().map(LayerEffect::from).collect(),
            color_profile: l.color_profile.map(ColorProfile::from),
        }
    }
}

impl From<&LayerStack> for LayerStackV1 {
    fn from(s: &LayerStack) -> Self {
        Self {
            version: s.version,
            canvas_width: s.canvas_width,
            canvas_height: s.canvas_height,
            layers: s.layers.iter().map(LayerV1::from).collect(),
            color_profile: (&s.color_profile).into(),
        }
    }
}

impl From<LayerStackV1> for LayerStack {
    fn from(s: LayerStackV1) -> Self {
        Self {
            version: s.version,
            canvas_width: s.canvas_width,
            canvas_height: s.canvas_height,
            layers: s.layers.into_iter().map(Layer::from).collect(),
            color_profile: s.color_profile.into(),
        }
    }
}

impl From<&AnimFrame> for AnimFrameV1 {
    fn from(f: &AnimFrame) -> Self {
        Self {
            image: (&f.image).into(),
            delay_ms: f.delay_ms,
            offset_xy: f.offset_xy,
            dispose_op: (&f.dispose_op).into(),
            blend_op: (&f.blend_op).into(),
            transparent_index: f.transparent_index,
        }
    }
}

impl From<AnimFrameV1> for AnimFrame {
    fn from(f: AnimFrameV1) -> Self {
        Self {
            image: f.image.into(),
            delay_ms: f.delay_ms,
            offset_xy: f.offset_xy,
            dispose_op: f.dispose_op.into(),
            blend_op: f.blend_op.into(),
            transparent_index: f.transparent_index,
        }
    }
}

impl From<&VectorDoc> for VectorDocV1 {
    fn from(_v: &VectorDoc) -> Self {
        Self::default()
    }
}

impl From<VectorDocV1> for VectorDoc {
    fn from(_v: VectorDocV1) -> Self {
        Self::default()
    }
}

impl From<&DecodedImage> for Ph2dContentV1 {
    fn from(d: &DecodedImage) -> Self {
        match d {
            DecodedImage::Flat(b) => Self::Flat(b.into()),
            DecodedImage::FlatHdr(b) => Self::FlatHdr(b.into()),
            DecodedImage::Layered(s) => Self::Layered(s.into()),
            DecodedImage::Animated(frames) => {
                Self::Animated(frames.iter().map(AnimFrameV1::from).collect())
            }
            DecodedImage::Vector(v) => Self::Vector(v.into()),
        }
    }
}

impl From<Ph2dContentV1> for DecodedImage {
    fn from(c: Ph2dContentV1) -> Self {
        match c {
            Ph2dContentV1::Flat(b) => Self::Flat(b.into()),
            Ph2dContentV1::FlatHdr(b) => Self::FlatHdr(b.into()),
            Ph2dContentV1::Layered(s) => Self::Layered(s.into()),
            Ph2dContentV1::Animated(frames) => {
                Self::Animated(frames.into_iter().map(AnimFrame::from).collect())
            }
            Ph2dContentV1::Vector(v) => Self::Vector(v.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_color::SrgbRgba;

    /// Audit-8 Lens M (2026-05-26): boundary check for the J-3 walker.
    /// Without these, a refactor that loosens the per-buffer dimension
    /// check regresses an OOM defence silently.
    #[test]
    fn validate_dimensions_v1_accepts_exact_max_raster() {
        let buf = ImageBufferSrgbV1 {
            width: ph2d_imageio::MAX_RASTER_DIMENSION,
            height: ph2d_imageio::MAX_RASTER_DIMENSION,
            pixels: Vec::new(),
            color_profile: ColorProfileV1::Srgb,
        };
        let content = Ph2dContentV1::Flat(buf);
        assert!(validate_v1_inner_versions(&content).is_ok());
    }

    #[test]
    fn validate_dimensions_v1_rejects_above_max_raster() {
        let buf = ImageBufferSrgbV1 {
            width: ph2d_imageio::MAX_RASTER_DIMENSION + 1,
            height: 1,
            pixels: Vec::new(),
            color_profile: ColorProfileV1::Srgb,
        };
        let content = Ph2dContentV1::Flat(buf);
        assert!(matches!(
            validate_v1_inner_versions(&content),
            Err(Error::DimensionExceedsLimit)
        ));
    }

    #[test]
    fn validate_dimensions_v1_rejects_height_above_max_raster() {
        let buf = ImageBufferSrgbV1 {
            width: 1,
            height: ph2d_imageio::MAX_RASTER_DIMENSION + 1,
            pixels: Vec::new(),
            color_profile: ColorProfileV1::Srgb,
        };
        let content = Ph2dContentV1::Flat(buf);
        assert!(matches!(
            validate_v1_inner_versions(&content),
            Err(Error::DimensionExceedsLimit)
        ));
    }

    /// Pixel field unused in the assertion — silence warning.
    #[allow(dead_code)]
    fn _silence(_: SrgbRgba) {}
}

