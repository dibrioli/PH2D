//! **O que um `AdjustmentParams` responde** — o `kind`, os escalares da GPU (`gpu_params` /
//! `spatial_params`) e a semente neutra (`neutral_for`) —, irmão de `adjustments/mod.rs` por tecto
//! de LOC.
//!
//! Corte mecânico: o `impl AdjustmentParams` saiu inteiro, verbatim; são métodos, nenhum chamador
//! muda de endereço.

use super::*;

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
            // Coordinate-dependent per-pixel kinds (ADJ_NOISE/HALFTONE/COLOR_LOOKUP).
            // Enum discriminants cast directly (the WGSL switches on the same order).
            Self::Noise(p) => [
                p.amount,
                p.kind as u8 as f32,
                if p.monochromatic { 1.0 } else { 0.0 },
            ],
            Self::Halftone(p) => [p.dot_size, p.angle, p.shape as u8 as f32],
            Self::ColorLookupLut(p) => [p.lut_3d.0 as f32, p.intensity, 0.0],
            _ => [0.0, 0.0, 0.0],
        }
    }

    /// The scalars the spatial pass-graph reads (`LayerOp::SpatialAdjustment`
    /// `params: [f32; 8]`), or `None` for a non-spatial kind. Packing mirrors
    /// `ph2d_render::layer_compositor::SPATIAL_*` (validated by the spatial parity
    /// gates), in lock-step with [`AdjustmentKind::gpu_spatial_code`]. The tail is
    /// zero for the ≤4-param kinds; only `ShadowsHighlights` uses all 8:
    /// - `GaussianBlur` → `[radius, 0, …]`
    /// - `Sharpen` → `[amount, radius, 0, …]` (unsharp: `base + amount·(base−blur)`)
    /// - `MotionBlur` → `[distance, angle_rad, 0, …]`
    /// - `ChromaticAberration` → `[red_shift, green_shift, blue_shift, falloff_center, 0, …]`
    /// - `Bloom` → `[threshold, intensity, radius, falloff, 0, …]`
    /// - `ShadowsHighlights` → `[shad_amount, shad_tonal_width, shad_radius,
    ///   high_amount, high_tonal_width, high_radius, color_correction, midtone_contrast]`
    ///
    /// The painter flatten passes this verbatim into the op so the GPU and the CPU
    /// reference (`apply_*` in `compute.rs`/`spatial.rs`) read identical numbers.
    #[must_use]
    pub fn spatial_params(&self) -> Option<[f32; 8]> {
        Some(match self {
            Self::GaussianBlur(p) => [p.radius, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            Self::Sharpen(p) => [p.amount, p.radius, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            Self::MotionBlur(p) => [p.distance, p.angle, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            Self::ChromaticAberration(p) => [
                p.red_shift,
                p.green_shift,
                p.blue_shift,
                p.falloff_center,
                0.0,
                0.0,
                0.0,
                0.0,
            ],
            Self::Bloom(p) => [
                p.threshold,
                p.intensity,
                p.radius,
                p.falloff,
                0.0,
                0.0,
                0.0,
                0.0,
            ],
            Self::ShadowsHighlights(p) => [
                p.shadows_amount,
                p.shadows_tonal_width,
                p.shadows_radius,
                p.highlights_amount,
                p.highlights_tonal_width,
                p.highlights_radius,
                p.color_correction,
                p.midtone_contrast,
            ],
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
