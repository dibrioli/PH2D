//! **The NUMBER an artist reads on each adjustment slider** — the twin of
//! [`super::adjustment_slider_params`] for the value, not the thumb.
//!
//! Every slider stores a normalized `0..1` thumb, and [`super::set_adjustment_slider_param`]
//! maps it to the param. Until 2026-09-16 the layers panel showed only the thumb — no number at
//! all, so putting *Contrast* at `+25` meant dragging until it looked right. The panel now paints
//! each slider as the app's single box (name inside, editable number), and the number is
//! `thumb × scale + offset` in the artist's unit (`%`, `°`, `px`, levels).
//!
//! ⚠️ **The mapping lives HERE, beside the setter, and reuses the setter's own range constants** —
//! a second copy in the panel would be a second answer to *"what does this thumb mean?"*, and a
//! wrong mapping edits the wrong value **silently**. The panel's seam gate types a number into
//! each box and reads the param back through an independent oracle.
//!
//! ⚠️ **Units:** a signed unit-range param (`-1..1`) reads as `-100..100 %`; a `0..1` amount as
//! `0..100 %`; turns and radians as degrees; pixel extents as pixels; 8-bit levels as `0..255`.

use super::params::{
    BLOOM_INTENSITY_MAX, CHROMA_SHIFT_MAX, HALFTONE_DOT_MAX, SHARPEN_AMOUNT_MAX,
    SHARPEN_RADIUS_MAX, SPATIAL_PX_MAX,
};
use super::*;

/// How one slider's thumb reads as a number.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SliderNumber {
    /// `number = thumb × scale + offset` — the setter's own affine map, so the number can be
    /// typed and projected back onto the thumb. `integer`: the param is a count/level and the
    /// number shows (and snaps to) whole values.
    Affine {
        scale: f32,
        offset: f32,
        integer: bool,
    },
    /// The thumb is NOT affine in the param (Levels gamma): the number is shown, never typed.
    Shown(f32),
}

impl SliderNumber {
    /// The number for a given thumb position (`Shown` ignores the thumb — it carries its value).
    #[must_use]
    pub fn at(self, thumb01: f32) -> f32 {
        match self {
            Self::Affine { scale, offset, .. } => thumb01 * scale + offset,
            Self::Shown(v) => v,
        }
    }
}

/// Percent of a fraction.
const PCT: f32 = 100.0;
/// Degrees of a full turn.
const TURN_DEG: f32 = 360.0;
/// The top 8-bit level.
const BYTE_MAX: f32 = 255.0;

const fn affine(scale: f32, offset: f32) -> SliderNumber {
    SliderNumber::Affine {
        scale,
        offset,
        integer: false,
    }
}

const fn whole(scale: f32, offset: f32) -> SliderNumber {
    SliderNumber::Affine {
        scale,
        offset,
        integer: true,
    }
}

/// A `0..1` amount, read as `0..100 %`.
const PERCENT: SliderNumber = affine(PCT, 0.0);
/// A signed `-1..1` amount (`thumb × 2 − 1`), read as `-100..100 %`.
const SIGNED_PERCENT: SliderNumber = affine(2.0 * PCT, -PCT);
/// A full turn, read in degrees.
const DEGREES: SliderNumber = affine(TURN_DEG, 0.0);
/// A pixel extent `0..SPATIAL_PX_MAX`.
const SPATIAL_PX: SliderNumber = affine(SPATIAL_PX_MAX, 0.0);
/// An 8-bit level `0..255`.
const LEVEL_8BIT: SliderNumber = whole(BYTE_MAX, 0.0);

/// The numbers of [`super::adjustment_slider_params`], slot for slot (same length, same order).
#[must_use]
pub fn adjustment_slider_numbers(params: &AdjustmentParams) -> Vec<SliderNumber> {
    match params {
        AdjustmentParams::HueSaturationBrightness(_) => {
            vec![DEGREES, SIGNED_PERCENT, SIGNED_PERCENT]
        }
        AdjustmentParams::BrightnessContrast(_) => vec![SIGNED_PERCENT, SIGNED_PERCENT],
        // EV -4..4 · offset -0.5..0.5 · the EFFECTIVE gamma `1 + correction` (0.1..1.9), which is
        // what the kernel divides by and what reads as "1.00 = untouched".
        AdjustmentParams::Exposure(_) => {
            vec![affine(8.0, -4.0), affine(1.0, -0.5), affine(1.8, 0.1)]
        }
        AdjustmentParams::Vibrance(_) => vec![SIGNED_PERCENT, SIGNED_PERCENT],
        AdjustmentParams::Posterize(_) => vec![whole(30.0, 2.0)],
        AdjustmentParams::Threshold(_) => vec![LEVEL_8BIT],
        AdjustmentParams::PhotoFilter(_) => vec![SIGNED_PERCENT, PERCENT],
        AdjustmentParams::ColorBalance(_) => vec![SIGNED_PERCENT; 3],
        AdjustmentParams::BlackAndWhite(p) => {
            // Weights `-2..3` read as `-200..300 %`.
            let mut v = vec![affine(5.0 * PCT, -2.0 * PCT); 6];
            if p.tint_color.is_some() {
                v.push(DEGREES);
                v.push(PERCENT);
            }
            v
        }
        // Levels reads in 8-bit levels, like every levels dialog; the gamma thumb is a curve.
        AdjustmentParams::Levels(p) => vec![
            LEVEL_8BIT,
            SliderNumber::Shown(p.gamma),
            LEVEL_8BIT,
            LEVEL_8BIT,
            LEVEL_8BIT,
        ],
        AdjustmentParams::GaussianBlur(_) => vec![SPATIAL_PX],
        AdjustmentParams::MotionBlur(_) => vec![SPATIAL_PX, DEGREES],
        AdjustmentParams::Sharpen(_) => vec![
            affine(SHARPEN_AMOUNT_MAX * PCT, 0.0),
            affine(SHARPEN_RADIUS_MAX, 0.0),
        ],
        AdjustmentParams::ChromaticAberration(_) => {
            vec![affine(2.0 * CHROMA_SHIFT_MAX, -CHROMA_SHIFT_MAX); 3]
        }
        AdjustmentParams::Noise(_) => vec![PERCENT],
        AdjustmentParams::Halftone(_) => vec![affine(HALFTONE_DOT_MAX - 1.0, 1.0), DEGREES],
        // The look is a preset INDEX (the thumb snaps on the preset grid).
        AdjustmentParams::ColorLookupLut(_) => {
            vec![
                whole((super::super::lut::LUT_PRESET_COUNT - 1) as f32, 0.0),
                PERCENT,
            ]
        }
        AdjustmentParams::Bloom(_) => vec![
            PERCENT,
            affine(BLOOM_INTENSITY_MAX * PCT, 0.0),
            SPATIAL_PX,
            PERCENT,
        ],
        AdjustmentParams::ShadowsHighlights(_) => vec![
            PERCENT,
            PERCENT,
            SPATIAL_PX,
            PERCENT,
            PERCENT,
            SPATIAL_PX,
            SIGNED_PERCENT,
            SIGNED_PERCENT,
        ],
        _ => Vec::new(),
    }
}

/// The numbers of [`super::channel_mixer_slider_params`]: source weights `-2..2` read as
/// `-200..200 %`, the constant `-1..1` as `-100..100 %`.
#[must_use]
pub fn channel_mixer_slider_numbers() -> [SliderNumber; 4] {
    let weight = affine(4.0 * PCT, -2.0 * PCT);
    [weight, weight, weight, SIGNED_PERCENT]
}

/// The numbers of [`super::selective_color_slider_params`]: each CMYK shift `-1..1` as
/// `-100..100 %`.
#[must_use]
pub fn selective_color_slider_numbers() -> [SliderNumber; 4] {
    [SIGNED_PERCENT; 4]
}

/// The numbers of [`super::gradient_stop_color_params`]: each channel as an 8-bit level.
#[must_use]
pub fn gradient_stop_color_numbers() -> [SliderNumber; 3] {
    [LEVEL_8BIT; 3]
}
