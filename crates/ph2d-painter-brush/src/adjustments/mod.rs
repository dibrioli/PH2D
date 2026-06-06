//! Adjustment-layer contract surface (ADR-0045 + [`0045-amendment-1`]).
//!
//! FROZEN CONTRACT (gate `architecture_painter_contract_surface::adjustments`):
//! - [`AdjustmentKind`] ≤ 32 variants (v1 ship 24) — §2.3
//! - [`AdjustmentParams`] ≤ 32 variants (variant name == kind) — §2.5
//! - [`DestructiveAdjustment`] ≤ 8 variants (v1 = 5) — §2.4
//! - [`AdjustmentLayer`] ≤ 12 fields — §2.2
//! - per-kind sub-`*Params` structs with the field caps in §2.6.
//!
//! This module defines ONLY the data + sensible `Default`s + serde. The compute
//! logic (`apply_adjustment(kind, params, &mut [[f32; 4]])` per ADR-0045 §2.7,
//! plus the W4-triage Coord decision — straight LINEAR f32 acc, not 8-bit, so
//! the per-frame composite never round-trips through sRGB8) is the
//! implementer's (T4.3+). T4.2 ships the no-op stub + the compositor wiring.
//!
//! **Amendment-1 crate-placement:** `AdjustmentLayer.{id, clipped_by, mask}` are
//! raw `u64` (LayerId values), not the `LayerId` newtype, because `LayerId` lives
//! in `ph2d-tool-painter` (which depends on this crate — a cycle). The
//! `LayerStack` converts at the boundary (`LayerId(x)` / `x.0`).

use crate::blend::BlendMode;
use ph2d_color::oklab::OklabColor;
use ph2d_color::{LinearRgba, OklchColor};
use serde::{Deserialize, Serialize};

// ─────────────────────────── shared sub-types ───────────────────────────

/// Tonal range a tone-scoped adjustment targets.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ToneScope {
    Shadows,
    #[default]
    Midtones,
    Highlights,
}

/// Interpolation between gradient-map stops.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GradientInterp {
    #[default]
    Linear,
    Smooth,
}

/// Noise distribution.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NoiseKind {
    #[default]
    Gaussian,
    Uniform,
}

/// Halftone cell shape.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HalftoneShape {
    #[default]
    Dot,
    Line,
    Circle,
}

/// Selective-color application method.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SelectiveMethod {
    #[default]
    Relative,
    Absolute,
}

/// `.cube` LUT cache handle (resolved to the cached 3D LUT at compute time).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LutHandle(pub u64);

/// A curve's control points (≤ 8 per channel, §2.6). Normalized `[x, y]` in
/// `0..=1`.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ControlPoints {
    pub points: Vec<[f32; 2]>,
}

/// A gradient-map stop: offset `0..=1` + sRGB8 color.
#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ColorStop {
    pub offset: f32,
    pub color: [u8; 4],
}

/// Per-color CMYK adjustment for Selective Color (§2.6).
#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct CmykAdjust {
    pub cyan: f32,
    pub magenta: f32,
    pub yellow: f32,
    pub black: f32,
}

// ─────────────────────────── Tier 1 sub-params ──────────────────────────

/// `HueSaturationBrightness` — Day-4 smoke kind. h in turns, s/b in `-1..=1`.
#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct HsbParams {
    pub h: f32,
    pub s: f32,
    pub b: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ColorBalanceParams {
    pub cyan_red: f32,
    pub magenta_green: f32,
    pub yellow_blue: f32,
    pub scope: ToneScope,
    pub preserve_luminosity: bool,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct CurvesParams {
    pub points_rgb: ControlPoints,
    pub points_r: ControlPoints,
    pub points_g: ControlPoints,
    pub points_b: ControlPoints,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GradientMapParams {
    pub stops: Vec<ColorStop>,
    pub interpolation: GradientInterp,
}

impl Default for GradientMapParams {
    /// A black→white duotone — the canonical neutral Gradient Map (maps luma to a
    /// grayscale ramp). (A derived empty-`stops` default would have no gradient to
    /// sample; the duotone editor also relies on the two endpoint stops existing.)
    fn default() -> Self {
        Self {
            stops: vec![
                ColorStop {
                    offset: 0.0,
                    color: [0, 0, 0, 255],
                },
                ColorStop {
                    offset: 1.0,
                    color: [255, 255, 255, 255],
                },
            ],
            interpolation: GradientInterp::Linear,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct BrightnessContrastParams {
    pub brightness: f32,
    pub contrast: f32,
    pub legacy: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct GaussianBlurParams {
    pub radius: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct MotionBlurParams {
    pub distance: f32,
    pub angle: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct BloomParams {
    pub threshold: f32,
    pub intensity: f32,
    pub radius: f32,
    pub falloff: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct NoiseParams {
    pub amount: f32,
    pub kind: NoiseKind,
    pub monochromatic: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SharpenParams {
    pub amount: f32,
    pub radius: f32,
    pub mask_edges: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct HalftoneParams {
    pub dot_size: f32,
    pub angle: f32,
    pub shape: HalftoneShape,
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ChromaticAberrationParams {
    pub red_shift: f32,
    pub green_shift: f32,
    pub blue_shift: f32,
    pub falloff_center: f32,
}

// ─────────────────────────── Tier 2 sub-params ──────────────────────────

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct VibranceParams {
    pub vibrance: f32,
    pub saturation: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColorLookupLutParams {
    pub lut_3d: LutHandle,
    pub intensity: f32,
    pub profile: LutProfile,
}

impl Default for ColorLookupLutParams {
    /// Neutral on creation — handle `0` (`None`) is a pass-through regardless of
    /// intensity, so the layer is an identity until the user scrubs the "Look".
    /// `intensity` seeds at full (`1.0`) so picking a look is immediately visible;
    /// the user then dials "Amount" back to taste. (A derived all-zero default
    /// would leave intensity at 0 → a picked look would show nothing.)
    fn default() -> Self {
        Self {
            lut_3d: LutHandle(0),
            intensity: 1.0,
            profile: LutProfile::Srgb,
        }
    }
}

/// Color-management profile a LUT is authored in.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LutProfile {
    #[default]
    Srgb,
    Linear,
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PhotoFilterParams {
    pub temperature: f32,
    pub density: f32,
    pub preserve_luminosity: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PosterizeParams {
    /// `2..=32`.
    pub levels: u8,
}

impl Default for PosterizeParams {
    fn default() -> Self {
        Self { levels: 8 }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThresholdParams {
    /// `0..=255`.
    pub threshold: u8,
}

impl Default for ThresholdParams {
    fn default() -> Self {
        Self { threshold: 128 }
    }
}

/// `Invert` is parameterless (a toggle); the struct exists so the discriminated
/// union stays variant==kind uniform.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct InvertParams {}

/// `Levels` — black/gamma/white in + output black/white, flat per §2.6. All
/// fields are `0..=1` except `gamma` (effective midtone power, neutral `1.0`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LevelsParams {
    pub black_point: f32,
    pub gamma: f32,
    pub white_point: f32,
    pub output_black: f32,
    pub output_white: f32,
}

impl Default for LevelsParams {
    /// The NEUTRAL identity — a freshly-created Levels layer is a no-op until the
    /// user drags a handle. (A derived all-zero default would be degenerate:
    /// `white_point == black_point == 0` collapses the input range and `gamma == 0`
    /// is not a valid power.)
    fn default() -> Self {
        Self {
            black_point: 0.0,
            gamma: 1.0,
            white_point: 1.0,
            output_black: 0.0,
            output_white: 1.0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SelectiveColorParams {
    pub reds: CmykAdjust,
    pub yellows: CmykAdjust,
    pub greens: CmykAdjust,
    pub cyans: CmykAdjust,
    pub blues: CmykAdjust,
    pub magentas: CmykAdjust,
    pub whites: CmykAdjust,
    pub neutrals: CmykAdjust,
    pub blacks: CmykAdjust,
    pub method: SelectiveMethod,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelMixerParams {
    /// `[r, g, b, constant]` per output channel.
    pub red_out: [f32; 4],
    pub green_out: [f32; 4],
    pub blue_out: [f32; 4],
    pub monochromatic: bool,
}

impl Default for ChannelMixerParams {
    /// The NEUTRAL identity — a freshly-created Channel Mixer passes R/G/B through
    /// unmixed (the identity matrix) until the user drags a weight. (A derived
    /// all-zero default would be degenerate: every output collapses to black — the
    /// same trap the Levels default avoids.)
    fn default() -> Self {
        Self {
            red_out: [1.0, 0.0, 0.0, 0.0],
            green_out: [0.0, 1.0, 0.0, 0.0],
            blue_out: [0.0, 0.0, 1.0, 0.0],
            monochromatic: false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ExposureParams {
    pub exposure_ev: f32,
    pub offset: f32,
    pub gamma_correction: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ShadowsHighlightsParams {
    pub shadows_amount: f32,
    pub shadows_tonal_width: f32,
    pub shadows_radius: f32,
    pub highlights_amount: f32,
    pub highlights_tonal_width: f32,
    pub highlights_radius: f32,
    pub color_correction: f32,
    pub midtone_contrast: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlackAndWhiteParams {
    pub reds: f32,
    pub yellows: f32,
    pub greens: f32,
    pub cyans: f32,
    pub blues: f32,
    pub magentas: f32,
    pub tint_color: Option<OklchColor>,
    pub tint_amount: f32,
}

impl Default for BlackAndWhiteParams {
    /// Photoshop's default per-hue weights (40/60/40/60/20/80 %) — a sensible
    /// neutral grayscale on creation. (A derived all-zero default would collapse
    /// every pixel to its `min(r,g,b)` — a too-dark, hue-blind conversion; the
    /// same degenerate-default trap as Levels / Channel Mixer.)
    fn default() -> Self {
        Self {
            reds: 0.4,
            yellows: 0.6,
            greens: 0.4,
            cyans: 0.6,
            blues: 0.2,
            magentas: 0.8,
            tint_color: None,
            tint_amount: 0.0,
        }
    }
}

// ───────────────────────────── the enums ────────────────────────────────

/// Non-destructive adjustment kinds — cap ≤ 32 (v1 ship 24). §2.3.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdjustmentKind {
    // Tier 1
    HueSaturationBrightness,
    ColorBalance,
    Curves,
    GradientMap,
    BrightnessContrast,
    GaussianBlur,
    MotionBlur,
    Bloom,
    Noise,
    Sharpen,
    Halftone,
    ChromaticAberration,
    // Tier 2
    Vibrance,
    ColorLookupLut,
    PhotoFilter,
    Posterize,
    Threshold,
    Invert,
    Levels,
    SelectiveColor,
    ChannelMixer,
    Exposure,
    ShadowsHighlights,
    BlackAndWhite,
}

impl AdjustmentKind {
    /// Every v1 [`AdjustmentKind`], in canonical menu order (the "+ Adjustment"
    /// picker iterates this, and the layout-stable index is the wire value the
    /// panel forwards back to `add_adjustment_layer`). The order mirrors the enum
    /// and the params discriminated union; keep all three in lock-step. Tier 1 is
    /// the first 12 (the `psd_mapping_is_canonical` gate asserts the counts).
    pub const ALL: [AdjustmentKind; 24] = [
        // Tier 1
        Self::HueSaturationBrightness,
        Self::ColorBalance,
        Self::Curves,
        Self::GradientMap,
        Self::BrightnessContrast,
        Self::GaussianBlur,
        Self::MotionBlur,
        Self::Bloom,
        Self::Noise,
        Self::Sharpen,
        Self::Halftone,
        Self::ChromaticAberration,
        // Tier 2
        Self::Vibrance,
        Self::ColorLookupLut,
        Self::PhotoFilter,
        Self::Posterize,
        Self::Threshold,
        Self::Invert,
        Self::Levels,
        Self::SelectiveColor,
        Self::ChannelMixer,
        Self::Exposure,
        Self::ShadowsHighlights,
        Self::BlackAndWhite,
    ];

    /// Human-readable name for the "+ Adjustment" menu + the layer-row label
    /// (English — UI is always English, [[feedback-app-ui-english-only]]).
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::HueSaturationBrightness => "Hue/Saturation",
            Self::ColorBalance => "Color Balance",
            Self::Curves => "Curves",
            Self::GradientMap => "Gradient Map",
            Self::BrightnessContrast => "Brightness/Contrast",
            Self::GaussianBlur => "Gaussian Blur",
            Self::MotionBlur => "Motion Blur",
            Self::Bloom => "Bloom",
            Self::Noise => "Noise",
            Self::Sharpen => "Sharpen",
            Self::Halftone => "Halftone",
            Self::ChromaticAberration => "Chromatic Aberration",
            Self::Vibrance => "Vibrance",
            Self::ColorLookupLut => "Color Lookup",
            Self::PhotoFilter => "Photo Filter",
            Self::Posterize => "Posterize",
            Self::Threshold => "Threshold",
            Self::Invert => "Invert",
            Self::Levels => "Levels",
            Self::SelectiveColor => "Selective Color",
            Self::ChannelMixer => "Channel Mixer",
            Self::Exposure => "Exposure",
            Self::ShadowsHighlights => "Shadows/Highlights",
            Self::BlackAndWhite => "Black & White",
        }
    }

    /// GPU adjustment-kernel code for the real-time compositor
    /// (`ph2d-render::layer_composite.wgsl` `ADJ_*` / `apply_adjustment`), or
    /// `None` for a kind the GPU shader does not implement yet (the compositor
    /// falls back to the CPU path for those). This is the tool↔shader contract —
    /// the painter flatten emits `LayerOp::Adjustment { kind: gpu_code(), .. }`.
    /// Keep in lock-step with the WGSL `ADJ_*` consts + the GPU parity gate
    /// `gpu_adjustment_matches_cpu_reference_each_kind`.
    #[must_use]
    pub fn gpu_code(self) -> Option<u8> {
        Some(match self {
            Self::HueSaturationBrightness => 0,
            Self::BrightnessContrast => 1,
            Self::Invert => 2,
            Self::Posterize => 3,
            Self::Threshold => 4,
            Self::Exposure => 5,
            Self::Vibrance => 6,
            // W4 bespoke — display-space 1-D transfer LUTs uploaded to the
            // compositor's binding-6 `adj_luts` (Curves = 3×256, Levels = 1×256).
            Self::Curves => 7,
            Self::Levels => 8,
            // Not yet ported to the per-pixel GPU shader. Spatial kinds run on
            // the multi-pass pass-graph instead — see `gpu_spatial_code`.
            _ => return None,
        })
    }

    /// GPU **spatial** kernel code for the multi-pass pass-graph
    /// (`ph2d-render::LayerCompositor` `SpatialAdjustment` path), or `None` for a
    /// kind that is not a (ported) spatial/neighbourhood op. These are the kinds
    /// `gpu_code` returns `None` for AND the compositor can run as a separable /
    /// gather pass: the painter flatten emits `LayerOp::SpatialAdjustment { kernel:
    /// gpu_spatial_code(), .. }` for them (vs. the scalar `LayerOp::Adjustment` for
    /// `gpu_code` kinds, vs. the CPU fallback for kinds with neither).
    ///
    /// The codes MIRROR `ph2d_render::layer_compositor::SPATIAL_*` (kept in
    /// lock-step the same way `gpu_code` mirrors the WGSL `ADJ_*` consts — no
    /// `ph2d-render` dependency here, that would be a cycle). Reconciled with the
    /// pass-graph by the spatial parity gates (`gpu_<kind>_matches_cpu_reference`).
    ///
    /// `Bloom` / `ShadowsHighlights` are spatial too but need extra pass-graph
    /// infra (mip pyramid / tonal combine) that has not landed — they stay `None`
    /// (CPU fallback) until their kernel ships.
    #[must_use]
    pub fn gpu_spatial_code(self) -> Option<u8> {
        Some(match self {
            Self::GaussianBlur => 0,        // SPATIAL_GAUSSIAN
            Self::Sharpen => 1,             // SPATIAL_SHARPEN
            Self::MotionBlur => 2,          // SPATIAL_MOTION
            Self::ChromaticAberration => 3, // SPATIAL_CHROMA
            _ => return None,
        })
    }
}

/// Destructive-only adjustments — cap ≤ 8 (v1 = 5). Separate enum so the type
/// system blocks `AdjustmentLayer { kind: Liquify }` (§2.4).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DestructiveAdjustment {
    Liquify,
    Clone,
    Recolor,
    Glitch,
    MeshWarp,
}

/// Typed per-kind params (discriminated union; variant name == [`AdjustmentKind`]).
/// Cap ≤ 32 (v1 ship 24). §2.5.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AdjustmentParams {
    // Tier 1
    HueSaturationBrightness(HsbParams),
    ColorBalance(ColorBalanceParams),
    Curves(CurvesParams),
    GradientMap(GradientMapParams),
    BrightnessContrast(BrightnessContrastParams),
    GaussianBlur(GaussianBlurParams),
    MotionBlur(MotionBlurParams),
    Bloom(BloomParams),
    Noise(NoiseParams),
    Sharpen(SharpenParams),
    Halftone(HalftoneParams),
    ChromaticAberration(ChromaticAberrationParams),
    // Tier 2
    Vibrance(VibranceParams),
    ColorLookupLut(ColorLookupLutParams),
    PhotoFilter(PhotoFilterParams),
    Posterize(PosterizeParams),
    Threshold(ThresholdParams),
    Invert(InvertParams),
    Levels(LevelsParams),
    SelectiveColor(SelectiveColorParams),
    ChannelMixer(ChannelMixerParams),
    Exposure(ExposureParams),
    ShadowsHighlights(ShadowsHighlightsParams),
    BlackAndWhite(BlackAndWhiteParams),
}

impl AdjustmentParams {
    /// The [`AdjustmentKind`] this params variant corresponds to.
    #[must_use]
    pub fn kind(&self) -> AdjustmentKind {
        match self {
            Self::HueSaturationBrightness(_) => AdjustmentKind::HueSaturationBrightness,
            Self::ColorBalance(_) => AdjustmentKind::ColorBalance,
            Self::Curves(_) => AdjustmentKind::Curves,
            Self::GradientMap(_) => AdjustmentKind::GradientMap,
            Self::BrightnessContrast(_) => AdjustmentKind::BrightnessContrast,
            Self::GaussianBlur(_) => AdjustmentKind::GaussianBlur,
            Self::MotionBlur(_) => AdjustmentKind::MotionBlur,
            Self::Bloom(_) => AdjustmentKind::Bloom,
            Self::Noise(_) => AdjustmentKind::Noise,
            Self::Sharpen(_) => AdjustmentKind::Sharpen,
            Self::Halftone(_) => AdjustmentKind::Halftone,
            Self::ChromaticAberration(_) => AdjustmentKind::ChromaticAberration,
            Self::Vibrance(_) => AdjustmentKind::Vibrance,
            Self::ColorLookupLut(_) => AdjustmentKind::ColorLookupLut,
            Self::PhotoFilter(_) => AdjustmentKind::PhotoFilter,
            Self::Posterize(_) => AdjustmentKind::Posterize,
            Self::Threshold(_) => AdjustmentKind::Threshold,
            Self::Invert(_) => AdjustmentKind::Invert,
            Self::Levels(_) => AdjustmentKind::Levels,
            Self::SelectiveColor(_) => AdjustmentKind::SelectiveColor,
            Self::ChannelMixer(_) => AdjustmentKind::ChannelMixer,
            Self::Exposure(_) => AdjustmentKind::Exposure,
            Self::ShadowsHighlights(_) => AdjustmentKind::ShadowsHighlights,
            Self::BlackAndWhite(_) => AdjustmentKind::BlackAndWhite,
        }
    }

    /// The ≤3 scalar params the GPU shader reads (`layer_composite.wgsl`
    /// `apply_adjustment`), in `(p0, p1, p2)` order. The tool↔shader contract:
    /// the painter flatten emits `LayerOp::Adjustment { params: gpu_params(), .. }`
    /// alongside [`AdjustmentKind::gpu_code`]. Mirrors the WGSL param meaning per
    /// kind (validated by `gpu_adjustment_matches_cpu_reference_each_kind`).
    /// Kinds without a GPU code return zeros (unused — the compositor uses the
    /// CPU path for them).
    #[must_use]
    pub fn gpu_params(&self) -> [f32; 3] {
        match self {
            Self::HueSaturationBrightness(p) => [p.h, p.s, p.b],
            Self::BrightnessContrast(p) => [p.brightness, p.contrast, 0.0],
            Self::Invert(_) => [0.0, 0.0, 0.0],
            Self::Posterize(p) => [p.levels as f32, 0.0, 0.0],
            // Threshold's shader cut is normalized (`luma >= p0`); the CPU stores
            // a `0..=255` byte, so divide to match `apply_threshold`.
            Self::Threshold(p) => [p.threshold as f32 / 255.0, 0.0, 0.0],
            Self::Exposure(p) => [p.exposure_ev, p.offset, p.gamma_correction],
            Self::Vibrance(p) => [p.vibrance, p.saturation, 0.0],
            // W4 BATCH-1 — Photo Filter fits the ≤3 scalar GPU rack: temperature,
            // density, preserve-luminosity (as 0/1). This packing is INERT until
            // the Coord lands the `ADJ_PHOTO_FILTER` WGSL case + flips
            // `gpu_code(PhotoFilter)` from `None` to its code (CPU-first phase, the
            // compositor uses `apply_photo_filter` until then). See the W4 handoff.
            Self::PhotoFilter(p) => [
                p.temperature,
                p.density,
                if p.preserve_luminosity { 1.0 } else { 0.0 },
            ],
            _ => [0.0, 0.0, 0.0],
        }
    }

    /// The 4 scalars the spatial pass-graph reads (`LayerOp::SpatialAdjustment`
    /// `params: [f32; 4]`), or `None` for a non-spatial kind. Packing mirrors
    /// `ph2d_render::layer_compositor::SPATIAL_*` (validated by the spatial parity
    /// gates), in lock-step with [`AdjustmentKind::gpu_spatial_code`]:
    /// - `GaussianBlur` → `[radius, 0, 0, 0]`
    /// - `Sharpen` → `[amount, radius, 0, 0]` (unsharp: `base + amount·(base−blur)`)
    /// - `MotionBlur` → `[distance, angle_rad, 0, 0]`
    /// - `ChromaticAberration` → `[red_shift, green_shift, blue_shift, falloff_center]`
    ///
    /// The painter flatten passes this verbatim into the op so the GPU and the CPU
    /// reference (`apply_*` in `compute.rs`) read identical numbers.
    #[must_use]
    pub fn spatial_params(&self) -> Option<[f32; 4]> {
        Some(match self {
            Self::GaussianBlur(p) => [p.radius, 0.0, 0.0, 0.0],
            Self::Sharpen(p) => [p.amount, p.radius, 0.0, 0.0],
            Self::MotionBlur(p) => [p.distance, p.angle, 0.0, 0.0],
            Self::ChromaticAberration(p) => {
                [p.red_shift, p.green_shift, p.blue_shift, p.falloff_center]
            }
            _ => return None,
        })
    }

    /// Neutral (no-op) params for `kind` — the seed when a new adjustment layer
    /// is created. The Day-4 smoke creates `HueSaturationBrightness` here.
    #[must_use]
    pub fn neutral_for(kind: AdjustmentKind) -> Self {
        match kind {
            AdjustmentKind::HueSaturationBrightness => {
                Self::HueSaturationBrightness(HsbParams::default())
            }
            AdjustmentKind::ColorBalance => Self::ColorBalance(ColorBalanceParams::default()),
            AdjustmentKind::Curves => Self::Curves(CurvesParams::default()),
            AdjustmentKind::GradientMap => Self::GradientMap(GradientMapParams::default()),
            AdjustmentKind::BrightnessContrast => {
                Self::BrightnessContrast(BrightnessContrastParams::default())
            }
            AdjustmentKind::GaussianBlur => Self::GaussianBlur(GaussianBlurParams::default()),
            AdjustmentKind::MotionBlur => Self::MotionBlur(MotionBlurParams::default()),
            AdjustmentKind::Bloom => Self::Bloom(BloomParams::default()),
            AdjustmentKind::Noise => Self::Noise(NoiseParams::default()),
            AdjustmentKind::Sharpen => Self::Sharpen(SharpenParams::default()),
            AdjustmentKind::Halftone => Self::Halftone(HalftoneParams::default()),
            AdjustmentKind::ChromaticAberration => {
                Self::ChromaticAberration(ChromaticAberrationParams::default())
            }
            AdjustmentKind::Vibrance => Self::Vibrance(VibranceParams::default()),
            AdjustmentKind::ColorLookupLut => Self::ColorLookupLut(ColorLookupLutParams::default()),
            AdjustmentKind::PhotoFilter => Self::PhotoFilter(PhotoFilterParams::default()),
            AdjustmentKind::Posterize => Self::Posterize(PosterizeParams::default()),
            AdjustmentKind::Threshold => Self::Threshold(ThresholdParams::default()),
            AdjustmentKind::Invert => Self::Invert(InvertParams::default()),
            AdjustmentKind::Levels => Self::Levels(LevelsParams::default()),
            AdjustmentKind::SelectiveColor => Self::SelectiveColor(SelectiveColorParams::default()),
            AdjustmentKind::ChannelMixer => Self::ChannelMixer(ChannelMixerParams::default()),
            AdjustmentKind::Exposure => Self::Exposure(ExposureParams::default()),
            AdjustmentKind::ShadowsHighlights => {
                Self::ShadowsHighlights(ShadowsHighlightsParams::default())
            }
            AdjustmentKind::BlackAndWhite => Self::BlackAndWhite(BlackAndWhiteParams::default()),
        }
    }
}

/// A non-destructive adjustment layer. §2.2 + [`0045-amendment-1`]: `id` /
/// `clipped_by` / `mask` are raw `u64` LayerId values (the `LayerStack`
/// converts at the boundary). For a `LayerKind::Adjustment` node, these inner
/// fields are authoritative over the outer `Layer`'s.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdjustmentLayer {
    pub id: u64,
    pub name: String,
    pub kind: AdjustmentKind,
    pub params: AdjustmentParams,
    pub mask: Option<u64>,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub visible: bool,
    pub locked: bool,
    pub clipped_by: Option<u64>,
    pub version: u32,
}

impl AdjustmentLayer {
    /// Schema version of a freshly authored adjustment layer (HR-14).
    pub const VERSION: u32 = 1;

    /// New neutral adjustment layer of `kind` with raw layer `id`.
    #[must_use]
    pub fn new(id: u64, name: impl Into<String>, kind: AdjustmentKind) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            params: AdjustmentParams::neutral_for(kind),
            mask: None,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            visible: true,
            locked: false,
            clipped_by: None,
            version: Self::VERSION,
        }
    }

    /// Runtime invariant (§2.5): `kind` and `params` are the same variant.
    /// Gate `adjustment_layer_kind_params_match` asserts this for every kind.
    #[must_use]
    pub fn kind_params_match(&self) -> bool {
        self.params.kind() == self.kind
    }
}

/// PSD export classification — frozen mapping table (ADR-0045 §2.8). A
/// [`PsdExport::Layered`] kind round-trips as a native PSD adjustment layer
/// (the 4-char type key); a [`PsdExport::Baked`] kind has no PSD layer
/// equivalent and is rasterized into the pixel data on export (W16).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PsdExport {
    /// 1:1 native PSD adjustment layer with this 4-char type key.
    Layered(&'static str),
    /// No PSD layer equivalent — baked into pixels on export.
    Baked,
}

impl AdjustmentKind {
    /// The frozen PSD interop mapping (§2.8). Every kind maps to exactly one
    /// [`PsdExport`]; v1 = 16 layered (1:1) + 8 baked.
    #[must_use]
    pub fn psd_export(self) -> PsdExport {
        use AdjustmentKind::*;
        match self {
            // Tier 1 — 5 layered, 7 baked.
            HueSaturationBrightness => PsdExport::Layered("hsbr"),
            ColorBalance => PsdExport::Layered("cobl"),
            Curves => PsdExport::Layered("curv"),
            GradientMap => PsdExport::Layered("grdm"),
            BrightnessContrast => PsdExport::Layered("brit"),
            GaussianBlur | MotionBlur | Bloom | Noise | Sharpen | Halftone
            | ChromaticAberration => PsdExport::Baked,
            // Tier 2 — 11 layered, 1 baked.
            Vibrance => PsdExport::Layered("vibA"),
            ColorLookupLut => PsdExport::Layered("clrL"),
            PhotoFilter => PsdExport::Layered("phfl"),
            Posterize => PsdExport::Layered("post"),
            Threshold => PsdExport::Layered("thrs"),
            Invert => PsdExport::Layered("nvrt"),
            Levels => PsdExport::Layered("levl"),
            SelectiveColor => PsdExport::Layered("selc"),
            ChannelMixer => PsdExport::Layered("mixr"),
            Exposure => PsdExport::Layered("expA"),
            ShadowsHighlights => PsdExport::Baked,
            BlackAndWhite => PsdExport::Layered("blwh"),
        }
    }
}

// ── Submodules (god-module split, 2026-06-04; pure move) ──
mod compute;
mod lut;
mod spatial;
#[cfg(test)]
mod tests;
pub use compute::{
    DISPLAY_LUT_N, SELCOLOR_BUCKETS, add_gradient_stop, adjustment_segment_params,
    adjustment_slider_params, adjustment_toggle_params, apply_adjustment,
    channel_mixer_slider_params, colorbalance_display_luts, curve_value_at, curves_display_luts,
    gradient_map_lut, gradient_stop_color_params, levels_display_lut, move_gradient_stop,
    remove_gradient_stop, selective_color_slider_params, set_adjustment_segment_param,
    set_adjustment_slider_param, set_adjustment_toggle_param, set_channel_mixer_param,
    set_gradient_stop_color_param, set_selective_color_param,
};
// Color Lookup — built-in cinematic looks (per-pixel grade; `.cube` load is a
// shell follow-up).
pub use lut::{LUT_PRESET_COUNT, LUT_PRESETS, apply_color_lookup};
// Window-/coordinate-aware kernels (spatial blurs + Noise/Halftone) and the
// canonical spatial math the GPU pass-graph reconciles against (W4 spatial mesh).
pub use spatial::{
    AdjustWindow, MAX_BLUR_HALF, apply_adjustment_windowed, apply_chromatic_aberration,
    apply_gaussian, apply_halftone, apply_motion_blur, apply_noise, apply_sharpen, gaussian_weights,
    motion_weights,
};
