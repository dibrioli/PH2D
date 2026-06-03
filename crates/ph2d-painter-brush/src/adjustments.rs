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

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct GradientMapParams {
    pub stops: Vec<ColorStop>,
    pub interpolation: GradientInterp,
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

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ColorLookupLutParams {
    pub lut_3d: LutHandle,
    pub intensity: f32,
    pub profile: LutProfile,
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

/// `Levels` — black/gamma/white in + output black/white, flat per §2.6.
#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct LevelsParams {
    pub black_point: f32,
    pub gamma: f32,
    pub white_point: f32,
    pub output_black: f32,
    pub output_white: f32,
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

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ChannelMixerParams {
    /// `[r, g, b, constant]` per output channel.
    pub red_out: [f32; 4],
    pub green_out: [f32; 4],
    pub blue_out: [f32; 4],
    pub monochromatic: bool,
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

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
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

/// Apply a non-destructive adjustment to a window of the compositor's
/// accumulator IN PLACE. `acc` is **straight, LINEAR f32 RGBA** (the same space
/// `ph2d_tool_painter::compositor` blends in) — operating on f32 keeps a stack
/// of adjustments band-free (no 8-bit round-trip in the per-frame composite).
///
/// Mask / opacity / blend-mode are handled by the compositor AROUND this call
/// (copy → `apply_adjustment` → blend by mask×opacity in the layer's blend
/// mode), so this fn is the pure `kind` + `params` → pixel transform. Kinds
/// conventionally defined in display space (Curves / Levels / Posterize)
/// convert linear↔sRGB internally.
///
/// **STUB (W4 T4.1/T4.2 Coord):** the hook signature + wiring are landed (the
/// compositor calls this for every `LayerKind::Adjustment`), but the per-kind
/// compute is the implementer's (T4.3+, HSB first for the Day-4 smoke). Replace
/// the no-op body with `match kind { … }`; an implemented arm goes live the next
/// frame.
pub fn apply_adjustment(kind: &AdjustmentKind, params: &AdjustmentParams, acc: &mut [[f32; 4]]) {
    debug_assert_eq!(
        params.kind(),
        *kind,
        "apply_adjustment: kind/params variant mismatch"
    );
    // The match grows an arm per kind as T4.x land; the remaining kinds stay
    // no-ops (identity) until theirs ships.
    match (kind, params) {
        // T4.3 — Hue/Saturation/Brightness (Day-4 smoke).
        (AdjustmentKind::HueSaturationBrightness, AdjustmentParams::HueSaturationBrightness(p)) => {
            apply_hsb(p, acc)
        }
        // T4.7 — Brightness/Contrast.
        (AdjustmentKind::BrightnessContrast, AdjustmentParams::BrightnessContrast(p)) => {
            apply_brightness_contrast(p, acc)
        }
        // T4.x — Exposure (linear gain + offset + gamma).
        (AdjustmentKind::Exposure, AdjustmentParams::Exposure(p)) => apply_exposure(p, acc),
        // T4.x — Vibrance (OKLab chroma, low-saturation-weighted).
        (AdjustmentKind::Vibrance, AdjustmentParams::Vibrance(p)) => apply_vibrance(p, acc),
        // T4.x — Posterize (display-space level quantization).
        (AdjustmentKind::Posterize, AdjustmentParams::Posterize(p)) => apply_posterize(p, acc),
        // T4.x — Threshold (display-space luma → black/white).
        (AdjustmentKind::Threshold, AdjustmentParams::Threshold(p)) => apply_threshold(p, acc),
        // T4.x — Invert (display-space photographic negative).
        (AdjustmentKind::Invert, AdjustmentParams::Invert(_)) => apply_invert(acc),
        _ => {}
    }
}

// ─────────────────────── sRGB transfer (display space) ───────────────────
//
// Continuous f32 sRGB ↔ linear transfer (IEC 61966), for kinds conventionally
// defined in display space (Invert / Posterize / Threshold). The `ph2d_color`
// crate exposes only the 8-bit byte transfer; these f32 twins avoid the
// quantization round-trip while staying byte-identical at the sample points.

/// linear-light intensity → sRGB-encoded `0..=1` (display space).
#[inline]
fn linear_to_srgb_f32(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// sRGB-encoded `0..=1` (display space) → linear-light intensity.
#[inline]
fn srgb_to_linear_f32(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// Exposure — `acc` is straight LINEAR f32 RGBA (alpha preserved). A photographic
/// exposure in stops (`exposure_ev`, a `2^ev` gain on linear light), then a
/// linear `offset` (lifts/drops the floor), then a `gamma_correction` applied as
/// a power in linear (stored as an offset from 1.0 so the all-zero `Default` is
/// an exact identity). Neutral `{0,0,0}` early-returns identity.
fn apply_exposure(p: &ExposureParams, acc: &mut [[f32; 4]]) {
    if p.exposure_ev == 0.0 && p.offset == 0.0 && p.gamma_correction == 0.0 {
        return;
    }
    let gain = 2.0_f32.powf(p.exposure_ev);
    let inv_gamma = 1.0 / (1.0 + p.gamma_correction).max(1e-3);
    for px in acc.iter_mut() {
        for ch in px.iter_mut().take(3) {
            let v = (*ch * gain + p.offset).max(0.0);
            *ch = v.powf(inv_gamma);
        }
    }
}

/// Vibrance — OKLab chroma scaling. `vibrance` boosts chroma MORE for
/// low-saturation pixels (and tapers off as a pixel approaches full chroma — the
/// "protect already-saturated colors / skin tones" behavior), while `saturation`
/// scales every pixel's chroma uniformly. Both `-1..1`, neutral 0. Rotation-free
/// (hue preserved) and gray-safe (zero chroma stays zero — no rainbow). Alpha
/// preserved; neutral early-returns identity.
fn apply_vibrance(p: &VibranceParams, acc: &mut [[f32; 4]]) {
    if p.vibrance == 0.0 && p.saturation == 0.0 {
        return;
    }
    let sat_mul = (1.0 + p.saturation).max(0.0);
    // OKLab chroma is ~0..0.4 for in-gamut sRGB; normalize against that so the
    // vibrance weight (1 - normalized_chroma) reads "how unsaturated is this".
    const CHROMA_NORM: f32 = 0.4;
    for px in acc.iter_mut() {
        let lab = OklabColor::from_linear(LinearRgba::new(px[0], px[1], px[2], 1.0));
        let chroma = (lab.a * lab.a + lab.b * lab.b).sqrt();
        if chroma > 1e-6 {
            let nc = (chroma / CHROMA_NORM).min(1.0);
            let vib_mul = (1.0 + p.vibrance * (1.0 - nc)).max(0.0);
            let scale = sat_mul * vib_mul;
            let out = OklabColor::new(lab.l, lab.a * scale, lab.b * scale, 1.0).to_linear();
            px[0] = out.r();
            px[1] = out.g();
            px[2] = out.b();
        }
    }
}

/// Posterize — quantize each channel to `levels` (`2..=32`) evenly-spaced steps
/// in DISPLAY (sRGB) space (where the bands read as Photoshop's do), then convert
/// back to linear. `acc` is straight LINEAR f32 RGBA (alpha preserved). Always
/// applies (a freshly-created Posterize is a visible effect, like Photoshop).
fn apply_posterize(p: &PosterizeParams, acc: &mut [[f32; 4]]) {
    let levels = p.levels.clamp(2, 32) as f32;
    let steps = levels - 1.0;
    for px in acc.iter_mut() {
        for ch in px.iter_mut().take(3) {
            let s = linear_to_srgb_f32(*ch);
            let q = (s * steps).round() / steps;
            *ch = srgb_to_linear_f32(q);
        }
    }
}

/// Threshold — every pixel becomes pure black or white by comparing its display-
/// space luma (Rec.601 weights on sRGB, matching Photoshop's Threshold) against
/// `threshold` (`0..=255`). `acc` is straight LINEAR f32 RGBA (alpha preserved).
fn apply_threshold(p: &ThresholdParams, acc: &mut [[f32; 4]]) {
    let cut = p.threshold as f32 / 255.0;
    for px in acc.iter_mut() {
        let luma = 0.299 * linear_to_srgb_f32(px[0])
            + 0.587 * linear_to_srgb_f32(px[1])
            + 0.114 * linear_to_srgb_f32(px[2]);
        let v = if luma >= cut { 1.0 } else { 0.0 };
        px[0] = v;
        px[1] = v;
        px[2] = v;
    }
}

/// Invert — a photographic negative: `1 - x` per channel in DISPLAY (sRGB) space
/// (a linear `1 - x` would skew midtones), converted back to linear. `acc` is
/// straight LINEAR f32 RGBA (alpha preserved).
fn apply_invert(acc: &mut [[f32; 4]]) {
    for px in acc.iter_mut() {
        for ch in px.iter_mut().take(3) {
            *ch = srgb_to_linear_f32(1.0 - linear_to_srgb_f32(*ch));
        }
    }
}

/// Brightness/Contrast. `acc` is straight LINEAR f32 RGBA (alpha preserved).
/// Contrast scales each channel around the perceptual mid-gray pivot; brightness
/// then lerps toward black (`-1`) / white (`+1`) — exact extremes (the same
/// brightness model as [`apply_hsb`], for consistency). Both `-1..1`, neutral 0.
fn apply_brightness_contrast(p: &BrightnessContrastParams, acc: &mut [[f32; 4]]) {
    if p.brightness == 0.0 && p.contrast == 0.0 {
        return;
    }
    // sRGB 0.5 in linear light — contrast pivots around perceptual mid-gray
    // (a linear-0.5 pivot would sit far too bright).
    const PIVOT: f32 = 0.214_041_14;
    let scale = 1.0 + p.contrast; // -1 → flat to pivot, +1 → 2×
    for px in acc.iter_mut() {
        for ch in px.iter_mut().take(3) {
            // Contrast around the pivot, clamped so the brightness lerp below
            // stays well-defined (LDR; the compositor encode clamps anyway).
            let mut v = ((*ch - PIVOT) * scale + PIVOT).clamp(0.0, 1.0);
            if p.brightness > 0.0 {
                v += (1.0 - v) * p.brightness;
            } else if p.brightness < 0.0 {
                v *= 1.0 + p.brightness;
            }
            *ch = v;
        }
    }
}

/// The slider-editable params of an adjustment, as `(label, value01)` in slot
/// order — what the layers panel renders as a labeled slider per slot, and the
/// inverse of [`set_adjustment_slider_param`]. Kinds with bespoke controls
/// (Curves, Gradient Map, …) return empty here and get their own UI later.
#[must_use]
pub fn adjustment_slider_params(params: &AdjustmentParams) -> Vec<(&'static str, f32)> {
    match params {
        AdjustmentParams::HueSaturationBrightness(p) => vec![
            ("Hue", p.h.clamp(0.0, 1.0)),
            ("Sat", (p.s + 1.0) * 0.5),
            ("Bright", (p.b + 1.0) * 0.5),
        ],
        AdjustmentParams::BrightnessContrast(p) => vec![
            ("Bright", (p.brightness + 1.0) * 0.5),
            ("Contrast", (p.contrast + 1.0) * 0.5),
        ],
        // Exposure: EV -4..4, Offset -0.5..0.5, Gamma -0.9..0.9 (all centered).
        AdjustmentParams::Exposure(p) => vec![
            ("Expo", (p.exposure_ev + 4.0) / 8.0),
            ("Offset", p.offset + 0.5),
            ("Gamma", (p.gamma_correction + 0.9) / 1.8),
        ],
        AdjustmentParams::Vibrance(p) => vec![
            ("Vib", (p.vibrance + 1.0) * 0.5),
            ("Sat", (p.saturation + 1.0) * 0.5),
        ],
        // Posterize levels 2..=32; Threshold cutoff 0..=255.
        AdjustmentParams::Posterize(p) => {
            vec![("Levels", (p.levels.clamp(2, 32) as f32 - 2.0) / 30.0)]
        }
        AdjustmentParams::Threshold(p) => vec![("Level", p.threshold as f32 / 255.0)],
        _ => Vec::new(),
    }
}

/// Set slider `slot` of an adjustment from a normalized `0..1` value (inverse of
/// [`adjustment_slider_params`]). Out-of-range slots / non-slider kinds no-op.
pub fn set_adjustment_slider_param(params: &mut AdjustmentParams, slot: usize, value01: f32) {
    let v = value01.clamp(0.0, 1.0);
    match params {
        AdjustmentParams::HueSaturationBrightness(p) => match slot {
            0 => p.h = v,             // 0..1 turns
            1 => p.s = v * 2.0 - 1.0, // -1..1
            2 => p.b = v * 2.0 - 1.0, // -1..1
            _ => {}
        },
        AdjustmentParams::BrightnessContrast(p) => match slot {
            0 => p.brightness = v * 2.0 - 1.0,
            1 => p.contrast = v * 2.0 - 1.0,
            _ => {}
        },
        AdjustmentParams::Exposure(p) => match slot {
            0 => p.exposure_ev = v * 8.0 - 4.0,      // -4..4 EV
            1 => p.offset = v - 0.5,                 // -0.5..0.5
            2 => p.gamma_correction = v * 1.8 - 0.9, // -0.9..0.9 (effective γ 0.1..1.9)
            _ => {}
        },
        AdjustmentParams::Vibrance(p) => match slot {
            0 => p.vibrance = v * 2.0 - 1.0,
            1 => p.saturation = v * 2.0 - 1.0,
            _ => {}
        },
        AdjustmentParams::Posterize(p) if slot == 0 => {
            p.levels = (2.0 + v * 30.0).round().clamp(2.0, 32.0) as u8;
        }
        AdjustmentParams::Threshold(p) if slot == 0 => {
            p.threshold = (v * 255.0).round().clamp(0.0, 255.0) as u8;
        }
        _ => {}
    }
}

/// Hue / Saturation / Brightness in **OKLab** (the project's perceptual color
/// space — gold standard, and the brush engine's native space). `acc` is
/// straight LINEAR f32 RGBA; only RGB is transformed — the alpha (= coverage)
/// is preserved.
///
/// - **Hue** (`h`, in turns) is a RIGID rotation of the `(a, b)` chroma vector,
///   so chroma magnitude is preserved exactly and near-neutral pixels stay
///   neutral. HSL hue is numerically unstable for near-gray pixels (tiny chroma
///   → ill-defined hue), so rotating it scatters incoherent colors — the colored
///   speckle Enio hit on the gray background. An OKLab rotation has no such
///   instability.
/// - **Saturation** (`s`, `-1..1`) scales chroma (`-1` = grayscale, `+1` = 2×).
/// - **Brightness** (`b`, `-1..1`) lerps toward black (`-1`) / white (`+1`) in
///   linear light, so the extremes are EXACT black / white (matches Procreate).
///
/// Neutral `{0, 0, 0}` early-returns an EXACT identity (and skips the OKLab
/// round-trip — also the hot-path win while dragging).
fn apply_hsb(p: &HsbParams, acc: &mut [[f32; 4]]) {
    if p.h == 0.0 && p.s == 0.0 && p.b == 0.0 {
        return;
    }
    let hue_rad = p.h * std::f32::consts::TAU; // turns → radians
    let (hue_sin, hue_cos) = hue_rad.sin_cos();
    let chroma_scale = (1.0 + p.s).max(0.0); // -1 → 0 (gray), +1 → 2×
    for px in acc.iter_mut() {
        let lab = OklabColor::from_linear(LinearRgba::new(px[0], px[1], px[2], 1.0));
        // Hue rotation (rigid) + saturation (scale) of the chroma vector; L kept.
        let a = (lab.a * hue_cos - lab.b * hue_sin) * chroma_scale;
        let b = (lab.a * hue_sin + lab.b * hue_cos) * chroma_scale;
        let out = OklabColor::new(lab.l, a, b, 1.0).to_linear();
        let (mut r, mut g, mut bl) = (out.r(), out.g(), out.b());
        // Brightness: lerp toward black / white in linear (exact at the ends).
        if p.b > 0.0 {
            r += (1.0 - r) * p.b;
            g += (1.0 - g) * p.b;
            bl += (1.0 - bl) * p.b;
        } else if p.b < 0.0 {
            let k = 1.0 + p.b;
            r *= k;
            g *= k;
            bl *= k;
        }
        px[0] = r;
        px[1] = g;
        px[2] = bl;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Every `AdjustmentKind` v1 variant (24) — the public canonical list the
    /// "+ Adjustment" menu also iterates. A new variant added to the enum must be
    /// added to `AdjustmentKind::ALL` (and `psd_export` + `display_name`), so the
    /// behavioral gates below cover it.
    const ALL_KINDS: &[AdjustmentKind] = &AdjustmentKind::ALL;

    /// Gate `adjustment_layer_kind_params_match` (§2.10): the runtime invariant
    /// — a layer built with `neutral_for(kind)` has matching kind/params for
    /// every kind.
    #[test]
    fn adjustment_layer_kind_params_match() {
        for &kind in ALL_KINDS {
            let layer = AdjustmentLayer::new(1, "test", kind);
            assert!(
                layer.kind_params_match(),
                "kind/params mismatch for {kind:?}: params is {:?}",
                layer.params.kind()
            );
            assert_eq!(layer.params.kind(), kind);
        }
    }

    /// Gate `psd_mapping_is_canonical` (§2.10 + §2.8): every kind maps to a
    /// PsdExport; v1 = 16 layered (1:1) + 8 baked; Tier-1 split is 5 + 7.
    #[test]
    fn psd_mapping_is_canonical() {
        assert_eq!(ALL_KINDS.len(), 24, "v1 ships 24 kinds");
        let layered = ALL_KINDS
            .iter()
            .filter(|k| matches!(k.psd_export(), PsdExport::Layered(_)))
            .count();
        let baked = ALL_KINDS
            .iter()
            .filter(|k| matches!(k.psd_export(), PsdExport::Baked))
            .count();
        assert_eq!(layered, 16, "16 kinds round-trip 1:1 as PSD layers");
        assert_eq!(baked, 8, "8 kinds bake on export");
        // Tier 1 (first 12) splits 5 layered / 7 baked per §2.10.
        let tier1_layered = ALL_KINDS[..12]
            .iter()
            .filter(|k| matches!(k.psd_export(), PsdExport::Layered(_)))
            .count();
        assert_eq!(tier1_layered, 5, "Tier 1: 5 layered, 7 baked");
    }

    #[test]
    fn all_kinds_have_nonempty_distinct_display_names() {
        // Every kind in the menu shows a non-empty, unique English label.
        assert_eq!(AdjustmentKind::ALL.len(), 24);
        let mut seen = std::collections::BTreeSet::new();
        for &kind in &AdjustmentKind::ALL {
            let name = kind.display_name();
            assert!(!name.is_empty(), "empty display_name for {kind:?}");
            assert!(seen.insert(name), "duplicate display_name {name:?}");
        }
    }

    #[test]
    fn neutral_params_round_trip_via_postcard() {
        // Postcard serialize/deserialize is the persist path (HR-14); a neutral
        // layer must round-trip identically (no ID-shift in the discriminated
        // union — §2.6).
        for &kind in ALL_KINDS {
            let layer = AdjustmentLayer::new(7, "x", kind);
            let bytes = postcard::to_allocvec(&layer).expect("serialize");
            let back: AdjustmentLayer = postcard::from_bytes(&bytes).expect("deserialize");
            assert_eq!(layer, back, "postcard round-trip for {kind:?}");
        }
    }

    // ── T4.3 — Hue/Saturation/Brightness compute (Day-4 smoke) ────────────
    //
    // Invariant/property tests (neutral identity, hue rotation, desaturation,
    // brightness extremes, alpha preservation). A pixel-exact golden vs a
    // Photoshop reference (SSIM ≥ 0.999, plan §7) needs a real PS-exported
    // fixture asset — flagged to the Coord, not faked here.

    fn hsb(h: f32, s: f32, b: f32) -> AdjustmentParams {
        AdjustmentParams::HueSaturationBrightness(HsbParams { h, s, b })
    }

    fn apply(h: f32, s: f32, b: f32, px: [f32; 4]) -> [f32; 4] {
        let mut acc = [px];
        apply_adjustment(
            &AdjustmentKind::HueSaturationBrightness,
            &hsb(h, s, b),
            &mut acc,
        );
        acc[0]
    }

    #[test]
    fn hsb_neutral_is_exact_identity() {
        let px = [0.3, 0.6, 0.1, 0.8];
        assert_eq!(apply(0.0, 0.0, 0.0, px), px, "neutral {{0,0,0}} is a no-op");
    }

    #[test]
    fn hsb_preserves_alpha() {
        let out = apply(0.25, 0.5, -0.3, [0.8, 0.2, 0.2, 0.42]);
        assert_eq!(out[3], 0.42, "alpha (coverage) is never touched");
    }

    #[test]
    fn hsb_saturation_minus_one_is_grayscale() {
        // s = -1 collapses saturation → R == G == B (achromatic).
        let out = apply(0.0, -1.0, 0.0, [1.0, 0.0, 0.0, 1.0]);
        assert!(
            (out[0] - out[1]).abs() < 1e-5 && (out[1] - out[2]).abs() < 1e-5,
            "fully desaturated red is gray: {out:?}"
        );
    }

    #[test]
    fn hsb_brightness_extremes_clamp_to_black_and_white() {
        let black = apply(0.0, 0.0, -1.0, [1.0, 0.0, 0.0, 1.0]);
        assert!(
            black[0] < 1e-4 && black[1] < 1e-4 && black[2] < 1e-4,
            "brightness -1 → black: {black:?}"
        );
        let white = apply(0.0, 0.0, 1.0, [1.0, 0.0, 0.0, 1.0]);
        assert!(
            white[0] > 0.999 && white[1] > 0.999 && white[2] > 0.999,
            "brightness +1 → white: {white:?}"
        );
    }

    #[test]
    fn hsb_hue_rotation_shifts_red_toward_green() {
        // +1/3 turn (120°) rotates red's OKLab chroma vector into the green
        // half-plane → green becomes the dominant channel.
        let out = apply(1.0 / 3.0, 0.0, 0.0, [1.0, 0.0, 0.0, 1.0]);
        assert!(
            out[1] > out[0] && out[1] > out[2],
            "red rotated +1/3 turn is green-dominant: {out:?}"
        );
    }

    #[test]
    fn hsb_full_turn_hue_is_identity() {
        // A whole-turn hue rotation is a 2π chroma rotation → original color
        // (within the OKLab linear round-trip, ~1e-3).
        let px = [0.7, 0.2, 0.4, 1.0];
        let out = apply(1.0, 0.0, 0.0, px);
        for c in 0..3 {
            assert!(
                (out[c] - px[c]).abs() < 5e-3,
                "full-turn ~identity: {out:?}"
            );
        }
    }

    // ── T4.7 — Brightness/Contrast + generic slider plumbing ──────────────

    fn apply_bc(brightness: f32, contrast: f32, px: [f32; 4]) -> [f32; 4] {
        let mut acc = [px];
        apply_adjustment(
            &AdjustmentKind::BrightnessContrast,
            &AdjustmentParams::BrightnessContrast(BrightnessContrastParams {
                brightness,
                contrast,
                legacy: false,
            }),
            &mut acc,
        );
        acc[0]
    }

    #[test]
    fn brightness_contrast_neutral_is_identity() {
        let px = [0.3, 0.6, 0.1, 0.5];
        assert_eq!(apply_bc(0.0, 0.0, px), px);
        assert_eq!(
            apply_bc(0.4, -0.3, [0.5, 0.5, 0.5, 0.42])[3],
            0.42,
            "alpha kept"
        );
    }

    #[test]
    fn contrast_pushes_brights_up_darks_down() {
        // +contrast pushes a bright pixel brighter and a dark pixel darker
        // (around the perceptual mid-gray pivot).
        let bright = apply_bc(0.0, 0.5, [0.8, 0.8, 0.8, 1.0]);
        let dark = apply_bc(0.0, 0.5, [0.05, 0.05, 0.05, 1.0]);
        assert!(bright[0] > 0.8, "bright got brighter: {bright:?}");
        assert!(dark[0] < 0.05, "dark got darker: {dark:?}");
    }

    #[test]
    fn bc_brightness_extremes_clamp_to_black_and_white() {
        assert!(
            apply_bc(-1.0, 0.0, [0.7, 0.4, 0.2, 1.0])
                .iter()
                .take(3)
                .all(|&v| v < 1e-4)
        );
        assert!(
            apply_bc(1.0, 0.0, [0.7, 0.4, 0.2, 1.0])
                .iter()
                .take(3)
                .all(|&v| v > 0.999)
        );
    }

    // ── T4.x — per-pixel fan-out kinds (Invert/Exposure/Vibrance/Posterize/
    // Threshold): invariant tests (neutral identity, extremes, alpha, gray-safe).

    fn run(kind: AdjustmentKind, params: AdjustmentParams, px: [f32; 4]) -> [f32; 4] {
        let mut acc = [px];
        apply_adjustment(&kind, &params, &mut acc);
        acc[0]
    }

    #[test]
    fn invert_is_a_display_space_negative() {
        let white = run(
            AdjustmentKind::Invert,
            AdjustmentParams::Invert(InvertParams {}),
            [1.0, 1.0, 1.0, 0.7],
        );
        assert!(
            white.iter().take(3).all(|&v| v < 1e-4),
            "white → black: {white:?}"
        );
        assert_eq!(white[3], 0.7, "alpha preserved");
        let black = run(
            AdjustmentKind::Invert,
            AdjustmentParams::Invert(InvertParams {}),
            [0.0, 0.0, 0.0, 1.0],
        );
        assert!(
            black.iter().take(3).all(|&v| v > 0.999),
            "black → white: {black:?}"
        );
    }

    #[test]
    fn exposure_neutral_identity_and_ev_brightens() {
        let px = [0.2, 0.4, 0.1, 0.55];
        let neutral = run(
            AdjustmentKind::Exposure,
            AdjustmentParams::Exposure(ExposureParams::default()),
            px,
        );
        assert_eq!(neutral, px, "all-zero Exposure is an exact identity");
        // +1 EV doubles linear light.
        let up = run(
            AdjustmentKind::Exposure,
            AdjustmentParams::Exposure(ExposureParams {
                exposure_ev: 1.0,
                offset: 0.0,
                gamma_correction: 0.0,
            }),
            [0.25, 0.25, 0.25, 0.4],
        );
        assert!((up[0] - 0.5).abs() < 1e-5, "+1 EV: 0.25 → 0.5: {up:?}");
        assert_eq!(up[3], 0.4, "alpha preserved");
    }

    #[test]
    fn vibrance_neutral_identity_and_gray_stays_gray() {
        let px = [0.6, 0.3, 0.15, 0.9];
        let neutral = run(
            AdjustmentKind::Vibrance,
            AdjustmentParams::Vibrance(VibranceParams::default()),
            px,
        );
        assert_eq!(neutral, px, "neutral Vibrance is an exact identity");
        // A gray pixel has zero chroma → any vibrance/saturation leaves it gray
        // (no rainbow speckle — the same gray-safety as the OKLab HSB).
        let gray = run(
            AdjustmentKind::Vibrance,
            AdjustmentParams::Vibrance(VibranceParams {
                vibrance: 1.0,
                saturation: 1.0,
            }),
            [0.5, 0.5, 0.5, 1.0],
        );
        assert!(
            (gray[0] - gray[1]).abs() < 1e-4 && (gray[1] - gray[2]).abs() < 1e-4,
            "gray stays gray under max vibrance: {gray:?}"
        );
    }

    #[test]
    fn vibrance_full_desaturation_is_grayscale() {
        let out = run(
            AdjustmentKind::Vibrance,
            AdjustmentParams::Vibrance(VibranceParams {
                vibrance: 0.0,
                saturation: -1.0,
            }),
            [1.0, 0.0, 0.0, 1.0],
        );
        assert!(
            (out[0] - out[1]).abs() < 1e-4 && (out[1] - out[2]).abs() < 1e-4,
            "saturation -1 collapses red to gray: {out:?}"
        );
    }

    #[test]
    fn posterize_two_levels_snaps_to_black_or_white() {
        // levels = 2 → each channel rounds to display 0 or 1.
        for &c in &[0.05_f32, 0.3, 0.7, 0.95] {
            let out = run(
                AdjustmentKind::Posterize,
                AdjustmentParams::Posterize(PosterizeParams { levels: 2 }),
                [c, c, c, 0.5],
            );
            assert!(
                out[0] < 1e-4 || out[0] > 0.999,
                "posterize-2 snaps {c} to an endpoint: {out:?}"
            );
            assert_eq!(out[3], 0.5, "alpha preserved");
        }
    }

    #[test]
    fn threshold_splits_on_luma() {
        let bright = run(
            AdjustmentKind::Threshold,
            AdjustmentParams::Threshold(ThresholdParams { threshold: 128 }),
            [0.9, 0.9, 0.9, 0.6],
        );
        assert!(
            bright.iter().take(3).all(|&v| v > 0.999),
            "bright → white: {bright:?}"
        );
        assert_eq!(bright[3], 0.6, "alpha preserved");
        let dark = run(
            AdjustmentKind::Threshold,
            AdjustmentParams::Threshold(ThresholdParams { threshold: 128 }),
            [0.02, 0.02, 0.02, 1.0],
        );
        assert!(
            dark.iter().take(3).all(|&v| v < 1e-4),
            "dark → black: {dark:?}"
        );
    }

    #[test]
    fn fanout_slider_round_trips() {
        // Each new slider kind: set then read returns the same 0..1 (the panel
        // relies on this for stable thumb positions). Quantized kinds (Posterize/
        // Threshold) round-trip on the canonical step grid.
        let mut exp = AdjustmentParams::Exposure(ExposureParams::default());
        for (slot, want) in [(0usize, 0.2_f32), (1, 0.8), (2, 0.35)] {
            set_adjustment_slider_param(&mut exp, slot, want);
        }
        let read = adjustment_slider_params(&exp);
        for (got, want) in read.iter().map(|r| r.1).zip([0.2, 0.8, 0.35]) {
            assert!(
                (got - want).abs() < 1e-6,
                "exposure round-trip {got} vs {want}"
            );
        }
        let mut vib = AdjustmentParams::Vibrance(VibranceParams::default());
        set_adjustment_slider_param(&mut vib, 0, 0.9);
        set_adjustment_slider_param(&mut vib, 1, 0.1);
        let vr = adjustment_slider_params(&vib);
        assert!((vr[0].1 - 0.9).abs() < 1e-6 && (vr[1].1 - 0.1).abs() < 1e-6);
    }

    #[test]
    fn slider_params_round_trip() {
        // adjustment_slider_params is the inverse of set_adjustment_slider_param.
        let mut hsb = AdjustmentParams::HueSaturationBrightness(HsbParams::default());
        set_adjustment_slider_param(&mut hsb, 0, 0.25);
        set_adjustment_slider_param(&mut hsb, 1, 0.75);
        set_adjustment_slider_param(&mut hsb, 2, 0.10);
        let read = adjustment_slider_params(&hsb);
        assert_eq!(read.len(), 3);
        for (got, want) in read.iter().map(|r| r.1).zip([0.25, 0.75, 0.10]) {
            assert!(
                (got - want).abs() < 1e-6,
                "hsb slot round-trip: {got} vs {want}"
            );
        }
        let mut bc = AdjustmentParams::BrightnessContrast(BrightnessContrastParams::default());
        set_adjustment_slider_param(&mut bc, 1, 0.9);
        let bc_read = adjustment_slider_params(&bc);
        assert_eq!(bc_read.len(), 2);
        assert!((bc_read[1].1 - 0.9).abs() < 1e-6);
    }
}
