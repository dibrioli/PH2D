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

use ph2d_blend_mode::BlendMode;
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

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BloomParams {
    pub threshold: f32,
    pub intensity: f32,
    pub radius: f32,
    pub falloff: f32,
}

impl Default for BloomParams {
    /// Neutral on creation (`intensity 0` → no glow), but the threshold / radius /
    /// falloff are seeded to usable values so raising "Intensity" immediately
    /// blooms the bright areas with a sensible soft glow (a derived all-zero
    /// default would bloom the WHOLE image with a 0-px radius — a hard double).
    fn default() -> Self {
        Self {
            threshold: 0.7,
            intensity: 0.0,
            radius: 20.0,
            falloff: 0.15,
        }
    }
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

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
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

impl Default for ShadowsHighlightsParams {
    /// Neutral on creation (both `*_amount` and `midtone_contrast` = 0 → identity),
    /// but the tonal widths + local radii seed to usable values so the first
    /// amount-drag behaves like a smooth Photoshop-style local correction (a
    /// derived all-zero default would make the tonal widths a hard 0-width step).
    fn default() -> Self {
        Self {
            shadows_amount: 0.0,
            shadows_tonal_width: 0.5,
            shadows_radius: 30.0,
            highlights_amount: 0.0,
            highlights_tonal_width: 0.5,
            highlights_radius: 30.0,
            color_correction: 0.0,
            midtone_contrast: 0.0,
        }
    }
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

// ── Submodules (god-module split, 2026-06-04; pure move) ──
mod compute;
mod gpu_codes; // which GPU path each kind takes (LOC cap: sibling module)
mod lut;
mod params_queries; // what an AdjustmentParams answers: kind, GPU scalars, neutral seed (LOC cap: sibling module)
mod psd_export; // the frozen PSD interop mapping, ADR-0045 §2.8 (LOC cap: sibling module)
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
// The number an artist reads on each slider (the twin of the `*_slider_params` above).
pub use compute::{
    SliderNumber, adjustment_slider_numbers, channel_mixer_slider_numbers,
    gradient_stop_color_numbers, selective_color_slider_numbers,
};
// Color Lookup — built-in cinematic looks (per-pixel grade; `.cube` load is a
// shell follow-up).
pub use lut::{LUT_PRESET_COUNT, LUT_PRESETS, apply_color_lookup};
pub use psd_export::PsdExport;
// Window-/coordinate-aware kernels (spatial blurs + Noise/Halftone). The Gaussian
// + Motion math is the canonical reference the GPU pass-graph reconciles against
// (gated by `spatial_weights_parity`); `apply_chromatic_aberration` is NOT yet
// reconciled and DIVERGES from the GPU (see `spatial.rs` §Two roles, audit 2026-06-18).
pub use spatial::{
    AdjustWindow, MAX_BLUR_HALF, apply_adjustment_windowed, apply_bloom,
    apply_chromatic_aberration, apply_gaussian, apply_halftone, apply_motion_blur, apply_noise,
    apply_shadows_highlights, apply_sharpen, gaussian_weights, motion_weights,
};
